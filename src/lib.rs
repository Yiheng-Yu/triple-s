mod error;
mod survey;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use pyo3::{exceptions::PyOSError, prelude::*};
pub use survey::{
    Choice,
    ChoiceProxy,
    Location,
    Meta,
    Question,
    QuestionProxy,
    Record,
    Response,
    ResponseProxy,
    SSSType,
    Spread,
    Survey,
};

#[pyfunction]
pub fn read_sss(sss_path: &str) -> PyResult<Survey> {
    // TODO: Read non-utf8 files!!
    let input = File::open(sss_path)
        .map_err(error::IoError::from)
        .map_err(|error| PyOSError::new_err(error.to_string()))?;

    let mut lines = BufReader::new(input).lines();
    let first_line = lines
        .next()
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "file seems to be empty.",
        ))
        .map_err(|error| PyOSError::new_err(error.to_string()))?
        .map_err(error::IoError::from)
        .map_err(|error| PyOSError::new_err(error.to_string()))?;

    match SSSType::parse(&first_line).map_err(|error| PyOSError::new_err(error.to_string()))? {
        SSSType::Classic => survey::read_sss_classic(lines)
            .map_err(|error| PyOSError::new_err(error.to_string())),
        _ => todo!(),
    }
}

/// A Python module implemented in Rust.
#[pymodule]
mod _triple_s {
    use super::*;
    #[pymodule_export]
    use super::{
        Choice,
        ChoiceProxy,
        Location,
        Meta,
        Question,
        QuestionProxy,
        Record,
        Response,
        ResponseProxy,
        Spread,
        Survey,
    };

    #[pyfunction]
    pub fn read_sss(sss_path: &str) -> PyResult<Survey> {
        super::read_sss(sss_path)
    }
}
