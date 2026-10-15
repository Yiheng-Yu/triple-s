//! Text values and specialised text variants.
//!
//! This module provides types for representing textual content in SSS XML.
//! A [`Text`] contains a default text value and may additionally contain
//! specialised [`Variant`] values for different languages and usage modes (from sss xml 2.0 onwards).
//! Currently only two different variant options are available: [`Mode`] and `lang`.  
//! 
//! [`Mode`]: identifies whether a variant is intended for interviewing or analysis.
//! `language`: identifies the same text but getting translated into various languages. `lang` is basically the same as `xml::lang`.
//!
//! # Example
//!
//! ```rust
//! use triple_s::text::{Text, Variant};
//!
//! let mut text = Text::new("Age");
//!
//! // adding variants
//! text.add_variant(
//!     Variant::new(
//!         "How old are you?",
//!         Some("interview"),
//!         Some("en-GB"),
//!     )?,
//! )?;
//!
//! text.add_variant(
//!     Variant::new(
//!         "Ama hiç türkçe bilmiyorum",
//!         Some("analysis"),
//!         Some("tr"),
//!     )?,
//! )?;
//!
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! Cloning [`Text`] or [`Variant`] creates another handle to the same
//! underlying data. Use [`DeepCopy::deepcopy`] when an independent copy is
//! required.

#[cfg(feature = "pyo3")]
use std::hash::DefaultHasher;
use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    str::FromStr,
    sync::Arc,
};

use anyhow::{Context, Result};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*, types::PyTuple};
use quick_xml::{
    events::{BytesText, Event},
    writer::Writer,
};
use strum::{EnumIter, EnumString};

#[cfg(feature = "pyo3")]
use crate::utils::{normalize_py_index, py_value_error, single_quote};
use crate::{
    prelude::*,
    utils::{ArcLock, lock_it_up, validate_language_tag},
};

#[doc = include_str!("../doc/mode.md")]
#[allow(missing_docs)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Debug, PartialEq, Hash, Clone, EnumString, Eq, EnumIter)]
#[strum(ascii_case_insensitive)]
pub enum Mode {
    #[strum(serialize = "analysis")]
    Analysis,
    #[strum(serialize = "interview")]
    Interview,
}

impl Flag for Mode {
    fn to_string(&self) -> String {
        match self {
            Self::Analysis => "analysis".into(),
            Self::Interview => "interview".into(),
        }
    }
}

impl DeepCopy for Mode {
    #[doc = include_str!("../doc/rs_deep_copy_impl.md")]
    fn deepcopy(&self) -> Self {
        self.clone()
    }
}

/// `Text` represents a default text value together with zero or more
/// specialised [`Variant`] values.
///
#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Text", module = "triple_s", from_py_object)
)]
#[derive(Debug, Default)]
pub struct Text {
    /// the default value
    pub(crate) value: ArcLock<String>,
    /// variants
    pub(crate) variants: ArcLock<Vec<Variant>>,
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
    /// variants comparisons are order-independentent (i.e variant [A, B] is
    /// considered to be equal with variant [B, A])
    fn eq(&self, other: &Self) -> bool {
        if self.value.read().as_str() != other.value.read().as_str() {
            return false;
        };
        self.has_same_variants(other)
    }
}

impl From<Arc<str>> for Text {
    fn from(value: Arc<str>) -> Self {
        Self::new(value.as_ref())
    }
}

/// # Fields
///
/// ---
impl Text {
    /// Default value
    pub fn value(&self) -> String {
        self.value.read().to_string()
    }

    /// Returns a mutable `value` field for updating it.
    ///
    /// # Example
    /// ```rust
    /// use triple_s::text::Text;
    ///
    /// let mut example = Text::new("Original text");
    ///
    /// *example.value_mut() = "Updated text".into();
    /// assert_eq!(example.value(), "Updated text");
    /// ```
    pub fn value_mut(&mut self) -> RwLockWriteGuard<'_, String> {
        self.value.write()
    }

    /// list of text variants
    pub fn variants(&self) -> RwLockReadGuard<'_, Vec<Variant>> {
        self.variants.read()
    }

    /// Returns a mutable `variants` field for updating it.
    ///
    /// # Example
    /// ```rust
    /// use triple_s::text::{Text, Variant};
    ///
    /// let mut example = Text::new("Hello");
    /// let variant = Variant::new("bir iki üç", None, Some("tr")).unwrap();
    ///
    /// example.variants_mut().push(variant);
    /// assert_eq!(example.variants()[0].value(), "bir iki üç");
    /// ```
    pub fn variants_mut(&mut self) -> RwLockWriteGuard<'_, Vec<Variant>> {
        self.variants.write()
    }
}

impl Text {
    /// Create default text with no language or mode variants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::text::Text;
    ///
    /// let text = Text::new("variant a");
    /// ```
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: lock_it_up(value.into()),
            variants: lock_it_up(Vec::new()),
        }
    }

    /// Adds a specialised text variant. 
    /// Returns an error if a variant with the same mode and language is already
    /// present.
    ///
    /// # Example
    ///
    /// ```rust
    /// use triple_s::text::{Text, Variant};
    ///
    /// let mut text = Text::new("Age");
    /// let variant = Variant::new(
    ///     "How old are you?",
    ///     Some("interview"),
    ///     Some("en-GB"),
    /// )?;
    ///
    /// text.add_variant(variant)?;
    ///
    /// assert_eq!(text.variants().len(), 1);
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn add_variant(&mut self, variant: Variant) -> Result<()> {
        let copied = variant.deepcopy();
        anyhow::ensure!(
            !self.variants.read().iter().any(|item| item.has_same_attribute(&copied)),
            "A variant with this mode and language already exists"
        );
        self.variants.write().push(copied);
        Ok(())
    }

    /// Removes and returns the variant at the given index.
    /// This method behaves simlar to [`Vec::remove`]:
    /// - Returns [`None`] if `index` is outside the bounds of the variants list.
    /// - Removing a variant shifts all variants after it one position to the left.
    ///
    /// # Example
    ///
    /// ```rust
    /// use triple_s::text::{Text, Variant};
    ///
    /// let mut text = Text::new("Age");
    /// text.add_variant(
    ///     Variant::new("How old are you?", Some("interview"), Some("en-GB"))?,
    /// )?;
    ///
    /// let removed = text.remove_variant(0);
    ///
    /// assert!(removed.is_some());
    /// assert!(text.variants().is_empty());
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn remove_variant(&mut self, index: usize) -> Option<Variant> {
        if index < self.variants.read().len() {
            Some(self.variants.write().remove(index))
        } else {
            None
        }
    }

    /// Copy and validate inputs before replacing membership.
    pub fn set_variants(&mut self, mut variants: Vec<Variant>) -> Result<()> {
        // drop duplicated variants first
        let mut unique = HashSet::new();
        variants.retain(|item| unique.insert((item.value(), item.mode(), item.language())));

        let variants =
            variants.into_iter().map(|item| item.deepcopy()).collect::<Vec<Variant>>();
        self.variants = lock_it_up(variants);
        Ok(())
    }

    /// Returns whether this text and `other` contain the same variants.
    pub fn has_same_variants(&self, other: &Self) -> bool {
        if self.variants.read().len() != other.variants.read().len() {
            false
        } else if self.variants.read().is_empty() {
            true
        } else {
            self.variants
                .read()
                .iter()
                .find(|self_var| {
                    other
                        .variants
                        .read()
                        .iter()
                        .find(|other_var| other_var == self_var)
                        .is_some()
                })
                .is_some()
        }
    }

    /// Returns the default text value in the classic SSS representation.
    /// This method does not output text variants since its not supported
    /// in triple-s classic.
    /// # Example
    ///
    /// ```rust
    /// use triple_s::text::{Text, Variant};
    ///
    /// let mut text = Text::new("Age");
    /// text.add_variant(
    ///     Variant::new("How old are you?", Some("interview"), Some("en-GB"))?,
    /// )?;
    ///
    /// assert_eq!(text.to_classic(), "Age");
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn to_classic(&self) -> String {
        self.value.read().clone()
    }

    /// Writes this text and its variants as XML.
    /// `tag` specifies the name of the enclosing XML element, while `attributes`
    /// contains attributes to add to that element.
    pub(crate) fn write_xml<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        tag: &str,
        attributes: Vec<(&str, &str)>,
    ) -> Result<(), std::io::Error> {
        let value = self.value();
        writer.create_element(tag).with_attributes(attributes).write_inner_content(
            |child_writer| {
                child_writer.write_event(Event::Text(BytesText::new(&value)))?;
                for variant in self.variants.read().iter() {
                    variant.write_xml(child_writer)?;
                }
                Ok(())
            },
        )?;

        Ok(())
    }
}

impl DeepCopy for Text {
    #[doc = include_str!("../doc/rs_deep_copy_impl.md")]
    fn deepcopy(&self) -> Self {
        let variants = self
            .variants
            .read()
            .clone()
            .iter()
            .map(Variant::deepcopy)
            .collect::<Vec<Variant>>();

        Self {
            value: self.value.deepcopy(),
            variants: lock_it_up(variants),
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Text {
    /// Create default text with an optional list of variants.
    ///
    /// # Examples
    ///
    /// ```python
    /// from triple_s import Text, Variant
    ///
    /// variant = Variant("original", mode="analysis", language="en-GB")
    /// text = Text("original", variants=[variant])
    /// ```
    #[new]
    #[pyo3(signature = (value, variants=None))]
    fn py_new(value: String, variants: Option<Vec<Variant>>) -> PyResult<Self> {
        let mut text = Self::new(value);
        text.set_variants(variants.unwrap_or_default()).map_err(py_value_error)?;
        Ok(text)
    }

    pub(crate) fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<String> {
        let value = self.value.read();
        let mut chars = value.chars();
        let mut preview: String = chars.by_ref().take(25).collect();
        if chars.next().is_some() {
            preview.push_str("...");
        }
        let preview = single_quote(preview);

        let variants = self.variants.read();
        if variants.is_empty() {
            return Ok(format!("Text(value={})", preview));
        }
        let variants = variants
            .iter()
            .map(|variant| variant.__repr__(py))
            .collect::<PyResult<Vec<_>>>()?
            .join(", ");

        Ok(format!(
            "Text(\n  value={},\n  variants=[{}]\n)",
            preview, variants
        ))
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

    /// Default value
    #[getter(value)]
    fn py_get_value(&self) -> String {
        self.value()
    }

    #[setter(value)]
    fn py_set_value(&mut self, value: String) {
        *self.value.write() = value;
    }

    // makes a shallow copy
    #[getter]
    fn get_variants<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let variants_view = self.variants.read().clone().into_iter().map(|item| item.clone());
        PyTuple::new(py, variants_view)
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
        let index = normalize_py_index(index, self.variants.read().len())
            .ok_or_else(|| exceptions::PyIndexError::new_err("variant index out of range"))?;
        Ok(self.variants.write().remove(index))
    }
}

/// Introduced since sss 2.0 onwards.
///
/// Official documentation:
/// 
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
#[derive(Debug, Default)]
pub struct Variant {
    value: ArcLock<String>,
    mode: ArcLock<Option<Mode>>,
    language: ArcLock<Option<String>>,
}

impl Clone for Variant {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            mode: self.mode.clone(),
            language: self.language.clone(),
        }
    }
}

impl DeepCopy for Variant {
    #[doc = include_str!("../doc/rs_deep_copy_impl.md")]
    fn deepcopy(&self) -> Self {
        Self {
            value: self.value.deepcopy(),
            mode: lock_it_up(self.mode()),
            language: self.language.deepcopy(),
        }
    }
}

impl PartialEq for Variant {
    fn eq(&self, other: &Self) -> bool {
        self.has_same_attribute(other) && self.value() == other.value()
    }
}

impl Eq for Variant {}

impl Hash for Variant {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value().hash(state);
        self.mode().hash(state);
        self.language().hash(state);
    }
}

/// # Fields
///
/// ---
impl Variant {
    /// content of the text variant
    pub fn value(&self) -> String {
        self.value.read().to_string()
    }

    /// Returns a mutable `value` field for updating it.
    ///
    /// # Example
    /// ```rust
    /// use triple_s::text::Variant;
    ///
    /// let mut example = Variant::new("Original text", None, None).unwrap();
    ///
    /// *example.value_mut() = "Updated text".into();
    /// assert_eq!(example.value(), "Updated text");
    /// ```
    pub fn value_mut(&mut self) -> RwLockWriteGuard<'_, String> {
        self.value.write()
    }

    #[doc = include_str!("../doc/mode.md")]
    pub fn mode(&self) -> Option<Mode> {
        self.mode.read().clone()
    }

    /// Returns a mutable `mode` field for updating it.
    ///
    /// # Example
    /// ```rust
    /// use triple_s::text::{Mode, Variant};
    ///
    /// let mut example = Variant::new("Age", None, None).unwrap();
    ///
    /// *example.mode_mut() = Some(Mode::Analysis);
    /// assert_eq!(example.mode(), Some(Mode::Analysis));
    /// ```
    pub fn mode_mut(&mut self) -> RwLockWriteGuard<'_, Option<Mode>> {
        self.mode.write()
    }

    /// Corresponds to the `xml::lang attribute of a <text> node.
    pub fn language(&self) -> Option<String> {
        self.language.read().as_ref().map(|item| item.clone())
    }

    /// Returns a mutable `language` field for updating it.
    ///
    /// Direct updates bypass language tag validation. Use
    /// [`Self::set_language`] to validate the new language tag.
    ///
    /// # Example
    /// ```rust
    /// use triple_s::text::Variant;
    ///
    /// let mut example = Variant::new("Hello", None, None).unwrap();
    ///
    /// *example.language_mut() = Some("en-GB".into());
    /// assert_eq!(example.language(), Some("en-GB".into()));
    /// ```
    pub fn language_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.language.write()
    }
}

impl Variant {
    /// Create a text variant with optional mode and language attributes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::text::Variant;
    ///
    /// let variant = Variant::new("variant a", Some("interview"), Some("gb")).unwrap();
    /// ```
    pub fn new(
        value: impl Into<String>,
        mode: Option<&str>,
        language: Option<&str>,
    ) -> Result<Self> {
        let mut instance = Self {
            value: lock_it_up(value.into()),
            mode: lock_it_up(None),
            language: lock_it_up(None),
        };

        instance.set_mode(mode)?;
        instance.set_language(language)?;
        Ok(instance)
    }

    /// check if `other` has same variant as `self`.
    /// very handy.
    /// 
    /// ```rust
    /// use triple_s::Variant;
    ///
    /// let a = Variant::new("value 1", Some("interview"), None)?;
    /// let b = Variant::new("value 2", Some("interview"), None)?;
    /// let c = Variant::new("value 2", Some("analysis"), None)?;
    ///
    /// assert!(a.has_same_attribute(&b));
    /// assert!(!a.has_same_attribute(&c));
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn has_same_attribute(&self, other: &Self) -> bool {
        self.mode() == other.mode() && self.language() == other.language()
    }

    pub(crate) fn set_mode(&mut self, mode: Option<&str>) -> Result<()> {
        *self.mode.write() = mode
            .map(Mode::from_str)
            .transpose()
            .with_context(|| format!("Expect one of: {:?}", Mode::available_options()))?;
        Ok(())
    }

    /// updates `language` attribute, this method will validate language tag beforehand.
    pub fn set_language(&mut self, language: Option<&str>) -> Result<()> {
        let language =
            language.map(|s| validate_language_tag(s).map(str::to_owned)).transpose()?;
        // Update the shared value, rather than replacing the shared pointer.
        *self.language.write() = language;
        Ok(())
    }

    pub(crate) fn write_xml<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), std::io::Error> {
        let mode = self.mode().map(|mode| mode.to_string());
        let language = self.language();
        let text = self.value();

        let mut element = writer.create_element("text");
        if let Some(mode) = &mode {
            element = element.with_attribute(("mode", mode.as_str()));
        }
        if let Some(language) = &language {
            element = element.with_attribute(("xml:lang", language.as_str()));
        }

        element.write_text_content(BytesText::new(&text))?;
        Ok(())
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Variant {
    /// Create a text variant.
    /// 
    /// Mode and language default to `None`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from triple_s import Variant
    ///
    /// variant = Variant("original", mode="analysis", language="en-GB")
    /// ```
    #[new]
    #[pyo3(signature = (value:"str", mode:"'analysis'|'interview'|None"=None, language:"str|None"=None))]
    fn py_new(value: String, mode: Option<&str>, language: Option<&str>) -> PyResult<Self> {
        Self::new(value, mode, language).map_err(py_value_error)
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
        let value = self.value().into_pyobject(py)?.repr()?;
        let mode = self.py_get_mode().into_pyobject(py)?.repr()?;
        let language = self.language().into_pyobject(py)?.repr()?;
        Ok(format!(
            "Variant(value={value}, mode={mode}, language={language})"
        ))
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.eq(other)
    }

    fn __hash__(&self) -> u64 {
        let mut state = DefaultHasher::new();
        self.hash(&mut state);
        state.finish()
    }

    #[getter(value)]
    fn py_get_value(&self) -> String {
        self.value()
    }

    #[setter(value)]
    fn py_set_value(&mut self, value: String) {
        *self.value_mut() = value;
    }

    #[doc = include_str!("../doc/mode.md")]
    #[getter(mode)]
    fn py_get_mode(&self) -> Option<String> {
        self.mode().map(|mode| mode.to_string())
    }

    #[setter(mode)]
    fn py_set_mode(&mut self, mode: Option<&str>) -> PyResult<()> {
        self.set_mode(mode).map_err(py_value_error)
    }

    /// Corresponds to the `xml::lang attribute of a <text> node.
    #[getter(language)]
    fn py_get_language(&self) -> Option<String> {
        self.language()
    }

    #[setter(language)]
    fn py_set_language(&mut self, language: Option<&str>) -> PyResult<()> {
        self.set_language(language).map_err(py_value_error)
    }

    /// makes deepcopy of the current instance.
    fn copy(&self) -> Self {
        self.deepcopy()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::member::test::normalize_xml;

    #[test]
    fn test_variant_equal() {
        let a = Variant::new("variant a", Some("interview"), Some("gb")).unwrap();
        let b = Variant::new("variant a", Some("interview"), Some("gb")).unwrap();
        assert_eq!(a, b)
    }

    #[test]
    fn test_text_equal() {
        let a = Text::new("variant a");
        let b = Text::new("variant a");
        assert_eq!(a, b)
    }

    #[test]
    fn test_variant_write_xml() {
        let variant = Variant::new("variant a", Some("interview"), Some("gb")).unwrap();
        let mut writer = Writer::new(Vec::new());
        writer
            .create_element("label")
            .write_inner_content(|inner| variant.write_xml(inner))
            .unwrap();

        let content = String::from_utf8(writer.into_inner()).unwrap();
        let correct =
            r#"<label><text mode="interview" xml:lang="gb">variant a</text></label>"#;
        assert_eq!(content, correct);
    }

    #[test]
    fn test_text_write_xml() {
        let mut text = Text::new("variant a");
        let variant = Variant::new("variant a", Some("interview"), Some("gb")).unwrap();
        text.add_variant(variant).unwrap();

        let mut writer = Writer::new(Vec::new());
        text.write_xml(&mut writer, "some_tag_name", Vec::new()).unwrap();
        let content = String::from_utf8(writer.into_inner()).unwrap();
        let correct = r#"<some_tag_name>variant a<text mode="interview" xml:lang="gb">variant a</text></some_tag_name>"#;
        assert_eq!(normalize_xml(&content), normalize_xml(correct));
    }
}
