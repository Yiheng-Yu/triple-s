use std::{range::Range, str::FromStr, sync::Arc};

use anyhow::Result;
use chrono::{NaiveDate, NaiveTime};
use parking_lot::RwLock;
use strum_macros::EnumString;

use crate::error::BadLine;

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
}

#[derive(Debug, Clone)]
pub struct Survey {
    pub name: String,
    pub title: Option<String>,
    pub meta: Meta,
    pub record: Record,
}

impl Survey {
    pub fn empty() -> Self {
        Self {
            title: None,
            meta: Meta::with_version(Version::V1_1),
            record: Record::default(),
            name: String::new(),
        }
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

#[derive(Debug, Clone, EnumString, Default)]
pub enum Format {
    #[default]
    #[strum(serialize = "numeric")]
    Numeric,
    #[strum(serialize = "literal")]
    Literal,
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Character {
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Single<T: Clone> {
    pub range: Option<Range<T>>,
    pub values: Vec<Code<T>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Multiple {
    pub range: Option<Range<u64>>,
    pub spread: Option<Spread>,
    pub values: Vec<Code<u64>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RangeDate {
    pub range: Option<Range<NaiveDate>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RangeTime {
    pub range: Option<Range<NaiveTime>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Code<T>
where T: Clone
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

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Position {
    pub from: u64,
    pub to: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spread {
    /// The number of subfields within the overall field that is defined by the
    /// <position> element.
    pub num_subfields: u64,
    /// Denotes the width of each subfield.
    pub width: Option<u64>,
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
    pub fn some(self) -> Option<Self> {
        Some(self)
    }

    pub fn ok<E: std::error::Error>(self) -> Result<Self, E> {
        Ok(self)
    }

    pub fn to_string(self) -> String {
        self.0
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

// #[derive(Debug, Clone, PartialEq)]
// pub struct Value {
//     pub code: u64,
//     /// text label
//     pub label: Text,
//     /// Response filter.
//     pub filter: Option<String>,
//     pub score: Option<f64>,
//     /// (Optional) indicator for special values (e.g. mannotationissing data
// codes).     /// Removed from SSS 1.2 onwards.
//     pub is_special: bool,
// }
