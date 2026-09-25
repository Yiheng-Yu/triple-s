#[cfg(feature = "pyo3")]
use std::str::FromStr;

#[cfg(feature = "pyo3")]
use anyhow::Context;
#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*};

#[cfg(feature = "pyo3")]
use crate::{prelude::Flag, utils::{lock_it_up, get_type_name}};
use crate::{
    entities::{CodeFormat, Text, Usage},
    prelude::DeepCopy,
    utils::ArcLock,
};

#[cfg_attr(feature = "pyo3", pyclass(module = "triple_s", name = "Question", from_py_object))]
#[derive(Debug)]
pub struct Question {
    pub index: u64,
    pub name: ArcLock<String>,
    // pub label: ArcLock<Text>,
    pub label: Option<Text>,
    pub filter: ArcLock<Option<String>>,
    pub format: ArcLock<CodeFormat>,
    pub usage: ArcLock<Usage>,
    pub location: ArcLock<Location>,
    // pub response: Response,
}

impl Clone for Question {
    /// creates a shallow copy of the current instance (only clones the pointer, not the data it points to.)
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
    fn new(
        name: &str,
        label: Option<&str>,
        filter: Option<&str>,
        format: Option<&str>,
        usage: Option<&str>,
        location: Option<Location>
    ) -> PyResult<Self> {
        let name = name.to_string();
        let label = label.map(Text::new);
        let filter = lock_it_up(filter.map(|s| s.to_string()));
        let format = format.map(CodeFormat::from_str)
            .transpose()
            .with_context(|| format!("Expect value to be one of: {:?} or None", CodeFormat::available_options()))
            .map_err(|err| exceptions::PyValueError::new_err(err.backtrace().to_string()))?
            .unwrap_or_default();
        let format = lock_it_up(format);

        let usage = usage.map(Usage::from_str)
            .transpose()
            .with_context(|| format!("Expect value to be one of: {:?} or None", Usage::available_options()))
            .map_err(|err| exceptions::PyValueError::new_err(err.backtrace().to_string()))?
            .unwrap_or_default();
        let usage = lock_it_up(usage);

        let location = lock_it_up(location.unwrap_or_default());

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
                    "Expect one of: Text, str, None, got: {}", type_name
                )))
            }
        } else {
            self.label = None;
            Ok(())
        }
    }
}

/// Defines the part of the data record that is allocated to holding the value
/// of the variable.
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Location", from_py_object)
)]
#[derive(Debug, Clone, PartialEq, Default, Eq)]
pub struct Location {
    pub from: u64,
    pub to: u64,
}

impl DeepCopy for Location {
    fn deepcopy(&self) -> Self {
        Self {
            from: self.from,
            to: self.to,
        }
    }
}

impl Location {
    pub fn to_sss_classic(&self) -> String {
        if self.to != self.from {
            format!("POSITION {} TO {}", self.from, self.to)
        } else {
            format!("POSITION {}", self.from)
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

        Ok(Self { from, to })
    }

    #[getter(from)]
    fn py_get_from(&self) -> u64 {
        self.from.clone()
    }

    #[setter(from)]
    fn py_set_from(&mut self, from: u64) -> PyResult<()> {
        self.from = from;
        Ok(())
    }

    #[getter(to)]
    fn py_get_to(&self) -> u64 {
        self.to.clone()
    }

    #[setter(to)]
    fn py_set_to(&mut self, to: u64) -> PyResult<()> {
        if to >= self.from {
            self.to = to;
            Ok(())
        } else {
            Err(exceptions::PyValueError::new_err(format!(
                "Value of `to` cannot be greater than `from` ({}).",
                &self.from
            )))
        }
    }
}
