use std::str::FromStr;

use anyhow::{Context, Result};
#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*};

#[cfg(feature = "pyo3")]
use crate::utils::{get_type_name, normalize_py_index, py_value_error};
use crate::{
    entities::{CodeFormat, Text, Usage},
    error,
    prelude::{DeepCopy, *},
    utils::{ArcLock, lock_it_up},
};

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Question", from_py_object)
)]
#[derive(Debug)]
pub struct Question {
    pub index: u64,
    pub name: ArcLock<String>,
    pub label: Option<Text>,
    pub filter: ArcLock<Option<String>>,
    pub format: ArcLock<Option<CodeFormat>>,
    pub usage: ArcLock<Option<Usage>>,
    // pub location: ArcLock<Location>,
    pub location: Location,
    // pub response: Response,
}

impl Clone for Question {
    /// creates a shallow copy of the current instance (only clones the pointer,
    /// not the data it points to.)
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            filter: self.filter.clone(),
            format: self.format.clone(),
            usage: self.usage.clone(),
            location: self.location.clone(),
        }
    }
}

impl DeepCopy for Question {
    fn deepcopy(&self) -> Self {
        Self {
            index: self.index,
            name: self.name.deepcopy(),
            label: self.label.deepcopy(),
            filter: self.filter.deepcopy(),
            format: self.format.deepcopy(),
            usage: self.usage.deepcopy(),
            location: self.location.deepcopy(),
        }
    }
}

impl Question {
    pub fn new(
        name: &str,
        label: Option<&str>,
        filter: Option<&str>,
        format: Option<&str>,
        usage: Option<&str>,
        location: Option<Location>,
    ) -> Result<Self> {
        let name = name.to_string();
        let label = label.map(Text::new);
        let filter = lock_it_up(filter.map(|s| s.to_string()));
        let format = format.map(CodeFormat::from_str).transpose().with_context(|| {
            format!(
                "Expect value to be one of: {:?} or None",
                CodeFormat::available_options()
            )
        })?;
        let format = lock_it_up(format);

        let usage = usage.map(Usage::from_str).transpose().with_context(|| {
            format!(
                "Expect value to be one of: {:?} or None",
                Usage::available_options()
            )
        })?;
        let usage = lock_it_up(usage);

        // let location = lock_it_up(location.unwrap_or_default());
        let location = location.unwrap_or_default();

        let instance = Self {
            index: 0,
            name: lock_it_up(name),
            label: label,
            filter: filter,
            format: format,
            usage: usage,
            location: location,
        };

        Ok(instance)
    }

    pub fn get_format_str(&self) -> Option<&str> {
        self.format.read().as_ref().map(|item| item.as_str())
    }

    pub fn set_format_from_str(&self, value: Option<&str>) -> Result<()> {
        match value {
            None => {
                *self.format.write() = None;
                Ok(())
            }
            Some(format) => {
                let format = CodeFormat::from_str(format).with_context(|| {
                    format!(
                        "Expected to be `None` or one of: {:?}",
                        CodeFormat::available_options()
                    )
                })?;
                *self.format.write() = Some(format);
                Ok(())
            }
        }
    }

    pub fn set_usage_from_str(&self, value: Option<&str>) -> Result<()> {
        match value {
            None => {
                *self.usage.write() = None;
                Ok(())
            }
            Some(usage) => {
                let usage = Usage::from_str(usage).with_context(|| {
                    format!(
                        "Expected to be `None` or one of: {:?}",
                        Usage::available_options()
                    )
                })?;
                *self.usage.write() = Some(usage);
                Ok(())
            }
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Question {
    #[new]
    #[pyo3(signature=(
        name:"str", 
        label:"str"=None, 
        filter:"str"=None, 
        format:"'numeric'|'literal'|None"=None,
        usage:"'serial'|'weight'|None"=None,
        location=None,
    ))]
    fn py_new(
        name: &str,
        label: Option<&str>,
        filter: Option<&str>,
        format: Option<&str>,
        usage: Option<&str>,
        location: Option<Location>,
    ) -> PyResult<Self> {
        Self::new(name, label, filter, format, usage, location).map_err(py_value_error)
    }

    fn __repr__(&self) -> String {
        format!("Question(name='{}')", self.name.read().as_str())
    }

    #[getter]
    fn get_label(&self) -> PyResult<Option<Text>> {
        let output = self.label.as_ref().map(|label| label.clone());
        Ok(output)
    }

    #[setter]
    fn set_label<'py>(&mut self, label: Option<&Bound<'py, PyAny>>) -> PyResult<()> {
        if let Some(bound) = label {
            if let Ok(string) = bound.extract::<&str>() {
                if let Some(current) = &mut self.label {
                    current.set_value(string);
                } else {
                    self.label = Some(Text::new(string))
                };
                Ok(())
            } else if let Ok(object) = bound.extract::<Text>() {
                self.label = Some(object);
                Ok(())
            } else {
                let type_name = get_type_name(bound)?;
                Err(exceptions::PyTypeError::new_err(format!(
                    "Expect one of: Text, str, None, got: {}",
                    type_name
                )))
            }
        } else {
            self.label = None;
            Ok(())
        }
    }

    #[getter]
    fn get_filter(&self) -> PyResult<Option<String>> {
        Ok(self.filter.read().clone())
    }

    #[setter]
    fn set_filter(&mut self, filter: Option<String>) -> PyResult<()> {
        self.filter = lock_it_up(filter);
        Ok(())
    }

    #[getter]
    fn get_location(&self) -> PyResult<Location> {
        Ok(self.location.clone())
    }

    #[getter(format)]
    fn py_get_format(&self) -> PyResult<Option<&str>> {
        Ok(self.get_format_str())
    }

    #[setter(format)]
    fn py_set_format<'py>(&self, format: &Bound<'py, PyAny>) -> PyResult<()> {
        if let Ok(value) = format.extract::<Option<CodeFormat>>() {
            *self.format.write() = value;
            Ok(())
        } else if let Ok(value) = format.extract::<Option<&str>>() {
            self.set_format_from_str(value).map_err(py_value_error)
        } else {
            let type_name = get_type_name(format)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect 'format' to be one of: 'numeric'|'literal'|None, got: {}",
                type_name
            )))
        }
    }

    #[getter(usage)]
    fn py_get_usage(&self) -> PyResult<Option<&str>> {
        Ok(self.usage.read().as_ref().map(|item| item.as_str()))
    }

    #[setter(usage)]
    fn py_set_usage<'py>(&self, value: &Bound<'py, PyAny>) -> PyResult<()> {
        if let Ok(value) = value.extract::<Option<Usage>>() {
            *self.usage.write() = value;
            Ok(())
        } else if let Ok(value) = value.extract::<Option<&str>>() {
            self.set_usage_from_str(value).map_err(py_value_error)
        } else {
            let type_name = get_type_name(value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect 'usage' to be one of: 'serial'|'weight'|None, got: {}",
                type_name
            )))
        }
    }
}

/// Defines the part of the data record that is allocated to holding the value
/// of the variable.
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Location", from_py_object)
)]
#[derive(Debug, Default)]
pub struct Location {
    pub from: ArcLock<u64>,
    pub to: ArcLock<u64>,
}

impl DeepCopy for Location {
    fn deepcopy(&self) -> Self {
        Self {
            from: lock_it_up(self.from.read().clone()),
            to: lock_it_up(self.to.read().clone()),
        }
    }
}

impl Clone for Location {
    fn clone(&self) -> Self {
        Self {
            from: self.from.clone(),
            to: self.to.clone(),
        }
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        (self.from.read().clone() == other.from.read().clone())
            & (self.to.read().clone() == other.to.read().clone())
    }
}

impl Location {
    pub fn to_sss_classic(&self) -> String {
        if self.to.read().clone() != self.from.read().clone() {
            format!("POSITION {} TO {}", self.from.read(), self.to.read())
        } else {
            format!("POSITION {}", self.from.read())
        }
    }

    #[inline]
    pub fn set_from(&mut self, from: u64) -> Result<(), error::InvalidArg> {
        *self.from.write() = from;
        Ok(())
    }

    #[inline]
    pub fn set_to(&mut self, to: u64) -> Result<(), error::InvalidArg> {
        if to >= self.from.read().clone() {
            *self.to.write() = to;
            Ok(())
        } else {
            Err(error::InvalidArg::new(format!(
                "The `end` of a Location cannot be larger than its `start` location ({}).",
                &self.from.read()
            )))
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Location {
    #[new]
    #[pyo3(signature=(from:"int", to:"int"=None))]
    fn new(from: u64, to: Option<u64>) -> PyResult<Self> {
        let to = if let Some(to) = to {
            if to < from {
                return Err(exceptions::PyValueError::new_err(format!(
                    "Value of `to` cannot be greater than `from`."
                )));
            } else {
                to
            }
        } else {
            from.clone()
        };

        Ok(Self {
            from: lock_it_up(from),
            to: lock_it_up(to),
        })
    }

    // `from` is a preserved word in Python hence the alias
    #[getter(start)]
    fn py_get_from(&self) -> u64 {
        self.from.read().clone()
    }

    #[setter(start)]
    fn py_set_from(&mut self, from: u64) -> PyResult<()> {
        self.set_from(from)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
    }

    #[getter(end)]
    fn py_get_to(&self) -> u64 {
        self.to.read().clone()
    }

    #[setter(end)]
    fn py_set_to(&mut self, to: u64) -> PyResult<()> {
        self.set_to(to)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Variant;

    fn new_empty_question() -> Question {
        Question::new("Q1", None, None, None, None, None).unwrap()
    }

    fn new_filled_question() -> Question {
        let mut question = new_empty_question();
        question.location.set_from(23).unwrap();
        question.location.set_to(55).unwrap();
        question.label = Some(Text::new("some label"));
        question
    }

    #[test]
    fn test_set_question_data_location_success() {
        let mut question = new_empty_question();
        question.location.set_from(23).unwrap();
        question.location.set_to(55).unwrap();

        assert_eq!(question.location.from.read().clone(), 23);
        assert_eq!(question.location.to.read().clone(), 55);
    }

    #[test]
    fn test_set_question_data_location_fail() {
        let mut question = new_empty_question();
        assert!(question.location.set_from(23).is_ok());
        assert!(question.location.set_to(23).is_ok());
        assert!(question.location.set_to(3).is_err());
    }

    #[test]
    fn test_sharable_mut_question_location() {
        let question = new_filled_question();
        let mut location = question.location.clone();

        location.set_from(2).unwrap();
        assert_eq!(question.location.from.read().clone(), 2);
        location.set_to(55).unwrap();
        assert_eq!(question.location.to.read().clone(), 55);
    }

    #[test]
    fn test_sharable_mut_question_label() {
        let question = new_filled_question();
        let mut label = question.label.as_ref().unwrap().clone();
        label.set_value("THIS");
        assert_eq!(
            "THIS",
            question.label.as_ref().map(|item| item.get_value().to_string()).unwrap()
        );

        label
            .add_variant(Variant::new("variant1", Some("analysis"), None).unwrap())
            .unwrap();
        assert_eq!(
            label.variants.read().get(0).unwrap().clone(),
            question
                .label
                .as_ref()
                .map(|item| item.variants.read().get(0).unwrap().clone())
                .unwrap()
        )
    }

    #[cfg(feature = "pyo3")]
    #[test]
    fn test_question_format_getter_setter() {
        Python::initialize();
        // somehoe seems that i need `Python` to set these up,,,
        Python::attach(|py| {
            let question = new_empty_question();
            assert_eq!(question.py_get_format().unwrap(), None);

            for value in ["numeric", "literal"] {
                let input = pyo3::types::PyString::new(py, value);
                question.py_set_format(&input).unwrap();
                assert_eq!(question.py_get_format().unwrap(), Some(value));
            }
        });
    }

    #[cfg(feature = "pyo3")]
    #[test]
    fn test_question_usage_getter_setter() {
        Python::initialize();
        Python::attach(|py| {
            let question = new_empty_question();
            assert_eq!(question.py_get_usage().unwrap(), None);

            for value in ["serial", "weight"] {
                let input = pyo3::types::PyString::new(py, value);
                question.py_set_usage(&input).unwrap();
                assert_eq!(question.py_get_usage().unwrap(), Some(value));
            }
        });
    }
}
