mod survey;
use pyo3::prelude::*;


#[pymodule(gil_used = false)]
pub mod triple_s {
    use super::*;

    #[pymodule_export]
    pub use super::survey::PySurvey;

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}