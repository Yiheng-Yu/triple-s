use std::range::Range;

use crate::{
    utils::ArcLock,
};

#[cfg(feature="pyo3")]
use pyo3::prelude::*;


#[cfg_attr(feature="pyo3", pyclass(name="Response", module="triple_s"))]
pub enum Response {
    Quantity(Quantity),
    Category(Category),
}

#[cfg_attr(feature="pyo3", pyclass(name="Category", module="triple_s", from_py_object))]
pub struct Category {
    pub values: ArcLock<Vec<ArcLock<Code>>>
}

impl Clone for Category {
    fn clone(&self) -> Self {
        todo!()
    }
}

#[cfg_attr(feature="pyo3", pyclass(name="Code", module="triple_s", from_py_object))]
pub struct Code {
    value: Value,
    label: Option<String>,
    filter: Option<String>,
    score: Option<f64>,
}

impl Clone for Code {
    fn clone(&self) -> Self {
        todo!()
    }
}

#[cfg_attr(feature="pyo3", pyclass(name="Value", module="triple_s", from_py_object))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(u64),
    Str(String)
}

#[cfg_attr(feature="pyo3", pyclass(name="Quantity", module="triple_s", from_py_object))]
pub struct Quantity {
    range: Range<f64>,
    /// [(special_value, special_label)]
    special: ArcLock<Vec<(f64, String)>>
}

impl Clone for Quantity {
    fn clone(&self) -> Self {
        todo!()
    }
}