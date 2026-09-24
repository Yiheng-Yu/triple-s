use std::str::FromStr;

#[cfg(feature = "pyo3")]
use pyo3::{PyResult, exceptions, prelude::*, types::PyDict};

use crate::{prelude::*, utils};

use super::{Mode, Version};

#[cfg_attr(
    feature = "pyo3",
    pyclass(
        dict,
        weakref,
        module = "triple_s",
        name = "Meta",
        from_py_object,
        get_all,
        set_all
    )
)]
#[derive(Debug, Clone)]
pub struct Meta {
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

impl Meta {
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
impl Meta {
    /// Create survey metadata for the given Triple-S version.
    /// Optional text fields default to None; languages default to [], modes to
    /// [None]. Modes accept "analysis", "interview", or None. Invalid
    /// versions or modes raise ValueError.
    #[new]
    #[pyo3(signature = (
        version: "str",
        name: "str | None" = None,
        title: "str | None" = None,
        date: "str | None" = None,
        time: "str | None" = None,
        origin: "str | None" = None,
        user: "str | None" = None,
        style: "str | None" = None,
        language_options: "list[str] | None" = None,
        mode_options: "list[str] | None" = None,
        standard: "str | None" = None,
    ))]
    fn py_new(
        version: &str,
        name: Option<String>,
        title: Option<String>,
        date: Option<String>,
        time: Option<String>,
        origin: Option<String>,
        user: Option<String>,
        style: Option<String>,
        language_options: Option<Vec<String>>,
        mode_options: Option<Vec<String>>,
        standard: Option<String>,
    ) -> PyResult<Self> {
        println!("TODO: MAKE META::NEW AVAIBALE TO ALL FEATURES");
        let version = Version::from_str(version)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))?;

        let mode_options = mode_options
            .map(|options| {
                options
                    .into_iter()
                    .map(|mode| {
                        Mode::from_str(&mode).map_err(|err| {
                            exceptions::PyValueError::new_err(format!(
                                "{err}. Mode options should be one of: {:?}",
                                Mode::available_options()
                            ))
                        })
                    })
                    .collect::<PyResult<Vec<Mode>>>()
            })
            .transpose()?
            .unwrap_or_default();

        let language_options = language_options
            .map(|languages| {
                languages
                    .iter()
                    .map(|s| {
                        utils::validate_language_tag(&s)
                            .map(|s| s.to_string())
                            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
                    })
                    .collect::<PyResult<Vec<String>>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            version: version,
            name: name,
            title: title,
            date: date,
            time: time,
            origin: origin,
            user: user,
            style: style,
            language_options: language_options,
            mode_options: mode_options,
            standard: standard,
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
