mod survey;
// mod configuration;
mod error;

use pyo3::prelude::*;

#[pymodule(gil_used = false)]
pub mod triple_s {
    use super::*;

    #[pymodule_export]
    pub use super::error::RustBackendError;

    #[pymodule_export]
    pub use super::survey::{
        // PyCode,
        PyMeta,
        PyQuestion,
        PyRecord,
        // PyResponse,
        PySurvey,
        PyText,
        PyVariant,
        PyMode,
        PyFormat,
        PyLanguage,
        read_file
    };

    // #[pymodule(gil_used = false)]
    // pub mod config {
    //     #[pymodule_export]
    //     pub use super::configuration::{
    //         PyFormat,
    //         PyMode,
    //     };
    // }

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}