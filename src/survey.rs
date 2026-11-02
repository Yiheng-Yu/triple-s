use std::sync::Arc;

use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*, types::PyDict};
#[cfg(feature = "pyo3")]
use pyo3_polars::error::PyPolarsErr;

use crate::{
    data::{QuestionData, SurveyData},
    datamap::Datamap,
    utils::{ArcLock, lock_it_up},
};
#[cfg(feature = "pyo3")]
use crate::{
    SurveyInfo,
    utils::{anyvalue_to_python, get_type_name},
};

/// Triple-s survey.
///
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

    /// Gets a shared [`QuestionData`] view, or `None` if the question name doesn't exist.
    /// Existing views follow replacements made through [`Self::data_mut`].
    pub fn get_question_data(&self, name: &str) -> Option<QuestionData> {
        let question = self.datamap.get(name)?.clone();

        Some(QuestionData::new(Arc::clone(&self.data), question))
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

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<crate::data::ResponseIterator>> {
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
