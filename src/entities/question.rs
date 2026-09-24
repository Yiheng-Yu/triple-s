#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    entities::{CodeFormat, Text, Usage},
    utils::ArcLock,
};

#[cfg_attr(feature = "pyo3", pyclass(module = "triple_s", name = "question"))]
#[derive(Debug)]
pub struct Question {
    pub index: u64,
    pub name: ArcLock<String>,
    pub label: ArcLock<Text>,
    pub filter: ArcLock<Option<String>>,
    pub format: ArcLock<CodeFormat>,
    pub usage: ArcLock<Usage>,
    // pub location: Position,
    // pub response: Response,
}
