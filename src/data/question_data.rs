use std::collections::HashMap;

#[cfg(feature = "pyo3")]
use chrono::{NaiveDate, NaiveTime};
use parking_lot::RwLockReadGuard;
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::{
    IntoPyObjectExt,
    exceptions,
    prelude::*,
    types::{PyDict, PyTuple},
};
#[cfg(feature = "pyo3")]
use pyo3_polars::{PyDataFrame, error::PyPolarsErr};

#[cfg(feature = "pyo3")]
use crate::{
    DataRange,
    config::DataFrameFormat,
    error::InternalError,
    prelude::*,
    utils::{
        anyvalue_to_python,
        convert_date,
        convert_time,
        normalize_py_index,
        polars_to_pandas,
    },
};
use crate::{
    QuestionType,
    config::ResponseDisplay,
    data::{config::Config, survey_data::SurveyData},
    datamap::{Question, Text},
    utils::ArcLock,
};

/// Data for a single question.
#[cfg_attr(
    all(feature = "pyo3", not(doc)),
    doc = r###"
Raises:
    NotImplementedError: Direct construction is not supported. Obtain question data from a Survey instance.
"###
)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "QuestionData", from_py_object)
)]
#[derive(Clone)]
pub struct QuestionData {
    data: ArcLock<SurveyData>,
    config: ArcLock<Config>,
    question: Question,
}

/// ---
///
/// # Fields
impl QuestionData {
    /// name of the question
    #[inline]
    pub fn name(&self) -> String {
        self.question.name()
    }

    /// Type of the question
    #[inline]
    pub fn question_type(&self) -> QuestionType {
        self.question.question_type().clone()
    }

    /// Get [`LazyFrame`] of the question data
    #[inline]
    pub fn lazy(&self) -> PolarsResult<LazyFrame> {
        Ok(self.data.read().lazy()?.clone().select([col(self.name())]))
    }

    /// Get datamap for the current question
    #[inline]
    pub fn datamap(&self) -> &Question {
        &self.question
    }

    /// Get mutable view of the datamap for the current question
    #[inline]
    pub fn datamap_mut(&mut self) -> &mut Question {
        &mut self.question
    }

    /// Get the lookup table for items in the [`Question::Response()::values()`] field, as: { [`Code::value`]: [`Code::label()`] }
    ///
    /// Note that this methods returns a *deepcopy* of the data, and modifying items returned by this
    /// method will not mutate the caller.
    #[allow(clippy::mutable_key_type)] // can only be string, float, or int
    pub fn response_label(&self) -> HashMap<AnyValue<'static>, Text> {
        self.question.response_label()
    }

    /// Configuration options
    pub fn config(&self) -> RwLockReadGuard<'_, Config> {
        self.config.read()
    }
}

/// Methods
///
/// ---
impl QuestionData {
    /// Create new [`QuestionData`] instance
    pub(crate) fn new(
        data: ArcLock<SurveyData>,
        question: Question,
        config: ArcLock<Config>,
    ) -> Self {
        Self {
            data,
            question,
            config,
        }
    }

    /// Export contents to a polars [`Column`]
    pub fn to_column(&self) -> PolarsResult<Column> {
        match self.config().response_display {
            ResponseDisplay::Value => self.to_value_column(),
            ResponseDisplay::Label => match self.question_type() {
                QuestionType::Single => self.to_labelled_column_single_choice(),
                QuestionType::Multiple => self.to_labelled_column_multi_choice(),
                _ => self.to_value_column(),
            },
        }
    }

    /// Export contents to a polars [`Series`]
    pub fn to_series(&self) -> PolarsResult<Series> {
        self.to_column().map(Column::take_materialized_series)
    }

    /// Count unique values of this Series and a column "counts" with dtype IdxType
    ///
    /// This function basically calls [`polars::Series::value_counts`] under the hood, apart from multi-choice
    /// questions ([`QuestionData::question_type()`] is [`QuestionType::Multiple`]),
    /// where the choice for each respondant is counted individually.
    ///
    /// # Arguments
    /// - `count_nan` (`bool`):
    ///     - Toogle on/ off for counting Null values, note that multi-choices entries of an empty list (`[]`) will also count as `null`
    pub fn value_counts(&self, count_nan: bool) -> PolarsResult<DataFrame> {
        // Get raw value counts, always use the unlabelled version
        let series = match self.question_type() {
            QuestionType::Multiple => {
                let config = ExplodeOptions {
                    empty_as_null: true,
                    keep_nulls: true, // cell with value `null` gets exploded into `null`
                };
                self.to_value_column()?.explode(config)?.take_materialized_series()
            }
            _ => self.to_series()?,
        };

        let series = if count_nan {
            series.drop_nulls()
        } else {
            series
        };

        let df = series.value_counts(true, true, "count".into(), false)?.sort(
            [self.name()],
            SortMultipleOptions::default()
                .with_nulls_last(true)
                .with_maintain_order(false)
                .with_order_descending(false),
        )?;

        // append data label
        let (keys, labels) = self.get_datalabel_key_value()?;
        let keys = keys.cast(series.dtype())?;
        let height = labels.len();
        let response_label = DataFrame::new(height, vec![keys.into(), labels.into()])?;

        df.lazy()
            .left_join(response_label.lazy(), col(self.name()), col(self.name()))
            .select([col(self.name()), col("label"), col("count")])
            .collect()
    }

    /// Checks if `other` is comes from the same [`Survey`] as `self`.
    pub fn is_same_survey(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    fn get_datalabel_key_value(&self) -> PolarsResult<(Series, Series)> {
        let (keys, values): (Vec<AnyValue<'_>>, Vec<String>) =
            self.response_label().into_iter().map(|(key, val)| (key, val.value())).unzip();
        let keys = Series::from_any_values(self.name().into(), &keys, true)?;
        let labels = Series::new("label".into(), values);
        Ok((keys, labels))
    }

    /// Convert single quesiton labels -> values
    fn to_labelled_column_single_choice(&self) -> PolarsResult<Column> {
        let (keys, labels) = self.get_datalabel_key_value()?;
        let question_name = self.name();
        let query = self.lazy()?.select([col(&question_name).replace_strict(
            lit(keys.clone()),
            lit(labels),
            None,
            Some(DataTypeExpr::Literal(DataType::String)),
        )]);

        query
            .collect()
            .map_err(|err| Self::map_missing_label_error(err, &self.name(), &keys))?
            .into_columns()
            .into_iter()
            .next()
            .ok_or(PolarsError::ColumnNotFound(
                format!("Unable to find column '{}' in `SurveyData`", self.name()).into(),
            ))
    }

    fn to_labelled_column_multi_choice(&self) -> PolarsResult<Column> {
        let (keys, labels) = self.get_datalabel_key_value()?;
        let question_name = self.name();

        let query = col(&question_name)
            .list()
            .eval(element().replace_strict(
                lit(keys.clone()),
                lit(labels),
                None,
                Some(DataTypeExpr::Literal(DataType::String)),
            ))
            .alias(&question_name);
        let query = self.lazy()?.select([query]);

        query
            .collect()
            .map_err(|err| Self::map_missing_label_error(err, &self.name(), &keys))?
            .into_columns()
            .into_iter()
            .next()
            .ok_or(PolarsError::ColumnNotFound(
                format!("Unable to find column '{}' in `SurveyData`", self.name()).into(),
            ))
    }

    #[inline]
    fn map_missing_label_error(
        err: PolarsError,
        question_name: &str,
        unique_keys: &Series,
    ) -> PolarsError {
        match &err {
            PolarsError::ExprContext { error, expr } if expr.contains(".replace_strict") => {
                if expr.contains(").replace_strict") {
                    PolarsError::ComputeError(format!(
                        "Question '{}' contains response values with no matching label, expect to be one of: {}",
                        question_name,
                        unique_keys.fmt_list()
                    ).into())
                } else {
                    err
                }
            }
            _ => err,
        }
    }

    fn to_value_column(&self) -> PolarsResult<Column> {
        self.lazy()?.collect()?.into_columns().into_iter().next().ok_or(
            PolarsError::ColumnNotFound(
                format!("Unable to find column '{}' in `SurveyData`", self.name()).into(),
            ),
        )
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl QuestionData {
    /// Create a QuestionData instance.
    ///
    /// Raises:
    ///     NotImplementedError: Direct construction is not supported. Obtain question data from a Survey instance.
    #[new]
    fn py_new() -> PyResult<Self> {
        Err(exceptions::PyNotImplementedError::new_err(
            "Not implemented yet!",
        ))
    }

    /// Name of the question
    #[getter(name)]
    fn py_get_name(&self) -> String {
        self.name()
    }

    /// Title of the question
    #[getter(label)]
    fn py_get_label(&self) -> Text {
        self.question.label().clone()
    }

    /// Minimum and maximum values allowed for a quantity response,
    /// or an implicit range of codes for a numeric categorical response.
    /// For character responses, the end stores the maximum size plus one.
    /// String categorical responses do not support a range.
    #[getter(range)]
    fn py_get_range(&self) -> Option<DataRange> {
        self.question.response().range().clone()
    }

    /// Type of the question ('single', 'multiple' etc.,)
    #[getter]
    // `type` is a preserved rust keyword, hence name of this function NEEDS TO BE `get_type`
    fn get_type(&self) -> Option<String> {
        let qtype = self.question.question_type();
        match qtype.clone() {
            QuestionType::None => None,
            _ => Some(qtype.to_string()),
        }
    }

    /// Datamap of the current question
    ///
    /// Note:
    ///
    ///     *Modifying the datamp would also modify the datamap in the survey that the
    ///     current instance being a part of.*
    #[getter(datamap)]
    fn py_get_datamap(&self) -> Question {
        self.question.clone()
    }

    /// Get a copy of question data as a Python tuple.
    #[getter(data)]
    fn py_get_data<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let series = self.to_series().map_err(PyPolarsErr::Polars)?;
        let dtype = self.question.polars_dtype();
        match dtype {
            DataType::Null => PyTuple::new(py, (0..series.len()).map(|_| py.None())),
            DataType::String => {
                PyTuple::new(py, series.str().map_err(PyPolarsErr::Polars)?.iter())
            }
            DataType::UInt32 => {
                PyTuple::new(py, series.u32().map_err(PyPolarsErr::Polars)?.iter())
            }
            DataType::Float64 => {
                PyTuple::new(py, series.f64().map_err(PyPolarsErr::Polars)?.iter())
            }
            DataType::Date => {
                let iterator = series
                    .date()
                    .map_err(PyPolarsErr::Polars)?
                    .physical()
                    .iter()
                    .map(|days| days.map(convert_date).transpose())
                    .collect::<PyResult<Vec<Option<NaiveDate>>>>()?;
                PyTuple::new(py, iterator)
            }
            DataType::Time => {
                let iterator = series
                    .time()
                    .map_err(PyPolarsErr::Polars)?
                    .physical()
                    .iter()
                    .map(|days| days.map(convert_time).transpose())
                    .collect::<PyResult<Vec<Option<NaiveTime>>>>()?;
                PyTuple::new(py, iterator)
            }
            DataType::List(inner) if *inner == DataType::UInt32 => {
                let values = series
                    .iter()
                    .map(|s| anyvalue_to_python(py, s))
                    .collect::<PyResult<Vec<_>>>()?;
                PyTuple::new(py, values)
            }
            _ => Err(
                InternalError::new(format!("Undefined question dtype: '{:?}'", dtype)).into(),
            ),
        }
    }

    /// Get the lookup table for items in the `Question.response.values`, asdictionary of { value: label }
    ///
    /// Note that this methods returns a *deepcopy* of the data, and modifying items returned by this
    /// method will not mutate the caller.
    #[getter(response_label)]
    fn py_get_response_label<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.question.response().py_get_response_label(py)
    }

    /// Default DataFrame library used when returning results to Python.
    /// Supported values are `"pandas"` and `"polars"`, default `"polars"`
    ///
    /// Note:
    ///
    ///     All DataFrame operations are performed internally using Rust Polars,
    ///     regardless of the selected output format.
    ///
    ///     When `"pandas"` is selected, the resulting Polars DataFrame is converted
    ///     to pandas using `to_pandas()`. This may take a bit longer but shouldn't be too much, since majorities of the
    ///     computation-heavy stuff are already handled on the Rust side.
    #[getter(dataframe_format)]
    fn py_get_dataframe_format(&self) -> String {
        self.config().py_dataframe_format.to_string()
    }

    /// Controls whether choice responses are represented by their values or labels.
    #[getter(response_display)]
    fn py_get_response_display(&self) -> String {
        self.config().response_display.to_string()
    }

    fn __repr__(&self) -> PyResult<String> {
        let mut content = Vec::new();

        // Name of the question
        content.push(("name", format!("{:?}", self.name())));

        // response type
        match self.get_type() {
            Some(value) => content.push(("type", format!("{:?}", value))),
            None => content.push(("type", "None".into())),
        }

        // question text
        content.push(("title", self.py_get_label().__repr__()));

        // No. rows
        const MAX_ROWS: usize = 5;
        let series = self.to_series().map_err(PyPolarsErr::Polars)?;
        let mut sample = series
            .iter()
            .take(MAX_ROWS)
            .map(|v| match v {
                AnyValue::Null => "None".to_string(),
                AnyValue::String(s) => format!("{s:?}"),
                AnyValue::StringOwned(s) => format!("{s:?}"),
                AnyValue::Boolean(b) => (if b { "True" } else { "False" }).to_string(),
                _ => v.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ");
        if series.len() > MAX_ROWS {
            sample.push_str(", ...");
        }
        let sample = format!("[{}]", sample);

        // final formatting
        let value = content
            .into_iter()
            .chain(std::iter::once(("data", sample)))
            .map(|(name, value)| format!("{}: {}", name, value))
            .collect::<Vec<String>>()
            .join("\n  ");
        Ok(format!("QuestionData({{\n  {}\n}})", value))
    }

    fn __getitem__<'py>(
        &'py self,
        py: Python<'py>,
        index: isize,
    ) -> PyResult<Bound<'py, PyAny>> {
        let col = self.to_column().map_err(PyPolarsErr::Polars)?;
        let total = col.len();
        let index = normalize_py_index(index, total).unwrap_or(index as usize);
        let value = col.get(index).map_err(PyPolarsErr::Polars)?;
        anyvalue_to_python(py, value)
    }

    fn __iter__(&self) -> PyResult<ColumnIterator> {
        let column = self.to_column().map_err(PyPolarsErr::Polars)?;
        Ok(ColumnIterator { column, index: 0 })
    }

    /// Count unique values of this question data.
    ///
    /// This function basically calls ``poalrs.Series.value_counts`` under the hood, apart from multi-choice
    /// where the choice for each respondant is counted individually.
    ///
    /// Keyword Args:
    ///     count_nan (bool, *optional*): Toogle on/ off for counting Null values.
    ///         Note: multi-choice entries of an empty list (`[]`) will also count as `null`.
    ///         Default `False`
    ///     format ('polars' | 'pandas' |DataFrameFormat, *optional*): Select what kind of dataframe to output:
    ///         'polars' | DataFrameFormat.Polars: polars.DataFrame
    ///         'pandas' | DataFrameFormat.Pandas: pandas.DataFrame
    ///
    ///         Default:
    ///             consistent with the survey current instance created from.
    ///
    /// Returns:
    ///     polars.DataFrame | pandas.DataFrame: Value counts with response labels.
    ///
    /// Note:
    ///     The current implementation excludes null values when count_nan is True.
    #[pyo3(
        name = "value_counts", 
        signature=(
            count_nan:"bool" = None,
            format: "'polars'|'pandas'|DataFrameFormat" = None
        )
    )]
    fn py_value_counts<'py>(
        &self,
        py: Python<'py>,
        count_nan: Option<bool>,
        format: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let count_nan = count_nan.unwrap_or(false);
        let result = self.value_counts(count_nan).map_err(PyPolarsErr::Polars)?;

        // prep output
        let format = format
            .map(|value| DataFrameFormat::from_py_any(value))
            .transpose()?
            .unwrap_or(self.config.read().py_dataframe_format.clone());
        match format {
            DataFrameFormat::Polars => PyDataFrame(result).into_bound_py_any(py),
            DataFrameFormat::Pandas => polars_to_pandas(result, py),
        }
    }

    /// Return the answers as a polars Series.
    ///
    /// Returns:
    ///     polars.Series: The respondent data in Polars format.
    #[pyo3(name = "to_polars")]
    fn py_to_polars(&self) -> PyResult<pyo3_polars::PySeries> {
        self.to_series()
            .map(pyo3_polars::PySeries)
            .map_err(|err| PyPolarsErr::Polars(err).into())
    }

    /// Export the answers as a pandas Series object.
    ///
    /// Returns:
    ///     pandas.Series: The respondent data in pandas format.
    #[pyo3(name = "to_pandas")]
    pub fn py_to_pandas<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame = self.py_to_polars()?.into_bound_py_any(py)?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("use_pyarrow_extension_array", true)?;
        frame.call_method("to_pandas", (), Some(&kwargs))
    }

    /// Export the answers as a list.
    ///
    /// Returns:
    ///     list: The response values in row order.
    #[pyo3(name = "to_list")]
    pub fn py_to_list<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame = self.py_to_polars()?.into_bound_py_any(py)?;
        frame.call_method("to_list", (), None)
    }
}

// ---------- Helpers -----------
/// Iterates through response data of a question
#[cfg(feature = "pyo3")]
#[pyclass]
pub(crate) struct ColumnIterator {
    column: Column,
    index: usize,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl ColumnIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__<'py>(&mut self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        if self.index >= self.column.len() {
            return Ok(None);
        }

        // __next__ checks the index against the column length immediately above.
        let value = unsafe { self.column.get_unchecked(self.index) };

        self.index += 1;
        anyvalue_to_python(py, value).map(Some) // bound already checked
    }
}

#[cfg(test)]
mod test {
    use crate::{Survey, read_triple_s};

    const PATH_DATAMAP_V2_0: &str = "tests/sss-examples/sss-2.0/example1.sss";
    const PATH_DATAFILE_V2_0: &str = "tests/sss-examples/sss-2.0/example1.asc";

    fn get_test_survey() -> Survey {
        read_triple_s(PATH_DATAMAP_V2_0, Some(PATH_DATAFILE_V2_0)).unwrap()
    }

    #[test]
    fn test_is_same_survey() {
        let survey = get_test_survey();
        let questions = survey.question_names();
        let q1 = survey.get_question_data(&questions[0]).unwrap();
        let q2 = survey.get_question_data(&questions[1]).unwrap();
        assert!(q1.is_same_survey(&q2));
        assert!(q2.is_same_survey(&q1));

        assert!(q1.is_same_survey(&q1));
        assert!(q2.is_same_survey(&q2));
    }
}
