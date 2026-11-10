#[cfg(feature = "pyo3")]
use std::{collections::HashMap, sync::Arc};

use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use polars::{frame::row::Row, prelude::*};
#[cfg(feature = "pyo3")]
use pyo3::{
    IntoPyObjectExt,
    exceptions,
    prelude::*,
    types::{PyDict, PyList},
};
#[cfg(feature = "pyo3")]
use pyo3_polars::{PyDataFrame, error::PyPolarsErr};
use rayon::prelude::*;

#[cfg(feature = "pyo3")]
use crate::{
    SurveyInfo,
    config::{DataFrameFormat, ResponseDisplay},
    data::DataEntryIterator,
    prelude::*,
    utils::{anyvalue_to_python, get_type_name, polars_to_pandas},
};
use crate::{
    data::{QuestionData, SurveyData, config::Config},
    datamap::Datamap,
    utils::{ArcLock, lock_it_up},
};

#[cfg_attr(any(not(feature = "pyo3"), doc), doc = include_str!("./doc/survey_rs.md"))]
#[cfg_attr(all(feature = "pyo3", not(doc)), doc = include_str!("./doc/survey_py.md"))]
#[cfg_attr(
    all(feature = "pyo3", not(doc)),
    doc = r###"
Args:
    datamap (Datamap): Survey definition used to interpret the respondent data.

Keyword Args:
    data (SurveyData | None): Respondent data. None creates a survey without loaded data. Defaults to None.

Returns:
    Survey: The constructed Survey instance.
"###
)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Survey", skip_from_py_object)
)]
#[derive(Debug, Clone)]
pub struct Survey {
    datamap: Datamap,
    data: ArcLock<SurveyData>,
    config: ArcLock<Config>,
}

/// Fields
///
/// ---
impl Survey {
    /// Title of survey
    pub fn title(&self) -> Option<String> {
        self.datamap().title()
    }

    /// Sets survey title
    #[inline]
    pub fn set_title(&mut self, title: Option<String>) {
        *self.datamap_mut().info_mut().title_mut() = title;
    }

    /// Name of survey
    pub fn name(&self) -> Option<String> {
        self.datamap().name()
    }

    /// Sets name of a survey
    #[inline]
    pub fn set_name(&mut self, name: Option<String>) {
        *self.datamap_mut().info_mut().name_mut() = name;
    }

    /// triple-s datamap
    pub fn datamap(&self) -> &Datamap {
        &self.datamap
    }

    /// A read-write view of triple-s datamap
    pub fn datamap_mut(&mut self) -> &mut Datamap {
        &mut self.datamap
    }

    /// Read access to shared survey data.
    pub fn data(&self) -> RwLockReadGuard<'_, SurveyData> {
        self.data.read()
    }

    /// Write access to respondent data. Replacing it updates existing question views.
    pub fn data_mut(&self) -> RwLockWriteGuard<'_, SurveyData> {
        self.data.write()
    }

    /// List of question names in the current instance
    pub fn question_names(&self) -> Vec<String> {
        self.datamap().question_names()
    }

    /// Configuration options
    pub fn config(&self) -> RwLockReadGuard<'_, Config> {
        self.config.read()
    }

    /// Mutable view of survey configuration options
    pub fn config_mut(&mut self) -> RwLockWriteGuard<'_, Config> {
        self.config.write()
    }

    /// Creates a dataframe collating response labels for all questions in the survey.
    /// Very handy for look up some values.
    ///
    /// This method will return dataframe with the following columns:
    ///
    /// |column|dtype|
    /// |--|--|
    /// |`question_name`|String|
    /// |`response`|String|
    /// |`label`|String|
    ///
    /// Note:
    ///
    /// Polars only supports dataframe with *single data type*, hence all response values needs to
    /// be converted to string beforehand. You can convert them back to the correct type later.
    ///
    /// An example output may look like:
    ///
    /// ```bash
    /// shape: (26, 3)
    /// ┌───────────────┬──────────┬─────────────────────────────────┐
    /// │ question_name ┆ response ┆ label                           │
    /// │ ---           ┆ ---      ┆ ---                             │
    /// │ str           ┆ str      ┆ str                             │
    /// ╞═══════════════╪══════════╪═════════════════════════════════╡
    /// │ Q2            ┆ 0        ┆ No, this is the first visit     │
    /// │ Q2            ┆ 1        ┆ I have visited before within t… │
    /// │ Q2            ┆ 2        ┆ I visited before that           │
    /// │ Q3            ┆ 1        ┆ Sherwood � Forest               │
    /// │ Q3            ┆ 2        ┆ Nottingham � Castle             │
    /// │ …             ┆ …        ┆ …                               │
    /// │ Q6            ┆ 500.0    ┆ 500 or more                     │
    /// │ Q6            ┆ 999.0    ┆ Not stated                      │
    /// │ Q8            ┆ "A"      ┆ Within 3 months                 │
    /// │ Q8            ┆ "B"      ┆ Between 3 months and 1 year     │
    /// │ Q8            ┆ "C"      ┆ More than 1 years time          │
    /// └───────────────┴──────────┴─────────────────────────────────┘
    /// ```
    pub fn response_labels(&self) -> PolarsResult<DataFrame> {
        let responses = self
            .datamap()
            .record()
            .content()
            .clone()
            .into_par_iter()
            .flat_map_iter(|question| {
                let question = question.read();
                let name = question.name().to_owned();
                question.response_label_iterator().map(move |(key, value)| {
                    Row::new(vec![
                        AnyValue::StringOwned(name.clone().into()),
                        AnyValue::StringOwned(key.to_string().into()),
                        AnyValue::StringOwned(value.value().into()),
                    ])
                })
            })
            .collect::<Vec<Row<'_>>>();

        let schema = Schema::from_iter([
            Field::new("question_name".into(), DataType::String),
            Field::new("response".into(), DataType::String),
            Field::new("label".into(), DataType::String),
        ]);

        DataFrame::from_rows_and_schema(&responses, &schema)
    }
}

impl Survey {
    /// Create new `Survey` instance
    pub fn new(datamap: Datamap, data: SurveyData) -> Self {
        Self {
            datamap,
            data: lock_it_up(data),
            config: lock_it_up(Config::default()),
        }
    }

    /// Creates a new empty `Survey` instance
    pub fn empty() -> Self {
        let datamap = Datamap::empty();
        let data = SurveyData::empty();
        Self::new(datamap, data)
    }

    /// List out number of responses loaded
    ///
    /// This function may fail during polars' [`LazyFrame::collect()`] function call
    pub fn num_response(&self) -> PolarsResult<usize> {
        if self.has_data() {
            self.data().num_response()
        } else {
            Ok(0)
        }
    }

    /// Check if data loaded or not
    pub fn has_data(&self) -> bool {
        self.data().has_data()
    }

    /// Gets a shared [`QuestionData`] view, or `None` if the question name doesn't exist.
    pub fn get_question_data(&self, name: &str) -> Option<QuestionData> {
        let question = self.datamap.get(name)?.clone();
        Some(QuestionData::new(
            Arc::clone(&self.data),
            question,
            Arc::clone(&self.config),
        ))
    }

    fn to_labelled_dataframe(&self) -> PolarsResult<DataFrame> {
        // convert response value to response label
        let df = self.data().lazy()?.clone().collect()?;
        let data_label = self.response_labels()?.lazy();

        let schema = df.schema().clone();
        let query = schema
            .iter()
            .map(|(colname, dtype)| {
                let colname = colname.clone();
                let (dtype, is_multi) = match dtype {
                    DataType::List(inner) if inner.as_ref() == &DataType::UInt32 => {
                        (DataType::UInt32, true)
                    }
                    _ => (dtype.clone(), false),
                };
                let resposne_mapping = data_label
                    .clone()
                    .filter(col("question_name").eq(lit(colname.to_string())))
                    .with_column(col("response").cast(dtype))
                    .collect()?;

                let old = resposne_mapping.column("response")?.implode()?.into_series();

                // return as-is if no mathcing was found
                if old.is_empty() {
                    return Ok(col(colname));
                }

                let new = resposne_mapping.column("label")?.implode()?.into_series();
                if new.is_empty() {
                    return Ok(col(colname));
                }

                if !is_multi {
                    Ok(col(colname.clone()).replace_strict(
                        lit(old),
                        lit(new),
                        Some(col(colname)),
                        Some(DataTypeExpr::Literal(DataType::String)),
                    ))
                } else {
                    Ok(col(colname.clone()).list().eval(element().replace_strict(
                        lit(old),
                        lit(new),
                        Some(element()),
                        Some(DataTypeExpr::Literal(DataType::String)),
                    )))
                }
            })
            .collect::<PolarsResult<Vec<Expr>>>()?;

        df.lazy().select(query).collect()
    }

    #[cfg(feature="pyo3")]
    #[inline]
    fn create_response_iterator(&self) -> PyResult<ResponseIterator> {
        let data_iterator = self.data().create_data_entry_iterator()?;
        match self.config().response_display {
            ResponseDisplay::Value => {
                Ok(ResponseIterator {
                    data_iterator: data_iterator,
                    show_label: false,
                    label_lookup: HashMap::new(),
                })
            }

            ResponseDisplay::Label => {
                let label_lookup = self
                    .datamap()
                    .record()
                    .content()
                    .clone()
                    .into_par_iter()
                    .map(|question| {
                        let question = question.read();
                        let name = question.name();
                        let question_lookup = question
                            .response_label_iterator()
                            .into_iter()
                            .map(|(key, value)| (key, PlSmallStr::from_string(value.value())))
                            .collect::<HashMap<_, _>>();
                        (name, question_lookup)
                    })
                    .collect::<HashMap<String, _>>();
                Ok(ResponseIterator {
                    data_iterator: data_iterator,
                    show_label: true,
                    label_lookup: label_lookup,
                })
            }
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Survey {
    /// Creates new `Survey` instance
    ///
    /// Args:
    ///     datamap (Datamap): Survey definition used to interpret the respondent data.
    ///
    /// Keyword Args:
    ///     data (SurveyData | None): Respondent data. None creates a survey without loaded data. Defaults to None.
    ///
    /// Returns:
    ///     Survey: The constructed Survey instance.
    #[pyo3(signature=(datamap: "Datamap", data: "SurveyData"=None))]
    #[new]
    fn py_new(datamap: Datamap, data: Option<SurveyData>) -> Self {
        let data = data.unwrap_or_default();
        Self::new(datamap, data)
    }

    fn __repr__(&self) -> PyResult<String> {
        let mut content = Vec::new();

        // name of the survey
        let name = self.py_get_name().unwrap_or("None".to_string());
        content.push(("name", format!("{:?}", name)));

        // title of the survey
        let title = self.py_get_title().unwrap_or("None".to_string());
        content.push(("title", format!("{:?}", title)));

        // list of survey questions
        let data = self.data();
        let mut question_repr = if data.has_data() {
            data.py_repr_content().map_err(PyPolarsErr::Polars)?
        } else {
            let mut q_repr = self.datamap().py_question_repr();
            q_repr.push(("num_response", "0".to_string()));
            q_repr
        };
        content.append(&mut question_repr);

        // Config: dataframe library
        let dataframe_format = self.config().py_dataframe_format.to_string();
        content.push(("dataframe_format", format!("{:?}", dataframe_format)));

        // Config: return label or value
        let response_display = self.config().response_display.to_string();
        content.push(("response_display", format!("{:?}", response_display)));

        // formatting
        let value = content
            .into_iter()
            .map(|(fieldname, value)| format!("{}: {}", fieldname, value))
            .collect::<Vec<String>>()
            .join("\n  ");
        Ok(format!("Survey({{\n  {}\n}})", value))
    }

    fn __getitem__<'py>(&self, index: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let py = index.py();

        // Read individual row
        if let Ok(index) = index.extract::<i64>() {

            let df = self
                .data()
                .lazy()?
                .clone()
                .slice(index, 1)
                .collect()
                .map_err(PyPolarsErr::Polars)?;
            let row = df.get_row(0).map_err(PyPolarsErr::Polars)?;

            let dict = PyDict::new(py);
            for (name, value) in df.get_column_names().iter().zip(row.0) {
                let pyvalue = anyvalue_to_python(py, value)?;
                dict.set_item(name.as_str(), pyvalue)?;
            }
            Ok(dict.into_any())
        
        // get data for specific question
        } else if let Ok(question_name) = index.extract::<&str>() {
            let view = self
                .get_question_data(question_name)
                .ok_or_else(|| exceptions::PyKeyError::new_err(question_name.to_owned()))?;
            Ok(Py::new(py, view)?.into_bound(py).into_any())

        // Error
        } else {
            let type_name = get_type_name(&index)?;
            Err(exceptions::PyTypeError::new_err(type_name))
        }
    }

    /// Iterate each response row by row, same as `iter_rows()`
    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<ResponseIterator>> {
        let response_iterator = self.create_response_iterator()?;
        Py::new(py, response_iterator)
    }

    /// Iter each individual respondant data
    ///
    /// Returns:
    ///     Iterator[dict[str, object]]: An iterator over respondent rows.
    fn iter_rows(&self, py: Python<'_>) -> PyResult<Py<DataEntryIterator>> {
        self.data().__iter__(py)
    }

    /// Iterate over the survey questions and their response data.
    ///
    /// Returns:
    ///     Iterator[QuestionData]: An iterator over question data in survey order.
    fn iter_questions(&self, py: Python<'_>) -> PyResult<Py<QuestionDataIterator>> {
        let pointer = Arc::new(self.clone()); // clones a shallow copy
        let iterator = QuestionDataIterator {
            survey: pointer,
            index: 0,
        };

        Py::new(py, iterator)
    }

    /// (optional) Name of the survey
    #[getter(name)]
    fn py_get_name(&self) -> Option<String> {
        self.datamap().info().name().clone()
    }

    #[setter(name)]
    fn py_set_name(&mut self, name: Option<String>) -> PyResult<()> {
        self.set_name(name);
        Ok(())
    }

    /// (optional) Title of the survey
    #[getter(title)]
    fn py_get_title(&self) -> Option<String> {
        self.datamap().info().title().clone()
    }

    #[setter(title)]
    fn py_set_title(&mut self, title: Option<String>) -> PyResult<()> {
        self.set_title(title);
        Ok(())
    }

    /// Datamap
    #[getter(datamap)]
    fn py_get_datamap(&self) -> Datamap {
        self.datamap.clone()
    }

    /// Survey data
    #[getter(data)]
    fn py_get_data(&self) -> SurveyData {
        self.data().clone()
    }

    /// Replace respondent data while keeping existing question views attached.
    #[setter(data)]
    fn py_set_data(&self, data: SurveyData) {
        *self.data_mut() = data;
    }

    /// Survey metadata, shared with `datamap.info`.
    #[getter(info)]
    fn py_get_info(&self) -> SurveyInfo {
        self.datamap.info().clone()
    }

    /// Total number of responses
    #[getter(num_response)]
    fn py_get_num_response(&self) -> PyResult<usize> {
        self.num_response().map_err(|err| PyPolarsErr::Polars(err).into())
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

    #[setter(dataframe_format)]
    fn py_set_dataframe_format(&mut self, value: Bound<'_, PyAny>) -> PyResult<()> {
        self.config_mut().py_dataframe_format = DataFrameFormat::from_py_any(value)?;
        Ok(())
    }

    /// Controls whether choice responses are represented by their values or labels; can be 'value' or 'label'.
    ///
    /// Default 'value'
    #[getter(response_display)]
    fn py_get_response_display(&self) -> String {
        self.config().response_display.to_string()
    }

    #[setter(response_display)]
    fn py_set_response_display(&mut self, value: Bound<'_, PyAny>) -> PyResult<()> {
        self.config_mut().response_display = ResponseDisplay::from_py_any(value)?;
        Ok(())
    }

    /// Creates a dataframe collating response labels for all questions in the survey.
    /// Very handy for look up some values.
    ///
    /// This method will return dataframe with the following columns:
    ///
    /// =============== ===============
    /// column          dtype
    /// =============== ===============
    /// `question_name` String
    /// `response`      String
    /// `label`         String
    /// =============== ===============
    ///
    /// Note:
    ///
    ///     Polars only supports dataframe with *single data type*, hence all response values needs to
    ///     be converted to string beforehand. You can convert them back to the correct type later.
    ///
    /// .. code:: bash
    ///
    /// shape: (26, 3)
    /// ┌───────────────┬──────────┬─────────────────────────────────┐
    /// │ question_name ┆ response ┆ label                           │
    /// │ ---           ┆ ---      ┆ ---                             │
    /// │ str           ┆ str      ┆ str                             │
    /// ╞═══════════════╪══════════╪═════════════════════════════════╡
    /// │ Q2            ┆ 0        ┆ No, this is the first visit     │
    /// │ Q2            ┆ 1        ┆ I have visited before within t… │
    /// │ Q2            ┆ 2        ┆ I visited before that           │
    /// │ Q3            ┆ 1        ┆ Sherwood � Forest               │
    /// │ Q3            ┆ 2        ┆ Nottingham � Castle             │
    /// │ …             ┆ …        ┆ …                               │
    /// │ Q6            ┆ 500.0    ┆ 500 or more                     │
    /// │ Q6            ┆ 999.0    ┆ Not stated                      │
    /// │ Q8            ┆ "A"      ┆ Within 3 months                 │
    /// │ Q8            ┆ "B"      ┆ Between 3 months and 1 year     │
    /// │ Q8            ┆ "C"      ┆ More than 1 years time          │
    /// └───────────────┴──────────┴─────────────────────────────────┘
    ///
    /// Keyword Args:
    ///     format ('polars' | 'pandas' |DataFrameFormat, *optional*): Select what kind of dataframe to output:
    ///         'polars' | DataFrameFormat.Polars: polars.DataFrame
    ///         'pandas' | DataFrameFormat.Pandas: pandas.DataFrame
    ///
    ///         Default:
    ///             consistent with the survey.dataframe_format
    /// Returns:
    ///     polars.DataFrame | pandas.DataFrame: Value counts with response labels.
    fn py_get_response_labels<'py>(
        &self,
        py: Python<'py>,
        format: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let polars_frame = self.response_labels().map_err(|err| PyPolarsErr::Polars(err))?;

        let format = format
            .map(|value| DataFrameFormat::from_py_any(value))
            .transpose()?
            .unwrap_or(self.config.read().py_dataframe_format.clone());

        match format {
            DataFrameFormat::Polars => PyDataFrame(polars_frame).into_bound_py_any(py),
            DataFrameFormat::Pandas => polars_to_pandas(polars_frame, py),
        }
    }

    /// Check if data loaded or not
    ///
    /// Returns:
    ///     bool: Whether respondent data is loaded.
    #[pyo3(name = "has_data")]
    fn py_has_data(&self) -> bool {
        self.has_data()
    }
}

// ---------- Helpers -----------
/// Iterates each ``QuestionData``` in a survey
#[cfg(feature = "pyo3")]
#[pyclass]
pub(crate) struct QuestionDataIterator {
    survey: Arc<Survey>,
    index: usize,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl QuestionDataIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<QuestionData>> {
        let questions = self.survey.question_names();
        if self.index >= questions.len() {
            return Ok(None);
        };

        let target = questions
            .get(self.index)
            .ok_or(exceptions::PyIndexError::new_err(self.index))? // unlikely,,
            .clone();

        let instance = self
            .survey
            .get_question_data(&target)
            .ok_or(exceptions::PyKeyError::new_err(target))?; // unlikely,,

        self.index += 1;
        Ok(Some(instance))
    }
}

// ---------- Helpers -----------
/// Iterates through individual responses throughout the survey
#[cfg(feature = "pyo3")]
#[pyclass]
struct ResponseIterator {
    data_iterator: DataEntryIterator,
    show_label: bool,
    /// {question_name: {respons_code: response_label}}
    label_lookup: HashMap<String, HashMap<AnyValue<'static>, PlSmallStr>>,
}

#[cfg(feature = "pyo3")]
impl ResponseIterator {
    #[inline]
    fn index(&self) -> usize {
        self.data_iterator.index
    }

    #[inline]
    fn advance(&mut self) {
        self.data_iterator.index += 1;
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl ResponseIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__<'py>(&mut self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        if !self.show_label {
            return self.data_iterator.__next__(py);
        };

        let current_index = self.index();
        if current_index >= self.data_iterator.df.height() {
            return Ok(None);
        };

        let dict = PyDict::new(py);
        for row_result in self.data_iterator.get_row(current_index) {
            let (colname, value) = row_result.map_err(PyPolarsErr::Polars)?;
            match &value {
                AnyValue::List(inner) if inner.dtype() == &DataType::UInt32 => {
                    if let Some(question_label) = self.label_lookup.get(colname) {
                        let inner_data = inner
                            .iter()
                            .map(|entry| {
                                let labelled = question_label
                                    .get(&entry)
                                    .map(|label| AnyValue::StringOwned(label.clone()))
                                    .unwrap_or(entry);
                                anyvalue_to_python(py, labelled)
                            })
                            .collect::<PyResult<Vec<Bound<'py, PyAny>>>>()?;
                        let pyvalue = PyList::new(py, inner_data)?;
                        dict.set_item(colname, pyvalue.into_any())?;
                    } else {
                        let pyvalue = anyvalue_to_python(py, value)?;
                        dict.set_item(colname, pyvalue.into_any())?;
                    }
                }
                _ => {
                    let value = self
                        .label_lookup
                        .get(colname)
                        .map(|lookup| lookup.get(&value))
                        .flatten()
                        .map(|item| AnyValue::StringOwned(item.clone()))
                        .unwrap_or(value);
                    let pyvalue = anyvalue_to_python(py, value)?;
                    dict.set_item(colname.to_string(), pyvalue.into_any())?;
                }
            }
        }

        self.advance();
        Ok(Some(dict.into_any()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::read_triple_s;

    const PATH_DATAMAP_V2_0: &str = "tests/sss-examples/sss-2.0/example1.sss";
    const PATH_DATAFILE_V2_0: &str = "tests/sss-examples/sss-2.0/example1.asc";

    fn get_survey_struct() -> Survey {
        read_triple_s(PATH_DATAMAP_V2_0, Some(PATH_DATAFILE_V2_0)).unwrap()
    }

    #[test]
    fn test_response_label_dataframe() {
        let survey = get_survey_struct();
        let _ = survey.response_labels().unwrap();
    }

    #[test]
    fn test_export_to_labelled_polars_dataframe() {
        let survey = get_survey_struct();
        let _ = survey.to_labelled_dataframe().unwrap();
    }
}
