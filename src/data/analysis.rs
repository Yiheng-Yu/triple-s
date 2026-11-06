//! Analyse survey data

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;


/// Represents one topline table
///
/// # Fields:
///
/// side: name of question to be used as the rows of the table
/// top: list of question names used as columns of the table
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Clone)]
pub struct ToplineTable {
    /// name of question to be used as the rows of the table
    pub side: String,
    /// top: list of question names used as columns of the table
    pub top: Vec<String>
}

/// ---
///
/// # Methods
impl ToplineTable {
    /// Create new instance
    pub fn new(side: impl Into<String>, top: impl Iterator<Item = impl Into<String>>) -> Self {
        Self {
            side: side.into(),
            top: top.into_iter().map(|item| item.into()).collect()
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl ToplineTable {
    /// Creates a new `ToplineTable` instance.
    ///
    /// `top` accepts a QuestionData, None, or an iterable
    /// of QuestionData objects.
    ///
    /// Raises an error if questions belong to different surveys.
    // #[pyo3(signature = (
    //     side: "QuestionData",
    //     top: "QuestionData | Iterable[QuestionData] | None" = None
    // ))]
    #[new]
    fn py_new() -> PyResult<Self> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    // use crate::{Survey, read_triple_s};

    // const PATH_DATAMAP_V2_0: &str = "tests/sss-examples/sss-2.0/example1.sss";
    // const PATH_DATAFILE_V2_0: &str = "tests/sss-examples/sss-2.0/example1.asc";

    // fn get_test_survey() -> Survey {
    //     read_triple_s(PATH_DATAMAP_V2_0, Some(PATH_DATAFILE_V2_0)).unwrap()
    // }

    // fn create_topline_table() -> ToplineTable {
    //     let survey = get_test_survey();
    //     let questions = survey.question_names();
    //     let q1 = survey.get_question_data(&questions[0]).unwrap();
    //     let q2 = survey.get_question_data(&questions[1]).unwrap();
    //     ToplineTable::new(q1, vec![q2]).unwrap()
    // }

    // #[test]
    // fn test_create_analysis_success() {
    //     let _ = create_topline_table();
    // }

    // #[test]
    // fn test_create_analysis_fail() {
    //     let s1 = get_test_survey();
    //     let s2 = get_test_survey();

    //     let questions = s1.question_names();
    //     let q1 = s1.get_question_data(&questions[0]).unwrap();
    //     let q2 = s2.get_question_data(&questions[1]).unwrap();
    //     assert!(ToplineTable::new(q1, vec![q2]).is_err());
    // }
}
