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
        read_sss_file as read_file,
    };
    use super::*;

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn test_create_question_from_rust() {
        todo!("This is indended to create an API for python side to create new instance")
    }

    #[test]
    fn test_create_survey_from_rust() {
        todo!("This is indended to create an API for python side to create new instance")
    }
    
    #[test]
    fn test_create_response_from_rust() {
        todo!("This is indended to create an API for python side to create new instance")
    }

}