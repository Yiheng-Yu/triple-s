//! meta information about the datamap entity.

#[cfg(feature = "pyo3")]
use std::str::FromStr;

#[cfg(feature = "pyo3")]
use pyo3::{
    PyResult,
    exceptions,
    prelude::*,
    types::{PyDict, PyTuple},
};
use strum::{EnumIter, EnumString};

use super::text::Mode;
use crate::prelude::Flag;
#[cfg(feature = "pyo3")]
use crate::prelude::PyEnum;
#[cfg(feature = "pyo3")]
use crate::utils;

#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString, EnumIter)]
pub enum Version {
    #[strum(serialize = "1.0")]
    V1_0,
    #[strum(serialize = "1.1")]
    V1_1,
    #[strum(serialize = "1.2")]
    V1_2,
    #[strum(serialize = "2.0")]
    V2_0,
    #[strum(serialize = "3.0")]
    V3_0,
}

impl Flag for Version {
    fn to_string(&self) -> String {
        match self {
            Self::V1_0 => "1.0".into(),
            Self::V1_1 => "1.1".into(),
            Self::V1_2 => "1.2".into(),
            Self::V2_0 => "2.0".into(),
            Self::V3_0 => "3.0".into(),
        }
    }
}

#[cfg(feature = "pyo3")]
impl PyEnum for Version {}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Version {
    #[pyo3(name = "available_options")]
    #[staticmethod]
    fn py_available_options() -> Vec<String> {
        Self::available_options()
    }
}

/// meta information about the datamap entity.
#[cfg_attr(
    feature = "pyo3",
    pyclass(
        dict,
        weakref,
        module = "triple_s",
        name = "SurveyInfo",
        from_py_object,
        get_all,
        set_all
    )
)]
#[derive(Debug, Clone)]
pub struct SurveyInfo {
    /// The version attribute is used to indicate the version of the Triple-S
    /// standard that applies to this specification.
    pub version: Version,

    /// name of the datamap
    pub name: Option<String>,

    /// title of the datamap
    pub title: Option<String>,

    /// date of the creation
    pub date: Option<String>,

    /// time of the creation
    pub time: Option<String>,

    /// the originating system (program and operating system), for example:
    /// `<origin>MyProg v3, Windows 7</origin>`
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
    /// Creates an empty `SurveyInfo` instance with given version number.
    ///
    /// # Example
    /// ```rust
    /// use triple_s::{SurveyInfo, Version};
    ///
    /// let info = SurveyInfo::new(Version::V1_2);
    /// ```
    pub fn new(version: Version) -> Self {
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
            mode_options: Vec::new(),
            standard: None,
        }
    }

    /// Set name of the survey.
    ///
    /// # Example:
    /// ```rust
    /// use triple_s::{Survey, SurveyInfo, Version};
    ///
    /// let info = SurveyInfo::new(Version::V1_2).with_name(Some("Survey1"));
    /// let survey = Survey::new(info);
    /// assert_eq!(survey.name(), Some("Survey1".into()))
    /// ```
    pub fn with_name(mut self, name: Option<impl Into<String>>) -> Self {
        self.name = name.map(|item| item.into());
        self
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

    /// Create datamap metadata for a Triple-S version given as a string or
    /// float.
    ///
    /// # Examples
    ///
    /// ```python
    /// from triple_s import SurveyInfo
    ///
    /// meta = SurveyInfo("2.0", language_options=["en-GB"], mode_options=["analysis"])
    /// ```
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
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))?;

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
#[inline]
fn validate_language_options(values: Vec<String>) -> PyResult<Vec<String>> {
    for value in &values {
        utils::validate_language_tag(value)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))?;
    }
    Ok(values)
}

#[cfg(feature = "pyo3")]
#[inline]
fn parse_mode_options(values: Vec<Bound<'_, PyAny>>) -> PyResult<Vec<Mode>> {
    values.into_iter().map(Mode::from_py_any).collect()
}
