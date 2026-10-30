//! meta information about the datamap entity.

#[cfg(feature = "pyo3")]
use std::str::FromStr;

use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
#[cfg(feature = "pyo3")]
use pyo3::{
    PyResult,
    exceptions,
    prelude::*,
    types::{PyDict, PyTuple},
};
use strum::{EnumIter, EnumString};

use super::text::Mode;
#[cfg(feature = "pyo3")]
use crate::prelude::PyEnum;
#[cfg(feature = "pyo3")]
use crate::utils;
#[cfg(feature = "pyo3")]
use crate::{error::missing_key_err, utils::get_type_name};
use crate::{
    prelude::Flag,
    utils::{ArcLock, lock_it_up},
};

/// All available Triple-S file versions
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString, EnumIter)]
pub enum Version {
    /// 1.0 (triple-s classic)
    #[strum(serialize = "1.0")]
    V1_0,

    /// 1.1
    #[strum(serialize = "1.1")]
    V1_1,

    /// 1.2, this could either be classic or xml
    #[strum(serialize = "1.2")]
    V1_2,

    /// 2.0
    #[strum(serialize = "2.0")]
    V2_0,

    /// 3.0
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
    )
)]
#[derive(Debug, Clone)]
pub struct SurveyInfo {
    version: ArcLock<Version>,
    name: ArcLock<Option<String>>,
    title: ArcLock<Option<String>>,
    date: ArcLock<Option<String>>,
    time: ArcLock<Option<String>>,
    origin: ArcLock<Option<String>>,
    user: ArcLock<Option<String>>,
    style: ArcLock<Option<String>>,
    language_options: ArcLock<Vec<String>>,
    mode_options: ArcLock<Vec<Mode>>,
    standard: ArcLock<Option<String>>,
}

/// Field
///
/// ---
impl SurveyInfo {
    /// The version attribute is used to indicate the version of the Triple-S
    /// standard that applies to this specification.
    pub fn version(&self) -> RwLockReadGuard<'_, Version> {
        self.version.read()
    }

    /// Read-write view of `version` field
    pub fn version_mut(&mut self) -> RwLockWriteGuard<'_, Version> {
        self.version.write()
    }

    /// name of the survey
    pub fn name(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.name.read()
    }

    /// Read-write view of `name` field
    pub fn name_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.name.write()
    }

    /// title of the survey
    pub fn title(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.title.read()
    }

    /// Read-write view of `title` field
    pub fn title_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.title.write()
    }

    /// date of the creation
    pub fn date(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.date.read()
    }

    /// Read-write view of `date` field
    pub fn date_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.date.write()
    }

    /// time of the creation
    pub fn time(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.time.read()
    }

    /// Read-write view of `time` field
    pub fn time_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.time.write()
    }

    /// the originating system (program and operating system), for example(&self) -> RwLockReadGuard<'_,
    /// `<origin>MyProg v3, Windows 7</origin>`
    pub fn origin(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.origin.read()
    }

    /// Read-write view of `origin` field
    pub fn origin_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.origin.write()
    }

    /// user who created it
    pub fn user(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.user.read()
    }

    /// Read-write view of `user` field
    pub fn user_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.user.write()
    }

    /// html css styling
    pub fn style(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.style.read()
    }

    /// Read-write view of `style` field
    pub fn style_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.style.write()
    }

    /// ** Introduced since sss 1.2 **
    /// list of all available language options
    pub fn language_options(&self) -> RwLockReadGuard<'_, Vec<String>> {
        self.language_options.read()
    }

    /// Read-write view of `language_options` field
    pub fn language_options_mut(&mut self) -> RwLockWriteGuard<'_, Vec<String>> {
        self.language_options.write()
    }

    /// ** Introduced since sss 1.2 **
    /// list of all available `mode` options
    pub fn mode_options(&self) -> RwLockReadGuard<'_, Vec<Mode>> {
        self.mode_options.read()
    }

    /// Read-write view of `mode_options` field
    pub fn mode_options_mut(&mut self) -> RwLockWriteGuard<'_, Vec<Mode>> {
        self.mode_options.write()
    }

    /// ** Deprecated after sss version 1.2 **
    /// (Optional) indicate that some aspect of the triple-s definition meets a
    /// defined standard that may reduce the amount of checking required by an
    /// import program. The only value that option_setting may have in
    /// version 1.2 is 'standardnames'.
    pub fn standard(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.standard.read()
    }

    /// Read-write view of `standard` field
    pub fn standard_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.standard.write()
    }
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
            version: lock_it_up(version),
            name: lock_it_up(None),
            title: lock_it_up(None),
            date: lock_it_up(None),
            time: lock_it_up(None),
            origin: lock_it_up(None),
            user: lock_it_up(None),
            style: lock_it_up(None),
            language_options: lock_it_up(Vec::new()),
            mode_options: lock_it_up(Vec::new()),
            standard: lock_it_up(None),
        }
    }

    /// Set name of the survey.
    ///
    /// # Example:
    /// ```rust
    /// use triple_s::{Datamap, SurveyInfo, Version};
    ///
    /// let info = SurveyInfo::new(Version::V1_2).with_name(Some("Survey1"));
    /// let survey = Datamap::new(info);
    /// assert_eq!(survey.name(), Some("Survey1".into()))
    /// ```
    pub fn with_name(mut self, name: Option<impl Into<String>>) -> Self {
        *self.name_mut() = name.map(|item| item.into());
        self
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl SurveyInfo {
    /// The version attribute is used to indicate the version of the Triple-S
    /// standard that applies to this specification.
    #[getter(version)]
    fn py_get_version(&self) -> Version {
        self.version.read().clone()
    }

    #[setter(version)]
    fn py_set_version(&mut self, version: Bound<'_, PyAny>) -> PyResult<()> {
        let version = if let Ok(version) = version.extract::<String>() {
            version
        } else if let Ok(value) = version.extract::<f64>() {
            format!("{:.1}", value)
        } else {
            let type_name = get_type_name(&version)?;
            return Err(exceptions::PyTypeError::new_err(format!(
                "version must be a str, float or `Version` instance, got: {}",
                type_name
            )));
        };
        *self.version_mut() = Version::from_str(&version)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))?;
        Ok(())
    }

    /// name of the survey
    #[getter(name)]
    fn py_get_name(&self) -> Option<String> {
        self.name().clone()
    }

    #[setter(name)]
    fn py_set_name(&mut self, name: Option<String>) {
        *self.name_mut() = name;
    }

    /// title of the survey
    #[getter(title)]
    fn py_get_title(&self) -> Option<String> {
        self.title().clone()
    }

    #[setter(title)]
    fn py_set_title(&mut self, title: Option<String>) {
        *self.title_mut() = title;
    }

    /// date of the creation
    #[getter(date)]
    fn py_get_date(&self) -> Option<String> {
        self.date().clone()
    }

    #[setter(date)]
    fn py_set_date(&mut self, date: Option<String>) {
        *self.date_mut() = date;
    }

    /// time of the creation
    #[getter(time)]
    fn py_get_time(&self) -> Option<String> {
        self.time().clone()
    }

    #[setter(time)]
    fn py_set_time(&mut self, time: Option<String>) {
        *self.time_mut() = time;
    }

    /// the originating system (program and operating system), for example(&self) -> RwLockReadGuard<'_,
    /// `<origin>MyProg v3, Windows 7</origin>`
    #[getter(origin)]
    fn py_get_origin(&self) -> Option<String> {
        self.origin().clone()
    }

    #[setter(origin)]
    fn py_set_origin(&mut self, origin: Option<String>) {
        *self.origin_mut() = origin;
    }

    /// user who created it
    #[getter(user)]
    fn py_get_user(&self) -> Option<String> {
        self.user().clone()
    }

    #[setter(user)]
    fn py_set_user(&mut self, user: Option<String>) {
        *self.user_mut() = user;
    }

    /// html css styling
    #[getter(style)]
    fn py_get_style(&self) -> Option<String> {
        self.style().clone()
    }

    #[setter(style)]
    fn py_set_style(&mut self, style: Option<String>) {
        *self.style_mut() = style;
    }

    /// ** Introduced since sss 1.2 **
    /// list of all available language options
    #[getter(language_options)]
    fn get_language_options<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.language_options.read().iter())
    }

    #[setter(language_options)]
    fn set_language_options(&mut self, values: Vec<String>) -> PyResult<()> {
        *self.language_options_mut() = validate_language_options(values)?;
        Ok(())
    }

    /// ** Introduced since sss 1.2 **
    /// list of all available `mode` options
    #[getter(mode_options)]
    fn get_mode_options<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.mode_options().iter().cloned())
    }

    #[setter(mode_options)]
    fn set_mode_options(&mut self, values: Vec<Bound<'_, PyAny>>) -> PyResult<()> {
        *self.mode_options_mut() = parse_mode_options(values)?;
        Ok(())
    }

    /// ** Deprecated after sss version 1.2 **
    /// (Optional) indicate that some aspect of the triple-s definition meets a
    /// defined standard that may reduce the amount of checking required by an
    /// import program. The only value that option_setting may have in
    /// version 1.2 is 'standardnames'.
    #[getter(standard)]
    fn py_get_standard(&self) -> Option<String> {
        self.standard().clone()
    }

    #[setter(standard)]
    fn py_set_standard(&mut self, standard: Option<String>) {
        *self.standard_mut() = standard;
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
            version: lock_it_up(version),
            name: lock_it_up(name),
            title: lock_it_up(title),
            date: lock_it_up(date),
            time: lock_it_up(time),
            origin: lock_it_up(origin),
            user: lock_it_up(user),
            style: lock_it_up(style),
            language_options: lock_it_up(language_options),
            mode_options: lock_it_up(mode_options),
            standard: lock_it_up(standard),
        })
    }

    /// Create survey metadata from a dictionary. `version` is required;
    /// other metadata fields are optional and default to `None` or an empty
    /// options list.
    #[staticmethod]
    #[pyo3(name = "from_dict")]
    pub(crate) fn py_from_dict(content: Bound<'_, PyDict>) -> PyResult<Self> {
        let version = content.get_item("version")?.ok_or(missing_key_err("version"))?;
        let version = if let Ok(version) = version.extract::<Version>() {
            version
        } else if let Ok(version) = version.extract::<String>() {
            Version::from_str(&version)
                .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))?
        } else if let Ok(version) = version.extract::<f64>() {
            Version::from_str(&format!("{version:.1}"))
                .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))?
        } else {
            return Err(exceptions::PyTypeError::new_err(
                "version must be a Version, str, or float",
            ));
        };

        let name = content
            .get_item("name")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();
        let title = content
            .get_item("title")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();
        let date = content
            .get_item("date")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();
        let time = content
            .get_item("time")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();
        let origin = content
            .get_item("origin")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();
        let user = content
            .get_item("user")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();
        let style = content
            .get_item("style")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();
        let standard = content
            .get_item("standard")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();

        let language_options = content
            .get_item("language_options")?
            .or(content.get_item("language")?)
            .map(|item| item.extract::<Option<Vec<String>>>())
            .transpose()?
            .flatten()
            .unwrap_or_default();
        let language_options = validate_language_options(language_options)?;

        let mode_options = content
            .get_item("mode_options")?
            .map(|item| item.extract::<Option<Vec<Bound<'_, PyAny>>>>())
            .transpose()?
            .flatten()
            .unwrap_or_default();
        let mode_options = parse_mode_options(mode_options)?;

        Ok(Self {
            version: lock_it_up(version),
            name: lock_it_up(name),
            title: lock_it_up(title),
            date: lock_it_up(date),
            time: lock_it_up(time),
            origin: lock_it_up(origin),
            user: lock_it_up(user),
            style: lock_it_up(style),
            language_options: lock_it_up(language_options),
            mode_options: lock_it_up(mode_options),
            standard: lock_it_up(standard),
        })
    }

    #[pyo3(signature = () -> "dict[str, str]")]
    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);

        dict.set_item("version", self.version().clone())?;

        if let Some(value) = self.name().clone() {
            dict.set_item("name", value)?;
        };

        if let Some(value) = self.title().clone() {
            dict.set_item("title", value)?;
        }

        if let Some(value) = self.date().clone() {
            dict.set_item("date", value)?;
        };

        if let Some(value) = self.time().clone() {
            dict.set_item("time", value)?;
        };

        if let Some(value) = self.origin().clone() {
            dict.set_item("origin", value)?;
        };

        if let Some(value) = self.user().clone() {
            dict.set_item("user", value)?;
        };

        if let Some(value) = self.standard().clone() {
            dict.set_item("standard", value)?;
        };

        if let Some(value) = self.style().clone() {
            dict.set_item("style", value)?;
        };

        dict.set_item("language", self.language_options().clone())?;
        dict.set_item("mode_options", self.mode_options().clone())?;

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
