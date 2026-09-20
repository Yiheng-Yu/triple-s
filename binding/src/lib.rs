mod survey;
use pyo3::prelude::*;

#[pymodule(gil_used = false)]
pub mod triple_s {

    #[pymodule_export]
    pub use super::survey::{
        Kind,
        PyChoice,
        PyCategorical,
        PyCharacter,
        PyDate,
        PyLogical,
        PyLocation,
        PyMeta,
        PyNote,
        PyQuestion,
        PyQuantity,
        PyRecord,
        PyResponse,
        PySingleLiteral,
        PySurvey,
        PyTime,
        // PyProxy,
        read_sss_file,
    };
    use super::*;

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}
