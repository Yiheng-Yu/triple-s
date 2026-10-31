use std::sync::Arc;

use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*, types::PyDict};
#[cfg(feature = "pyo3")]
use pyo3_polars::error::PyPolarsErr;

use crate::{
    Question,
    QuestionType,
    data::SurveyData,
    datamap::Datamap,
    utils::{ArcLock, lock_it_up},
};
#[cfg(feature = "pyo3")]
use crate::{
    SurveyInfo,
    prelude::*,
    utils::{anyvalue_to_python, get_type_name, normalize_py_index},
};

/// Triple-s survey.
///
/// Internally, `Survey` A datamap together with respondent data.
#[cfg_attr(feature = "pyo3", pyclass(module = "triple_s", name = "Survey"))]
#[derive(Debug)]
pub struct Survey {
    datamap: Datamap,
    data: ArcLock<SurveyData>,
}

/// Fields
///
/// ---
impl Survey {
    /// Title of survey
    pub fn title<'a>(&'a self) -> Option<String> {
        self.datamap().title()
    }

    /// Sets survey title
    #[inline]
    pub fn set_title(&mut self, title: Option<String>) {
        *self.datamap_mut().info_mut().title_mut() = title;
    }

    /// Name of survey
    pub fn name<'a>(&'a self) -> Option<String> {
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

    /// Read access to shared survey data. Drop the guard before writing through a view.
    pub fn data(&self) -> RwLockReadGuard<'_, SurveyData> {
        self.data.read()
    }

    /// Write access to respondent data. Replacing it updates existing question views.
    /// Drop the guard before accessing the data through another view.
    pub fn data_mut(&self) -> RwLockWriteGuard<'_, SurveyData> {
        self.data.write()
    }
}

impl Survey {
    /// Create new `Survey` instance
    pub fn new(datamap: Datamap, data: SurveyData) -> Self {
        Self {
            datamap,
            data: lock_it_up(data),
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

    /// Gets data of a question in a survey, returns `None` if question name doesn't exist.
    pub fn get_question_data(&self, name: &str) -> Option<QuestionData> {
        let question = self.datamap.get_question(name)?.clone();

        Some(QuestionData {
            data: Arc::clone(&self.data),
            question,
        })
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Survey {
    /// Creates new `Survey` instance
    #[pyo3(signature=(datamap: "Datamap", data: "SurveyData"=None))]
    #[new]
    fn py_new(datamap: Datamap, data: Option<SurveyData>) -> Self {
        let data = data.unwrap_or_default();
        Self::new(datamap, data)
    }

    fn __repr__(&self) -> PyResult<String> {
        let mut content = Vec::new();

        let name = self.py_get_name().unwrap_or("None".to_string());
        content.push(("name", format!("{:?}", name)));

        let title = self.py_get_title().unwrap_or("None".to_string());
        content.push(("title", format!("{:?}", title)));

        let data = self.data();
        let question_repr = if data.has_data() {
            data.py_repr_content().map_err(PyPolarsErr::Polars)?
        } else {
            let mut q_repr = self.datamap().py_question_repr();
            q_repr.push(("num_response", "0".to_string()));
            q_repr
        };

        let value = content
            .into_iter()
            .chain(question_repr.into_iter())
            .map(|(fieldname, value)| format!("{}: {}", fieldname, value))
            .collect::<Vec<String>>()
            .join("\n  ");
        Ok(format!("Survey({{\n  {}\n}})", value))
    }

    fn __getitem__<'py>(&self, index: Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        let py = index.py();
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
        } else if let Ok(question_name) = index.extract::<&str>() {
            let view = self
                .get_question_data(question_name)
                .ok_or_else(|| exceptions::PyKeyError::new_err(question_name.to_owned()))?;

            Ok(Py::new(py, view)?.into_bound(py).into_any())
        } else {
            let type_name = get_type_name(&index)?;
            Err(exceptions::PyTypeError::new_err(type_name))
        }
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<crate::data::RowIterator>> {
        self.data().__iter__(py)
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

    #[getter(num_response)]
    fn py_get_num_response(&self) -> PyResult<usize> {
        self.num_response().map_err(|err| PyPolarsErr::Polars(err).into())
    }

    /// Check if data loaded or not
    #[pyo3(name = "has_data")]
    fn py_has_data(&self) -> bool {
        self.has_data()
    }
}

/// Survey data for a single question.
///
/// `QuestionData` provides a convenient way to work with an individual question within a `SurveyData` instance.
/// It shares the same underlying data as the `SurveyData` it was created from.
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "QuestionData", skip_from_py_object)
)]
#[derive(Clone)]
pub struct QuestionData {
    data: ArcLock<SurveyData>,
    question: Question,
}

impl QuestionData {
    /// name of the question
    #[inline]
    pub fn name(&self) -> String {
        self.question.name()
    }

    /// Type of the question
    #[inline]
    pub fn question_type(&self) -> RwLockReadGuard<'_, QuestionType> {
        self.question.question_type()
    }

    /// Export contents to [`polars::Column`]
    pub fn to_polars(&self) -> PolarsResult<Column> {
        let data = self.data.read();
        let name = self.name();

        data.lazy()?.clone().select([col(&name)]).collect()?.drop_in_place(&name)
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl QuestionData {
    /// Name of the question
    #[getter(name)]
    fn py_get_name(&self) -> String {
        self.name()
    }

    /// Type of the question ('single', 'multiple' etc.,)
    #[getter]
    // `type` is a preserved keyword in rust
    fn get_type(&self) -> Option<String> {
        let qtype = self.question.question_type();
        match qtype.clone() {
            QuestionType::None => None,
            _ => Some(qtype.to_string()),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        let mut content = Vec::new();

        content.push(("name", format!("{:?}", self.name())));
        match self.get_type() {
            Some(value) => content.push(("type", format!("{:?}", value))),
            None => content.push(("type", "None".into())),
        };

        const MAX_ROWS: usize = 5;
        let col = self.to_polars().map_err(PyPolarsErr::Polars)?;
        let mut sample = col
            .as_materialized_series()
            .iter()
            .take(MAX_ROWS)
            .map(|v| match v {
                AnyValue::Null => "None".to_string(),
                AnyValue::String(s) => format!("{s:?}"),
                AnyValue::StringOwned(s) => format!("{s:?}"),
                AnyValue::Boolean(b) => if b { "True" } else { "False" }.to_string(),
                _ => v.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ");
        if col.len() > MAX_ROWS {
            sample.push_str(", ...");
        }
        let sample = format!("[{}]", sample);

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
        let col = self.to_polars().map_err(PyPolarsErr::Polars)?;

        let total = col.len();
        let index = normalize_py_index(index, total).unwrap_or(index as usize);

        let value = col.get(index).map_err(PyPolarsErr::Polars)?;
        anyvalue_to_python(py, value)
    }

    /// Return the answers as a Polars Series (Python's column type).
    #[pyo3(name = "to_polars")]
    fn py_to_polars(&self) -> PyResult<pyo3_polars::PySeries> {
        self.to_polars()
            .map(|column| pyo3_polars::PySeries(column.take_materialized_series()))
            .map_err(|err| PyPolarsErr::Polars(err).into())
    }
}
