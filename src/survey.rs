use std::{collections::BTreeMap, fmt::Write};

use rayon::prelude::*;
use strum::EnumString;

use crate::error::{OperationError, UnsupportedFeature};

#[derive(Debug, PartialEq, Clone)]
pub struct Survey {
    pub title: Option<String>,
    pub meta: Meta,
    pub record: Record,
}

impl Survey {
    pub fn empty_classic() -> Self {
        Self {
            title: None,
            meta: Meta::default_classic(),
            record: Record::default(),
        }
    }

    pub fn empty_xml() -> Self {
        todo!()
        // Self {
        //     title: None,
        //     meta: Meta::default_classic(),
        //     record: Record::default(),
        // }
    }

    /// find question via question name, case insensitive
    pub fn get_question(&self, name: &str) -> Option<&Question> {
        self.record
            .content
            .par_iter()
            .find_first(|question| question.name.to_uppercase() == name.to_uppercase())
    }

    pub fn to_sss_classic(&self) -> String {
        let mut output = String::from("SSS\n");
        output.push_str(&self.meta.to_sss_classic());
        append_line(&mut output, "SURVEY");
        if let Some(title) = self.title.as_ref() {
            append_line(&mut output, format!("TITLE {}", quote(title)));
        }
        output.push_str(&self.record.to_sss_classic());
        append_line(&mut output, "END SURVEY");
        append_line(&mut output, "END SSS");
        output
    }
}

/// Major SSS file version
/// The main difference between different xml generations are on survey meta setting,
/// where <variable> tags aren't much different from one another.
/// Ref: https://triple-s.org/documentation/manuals/
/// Structured as Version::{Major Version}("version_text")
#[derive(Debug, PartialEq, Clone)]
pub enum Version {
    /// sss classic
    Classic(Option<String>),
    /// sss xml 1.x
    GenOne(Option<String>),
    /// sss xml 2.x
    GenTwo(Option<String>),
    /// sss xml 3.x
    GenThree(Option<String>),
}

impl Version {
    pub fn get_version_name(&self) -> Option<&str> {
        match &self {
            Self::Classic(value) => value.as_ref().map(|s| s.as_str()),
            Self::GenOne(value) => value.as_ref().map(|s| s.as_str()),
            Self::GenTwo(value) => value.as_ref().map(|s| s.as_str()),
            Self::GenThree(value) => value.as_ref().map(|s| s.as_str()),
        }
    }

    pub fn set_version_name<S>(&mut self, new_value: Option<S>)
    where
        S: Into<String>,
    {
        let new_value = new_value.map(|s| s.into());
        match self {
            Self::Classic(value) => *value = new_value.clone(),
            Self::GenOne(value) => *value = new_value.clone(),
            Self::GenTwo(value) => *value = new_value.clone(),
            Self::GenThree(value) => *value = new_value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Meta {
    Classic(Box<Classic>),
}

impl Meta {
    pub fn default_classic() -> Self {
        let meta = Classic::default();
        Self::Classic(Box::new(meta))
    }

    pub fn to_sss_classic(&self) -> String {
        match self {
            Self::Classic(item) => item.to_sss_classic(),
        }
    }

    pub fn version(&self) -> Result<Option<&str>, UnsupportedFeature> {
        match &self {
            Self::Classic(member) => Ok(member.version.get_version_name()),
        }
    }

    pub fn set_version(
        &mut self,
        value: Option<impl Into<String>>,
    ) -> Result<(), UnsupportedFeature> {
        match self {
            Self::Classic(member) => member.version.set_version_name(value),
        };

        Ok(())
    }

    pub fn date(&self) -> Result<Option<&str>, UnsupportedFeature> {
        match &self {
            Self::Classic(member) => Ok(member.date.as_ref().map(|s| s.as_str())),
        }
    }

    pub fn set_date(
        &mut self,
        value: Option<impl Into<String>>,
    ) -> Result<(), UnsupportedFeature> {
        match self {
            Self::Classic(member) => member.date = value.map(|s| s.into()),
        }
        Ok(())
    }

    pub fn time(&self) -> Result<Option<&str>, UnsupportedFeature> {
        match &self {
            Self::Classic(member) => Ok(member.time.as_ref().map(|s| s.as_str())),
        }
    }

    pub fn set_time(
        &mut self,
        value: Option<impl Into<String>>,
    ) -> Result<(), UnsupportedFeature> {
        match self {
            Self::Classic(member) => member.time = value.map(|s| s.into()),
        }
        Ok(())
    }

    pub fn origin(&self) -> Result<Option<&str>, UnsupportedFeature> {
        match &self {
            Self::Classic(member) => Ok(member.origin.as_ref().map(|s| s.as_str())),
        }
    }

    pub fn set_origin(
        &mut self,
        value: Option<impl Into<String>>,
    ) -> Result<(), UnsupportedFeature> {
        match self {
            Self::Classic(member) => member.origin = value.map(|s| s.into()),
        }
        Ok(())
    }

    pub fn user(&self) -> Result<Option<&str>, UnsupportedFeature> {
        match &self {
            Self::Classic(member) => Ok(member.user.as_ref().map(|s| s.as_str())),
        }
    }

    pub fn set_user(
        &mut self,
        value: Option<impl Into<String>>,
    ) -> Result<(), UnsupportedFeature> {
        match self {
            Self::Classic(member) => member.user = value.map(|s| s.into()),
        }
        Ok(())
    }

    pub fn style(&self) -> Result<Option<&str>, UnsupportedFeature> {
        match &self {
            Self::Classic(member) => Ok(member.style.as_ref().map(|s| s.as_str())),
        }
    }

    pub fn set_style(
        &mut self,
        value: Option<impl Into<String>>,
    ) -> Result<(), UnsupportedFeature> {
        match self {
            Self::Classic(member) => member.style = value.map(|s| s.into()),
        }
        Ok(())
    }

    pub fn standard(&self) -> Result<Option<&str>, UnsupportedFeature> {
        match &self {
            Self::Classic(member) => Ok(member.standard.as_ref().map(|s| s.as_str())),
        }
    }

    pub fn set_standard(
        &mut self,
        value: Option<impl Into<String>>,
    ) -> Result<(), UnsupportedFeature> {
        match self {
            Self::Classic(member) => member.standard = value.map(|s| s.into()),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Xml {
    pub version: Version,
    pub date: Option<String>,
    pub time: Option<String>,
    pub origin: Option<String>,
    pub user: Option<String>,
    pub style: Option<String>,
    // pub languages:
    pub mods: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Classic {
    pub version: Version,
    pub date: Option<String>,
    pub time: Option<String>,
    pub origin: Option<String>,
    pub user: Option<String>,
    pub style: Option<String>,
    pub standard: Option<String>,
}

impl Default for Classic {
    fn default() -> Self {
        Self {
            version: Version::Classic(None),
            date: None,
            time: None,
            origin: None,
            user: None,
            style: None,
            standard: None,
        }
    }
}

impl Classic {
    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        if let Some(value) = &self.version.get_version_name() {
            append_line(&mut output, format!("VERSION {}", value));
        }
        for (name, value) in [
            ("DATE", &self.date),
            ("TIME", &self.time),
            ("ORIGIN", &self.origin),
            ("USER", &self.user),
        ] {
            if let Some(value) = value {
                append_line(&mut output, format!("{name} {}", quote(value)));
            }
        }
        for (name, value) in [("STANDARD", &self.standard), ("STYLE", &self.style)] {
            if let Some(value) = value {
                append_line(&mut output, format!("{name} {}", quote(value)));
            }
        }
        output
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Record {
    pub id: String,
    pub content: Vec<Question>,
}

impl Record {
    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        append_line(&mut output, format!("RECORD {}", self.id));
        for question in self.content.iter() {
            output.push_str(&question.to_sss_classic());
        }
        append_line(&mut output, "END RECORD");
        output
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Question {
    pub index: u64,
    pub name: String,
    /// text in a question
    pub text: Text,
    pub filter: Option<String>,
    pub location: Location,
    pub response: Response,
}

impl Question {
    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        append_line(&mut output, format!("VARIABLE {}", self.index));
        append_line(&mut output, format!("NAME {}", quote(&self.name)));
        append_line(
            &mut output,
            format!("LABEL {}", quote(&self.text.to_sss_classic())),
        );
        append_line(&mut output, self.location.to_sss_classic());
        append_line(&mut output, format!("END VARIABLE {}", self.index));
        output.push_str(&self.response.to_sss_classic());
        output
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// Quantity type of response.
    /// min/ max: minimun & maximun value allowed
    /// na_value: default value for empty entries (removed after triple-s 1.1)
    /// na_label: text label for empty entries (removed after triple-s 1.1)
    Quantity {
        min: f64,
        max: f64,
        na_value: Option<f64>,
        na_label: Option<String>,
    },
    Character(usize),
    Note(String),
    Single {
        item: BTreeMap<u64, Choice>,
    },
    Multiple {
        item: BTreeMap<u64, Choice>,
        spread: Spread,
    },
}

impl Default for Response {
    fn default() -> Self {
        Self::Single {
            item: BTreeMap::new(),
        }
    }
}

impl Response {
    pub fn is_categorical(&self) -> bool {
        matches!(self, Self::Single { .. } | Self::Multiple { .. })
    }

    pub fn is_quantity(&self) -> bool {
        matches!(self, Self::Quantity { .. })
    }

    /// returns lookup table of code number <-> code label
    /// for single & multiple kinds of responses.
    pub fn codeframe(&self) -> Option<&BTreeMap<u64, Choice>> {
        match self {
            Self::Single { item } | Self::Multiple { item, .. } => Some(item),
            _ => None,
        }
    }

    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        match self {
            Self::Quantity {
                min,
                max,
                na_value,
                na_label,
            } => {
                append_line(&mut output, "TYPE QUANTITY");
                append_line(&mut output, "VALUES");

                // append_line(&mut output, format_range(*min, *max, "TO"));
                match na_value {
                    Some(na_value) => {
                        append_line(&mut output, format_range(*min, *max, "TO"));
                        let na_label = na_label.as_ref().map_or(r#""""#.into(), quote);
                        let na_line = format!("{} {} SPECIAL", na_value, na_label);
                        append_line(&mut output, na_line);
                    }
                    None => {
                        let base = format_range(*min, *max, "TO");
                        let range_line = format!("{} WITH", base);
                        append_line(&mut output, range_line);
                    }
                };
            }
            Self::Character(size) => {
                append_line(&mut output, "TYPE CHARACTER");
                append_line(&mut output, self.size_line(*size));
            }
            Self::Note(value) => {
                append_line(&mut output, "TYPE NOTE");
                append_line(&mut output, format!("LABEL {}", quote(value)));
            }
            Self::Single { item } => {
                append_line(&mut output, "TYPE SINGLE");
                write_choices(&mut output, item);
            }
            Self::Multiple { item, spread } => {
                append_line(&mut output, "TYPE MULTIPLE");
                append_line(&mut output, spread.to_sss_classic());
                write_choices(&mut output, item);
            }
        }
        output
    }

    fn size_line(&self, size: usize) -> String {
        format!("SIZE {size}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub code: u64,
    /// text label
    pub label: Text,
    /// Response filter.
    pub filter: Option<String>,
    /// (Optional) indicator for special values (e.g. mannotationissing data codes).
    /// Removed from SSS 1.2 onwards.
    pub is_special: bool,
}

impl Choice {
    pub fn new<T>(code: u64, label: T, filter: Option<T>) -> Self
    where
        T: Into<String>,
    {
        Self {
            code,
            label: Text::new(label.into()),
            filter: filter.map(Into::into),
            is_special: false,
        }
    }

    pub fn to_sss_classic(&self) -> String {
        let base = format!("{} {}", self.code, quote(&self.label.to_sss_classic()));
        if self.is_special {
            format!("{} SPECIAL", base)
        } else {
            base
        }
    }

    pub fn _set_special(mut self, is_special: bool) -> Self {
        self.is_special = is_special;
        self
    }

    pub fn set_label(&mut self, value: impl Into<String>) {
        self.label.default = value.into();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub from: u64,
    pub to: u64,
}

impl Default for Location {
    fn default() -> Self {
        Self { from: 1, to: 1 }
    }
}

impl Location {
    pub fn to_sss_classic(&self) -> String {
        if self.from == self.to {
            format!("POSITION {}", self.from)
        } else {
            format!("POSITION {} TO {}", self.from, self.to)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spread {
    pub num_subfields: u64,
    pub width: u64,
}

impl Default for Spread {
    fn default() -> Self {
        Self {
            num_subfields: 999,
            width: 999,
        }
    }
}

impl Spread {
    pub fn to_sss_classic(&self) -> String {
        format!("SPREAD {} OF {}", self.num_subfields, self.width)
    }
}

/// TODO: sss xml 2.0: `mode` attribute
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Text {
    /// the default value
    pub default: String,
    /// variants
    pub variant: Vec<TextVariant>,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            default: content.into(),
            variant: Vec::new(),
        }
    }

    /// Will modify existing variant if they have the same `mode` and `language` attribute
    /// or append to the current `self.variant` if not
    pub fn set_variant(&mut self, variant: TextVariant) {
        for item in self.variant.iter_mut() {
            if item.has_same_attribute(&variant) {
                item.value = variant.value.clone();
                return;
            }
        }
        self.variant.push(variant)
    }

    pub fn to_sss_classic(&self) -> String {
        self.default.clone()
    }
}

/// From sss xml 2.0 onwards supports multi-language
///
/// For sss xml 2.0:
/// ```text
/// The text for a survey <title>, variable <label> or values
/// <value> element may contain any number of specialised text elements.
/// ```
///
/// Example sss xml 2.0 text, note that one entry can contain both `label` and `language` attribute
/// ```xml
/// <label>Age
///     <text xml:lang="en-GB" mode="interview">How old are you?</text>
///     <text xml:lang="fr" mode="interview">Quel est votre âge?</text>
///     <text xml:lang="en-GB" mode="analysis">Age of respondent</text>
///     <text xml:lang="fr" mode="analysis">Âge de répondant</text>
/// </label>
/// ```
///
/// -> TODO: Impl this AFTER the sss xml website when up again
#[derive(Debug, Clone, PartialEq)]
pub struct TextVariant {
    pub value: String,
    pub language: Language,
    pub mode: Mode,
}

impl TextVariant {
    pub fn new<S>(value: S, mode: Mode, language: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            value: value.into(),
            mode: mode,
            language: Language::Named(language.into()),
        }
    }

    pub fn with_language(
        content: impl Into<String>,
        language_name: impl Into<String>,
    ) -> Self {
        Self {
            value: content.into(),
            language: Language::Named(language_name.into()),
            mode: Mode::None,
        }
    }

    /// set `language` to `Language::Name(value)`
    pub fn set_language(&mut self, new: impl Into<String>) {
        self.language = Language::Named(new.into());
    }

    /// reset `language` back to `Default`
    pub fn reset_language(&mut self) {
        self.language = Language::None;
    }

    /// check if instance of `other` has the same attributes as `self`
    pub fn has_same_attribute(&self, other: &Self) -> bool {
        (self.mode == other.mode) & (self.language == other.language)
    }
}

/// text language
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    /// undefined
    None,
    /// Language as defined by W3C XML version 1.0 specification
    Named(String),
}

impl Default for Language {
    fn default() -> Self {
        Self::None
    }
}

/// Introduced in SSS Xml 2.0.
///
/// Official documentation:
///
/// ```text
/// (Optional) The use of specialised texts for interviewing and/or analysis within the
/// specification must be signalled by a modes="mode_identifier_list"
/// attribute on the initial <sss> element.   
/// Two explicit modes are available: “interview” and “analysis”. In the absence of
/// a mode specification, the appropriate text is assumed to be used in both
/// modes.
/// ```
///
/// Variants:
/// - None: default value when no 'mode' value was defined.
/// - Analysis: mode 'analysis'
/// - Interview: mode 'interview'
#[derive(Debug, PartialEq, Clone, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Mode {
    None,
    #[strum(serialize = "analysis")]
    Analysis,
    #[strum(serialize = "interview")]
    Interview,
}

impl Default for Mode {
    fn default() -> Self {
        Self::None
    }
}

// Helper functions
// ------------------------------------------------------------------------
#[inline]
fn quote(value: impl std::fmt::Display) -> String {
    format!(r#""{}""#, value)
}

#[inline]
fn append_line(output: &mut String, content: impl std::fmt::Display) {
    writeln!(output, "{content}").expect("writing to a String shouldn't fail");
}

#[inline]
fn write_choices(output: &mut String, choices: &BTreeMap<u64, Choice>) {
    append_line(output, "VALUES");
    for choice in choices.values() {
        append_line(output, choice.to_sss_classic());
    }
    append_line(output, "END VALUES");
}

#[inline]
fn format_range<T: std::fmt::Display>(from: T, to: T, delimiter: &str) -> String {
    if from.to_string() == to.to_string() {
        from.to_string()
    } else {
        format!("{from} {delimiter} {to}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_write_spread() {
        let instance = Spread {
            num_subfields: 5,
            width: 10,
        };
        assert_eq!(
            instance.to_sss_classic(),
            format!("SPREAD {} OF {}", 5, 10)
        );
    }

    #[test]
    fn test_write_location() {
        let instance = Location { from: 0, to: 100 };
        assert_eq!(instance.to_sss_classic(), "POSITION 0 TO 100");
    }

    #[test]
    fn test_text_variant_set() {
        let mut instance = Text::new("base text");
        let v1 = TextVariant::new("variant 1", Mode::None, "GB");
        let v2 = TextVariant::new("variant 2", Mode::None, "GB");

        instance.set_variant(v1);
        assert_eq!(instance.variant.len(), 1);
        instance.set_variant(v2);
        assert_eq!(instance.variant.len(), 1);

        assert_eq!(&instance.variant[0].value, "variant 2");
    }

    #[test]
    fn test_choice_variant() {
        todo!("Set up unit tests for testing different text variants!!!")
    }
}
