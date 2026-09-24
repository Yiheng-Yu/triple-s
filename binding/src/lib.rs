mod survey;
mod utils;
mod repr;

use pyo3::prelude::*;

#[pymodule(gil_used = false)]
pub mod triple_s {
    use super::*;

    #[pymodule_export]
    pub use super::utils::RustBackendError;

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
        PyFormat,
        VariantList,
        read_file
    };

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}