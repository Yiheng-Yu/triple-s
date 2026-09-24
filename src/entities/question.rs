#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    entities::{CodeFormat, Usage, Text},
};

#[cfg_attr(feature="pyo3", pyclass(module="triple_s", name="question", skip_from_py_object))]
#[derive(Debug)]
pub struct Question {
    pub index: u64,
    pub name: String,
    pub label: Text,
    pub filter: Option<String>,
    pub format: CodeFormat,
    pub usage: Usage,
    // pub location: Position,
    // pub response: Response,
}