use std::{fmt::Write, range::Range, str::FromStr, sync::Arc};

use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveTime};
use parking_lot::RwLock;
use rayon::prelude::*;
use strum_macros::EnumString;

use crate::error::{BadLine, UnsupportedFeature};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString)]
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

impl Version {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1_0 => "1.0",
            Self::V1_1 => "1.1",
            Self::V1_2 => "1.2",
            Self::V2_0 => "2.0",
            Self::V3_0 => "3.0",
        }
    }

    /// Parses a version literal, a Classic `VERSION` line, or an XML `<sss>`
    /// document.
    pub fn from_string(_input: &str) -> Result<Self> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct Survey {
    pub name: String,
    pub title: Option<String>,
    pub meta: Meta,
    pub record: Record,
}

impl Survey {
    /// Parses a complete Classic or XML SSS document, or a `<survey>` fragment.
    pub fn from_string(_input: &str) -> Result<Self> {
        todo!()
    }

    pub fn empty() -> Self {
        Self {
            title: None,
            meta: Meta::with_version(Version::V1_1),
            record: Record::default(),
            name: String::new(),
        }
    }

    /// find question via question name, case insensitive
    pub fn get_question(&self, name: &str) -> Option<&Question> {
        self.record
            .content
            .par_iter()
            .find_first(|question| question.name.to_uppercase() == name.to_uppercase())
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::from("SSS\n");
        output.push_str(&self.meta.to_sss_classic());
        append_line(&mut output, "SURVEY");
        if let Some(title) = self.title.as_ref() {
            append_line(&mut output, format!("TITLE {}", quote(title)));
        }

        let record = &self.record.to_sss_classic()?;
        output.push_str(record);
        append_line(&mut output, "END SURVEY");
        append_line(&mut output, "END SSS");
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    /// The version attribute is used to indicate the version of the Triple-S
    /// standard that applies to this specification.
    pub version: Version,
    pub name: Option<String>,
    pub date: Option<String>,
    pub time: Option<String>,
    pub origin: Option<String>,
    pub user: Option<String>,
    pub style: Option<String>,

    /// ** Introduced since sss 1.2 **
    /// list of all available language options
    pub language_options: Vec<String>,

    /// ** Introduced since sss 1.2 **
    /// list of all available `mod` options
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

    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        append_line(&mut output, format!("VERSION {}", self.version.as_str()));
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

#[derive(Debug, Default, Clone)]
pub struct Record {
    pub id: char,

    // pub format: Option<Format>,
    /// (Optional) 'skip' attribute can be used to ignore one or more initial
    /// records in the Data File. This will be most useful for csv Data
    /// Files where the first line is often used as documentation (e.g.
    /// names for the columns/fields) for the succeeding values.
    pub skip: Option<String>,

    /// (Optional) Used to specify an explicit location for the Data
    /// File that is described by this Triple-S XML specification. Note that
    /// using an href attribute ties the specification to the Data File and
    /// may cause problems if the Metadata File and Data File are moved.
    pub href: Option<String>,
    pub content: Vec<Question>,
}

impl Record {
    /// Parses a Classic `RECORD` block or an XML `<record>` fragment.
    pub fn from_string(_input: &str) -> Result<Self> {
        todo!()
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::new();
        append_line(&mut output, format!("RECORD {}", self.id));
        for question in self.content.iter() {
            output.push_str(&question.to_sss_classic()?);
        }
        append_line(&mut output, "END RECORD");
        Ok(output)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Question {
    pub index: u64,
    pub name: String,
    pub label: Text,
    pub filter: Option<String>,
    pub format: Format,
    pub response: Response,
    pub location: Position,
}

impl Question {
    /// Parses one Classic `VARIABLE` block or XML `<variable>` fragment.
    pub fn from_string(_input: &str) -> Result<Self> {
        todo!()
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::new();
        append_line(&mut output, format!("VARIABLE {}", self.index));
        append_line(&mut output, format!("NAME {}", quote(&self.name)));
        append_line(
            &mut output,
            format!("LABEL {}", quote(self.label.to_sss_classic())),
        );
        append_line(&mut output, self.location.to_sss_classic());
        let response = &self
            .response
            .to_sss_classic()
            .with_context(|| format!("Question '{}'", self.name))?;

        output.push_str(response);
        append_line(&mut output, format!("END VARIABLE {}", self.index));
        Ok(output)
    }
}

#[derive(Debug, Clone, EnumString, Default)]
pub enum Format {
    #[default]
    #[strum(serialize = "numeric")]
    Numeric,
    #[strum(serialize = "literal")]
    Literal,
}

impl Format {
    /// Parses a `numeric`/`literal` value or obtains the format from one
    /// variable fragment.
    pub fn from_string(_input: &str) -> Result<Self> {
        todo!()
    }
}

#[derive(Debug, Clone, Default)]
pub enum Response {
    /// Quantity type of response.
    /// min/ max: minimun & maximun value allowed
    /// na_value: default value for empty entries (removed after triple-s 1.1)
    /// na_label: text label for empty entries (removed after triple-s 1.1)
    Quantity(Arc<RwLock<Quantity>>),

    /// character response type
    Character(Arc<RwLock<Character>>),

    /// Do not contain any values, not used in triple-s but lots of different
    /// companies inplement various variations of it. Main use case is a
    /// blank page with just 'Next Page' button on it, used for displaying
    /// instructions
    #[default]
    Note,

    /// Single response type
    Single(Arc<RwLock<Single<u64>>>),

    /// Multi response type
    Multiple(Arc<RwLock<Multiple>>),

    /// Logical response type
    Logical,

    /// Single response type but with code value as string literals,
    /// introduced since sss 2.0
    SingleLiteral(Arc<RwLock<Single<String>>>),
    Date(Arc<RwLock<RangeDate>>),
    Time(Arc<RwLock<RangeTime>>),
}

impl Response {
    /// Parses the response of one Classic `VARIABLE` block or XML `<variable>`
    /// fragment.
    pub fn from_string(_input: &str) -> Result<Self> {
        todo!()
    }

    pub fn new_single() -> Self {
        Self::Single(Arc::new(RwLock::new(Single::<u64>::default())))
    }

    pub fn new_multiple() -> Self {
        Self::Multiple(Arc::new(RwLock::new(Multiple::default())))
    }

    pub fn new_quantity() -> Self {
        Self::Quantity(Arc::new(RwLock::new(Quantity::default())))
    }

    pub fn new_character() -> Self {
        Self::Character(Arc::new(RwLock::new(Character::default())))
    }

    pub fn new_range_date() -> Self {
        Self::Date(Arc::new(RwLock::new(RangeDate::default())))
    }

    pub fn new_range_time() -> Self {
        Self::Time(Arc::new(RwLock::new(RangeTime::default())))
    }

    pub fn new_literal_single() -> Self {
        Self::SingleLiteral(Arc::new(RwLock::new(Single::<String>::default())))
    }
}

impl Response {
    pub fn to_sss_classic(&self) -> Result<String> {
        match self {
            Self::Character(item) => item.read().to_sss_classic(),
            Self::Single(item) => item.read().to_sss_classic(),
            Self::Multiple(item) => item.read().to_sss_classic(),
            Self::Quantity(item) => item.read().to_sss_classic(),
            Self::Logical => Ok("TYPE LOGICAL\n".into()),
            Self::Date(_) | Self::Time(_) | Self::SingleLiteral(_) | Self::Note => {
                let error = UnsupportedFeature {
                    current_version: "classic".into(),
                    feature: format!("{:?}", self),
                };
                Err(error.into())
            }
        }
    }
}

impl FromStr for Response {
    type Err = BadLine;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let to_match = s.trim().to_ascii_lowercase();
        let instance = match to_match.as_str() {
            "single" => Self::new_single(),
            "multiple" => Self::new_multiple(),
            "quantity" => Self::new_quantity(),
            "character" => Self::new_character(),
            "logical" => Self::Logical,
            "date" => Self::new_range_date(),
            "time" => Self::new_range_time(),
            _ => {
                return Err(BadLine::new(format!(
                    "Unidentified response kind: '{}'",
                    to_match
                )));
            }
        };
        Ok(instance)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Quantity {
    pub range: Range<f64>,
    pub values: Vec<Code<f64>>,
}

impl Quantity {
    pub fn from_string(_input: &str) -> Result<Self> {
        todo!()
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::new();
        append_line(&mut output, "TYPE QUANTITY");
        append_line(&mut output, "VALUES");
        append_line(
            &mut output,
            format_range(self.range.start, self.range.end, "TO"),
        );
        write_choices(&mut output, self.values.clone());
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Character {
    pub size: u64,
}

impl Character {
    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::new();
        append_line(&mut output, "TYPE CHARACTER");
        append_line(&mut output, format!("SIZE {}", self.size));
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Single<T: Clone>
where T: Clone + std::fmt::Display
{
    pub range: Option<Range<T>>,
    pub values: Vec<Code<T>>,
}

impl Single<u64> {
    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::new();
        append_line(&mut output, "TYPE SINGLE");
        append_line(&mut output, "VALUES");
        write_choices(&mut output, self.values.clone());
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Multiple {
    pub range: Option<Range<u64>>,
    pub spread: Option<Spread>,
    pub values: Vec<Code<u64>>,
}

impl Multiple {
    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::new();
        if let Some(spread) = self.spread.as_ref() {
            append_line(&mut output, spread.to_sss_classic());
        }
        append_line(&mut output, "TYPE MULTIPLE");
        append_line(&mut output, "VALUES");
        write_choices(&mut output, self.values.clone());
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RangeDate {
    pub range: Option<Range<NaiveDate>>,
}

impl RangeDate {
    pub fn to_sss_classic(&self) -> Result<String> {
        Err(UnsupportedFeature::new("SSS Classic", "DateTime response type").into())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RangeTime {
    pub range: Option<Range<NaiveTime>>,
}

impl RangeTime {
    pub fn to_sss_classic(&self) -> Result<String> {
        Err(UnsupportedFeature::new("SSS Classic", "DateTime response type").into())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Code<T>
where T: Clone + std::fmt::Display
{
    pub code: T,

    /// text label
    pub label: Text,
    /// Response filter.
    pub filter: Option<String>,

    pub score: Option<f64>,

    /// (Optional) indicator for special values (e.g. mannotationissing data
    /// codes). Removed from SSS 1.2 onwards.
    pub is_special: bool,
}

impl<T> Code<T>
where T: Clone + std::fmt::Display
{
    pub fn to_sss_classic(&self) -> String {
        let base = format!("{} {}", self.code, quote(self.label.to_sss_classic()));
        if self.is_special {
            format!("{} SPECIAL", base)
        } else {
            base
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Position {
    pub from: u64,
    pub to: u64,
}

impl Position {
    pub fn to_sss_classic(&self) -> String {
        if self.to != self.from {
            format!("POSITION {} TO {}", self.from, self.to)
        } else {
            format!("POSITION {}", self.from)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Spread {
    /// The number of subfields within the overall field that is defined by the
    /// <position> element.
    pub subfield: u64,
    /// Denotes the width of each subfield.
    pub width: Option<u64>,
}

impl Spread {
    pub fn to_sss_classic(&self) -> String {
        match self.width.as_ref() {
            Some(width) => format!("SPREAD {} OF {}", self.subfield, width),
            None => format!("SPREAD {}", self.subfield),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Text {
    /// the default value
    pub value: String,

    /// variants
    pub variant: Vec<TextVariant>,
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self {
            value,
            variant: Vec::new(),
        }
    }
}

impl From<std::sync::Arc<str>> for Text {
    fn from(value: std::sync::Arc<str>) -> Self {
        Self {
            value: value.to_string(),
            variant: Vec::new(),
        }
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _variants = format!(", [{}] variants", self.variant.len());
        let variants = "";

        write!(
            f,
            r"Text(\n\tvalue='{}'\n\tvariant={}>",
            self.value, variants
        )
    }
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            value: content.into(),
            variant: Vec::new(),
        }
    }

    /// Will modify existing variant if they have the same `mode` and `language`
    /// attribute or append to the current `self.variant` if not
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
        self.value.clone()
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
#[derive(Debug, Clone, PartialEq)]
pub struct TextVariant {
    pub value: String,
    pub language: Option<Language>,
    pub mode: Mode,
}

impl std::fmt::Display for TextVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"TextVariant(value="{}", language={}, mode={:?})"#,
            self.value,
            self.language.as_ref().map(|s| s.0.to_string()).unwrap_or("None".into()),
            self.mode
        )
    }
}

impl TextVariant {
    pub fn new<S>(value: S, mode: Mode, language: S) -> Self
    where S: Into<String> {
        Self {
            value: value.into(),
            mode,
            language: Language(language.into()).some(),
        }
    }

    pub fn with_language(
        content: impl Into<String>,
        language_name: impl Into<String>,
    ) -> Self {
        Self {
            value: content.into(),
            language: Language(language_name.into()).some(),
            mode: Mode::None,
        }
    }

    /// set `language` to `Language::Name(value)`
    pub fn set_language(&mut self, new: impl Into<String>) {
        self.language = Language(new.into()).some();
    }

    /// reset `language` back to `Default`
    pub fn reset_language(&mut self) {
        self.language = None;
    }

    /// check if instance of `other` has the same attributes as `self`
    pub fn has_same_attribute(&self, other: &Self) -> bool {
        (self.mode == other.mode) & (self.language == other.language)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Language(String);

impl Language {
    pub fn new(language: impl Into<String>) -> Self {
        Self(language.into())
    }

    pub fn some(self) -> Option<Self> {
        Some(self)
    }

    pub fn ok<E: std::error::Error>(self) -> Result<Self, E> {
        Ok(self)
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}


// make `Language` behave more like a string
impl AsRef<str> for Language {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Language {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<Language> for String {
    fn from(language: Language) -> Self {
        language.0
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
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
#[derive(Default)]
pub enum Mode {
    #[default]
    None,
    #[strum(serialize = "analysis")]
    Analysis,
    #[strum(serialize = "interview")]
    Interview,
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
fn write_choices<T>(output: &mut String, choices: Vec<Code<T>>)
where T: Clone + std::fmt::Display {
    for choice in choices {
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
    fn test_write_spread_with_width() {
        let instance = Spread {
            subfield: 5,
            width: Some(10),
        };
        assert_eq!(instance.to_sss_classic(), format!("SPREAD {} OF {}", 5, 10));
    }

    #[test]
    fn test_write_spread_without_width() {
        let instance = Spread {
            subfield: 5,
            width: None,
        };
        assert_eq!(instance.to_sss_classic(), format!("SPREAD {}", 5));
    }

    #[test]
    fn test_write_location() {
        let instance = Position { from: 0, to: 100 };
        assert_eq!(instance.to_sss_classic(), "POSITION 0 TO 100");
    }

    #[test]
    fn test_text_variant_overwrite() {
        let mut instance = Text::new("base text");
        let v1 = TextVariant::new("variant 1", Mode::None, "GB");
        let v2 = TextVariant::new("variant 2", Mode::None, "GB");

        instance.set_variant(v1);
        assert_eq!(instance.variant.len(), 1);
        instance.set_variant(v2);
        assert_eq!(instance.variant.len(), 1);
        assert_eq!(&instance.variant[0].value, "variant 2");

        let v3 = TextVariant::new("variant", Mode::None, "GBR");
        instance.set_variant(v3);
        assert_eq!(instance.variant.len(), 2);

        let v4 = TextVariant::new("variant", Mode::Analysis, "GBR");
        instance.set_variant(v4);
        assert_eq!(instance.variant.len(), 3)
    }
}
