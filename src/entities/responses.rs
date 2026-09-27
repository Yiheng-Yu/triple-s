use std::range::Range;

use anyhow::Result;
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
#[cfg(feature = "pyo3")]
use pyo3::{
    prelude::*,
    types::{PyFloat, PyInt, PyString},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    Text,
    error::{InvalidArg, UnsupportedFeature},
    entities::QuestionType,
    prelude::DeepCopy,
    utils::{ArcLock, lock_it_up},
};

#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Response", module = "triple_s", from_py_object)
)]
#[derive(Debug, Default)]
pub struct Response {
    pub range: ArcLock<Option<Range<Value>>>,
    pub values: ArcLock<Vec<ArcLock<Code>>>,
}

impl Clone for Response {
    fn clone(&self) -> Self {
        Self {
            range: self.range.clone(),
            values: self.values.clone(),
        }
    }
}

impl DeepCopy for Response {
    fn deepcopy(&self) -> Self {
        let values = self
            .values
            .read()
            .iter()
            .map(|item| lock_it_up(item.read().deepcopy()))
            .collect();
        Self {
            range: lock_it_up(self.range.read().as_ref().map(|item| item.clone())),
            values: lock_it_up(values),
        }
    }
}

impl Response {
    pub fn set_range(&self, value: Option<Range<Value>>) {
        *self.range.write() = value;
    }

    pub fn set_values(&self, value: Vec<ArcLock<Code>>) {
        *self.values.write() = value;
    }

    pub fn range(&self) -> RwLockReadGuard<'_, Option<Range<Value>>> {
        self.range.read()
    }

    pub fn range_mut(&self) -> RwLockWriteGuard<'_, Option<Range<Value>>> {
        self.range.write()
    }

    pub fn values(&self) -> RwLockReadGuard<'_, Vec<ArcLock<Code>>> {
        self.values.read()
    }

    pub fn values_mut(&self) -> RwLockWriteGuard<'_, Vec<ArcLock<Code>>> {
        self.values.write()
    }

    pub fn to_sss_classic(&self, question_type: &QuestionType) -> Result<String> {
        let mut output = String::from("VALUES\n");
        if let Some(range) = self.range.read().as_ref() {
            let start = range.start.to_sss_classic()?;
            let end = range.end.to_sss_classic()?;
            if *question_type == QuestionType::Quantity && start == end {
                output.push_str(&start);
                output.push('\n');
            } else {
                output.push_str(&format!("{start} TO {end}\n"));
            }
        }
        for code in self.values.read().iter() {
            output.push_str(&code.read().to_sss_classic()?);
            if *question_type == QuestionType::Quantity {
                output.push_str(" SPECIAL");
            }
            output.push('\n');
        }
        output.push_str("END VALUES\n");
        Ok(output)
    }

    pub fn new(range: Option<Range<Value>>, values: Vec<Code>) -> Self {
        Self {
            range: lock_it_up(range),
            values: lock_it_up(values.into_par_iter().map(lock_it_up).collect()),
        }
    }
}

#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Code", module = "triple_s", from_py_object)
)]
#[derive(Debug)]
pub struct Code {
    pub value: ArcLock<Value>,
    pub label: ArcLock<Text>,
    pub filter: ArcLock<Option<String>>,
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
            score: lock_it_up(*self.score.read()),
        }
    }
}

impl Code {
    pub fn value(&self) -> RwLockReadGuard<'_, Value> {
        self.value.read()
    }

    pub fn value_mut(&self) -> RwLockWriteGuard<'_, Value> {
        self.value.write()
    }

    pub fn label(&self) -> RwLockReadGuard<'_, Text> {
        self.label.read()
    }

    pub fn label_mut(&self) -> RwLockWriteGuard<'_, Text> {
        self.label.write()
    }

    pub fn filter(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.filter.read()
    }

    pub fn filter_mut(&self) -> RwLockWriteGuard<'_, Option<String>> {
        self.filter.write()
    }

    pub fn score(&self) -> RwLockReadGuard<'_, Option<f64>> {
        self.score.read()
    }

    pub fn score_mut(&self) -> RwLockWriteGuard<'_, Option<f64>> {
        self.score.write()
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        Ok(format!(
            "{} \"{}\"",
            self.value.read().to_sss_classic()?,
            self.label.read().to_sss_classic(),
        ))
    }

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
            score: lock_it_up(score),
        }
    }

    pub fn new_unsigned(
        value: u32,
        label: impl Into<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(Value::UnSigned(value)),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score),
        }
    }

    pub fn new_signed(
        value: i32,
        label: impl Into<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(Value::Signed(value)),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score),
        }
    }

    pub fn new_float(
        value: f64,
        label: impl Into<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(Value::Float(value)),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score),
        }
    }

    pub fn new_literal(
        value: impl Into<String>,
        label: impl Into<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(Value::Literal(value.into())),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score),
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Code {
    #[getter]
    fn py_get_label(&self) -> Text {
        self.label.read().clone()
    }

    #[getter]
    fn py_get_value<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        match &*self.value.read() {
            Value::Literal(item) => PyString::new(py, item).into_any(),
            Value::Float(item) => PyFloat::new(py, item.clone()).into_any(),
            Value::Signed(item) => PyInt::new(py, item).into_any(),
            Value::UnSigned(item) => PyInt::new(py, item).into_any(),
        }
    }
}

#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Value", module = "triple_s", from_py_object)
)]
#[derive(Debug, Clone)]
pub enum Value {
    UnSigned(u32),
    Signed(i32),
    Float(f64),
    Literal(String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::UnSigned(self_), Value::UnSigned(other_)) => self_ == other_,
            (Value::Literal(self_), Value::Literal(other_)) => self_ == other_,
            (Value::Float(self_), Value::Float(other_)) => {
                self_ == other_ || (self_.is_nan() && other_.is_nan())
            }
            _ => false,
        }
    }
}

impl Eq for Value {}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::UnSigned(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Signed(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
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

impl Value {
    pub fn to_sss_classic(&self) -> Result<String> {
        match self {
            Self::UnSigned(value) => Ok(value.to_string()),
            Self::Signed(value) => Ok(value.to_string()),
            Self::Float(value) => Ok(value.to_string()),
            Self::Literal(_) => Err(UnsupportedFeature::new(
                "SSS Classic", "None numeric code values.",
            ).into()),
        }
    }

    /// update content of current instance based on incoming `other`, raises
    /// error if `Self` and `Other` are of different variants.
    pub fn set_from(&mut self, other: Self) -> Result<(), InvalidArg> {
        match (&mut *self, &other) {
            (Value::Float(slf), Value::Float(oth)) => {
                *slf = *oth;
                Ok(())
            }
            (Value::Signed(slf), Value::Signed(oth)) => {
                *slf = *oth;
                Ok(())
            }
            (Value::UnSigned(slf), Value::UnSigned(oth)) => {
                *slf = *oth;
                Ok(())
            }
            (Value::Literal(slf), Value::Literal(oth)) => {
                *slf = oth.clone();
                Ok(())
            }
            _ => Err(InvalidArg::new(format!(
                "Cannot update value from instances of a different variant! Self: [{:?}], other: [{:?}]",
                self, other
            ))),
        }
    }

    pub fn parse_like(&self, text: &str) -> Result<Self, InvalidArg> {
        match self {
            Self::Float(_) => {
                let value = text.parse::<f64>().map_err(|err| {
                    InvalidArg::new(format!(
                        "Failed to parse value '{}' as float decimal: '{}'",
                        text, err
                    ))
                })?;
                Ok(Self::Float(value))
            }

            Self::Signed(_) => {
                let value = text.parse::<i32>().map_err(|err| {
                    InvalidArg::new(format!(
                        "Failed to parse value '{}' as integer: '{}'",
                        text, err
                    ))
                })?;

                Ok(Self::Signed(value))
            }

            Self::UnSigned(_) => {
                let value = text.parse::<u32>().map_err(|err| {
                    InvalidArg::new(format!(
                        "Failed to parse value '{}' as integer: '{}'",
                        text, err
                    ))
                })?;
                Ok(Self::UnSigned(value))
            }

            Self::Literal(_) => Ok(Self::Literal(text.to_owned())),
        }
    }
}
