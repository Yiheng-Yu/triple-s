use std::str::FromStr;

use anyhow::{Context, Result};

#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*};

use crate::{
    entities::Mode, prelude::*, utils::{ArcLock, lock_it_up, validate_language_tag},
};

#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Text", module = "triple_s", from_py_object)
)]
#[derive(Debug, Clone)]
pub struct Text {
    /// Default text
    pub value: ArcLock<String>,
    /// text variants
    pub variants: ArcLock<Vec<ArcLock<Variant>>>,
}

/// From sss xml 2.0 onwards supports multi-language
///
/// For sss xml 2.0:
/// ```text
/// The text for a survey <title>, variable <label> or values
/// <value> element may contain any number of specialised text elements.
/// ```
///
/// Example sss xml 2.0 text, note that one entry can contain both `label` and
/// `language` attribute
/// ```xml
/// <label>Age
///     <text xml:lang="en-GB" mode="interview">How old are you?</text>
///     <text xml:lang="fr" mode="interview">Quel est votre âge?</text>
///     <text xml:lang="en-GB" mode="analysis">Age of respondent</text>
///     <text xml:lang="fr" mode="analysis">Âge de répondant</text>
/// </label>
/// ```
#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Variant", module = "triple_s", from_py_object)
)]
#[derive(Debug, Clone)]
pub struct Variant {
    pub value: ArcLock<String>,
    mode: Mode,
    language: ArcLock<Option<String>>,
}

impl PartialEq for Variant {
    fn eq(&self, other: &Self) -> bool {
        (self.has_same_attribute(other))
            & (self.value.read().as_str() == other.value.read().as_str())
    }
}

impl Variant {
    /// check if instance of `other` has the same attributes as `self`
    pub fn has_same_attribute(&self, other: &Self) -> bool {
        (self.mode == other.mode)
            & (self.language.read().as_ref() == other.language.read().as_ref())
    }

    #[inline]
    pub fn get_mode(&self) -> &Mode {
        &self.mode
    }

    #[inline]
    pub fn set_mode(&mut self, mode: Option<&str>) -> Result<()> {
        let new = if let Some(mode) = mode {
            Mode::from_str(mode).with_context(|| {
                format!("Expect to be one of: {:?}", Mode::available_options())
            })?
        } else {
            Mode::None
        };
        self.mode = new;
        Ok(())
    }

    #[inline]
    pub fn get_language(&self) -> Option<String> {
        self.language.read().clone()
    }

    #[inline]
    pub fn set_language(&mut self, language: Option<String>) -> Result<()> {
        let language = language.map(|s| validate_language_tag(&s).map(|s| s.to_string()))
            .transpose()?;

        self.language = lock_it_up(language);
        Ok(())
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Variant {
    #[getter(get_mode)]
    fn py_get_mode(&self) -> String {
        self.mode.as_str().to_string()
    }

    #[setter(set_mode)]
    fn py_set_mode(&mut self, mode: Option<String>) -> PyResult<()> {
        self.set_mode(mode.as_ref().map(|s| s.as_str()))
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
    }

    #[getter(get_language)]
    fn py_get_language(&self) -> Option<String> {
        self.get_language()
    }

    #[setter(set_language)]
    fn py_set_language(&mut self, language: Option<String>) -> PyResult<()> {
        self.set_language(language)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
    }
}
