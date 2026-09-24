#[cfg(feature = "pyo3")]
use std::str::FromStr;

#[cfg(feature = "pyo3")]
use pyo3::{
    PyResult,
    exceptions,
    prelude::*,
    types::{PyDict, PyFloat, PyTuple},
};

use super::{Mode, Version};
#[cfg(feature = "pyo3")]
use crate::{prelude::*, utils};

#[cfg_attr(
    feature = "pyo3",
    pyclass(dict, weakref, module = "triple_s", name = "SurveyInfo", from_py_object, get_all, set_all)
)]
#[derive(Debug, Clone)]
pub struct SurveyInfo {
    /// The version attribute is used to indicate the version of the Triple-S
    /// standard that applies to this specification.
    pub version: Version,

    /// name of the survey
    pub name: Option<String>,

    /// title of the survey
    pub title: Option<String>,

    /// date of the creation
    pub date: Option<String>,

    /// time of the creation
    pub time: Option<String>,

    pub origin: Option<String>,

    /// user who created it
    pub user: Option<String>,

    /// html css styling
    pub style: Option<String>,

    /// ** Introduced since sss 1.2 **
    /// list of all available language options
    pub language_options: Vec<String>,

    /// ** Introduced since sss 1.2 **
    /// list of all available `mode` options
    pub mode_options: Vec<Mode>,

    /// ** Deprecated after sss version 1.2 **
    /// (Optional) indicate that some aspect of the triple-s definition meets a
    /// defined standard that may reduce the amount of checking required by an
    /// import program. The only value that option_setting may have in
    /// version 1.2 is 'standardnames'.
    pub standard: Option<String>,
}

impl SurveyInfo {
    pub fn with_version(version: Version) -> Self {
        Self {
            version,
            name: None,
            title: None,
            date: None,
            time: None,
            origin: None,
            user: None,
            style: None,
            language_options: Vec::new(),
            mode_options: vec![Mode::None],
            standard: None,
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl SurveyInfo {
    
    #[getter(language_options)]
    fn get_language_options<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.language_options.iter())
    }

    #[setter(language_options)]
    fn set_language_options(&mut self, values: Vec<String>) -> PyResult<()> {
        self.language_options = validate_language_options(values)?;
        Ok(())
    }

    #[getter(mode_options)]
    fn get_mode_options<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.mode_options.iter().cloned())
    }

    #[setter(mode_options)]
    fn set_mode_options(&mut self, values: Vec<Bound<'_, PyAny>>) -> PyResult<()> {
        self.mode_options = parse_mode_options(values)?;
        Ok(())
    }

    /// Create survey metadata for a Triple-S version given as a string or float.
    /// Optional text fields default to None; languages and modes default to
    /// empty tuples. Modes accept "analysis", "interview", or None. Invalid
    /// versions or modes raise ValueError.
    #[new]
    #[pyo3(signature = (
        version: "str | float",
        name: "str | None" = None,
        title: "str | None" = None,
        date: "str | None" = None,
        time: "str | None" = None,
        origin: "str | None" = None,
        user: "str | None" = None,
        style: "str | None" = None,
        language_options: "list[str] | None" = None,
        mode_options: "list[Mode | str | None] | None" = None,
        standard: "str | None" = None,
    ))]
    fn py_new(
        version: Bound<'_, PyAny>,
        name: Option<String>,
        title: Option<String>,
        date: Option<String>,
        time: Option<String>,
        origin: Option<String>,
        user: Option<String>,
        style: Option<String>,
        language_options: Option<Vec<String>>,
        mode_options: Option<Vec<Bound<'_, PyAny>>>,
        standard: Option<String>,
    ) -> PyResult<Self> {
        let version = if let Ok(version) = version.extract::<String>() {
            version
        } else if let Ok(value) = version.extract::<f64>() {
            format!("{:.1}", value)
        } else {
            return Err(exceptions::PyTypeError::new_err(
                "version must be a str or float",
            ));
        };
        let version = Version::from_str(&version)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))?;

        let mode_options = parse_mode_options(mode_options.unwrap_or_default())?;
        let language_options =
            validate_language_options(language_options.unwrap_or_default())?;

        Ok(Self {
            version,
            name,
            title,
            date,
            time,
            origin,
            user,
            style,
            language_options,
            mode_options,
            standard,
        })
    }

    #[pyo3(signature = () -> "dict[str, str]")]
    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);

        dict.set_item("version", self.version.clone())?;

        if let Some(value) = self.name.clone() {
            dict.set_item("name", value)?;
        };

        if let Some(value) = self.title.clone() {
            dict.set_item("title", value)?;
        }

        if let Some(value) = self.date.clone() {
            dict.set_item("date", value)?;
        };

        if let Some(value) = self.time.clone() {
            dict.set_item("time", value)?;
        };

        if let Some(value) = self.origin.clone() {
            dict.set_item("origin", value)?;
        };

        if let Some(value) = self.user.clone() {
            dict.set_item("user", value)?;
        };

        if let Some(value) = self.standard.clone() {
            dict.set_item("standard", value)?;
        };

        if let Some(value) = self.style.clone() {
            dict.set_item("style", value)?;
        };

        dict.set_item("language", self.language_options.clone())?;
        dict.set_item("mode_options", self.mode_options.clone())?;

        Ok(dict.unbind())
    }
}

#[cfg(feature = "pyo3")]
fn validate_language_options(values: Vec<String>) -> PyResult<Vec<String>> {
    for value in &values {
        utils::validate_language_tag(value)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))?;
    }
    Ok(values)
}

#[cfg(feature = "pyo3")]
fn parse_mode_options(values: Vec<Bound<'_, PyAny>>) -> PyResult<Vec<Mode>> {
    values
        .into_iter()
        .map(|value| {
            if value.is_none() {
                Ok(Mode::None)
            } else if let Ok(mode) = value.extract::<Mode>() {
                Ok(mode)
            } else if let Ok(mode) = value.extract::<String>() {
                Mode::from_str(&mode).map_err(|err| {
                    exceptions::PyValueError::new_err(format!(
                        "{err}. Mode options should be one of: {:?}",
                        Mode::available_options()
                    ))
                })
            } else {
                Err(exceptions::PyTypeError::new_err(
                    "mode options must contain Mode objects, strings, or None",
                ))
            }
        })
        .collect()
}
