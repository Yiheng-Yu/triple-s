#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    data::SurveyData,
    datamap::Datamap,
};

/// Survey
#[cfg_attr(feature = "pyo3", pyclass(module = "triple_s", name = "Survey"))]
#[derive(Debug)]
pub struct Survey {
    datamap: Datamap,
    data: SurveyData,
}

/// Fields
///
/// ---
impl Survey {
    /// triple-s datamap
    pub fn datamap(&self) -> &Datamap {
        &self.datamap
    }

    /// A read-write view of triple-s datamap
    pub fn datamap_mut(&mut self) -> &mut Datamap {
        &mut self.datamap
    }

    /// survey data
    pub fn data(&self) -> &SurveyData {
        &self.data
    }

    /// A read-write view of survey data
    pub fn data_mut(&mut self) -> &mut SurveyData {
        &mut self.data
    }
}

impl Survey {
    /// Create new `Survey` instance
    pub fn new(datamap: Datamap, data: SurveyData) -> Self {
        Self { datamap, data }
    }

    /// Check if data loaded or not
    pub fn has_data(&self) -> bool {
        self.data().has_data()
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Survey {
    /// (optional) Title of the survey
    #[getter(title)]
    fn py_get_title(&self) -> Option<String> {
        self.datamap().info().title().clone()
    }

    #[setter(title)]
    fn py_set_title(&mut self, title: Option<String>) -> PyResult<()> {
        *self.datamap.info_mut().title_mut() = title;
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
        self.data.clone()
    }

    /// Check if data loaded or not
    #[pyo3(name = "has_data")]
    fn py_has_data(&self) -> bool {
        self.has_data()
    }
}
