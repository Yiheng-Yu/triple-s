use std::{fmt::Write, range::Range, str::FromStr, sync::Arc};

use anyhow::{Context, Result, bail, ensure};
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
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        let input = input.as_ref().trim();
        if is_sss_classic(input) {
            return Ok(parse_sss_classic(input)?.meta.version);
        }
        let value = input.strip_prefix("VERSION").map(str::trim).unwrap_or(input);
        Ok(Self::from_str(value)?)
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
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        let input = input.as_ref();
        if is_sss_classic(input) {
            return parse_sss_classic(input);
        }
        if is_xml_fragment(input) {
            return parse_sss_classic(&format!(r#"<sss version="1.2">{input}</sss>"#));
        }
        bail!("a Classic survey must include the surrounding SSS document")
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
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        let input = input.as_ref();
        let document = if is_sss_classic(input) {
            input.to_owned()
        } else if is_xml_fragment(input) {
            format!(r#"<sss version="1.2"><survey>{input}</survey></sss>"#)
        } else {
            format!("SSS\nVERSION 1.1\nSURVEY\n{input}\nEND SURVEY\nEND SSS")
        };
        Ok(parse_sss_classic(&document)?.record)
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
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        parse_one_question(input.as_ref())
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
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        let input = input.as_ref().trim();
        if input.eq_ignore_ascii_case("numeric") || input.eq_ignore_ascii_case("literal") {
            return Ok(Self::from_str(input)?);
        }
        Ok(parse_one_question(input)?.format)
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
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        let input = input.as_ref();
        if input.trim_start().starts_with("TYPE ") {
            let document = format!(
                "SSS\nVERSION 1.1\nSURVEY\nRECORD A\nVARIABLE 1\nNAME \"VALUE\"\nLABEL \"Value\"\nPOSITION 1\n{input}\nEND VARIABLE\nEND RECORD\nEND SURVEY\nEND SSS"
            );
            return response_from_fragment(&document);
        }
        response_from_fragment(input)
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
    /// Parses a variable fragment whose response type is `quantity`.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        match response_template(input.as_ref(), "quantity")? {
            Response::Quantity(value) => Ok(value.read().clone()),
            response => bail!("expected a quantity response, found {response:?}"),
        }
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
    /// Parses a variable fragment whose response type is `character`.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        match response_template(input.as_ref(), "character")? {
            Response::Character(value) => Ok(value.read().clone()),
            response => bail!("expected a character response, found {response:?}"),
        }
    }

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
    /// Parses a variable fragment whose response type is numeric `single`.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        match response_template(input.as_ref(), "single")? {
            Response::Single(value) => Ok(value.read().clone()),
            response => bail!("expected a numeric single response, found {response:?}"),
        }
    }

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
    /// Parses a variable fragment whose response type is `multiple`.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        match response_template(input.as_ref(), "multiple")? {
            Response::Multiple(value) => Ok(value.read().clone()),
            response => bail!("expected a multiple response, found {response:?}"),
        }
    }

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
    /// Parses a variable fragment whose response type is `date`.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        match response_template(input.as_ref(), "date")? {
            Response::Date(value) => Ok(value.read().clone()),
            response => bail!("expected a date response, found {response:?}"),
        }
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        Err(UnsupportedFeature::new("SSS Classic", "DateTime response type").into())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RangeTime {
    pub range: Option<Range<NaiveTime>>,
}

impl RangeTime {
    /// Parses a variable fragment whose response type is `time`.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        match response_template(input.as_ref(), "time")? {
            Response::Time(value) => Ok(value.read().clone()),
            response => bail!("expected a time response, found {response:?}"),
        }
    }

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

impl Code<u64> {
    /// Parses the sole value from a numeric single or multiple variable
    /// fragment.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        let question = parse_one_question(input.as_ref())?;
        let values = match question.response {
            Response::Single(value) => value.read().values.clone(),
            Response::Multiple(value) => value.read().values.clone(),
            response => bail!("expected a single or multiple response, found {response:?}"),
        };
        ensure!(
            values.len() == 1,
            "expected exactly one value, found {}",
            values.len()
        );
        Ok(values.into_iter().next().expect("length checked"))
    }
}

impl Code<f64> {
    /// Parses the sole labelled value from a quantity variable fragment.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        let quantity = Quantity::from_string(input)?;
        ensure!(
            quantity.values.len() == 1,
            "expected exactly one value, found {}",
            quantity.values.len()
        );
        Ok(quantity.values.into_iter().next().expect("length checked"))
    }
}

impl Single<String> {
    /// Parses a variable fragment whose response is a literal `single`.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        match response_template(input.as_ref(), "single")? {
            Response::SingleLiteral(value) => Ok(value.read().clone()),
            response => bail!("expected a literal single response, found {response:?}"),
        }
    }
}

impl Code<String> {
    /// Parses the sole value from a literal single variable fragment.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        let single = Single::<String>::from_string(input)?;
        ensure!(
            single.values.len() == 1,
            "expected exactly one value, found {}",
            single.values.len()
        );
        Ok(single.values.into_iter().next().expect("length checked"))
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Position {
    pub from: u64,
    pub to: u64,
}

impl Position {
    /// Parses the position from one variable fragment.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        Ok(parse_one_question(input.as_ref())?.location)
    }

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
    /// Parses the spread from one multiple-response variable fragment.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        match response_from_fragment(input.as_ref())? {
            Response::Multiple(value) => value
                .read()
                .spread
                .clone()
                .ok_or_else(|| anyhow::anyhow!("multiple response has no spread")),
            response => bail!("expected a multiple response, found {response:?}"),
        }
    }

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
    /// Parses the label text from one variable fragment.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        Ok(parse_one_question(&survey_template(input.as_ref()))?.label)
    }

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
    /// Parses the sole specialised label text from one XML variable fragment.
    ///
    /// Classic SSS has no representation for specialised text variants.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        let variants = Text::from_string(input)?.variant;
        ensure!(
            variants.len() == 1,
            "expected exactly one text variant, found {}",
            variants.len()
        );
        Ok(variants.into_iter().next().expect("length checked"))
    }

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
    /// Parses a language identifier, such as `en-GB`.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        let value = input.as_ref().trim();
        if value.starts_with("<text") {
            return TextVariant::from_string(value)?
                .language
                .ok_or_else(|| anyhow::anyhow!("text variant has no language"));
        }
        ensure!(!value.is_empty(), "language identifier cannot be empty");
        Ok(Self(value.to_owned()))
    }

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

impl Mode {
    /// Parses a mode identifier (`analysis` or `interview`). An empty value is
    /// `None`.
    pub fn from_string(input: impl AsRef<str>) -> Result<Self> {
        let value = input.as_ref().trim();
        if value.starts_with("<text") {
            return Ok(TextVariant::from_string(value)?.mode);
        }
        if value.is_empty() {
            return Ok(Self::None);
        }
        Ok(Self::from_str(value)?)
    }
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

#[inline]
fn is_xml_fragment(input: &str) -> bool {
    input.trim_start().starts_with('<')
}

#[inline]
fn is_sss_classic(input: &str) -> bool {
    let input = input.trim_start();
    input.starts_with("SSS") || input.starts_with("<?xml") || input.starts_with("<sss")
}

#[inline]
fn parse_sss_classic(input: &str) -> Result<Survey> {
    crate::parser::read_from_string(input.to_owned())
}

#[inline]
fn wrap_question_fragment(input: &str) -> String {
    if is_sss_classic(input) {
        return input.to_owned();
    }
    if is_xml_fragment(input) {
        format!(
            r#"<sss version="1.2"><survey><record ident="A">{input}</record></survey></sss>"#
        )
    } else {
        format!("SSS\nVERSION 1.1\nSURVEY\nRECORD A\n{input}\nEND RECORD\nEND SURVEY\nEND SSS")
    }
}

#[inline]
fn parse_one_question(input: &str) -> Result<Question> {
    let survey = parse_sss_classic(&wrap_question_fragment(input))?;
    ensure!(
        survey.record.content.len() == 1,
        "expected exactly one variable fragment, found {}",
        survey.record.content.len()
    );
    Ok(survey.record.content.into_iter().next().expect("length checked"))
}

#[inline]
fn response_from_fragment(input: &str) -> Result<Response> {
    Ok(parse_one_question(input)?.response)
}

#[inline]
fn response_template(input: &str, xml_type: &str) -> Result<Response> {
    if is_sss_classic(input)
        || input.trim_start().starts_with("<variable")
        || input.trim_start().starts_with("VARIABLE")
    {
        return response_from_fragment(input);
    }

    let document = if is_xml_fragment(input) {
        format!(
            r#"<sss version="1.2"><survey><record ident="A"><variable ident="1" type="{xml_type}"><name>VALUE</name><label>Value</label><position start="1"/>{input}</variable></record></survey></sss>"#
        )
    } else {
        format!(
            "SSS\nVERSION 1.1\nSURVEY\nRECORD A\nVARIABLE 1\nNAME \"VALUE\"\nLABEL \"Value\"\nPOSITION 1\n{input}\nEND VARIABLE\nEND RECORD\nEND SURVEY\nEND SSS"
        )
    };
    response_from_fragment(&document)
}

#[inline]
fn survey_template(input: &str) -> String {
    if is_sss_classic(input)
        || input.trim_start().starts_with("<variable")
        || input.trim_start().starts_with("VARIABLE")
    {
        return wrap_question_fragment(input);
    }
    if is_xml_fragment(input) {
        let label = if input.trim_start().starts_with("<label") {
            input.to_owned()
        } else {
            format!("<label>Value{input}</label>")
        };
        format!(
            r#"<sss version="1.2"><survey><record ident="A"><variable ident="1" type="logical"><name>VALUE</name>{label}<position start="1"/></variable></record></survey></sss>"#
        )
    } else {
        format!(
            "SSS\nVERSION 1.1\nSURVEY\nRECORD A\nVARIABLE 1\nNAME \"VALUE\"\n{input}\nPOSITION 1\nTYPE LOGICAL\nEND VARIABLE\nEND RECORD\nEND SURVEY\nEND SSS"
        )
    }
}