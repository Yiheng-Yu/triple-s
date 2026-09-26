use std::{mem::type_info::Str, range::Range};
use anyhow::Result;
use crate::{
    Text, prelude::DeepCopy, utils::{ArcLock, lock_it_up},
};

#[cfg(feature="pyo3")]
use pyo3::prelude::*;


#[cfg_attr(feature="pyo3", pyclass(name="Response", module="triple_s"))]
pub enum Response {
    Quantity(Quantity),
    Category(Category),
}

impl Response {
    pub fn get_code(key: impl Into<Value>) -> Result<Option<Code>> {
        todo!()
    }
}

#[cfg_attr(feature="pyo3", pyclass(name="Category", module="triple_s", from_py_object))]
pub struct Category {
    pub range: ArcLock<Option<Range<u64>>>,
    // pub spread: ArcLock<Option<Range<u64>>>,
    pub values: ArcLock<Vec<ArcLock<Code>>>,
}

impl Clone for Category {
    fn clone(&self) -> Self {
        Self {
            range: self.range.clone(),
            values: self.values.clone()
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Category {
}

#[cfg_attr(feature="pyo3", pyclass(name="Code", module="triple_s", from_py_object))]
#[derive(Debug)]
pub struct Code {
    pub value: ArcLock<Value>,
    pub label: ArcLock<Text>,
    pub filter:ArcLock<Option<String>>,
    pub score: ArcLock<Option<f64>>,
}

impl Clone for Code {
    /// makes a shallow copy of the data
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            label: self.label.clone(),
            filter: self.filter.clone(),
            score: self.score.clone(),
        }
    }
}

impl DeepCopy for Code {
    fn deepcopy(&self) -> Self {
        Self {
            value: lock_it_up(self.value.read().clone()),
            label: lock_it_up(self.label.read().clone()),
            filter: lock_it_up(self.filter.read().clone()),
            score: lock_it_up(self.score.read().clone()),
        }
    }
}

impl Code {
    pub fn new(
        value: Value,
        label: impl Into<String>,
        filter: Option<impl Into<String>>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(value),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score)
        }
    }

    pub fn new_numeric(
        value: u64,
        label: impl Into<String>,
        filter: Option<impl Into<String>>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(Value::Numeric(value)),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score)
        }
    }

    pub fn new_literal(
        value: impl Into<String>,
        label: impl Into<String>,
        filter: Option<impl Into<String>>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(Value::Literal(value.into())),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score)
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Code {

}

#[cfg_attr(feature="pyo3", pyclass(name="Value", module="triple_s", from_py_object))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Numeric(u64),
    Literal(String)
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::Numeric(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Literal(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::Literal(value.to_owned())
    }
}


#[cfg_attr(feature="pyo3", pyclass(name="Quantity", module="triple_s", from_py_object))]
pub struct Quantity {
    dtype: DType,
    range: ArcLock<Range<f64>>,
    /// [(special_value, special_label)]
    special: ArcLock<Vec<(f64, String)>>
}

impl Clone for Quantity {
    fn clone(&self) -> Self {
        todo!()
    }
}

enum DType {
    Date,
    Time,
    Float,
}