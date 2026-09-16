mod survey;
use pyo3::prelude::*;

#[pymodule(gil_used = false)]
pub mod triple_s {

    #[pymodule_export]
    pub use super::survey::{
        // Kind,
        // PyChoice,
        // PyMeta,
        // PyQuestion,
        // PyRecord,
        // PyResponse,
        PySurvey,
        // PyProxy,
        read_sss,
    };
    use super::*;

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}
