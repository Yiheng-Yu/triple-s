mod survey;
mod configuration;
use pyo3::prelude::*;

#[pymodule(gil_used = false)]
pub mod triple_s {
    use super::*;

    #[pymodule_export]
    pub use super::survey::{
        // PyCode,
        PyMeta,
        // PyQuestion,
        PyRecord,
        // PyResponse,
        PySurvey,
        read_file
    };

    #[pymodule(gil_used = false)]
    pub mod config {
        #[pymodule_export]
        pub use super::configuration::{
            PyFormat,
            PyMode,
        };
    }

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}