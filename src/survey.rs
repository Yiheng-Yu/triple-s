use std::{collections::BTreeMap, fmt::Write, str::FromStr};

use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveTime};
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
}

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
            meta: Meta::with_version(Version::V1_1),
            record: Record::default(),
        }
    }

    pub fn empty_xml() -> Self {
        Self {
            title: None,
            meta: Meta::with_version(Version::V2_0),
            record: Record::default(),
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
    /// The version attribute is used to indicate the version of the Triple-S standard that applies to this specification.
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
    /// defined standard that may reduce the amount of checking required by an import program.
    /// The only value that option_setting may have in version 1.2 is 'standardnames'.
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
        append_line(
            &mut output,
            format!("VERSION {}", self.version.as_str()),
        );
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
    /// Record id, any single character A to Z or a to z.
    pub id: char,
    pub format: Option<Format>,

    /// (Optional) 'skip' attribute can be used to ignore one or more initial records
    /// in the Data File. This will be most useful for csv Data Files where the first line
    /// is often used as documentation (e.g. names for the columns/fields) for the
    /// succeeding values.
    pub skip: Option<String>,

    /// (Optional) Used to specify an explicit location for the Data
    /// File that is described by this Triple-S XML specification. Note that using an
    /// href attribute ties the specification to the Data File and may cause problems
    /// if the Metadata File and Data File are moved.
    pub href: Option<String>,

    pub content: Vec<Question>,
}

impl Record {
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

/// (Optional) used to declare the format of the Data File that corresponds to this specification.
/// Introduced after sss xml 1.1.
#[derive(Debug, Clone, PartialEq, EnumString)]
pub enum Format {
    /// the data representation is fixed format fields. For data in
    /// fixed format the position element refers to the character
    /// number.
    #[strum(ascii_case_insensitive)]
    Fixed,
    /// the data representation is comma-separated values, using
    /// one field for each variable, with data values similar to the
    /// fixed format. Note that for data in csv format the
    /// <position> element refers to the field number
    #[strum(ascii_case_insensitive)]
    Csv,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Question {
    pub index: u64,
    pub name: String,
    pub text: Text,
    pub filter: Option<String>,
    pub location: Location,
    pub response: Response,
}

impl Question {
    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::new();
        append_line(&mut output, format!("VARIABLE {}", self.index));
        append_line(&mut output, format!("NAME {}", quote(&self.name)));
        append_line(
            &mut output,
            format!("LABEL {}", quote(self.text.to_sss_classic())),
        );
        append_line(&mut output, self.location.to_sss_classic());
        append_line(&mut output, format!("END VARIABLE {}", self.index));
        let response = &self
            .response
            .to_sss_classic()
            .with_context(|| format!("Question '{}'", self.name))?;
        output.push_str(response);
        Ok(output)
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
        spread: Option<Spread>,
    },
    Logical,
    Date {
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    },
    Time {
        from: Option<NaiveTime>,
        to: Option<NaiveTime>,
    },
}

impl FromStr for Response {
    type Err = BadLine;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let to_match = s.trim().to_ascii_lowercase();
        let instance = match to_match.as_str() {
            "single" => Self::Single {
                item: BTreeMap::new(),
            },
            "multiple" => Self::Multiple {
                item: BTreeMap::new(),
                spread: None,
            },
            "quantity" => Self::Quantity {
                min: 0.0,
                max: 0.0,
                na_value: None,
                na_label: None,
            },
            "character" => Self::Character(1024),
            "logical" => Self::Logical,
            "date" => Self::Date {
                from: None,
                to: None,
            },
            "time" => Self::Time {
                from: None,
                to: None,
            },
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

    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::new();
        match self {
            Self::Date { .. } | Self::Time { .. } => {
                let error = UnsupportedFeature {
                    current_version: "classic".into(),
                    feature: format!("{:?}", &self),
                };
                return Err(error.into());
            }
            Self::Logical => append_line(&mut output, "TYPE LOGICAL"),
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
            Self::Multiple { item,spread} => {
                append_line(&mut output, "TYPE MULTIPLE");
                if let Some(item) = spread.as_ref() {
                    append_line(&mut output, item.to_sss_classic())
                };
                write_choices(&mut output, item);
            }
        }

        Ok(output)
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
    pub score: Option<f64>,
    /// (Optional) indicator for special values (e.g. mannotationissing data codes).
    /// Removed from SSS 1.2 onwards.
    pub is_special: bool,
}

impl Choice {
    pub fn new<T>(code: u64, label: T, score: Option<f64>, filter: Option<T>) -> Self
    where
        T: Into<String>,
    {
        Self {
            code,
            label: Text::new(label.into()),
            filter: filter.map(Into::into),
            score: score,
            is_special: false,
        }
    }

    pub fn to_sss_classic(&self) -> String {
        let base = format!("{} {}", self.code, quote(self.label.to_sss_classic()));
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
    pub to: Option<u64>,
}

impl Default for Location {
    fn default() -> Self {
        Self { from: 1, to: None }
    }
}

impl Location {
    pub fn to_sss_classic(&self) -> String {
        match self.to {
            Some(value) => format!("POSITION {} TO {}", self.from, value),
            None => format!("POSITION {}", self.from),
        }
    }
}

/// Indicates that the data values are coded as a series of category values in consecutive subfields,
/// rather than the default multiple format of a series of 0/1 characters).
///
/// For fixed Data Files, the <position> element must
/// define a width of at least (num_subfields * subfield_width).
/// The subfield_width must be large enough to hold the largest category value
/// specified for the multiple, for example:
///  <spread subfields="5" width="3"/>
///
/// For fixed data:
/// if the num_subfields exactly fills the space defined by the <position>
/// element. In this case the subfield_width is determined by dividing the width derived from the <position> element by num_subfields.
///
/// For csv data:
/// the width attribute must always be specified as it cannot be determined from the overall width defined in the <position> element.
#[derive(Debug, Clone, PartialEq)]
pub struct Spread {
    /// The number of subfields within the overall field that is defined by the <position> element.
    pub num_subfields: u64,
    /// Denotes the width of each subfield.
    pub width: Option<u64>,
}

impl Default for Spread {
    fn default() -> Self {
        Self {
            num_subfields: 999,
            width: None,
        }
    }
}

impl Spread {
    pub fn to_sss_classic(&self) -> String {
        match self.width.as_ref() {
            Some(width) => format!("SPREAD {} OF {}", self.num_subfields, width),
            None => format!("SPREAD {}", self.num_subfields),
        }
    }
}

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
            mode,
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
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Language {
    /// undefined
    #[default]
    None,
    /// Language as defined by W3C XML version 1.0 specification
    Named(String),
}

impl Language {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
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
    fn test_write_spread_with_width() {
        let instance = Spread {
            num_subfields: 5,
            width: Some(10),
        };
        assert_eq!(
            instance.to_sss_classic(),
            format!("SPREAD {} OF {}", 5, 10)
        );
    }

    #[test]
    fn test_write_spread_without_width() {
        let instance = Spread {
            num_subfields: 5,
            width: None,
        };
        assert_eq!(instance.to_sss_classic(), format!("SPREAD {}", 5));
    }

    #[test]
    fn test_write_location() {
        let instance = Location {
            from: 0,
            to: Some(100),
        };
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

    // #[test]
    // fn test_choice_variant() {
    //     todo!("Set up unit tests for testing different text variants!!!")
    // }
}
