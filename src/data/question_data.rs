use std::collections::HashMap;

#[cfg(feature = "pyo3")]
use chrono::{NaiveDate, NaiveTime};
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::{
    IntoPyObjectExt,
    exceptions,
    prelude::*,
    types::{PyDict, PyTuple},
};
#[cfg(feature = "pyo3")]
use pyo3_polars::{PyDataFrame, PySeries, error::PyPolarsErr};

#[cfg(feature = "pyo3")]
use crate::{
    DataRange,
    error::InternalError,
    options::DataFrameFormat,
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
    data::{operation, survey_data::SurveyData},
    datamap::{Question, text::Text},
    options::{LabelOptions, ResponseDisplay, SurveyOptions},
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
    config: SurveyOptions,
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

    /// Returns the label or original question text.
    ///
    /// The text may include language-specific or mode-specific variants.
    pub fn label(&self) -> &Text {
        self.question.label()
    }

    /// Returns a mutable reference for updating the question text and its
    /// variants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::question::QuestionOptions;
    ///
    /// let mut question = QuestionOptions::new("Q1").finish()?;
    ///
    /// *question.label_mut().value_mut() = "Updated question".to_string();
    /// assert_eq!(question.label().value(), "Updated question");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn label_mut(&mut self) -> &mut Text {
        self.question.label_mut()
    }

    /// Get the lookup table for items in the [`Response::values`](crate::datamap::response::Response) field of of the [`Question`] instance,
    /// formatted as: { [`Code::value()`](crate::datamap::response::Code): [`Code::label()`](crate::datamap::response::Code) }
    ///
    /// This method returns a `Arc<RwLock<String>>` for the code label text, this means mutating
    /// code label elsewhere will the hashmap returned from this function. Details on this mutation
    /// behaviour can be found in the documentation in [`Response::response_label_map`](crate::datamap::response::Response).
    #[allow(clippy::mutable_key_type)] // can only be string, float, or int
    pub fn response_label_map(&self) -> HashMap<AnyValue<'static>, ArcLock<String>> {
        self.question.response_label_map()
    }

    /// Configuration options copied from the survey when this view was created.
    pub fn config(&self) -> &SurveyOptions {
        &self.config
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
        config: SurveyOptions,
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
            ResponseDisplay::Label => self.to_labelled_column(),
        }
    }

    /// Export contents to a polars [`Series`]
    pub fn to_series(&self) -> PolarsResult<Series> {
        self.to_column().map(Column::take_materialized_series)
    }

    /// export to labelled polars column
    pub(crate) fn to_labelled_column(&self) -> PolarsResult<Column> {
        match self.question_type() {
            QuestionType::Single => self.to_labelled_column_single_choice(),
            QuestionType::Multiple => self.to_labelled_column_multi_choice(),
            _ => self.to_value_column(),
        }
    }

    /// Count unique values of this Series and a column "counts" with dtype IdxType
    ///
    /// This function basically calls `value_counts` method of a [`Series`](polars::prelude::Series) under the hood, apart from multi-choice
    /// questions ([`QuestionData::question_type`] is [`QuestionType::Multiple`]),
    /// where the choice for each respondant is counted individually.
    ///
    /// # Arguments
    /// - `name` (`impl Into<PlSmallStr>`, Optional):
    ///     - Name of the column, default `count`
    ///
    /// This function would give you an example output that looks like:
    ///
    /// ```bash
    /// shape: (6, 3)
    /// ┌──────┬───────────────────────────┬───────┐
    /// │ Q3   ┆ label                     ┆ count │
    /// │ ---  ┆ ---                       ┆ ---   │
    /// │ u64  ┆ str                       ┆ u64   │
    /// ╞══════╪═══════════════════════════╪═══════╡
    /// │ null ┆ Total                     ┆ 3     │
    /// │ 1    ┆ Sherwood � Forest         ┆ 2     │
    /// │ 2    ┆ Nottingham � Castle       ┆ 2     │
    /// │ 3    ┆ "Friar � Tuck" Restaurant ┆ 1     │
    /// │ 5    ┆ Mining museum             ┆ 1     │
    /// │ 9    ┆ Other                     ┆ 2     │
    /// └──────┴───────────────────────────┴───────┘
    /// ```
    ///
    /// _The first row is the total count of not-null values_
    #[allow(clippy::mutable_key_type)]
    pub fn value_counts(
        &self,
        normalise: bool,
        options: LabelOptions<'_>,
    ) -> PolarsResult<DataFrame> {
        // calling `value_counts_internal` instead of putting contents of `value_counts_internal` here
        // because `value_counts_internal` gets used elsewhere in parallel iterator later.
        let plan = self.value_counts_internal(normalise, options)?;
        plan.collect()
    }

    /// Builds query plan for running value_counts() to be collected later
    /// This is used for multi-threading, internal only.
    pub(crate) fn value_counts_internal(
        &self,
        normalise: bool,
        options: LabelOptions<'_>,
    ) -> PolarsResult<LazyFrame> {
        let question_name = self.name();

        let lazyframe = self
            .data
            .read()
            .lazy()?
            .clone()
            .select([col(&question_name).cast(self.datamap().polars_dtype())]);

        let result = operation::frequency_counts(
            lazyframe,
            &question_name,
            options.col_count,
            true,
            normalise,
        )?;

        // get response labels
        let match_expression =
            options.match_response_label(&question_name, self.response_label_map())?;
        let result = result.with_column(match_expression);

        // Fianl formatting: rename column names, add quesiton names etc.,
        let columns = options.into_expression(&question_name);

        Ok(result
            .select(columns)
            .with_simplify_expr(true)
            .with_comm_subplan_elim(true))
    }

    /// Checks if `other` is comes from the same survey as `self`.
    pub fn is_same_survey(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    fn get_datalabel_key_value(&self) -> PolarsResult<(Series, Series)> {
        let (keys, values): (Vec<AnyValue<'_>>, Vec<String>) = self
            .response_label_map()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone().read().to_string()))
            .unzip();

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
            None::<DataTypeExpr>,
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
                None::<DataTypeExpr>,
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
                    PolarsError::ComputeError(
                        format!(
                            "Question '{}' contains response values with no matching label, \
                             expect to be one of: {}",
                            question_name,
                            unique_keys.fmt_list()
                        )
                        .into(),
                    )
                } else {
                    err
                }
            }
            _ => err,
        }
    }

    fn to_value_column(&self) -> PolarsResult<Column> {
        self.lazy()?
            .collect()?
            .into_columns()
            .into_iter()
            .next()
            .ok_or(PolarsError::ColumnNotFound(
                format!("Unable to find column '{}' in `SurveyData`", self.name()).into(),
            ))
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

    /// Sets the label from a string or `Text`, or resets it to empty with
    /// `None`. A string updates the existing text; a `Text` object replaces
    /// it.
    #[setter(label)]
    fn py_set_label<'py>(&mut self, label: Option<&Bound<'py, PyAny>>) -> PyResult<()> {
        self.question.py_set_label(label)
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
        let dtype = series.dtype();
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
            DataType::List(inner)
                if matches!(inner.as_ref(), DataType::UInt32 | DataType::String) =>
            {
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

    fn __iter__(&self) -> PyResult<CellIterator> {
        let column = self.to_column().map_err(PyPolarsErr::Polars)?;
        Ok(CellIterator { column, index: 0 })
    }

    /// Count unique values of this question data.
    ///
    /// This calls `polars.Series.value_counts` under the hood. For multiple-choice
    /// questions, each selected response is counted individually. The first row,
    /// labelled `Total`, counts respondents with non-null answers, including empty lists.
    ///
    /// Keyword Arguments:
    ///     
    ///     normalise: (bool, *optional*):
    ///         Return counts as relative frequencies normalised to 1.0.
    ///         Default `False`
    ///
    ///     colname_stats (str, *optional*):
    ///         Give the resulting count column a specific name (same as `polars.Series.value_counts`)
    ///         Default `'total'`
    ///
    ///     colname_label (str, *optional*):
    ///         Column name for the response label column.
    ///         Default `'label'`
    ///
    ///     colname_code (str, *optional*):
    ///         Column name for the response code column.
    ///         Only used when `include_response_codes` is `True`.
    ///         Default `'code'`
    ///
    ///     include_question_name (bool, *optional*):
    ///         Include a `question` column with the question name in the first row
    ///         and nulls in the remaining rows.
    ///         Default `False`
    ///
    ///     include_response_codes (bool, *optional*):
    ///         Include the original response values in a separate column.
    ///         Default `False`
    ///
    ///     output ('polars' | 'pandas' | DataFrameFormat, *optional*): Select what kind of dataframe to output:
    ///         'polars' | DataFrameFormat.Polars: polars.DataFrame
    ///         'pandas' | DataFrameFormat.Pandas: pandas.DataFrame
    ///
    ///         Default:
    ///             consistent with the survey current instance created from.
    ///
    /// Returns:
    ///     polars.DataFrame | pandas.DataFrame: Value counts with response labels.
    #[pyo3(
        name = "value_counts", 
        signature=(
            normalise: "bool" = false,
            colname_stats: "str" = "total",
            colname_label: "str" = "label",
            colname_code: "str" = "code",
            include_response_codes: "bool" = false,
            include_question_name: "bool" = false,
            output: "'polars'|'pandas'|DataFrameFormat" = None
        )
    )]
    fn py_value_counts<'py>(
        &self,
        py: Python<'py>,
        normalise: Option<bool>,
        colname_stats: Option<&str>,
        colname_label: Option<&str>,
        colname_code: Option<&str>,
        include_response_codes: Option<bool>,
        include_question_name: Option<bool>,
        output: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let options = LabelOptions::new()
            .with_include_question_name(include_question_name.unwrap_or(false))
            .with_include_response_codes(include_response_codes.unwrap_or(false))
            // think this would be easier for the user and makes documentation less confusing
            // as the user can just call `ffill()` to fill nan values with question name
            // should they want to.
            .with_fill_question_name_column(false)
            .with_col_count(colname_stats)
            .with_col_label(colname_label)
            .with_col_code(colname_code);

        let normalise = normalise.unwrap_or(false);
        let result = self
            .value_counts(normalise, options)
            .map_err(PyPolarsErr::Polars)?;

        // prep output
        let format = output
            .map(|value| DataFrameFormat::from_py_any(value))
            .transpose()?
            .unwrap_or(self.config.py_dataframe_format.clone());
        match format {
            DataFrameFormat::Polars => PyDataFrame(result).into_bound_py_any(py),
            DataFrameFormat::Pandas => polars_to_pandas(result, py),
        }
    }

    /// Return the answers as a polars Series.
    ///
    /// Keyword Arguments:
    ///     response_label (bool, *optional*):
    ///         Set `True` to display response label text for single and multi-choice questions (`["London", "London", "Scotland", ...]`),
    ///         Set `False` to show values instead (`[1, 1, 2, ...]`).
    ///         Default `False`
    ///
    /// Returns:
    ///     polars.Series: The respondent data in Polars format.
    #[pyo3(name = "to_polars", signature=(response_label: "bool"=None))]
    fn py_to_polars(&self, response_label: Option<bool>) -> PyResult<PySeries> {
        let response_label = response_label.unwrap_or(false);

        if response_label {
            self.to_labelled_column()
                .map(|column| PySeries(column.take_materialized_series()))
                .map_err(|err| PyPolarsErr::Polars(err).into())
        } else {
            self.to_value_column()
                .map(|column| PySeries(column.take_materialized_series()))
                .map_err(|err| PyPolarsErr::Polars(err).into())
        }
    }

    /// Export the answers as a pandas Series object.
    ///
    /// Keyword Arguments:
    ///     response_label (bool, *optional*):
    ///         Set `True` to display response label text for single and multi-choice questions (`["London", "London", "Scotland", ...]`),
    ///         Set `False` to show values instead (`[1, 1, 2, ...]`).
    ///         Default `False`
    ///
    /// Returns:
    ///     pandas.Series: The respondent data in pandas format.
    #[pyo3(name = "to_pandas", signature=(response_label: "bool"=None))]
    pub fn py_to_pandas<'py>(
        &self,
        py: Python<'py>,
        response_label: Option<bool>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let series = self.py_to_polars(response_label)?.into_bound_py_any(py)?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("use_pyarrow_extension_array", true)?;
        series.call_method("to_pandas", (), Some(&kwargs))
    }

    /// Export the answers as a list.
    ///
    /// Keyword Arguments:
    ///     response_label (bool, *optional*):
    ///         Set `True` to display response label text for single and multi-choice questions (`["London", "London", "Scotland", ...]`),
    ///         Set `False` to show values instead (`[1, 1, 2, ...]`).
    ///         Default `False`
    ///
    /// Returns:
    ///     list: The response values in row order.
    #[pyo3(name = "to_list", signature=(response_label: "bool"=None))]
    pub fn py_to_list<'py>(
        &self,
        py: Python<'py>,
        response_label: Option<bool>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let frame = self.py_to_polars(response_label)?.into_bound_py_any(py)?;
        frame.call_method("to_list", (), None)
    }
}

// ---------- Helpers -----------
/// Iterates through response data of a question
#[cfg(feature = "pyo3")]
#[pyclass]
pub(crate) struct CellIterator {
    column: Column,
    index: usize,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl CellIterator {
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
