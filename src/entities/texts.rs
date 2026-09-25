use std::str::FromStr;

use anyhow::{Context, Result};
#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*, types::PyTuple};
use rayon::prelude::*;

#[cfg(feature = "pyo3")]
use crate::utils::normalize_py_index;
use crate::{
    entities::Mode,
    prelude::*,
    utils::{ArcLock, lock_it_up, validate_language_tag},
};

#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Text", module = "triple_s", from_py_object)
)]
#[derive(Debug)]
pub struct Text {
    pub value: ArcLock<String>,
    pub variants: Vec<Variant>,
}

impl Clone for Text {
    /// cloning the pointer to the actual data behind,
    /// this clone() will create shareable, mutable copy of the current
    /// instance.
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            variants: self.variants.clone(),
        }
    }
}

impl PartialEq for Text {
    /// compares current instance with other instance. 
    /// variants comparisons are order-independentent (i.e variant [A, B] is considered to be equal with variant [B, A])
    fn eq(&self, other: &Self) -> bool {
        if self.value.read().as_str() != other.value.read().as_str() {
            return false;
        };
        self.has_same_variants(other)
    }
}

impl Text {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: lock_it_up(value.into()),
            variants: Vec::new(),
        }
    }

    pub fn get_value(&self) -> String {
        self.value.read().clone()
    }

    pub fn set_value(&self, value: impl Into<String>) {
        *self.value.write() = value.into();
    }

    pub fn add_variant(&mut self, variant: Variant) -> Result<()> {
        let copied = variant.deepcopy();
        anyhow::ensure!(
            !self.variants.iter().any(|item| item.has_same_attribute(&copied)),
            "A variant with this mode and language already exists"
        );
        self.variants.push(copied);
        Ok(())
    }

    pub fn remove_variant(&mut self, index: usize) -> Option<Variant> {
        if index < self.variants.len() {
            Some(self.variants.remove(index))
        } else {
            None
        }
    }

    /// Copy and validate inputs before replacing membership.
    pub fn set_variants(&mut self, variants: Vec<Variant>) -> Result<()> {
        let mut replacement = Self::new("");
        for variant in variants {
            replacement.add_variant(variant)?;
        }
        self.variants = replacement.variants;
        Ok(())
    }

    pub fn has_same_variants(&self, other: &Self) -> bool {
        if self.variants.len() != other.variants.len() {
            return false;
        } else if self.variants.len() == 0 {
            return true
        } else {
            self.variants
                .par_iter()
                .find_first(|self_var| {
                    other
                        .variants
                        .par_iter()
                        .find_first(|other_var| other_var == self_var)
                        .is_some()
                })
                .is_some()
        }
    }
}

impl DeepCopy for Text {
    fn deepcopy(&self) -> Self {
        Self {
            value: self.value.deepcopy(),
            variants: self.variants.clone().iter().map(Variant::deepcopy).collect(),
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Text {
    #[new]
    #[pyo3(signature = (value, variants=None))]
    fn py_new(value: String, variants: Option<Vec<Variant>>) -> PyResult<Self> {
        let mut text = Self::new(value);
        text.set_variants(variants.unwrap_or_default()).map_err(py_value_error)?;
        Ok(text)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self)
    }

    fn __copy__(&self) -> Self {
        self.deepcopy()
    }

    fn __deepcopy__(&self) -> Self {
        self.deepcopy()
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.eq(other)
    }

    #[getter(value)]
    fn py_get_value(&self) -> String {
        self.get_value()
    }

    #[setter(value)]
    fn py_set_value(&self, value: String) {
        self.set_value(value);
    }

    /// Immutable membership snapshot, containing shared mutable variants.
    #[getter]
    fn get_variants<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, self.variants.iter().cloned())
    }

    #[setter(variants)]
    fn py_set_variants(&mut self, variants: Vec<Variant>) -> PyResult<()> {
        self.set_variants(variants).map_err(py_value_error)
    }

    #[pyo3(name = "add_variant", signature = (variant, mode=None, language=None))]
    fn py_add_variant(
        &mut self,
        variant: &Bound<'_, PyAny>,
        mode: Option<&str>,
        language: Option<&str>,
    ) -> PyResult<()> {
        let variant = if let Ok(variant) = variant.extract::<Variant>() {
            variant
        } else if let Ok(value) = variant.extract::<String>() {
            Variant::new(value, mode, language).map_err(py_value_error)?
        } else {
            let type_received = variant.get_type().name()?.extract::<String>()?;

            return Err(exceptions::PyTypeError::new_err(format!(
                "variant must be a Variant or str, received: '{}",
                type_received
            )));
        };
        self.add_variant(variant).map_err(py_value_error)
    }

    #[pyo3(name = "remove_variant")]
    fn py_remove_variant(&mut self, index: isize) -> PyResult<Variant> {
        let index = normalize_py_index(index, self.variants.len())
            .ok_or_else(|| exceptions::PyIndexError::new_err("variant index out of range"))?;
        Ok(self.variants.remove(index))
    }
}

#[cfg(feature = "pyo3")]
fn py_value_error(error: anyhow::Error) -> PyErr {
    exceptions::PyValueError::new_err(error.to_string())
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
/// `clone()` shares fields; `deepcopy()` creates independent fields.
#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Variant", module = "triple_s", from_py_object)
)]
#[derive(Debug, Clone, Default)]
pub struct Variant {
    pub value: ArcLock<String>,
    mode: ArcLock<Mode>,
    language: ArcLock<Option<String>>,
}

impl DeepCopy for Variant {
    fn deepcopy(&self) -> Self {
        Self {
            value: self.value.deepcopy(),
            mode: lock_it_up(self.get_mode()),
            language: self.language.deepcopy(),
        }
    }
}

impl PartialEq for Variant {
    fn eq(&self, other: &Self) -> bool {
        self.has_same_attribute(other) && self.get_value() == other.get_value()
    }
}

impl Variant {
    pub fn new(
        value: impl Into<String>,
        mode: Option<&str>,
        language: Option<&str>,
    ) -> Result<Self> {
        let instance = Self {
            value: lock_it_up(value.into()),
            mode: lock_it_up(Mode::None),
            language: lock_it_up(None),
        };

        instance.set_mode(mode)?;
        instance.set_language(language)?;
        Ok(instance)
    }

    pub fn has_same_attribute(&self, other: &Self) -> bool {
        self.get_mode() == other.get_mode() && self.get_language() == other.get_language()
    }

    pub fn get_value(&self) -> String {
        self.value.read().clone()
    }

    pub fn set_value(&self, value: impl Into<String>) {
        let value = value.into();
        *self.value.write() = value;
    }

    pub fn get_mode(&self) -> Mode {
        self.mode.read().clone()
    }

    pub fn set_mode(&self, mode: Option<&str>) -> Result<()> {
        let mode = mode
            .map(Mode::from_str)
            .transpose()
            .with_context(|| format!("Expect one of: {:?}", Mode::available_options()))?
            .unwrap_or_default();
        *self.mode.write() = mode;
        Ok(())
    }

    pub fn get_language(&self) -> Option<String> {
        self.language.read().clone()
    }

    pub fn set_language(&self, language: Option<&str>) -> Result<()> {
        let language =
            language.map(|s| validate_language_tag(&s).map(str::to_owned)).transpose()?;
        // Update the shared value, rather than replacing the shared pointer.
        *self.language.write() = language;
        Ok(())
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Variant {
    #[new]
    #[pyo3(signature = (value:"str", mode:"'analysis'|'interview'|None"=None, language:"str|None"=None))]
    fn py_new(value: String, mode: Option<&str>, language: Option<&str>) -> PyResult<Self> {
        Self::new(value, mode, language).map_err(py_value_error)
    }

    #[getter(value)]
    fn py_get_value(&self) -> String {
        self.get_value()
    }

    #[setter(value)]
    fn py_set_value(&self, value: String) {
        self.set_value(value);
    }

    #[getter(mode)]
    fn py_get_mode(&self) -> Option<String> {
        match self.get_mode() {
            Mode::None => None,
            mode => Some(mode.as_str().to_owned()),
        }
    }

    #[setter(mode)]
    fn py_set_mode(&self, mode: Option<&str>) -> PyResult<()> {
        self.set_mode(mode).map_err(py_value_error)
    }

    #[getter(language)]
    fn py_get_language(&self) -> Option<String> {
        self.get_language()
    }

    #[setter(language)]
    fn py_set_language(&self, language: Option<&str>) -> PyResult<()> {
        self.set_language(language).map_err(py_value_error)
    }

    /// makes deepcopy of the current instance.
    fn copy(&self) -> Self {
        self.deepcopy()
    }

    /// makes deepcopy of the current instance.
    fn __copy__(&self) -> Self {
        self.deepcopy()
    }

    /// makes deepcopy of the current instance.
    fn __deepcopy__(&self, _memo: &Bound<'_, PyAny>) -> Self {
        self.deepcopy()
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let value = self.get_value().into_pyobject(py)?.repr()?;
        let mode = self.py_get_mode().into_pyobject(py)?.repr()?;
        let language = self.get_language().into_pyobject(py)?.repr()?;
        Ok(format!(
            "Variant(value={value}, mode={mode}, language={language})"
        ))
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_equal_variant() {
        let a = Variant::new("variant a", Some("interview"), Some("gb")).unwrap();
        let b = Variant::new("variant a", Some("interview"), Some("gb")).unwrap();
        assert_eq!(a, b)
    }

    #[test]
    fn test_equal_text() {
        let a = Text::new("variant a");
        let b = Text::new("variant a");
        assert_eq!(a, b)
    }
}