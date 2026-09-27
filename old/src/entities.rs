use std::{
    any::type_name,
    collections::HashMap,
    fmt::{Display, Write},
    range::Range,
    str::FromStr,
    sync::{Arc, LazyLock},
};

use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveTime};
use parking_lot::RwLock;
use rayon::prelude::*;
use strum::{EnumIter, EnumString, IntoEnumIterator};

use crate::{
    error::{BadLine, DuplicatedKey, LanguageTagError, UnsupportedFeature},
    prelude::*,
};

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
    pub meta: SurveyInfo,
    pub record: Record,
}

impl Survey {
    /// Parses a complete Classic or XML SSS document, or a `<survey>` fragment.
    pub fn from_string(_input: &str) -> Result<Self> {
        todo!()
    }

    pub fn empty() -> Self {
        // let meta = SurveyInfo::with_version(Version::V1_1);
        Self {
            // meta: Arc::new(RwLock::new(meta)),
            meta: SurveyInfo::with_version(Version::V1_1),
            record: Record::default(),
        }
    }

    /// find question via question name, case insensitive
    pub fn get_question(&self, question_name: &str) -> Option<&Arc<RwLock<Question>>> {
        self.record.get_question(question_name)
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::from("SSS\n");
        // survey meta
        append_line(
            &mut output,
            format!("VERSION {}", self.meta.version.as_str()),
        );
        for (name, value) in [
            ("DATE", &self.meta.date),
            ("TIME", &self.meta.time),
            ("ORIGIN", &self.meta.origin),
            ("USER", &self.meta.user),
            ("STANDARD", &self.meta.standard), // sss classic does not have 'style' field
        ] {
            if let Some(value) = value {
                append_line(&mut output, format!("{name} {}", quote(value)));
            }
        }

        // survey content
        append_line(&mut output, "SURVEY");
        for (name, value) in [("NAME", &self.meta.name), ("TITLE", &self.meta.title)] {
            if let Some(value) = value {
                append_line(&mut output, format!("{name} {}", quote(value)));
            }
        }

        // list of questions
        let record = &self.record.to_sss_classic()?;
        output.push_str(record);

        // ending
        append_line(&mut output, "END SURVEY");
        append_line(&mut output, "END SSS");
        Ok(output)
    }
}

/// meta information about the survey entity.
#[derive(Debug, Clone, PartialEq)]
pub struct SurveyInfo {
    /// The version attribute is used to indicate the version of the Triple-S
    /// standard that applies to this specification.
    pub version: Version,
    pub name: Option<String>,
    pub title: Option<String>,
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

impl SurveyInfo {
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

    // pub fn to_sss_classic(&self) -> String {
    //     let mut output = String::new();
    //     append_line(&mut output, format!("VERSION {}",
    // self.version.as_str()));     for (name, value) in [
    //         ("DATE", &self.date),
    //         ("TIME", &self.time),
    //         ("ORIGIN", &self.origin),
    //         ("USER", &self.user),
    //         ("TITLE", &self.title),
    //     ] {
    //         if let Some(value) = value {
    //             append_line(&mut output, format!("{name} {}", quote(value)));
    //         }
    //     }
    //     for (name, value) in [("STANDARD", &self.standard), ("STYLE",
    // &self.style)] {         if let Some(value) = value {
    //             append_line(&mut output, format!("{name} {}", quote(value)));
    //         }
    //     }
    //     output
    // }
}

#[derive(Debug, EnumString, PartialEq, Default, Clone)]
pub enum DataSource {
    /// Supported from sss 2.0 onwards.
    /// the data representation is comma separated values, using one field
    /// for each variable, with data values similar to the fixed format. Note
    /// that for data in csv format the position element refers to the
    /// field number.
    Csv,

    /// The data representation is fixed format fields.
    /// Basically `.asc` text file that formatted bit like csv but without any
    /// column header.
    #[default]
    Fixed,
}

#[derive(Debug, Default, Clone)]
pub struct Record {
    pub id: char,

    // pub format: Option<CodeFormat>,
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

    pub data_source: DataSource,
    pub content: Vec<Arc<RwLock<Question>>>, /* NOTE: THIS KEPT AS ARC RWLOCK IS VERY
                                              * IMPORTANT FOR PYTHON BINDING TO WORK!!! */
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
            output.push_str(&question.read().to_sss_classic()?);
        }
        append_line(&mut output, "END RECORD");
        Ok(output)
    }

    /// check if question name `question_name` exists in `self.content` field
    /// case insensitive
    pub fn has_question(&self, question_name: &str) -> bool {
        self.content
            .par_iter()
            .find_first(|question| question.read().name.eq_ignore_ascii_case(question_name))
            .is_some()
    }

    pub fn get_question(&self, question_name: &str) -> Option<&Arc<RwLock<Question>>> {
        self.content.par_iter().find_first(|question| {
            question.read().name.to_uppercase() == question_name.to_uppercase()
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Question {
    pub index: u32,
    pub name: String,
    pub label: Text,
    pub filter: Option<String>,
    pub format: CodeFormat,
    pub usage: Usage,
    pub response: Response,
    pub location: Position,
}

impl Question {
    /// Parses one Classic `VARIABLE` block or XML `<variable>` fragment.
    pub fn from_string(input: &str) -> Result<Self> {
        let is_classic = is_sss_classic(input);
        let stream = input.lines().map(|s| Ok::<_, BadLine>(s.to_string()));
        let builder = crate::read::classic::collect_question_stream(stream, 0)?;
        builder.build(is_classic)
    }

    /// Export fragments of `Question` instance from from SSS classic
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

/// describes the role of this variable in the survey.
/// Only a subset of variable types may have a use attribute (as detailed below)
#[derive(Clone, Debug, Default, EnumString, EnumIter, PartialEq)]
pub enum Usage {
    /// plain data
    #[default]
    Null,

    /// this variable contains the serial number (or other identification field)
    #[strum(serialize = "serial")]
    Serial,
    /// this variable contains the weighting
    #[strum(serialize = "weight")]
    Weight,
}

// TODO! ADD DOCUMENTATION
#[derive(Debug, Clone, EnumString, EnumIter, Default, PartialEq)]
pub enum CodeFormat {
    #[default]
    #[strum(serialize = "numeric")]
    UnSigned,
    #[strum(serialize = "literal")]
    Literal,
}

impl CodeFormat {
    pub fn name(&self) -> &str {
        match self {
            Self::Literal => "literal",
            Self::UnSigned => "numeric",
        }
    }

    pub fn all_format_names() -> impl Iterator<Item = Arc<str>> {
        Self::iter().map(|format| format.name().into())
    }
}

#[derive(Debug, Clone)]
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
    Note(Arc<RwLock<Note>>),

    /// Single/ Multiple response type with code value being u32, `format`
    /// attribute `numeric`
    NumericCategory(Arc<RwLock<NumericCategory>>),

    /// Single response question with `format='literal'`, introced since SSS
    /// 2.0. /// Introduced since SSS 2.0, `TextCategory` type is converted
    /// from variables of type 'single' with `format` attribute 'literal'.
    /// An example would be:
    ///
    /// ```text
    /// <variable ident="11" type="single" format="literal">
    ///     <values>
    ///         <value code="A">Within 3 months</value>
    ///         <value code="B">Between 3 months and 1 year</value>
    ///         <value code="C">More than 1 years time</value>
    /// ```
    ///
    /// This formatting is EXTREMELY difficult to work around in rust, even with
    /// generics added in. Hence the they gets treated as their own type
    /// instead. `TextCategory` type can ONLY be single-response and No
    /// `range` field
    TextCategory(Arc<RwLock<TextCategory>>),

    /// Logical response type
    Logical(Arc<RwLock<Logical>>),

    Date(Arc<RwLock<RangeDate>>),
    Time(Arc<RwLock<RangeTime>>),
}

impl Wrap for Response {}

impl Default for Response {
    fn default() -> Self {
        Self::new_note().expect("Creation of 'Note' variant is EXTREMELY unlikely to fail")
    }
}

impl Response {
    pub fn from_string(_input: &str) -> Result<Self> {
        // plan: put input inside a template, parse as question, return the
        // question.resposne
        todo!()
    }

    #[inline]
    fn lock_it<T>(item: T) -> Arc<RwLock<T>> {
        Arc::new(RwLock::new(item))
    }

    pub fn new_note() -> Result<Self> {
        Self::Note(Arc::new(RwLock::new(Note::new()))).wrap()
    }

    pub fn new_single_numerical() -> Result<Self> {
        let instance = NumericCategory::new(vec![], true)?;
        Self::NumericCategory(Self::lock_it(instance)).wrap()
    }

    pub fn new_single_literal() -> Result<Self> {
        let instance = TextCategory::default();
        Self::TextCategory(Self::lock_it(instance)).wrap()
    }

    pub fn new_multiple() -> Result<Self> {
        let instance = NumericCategory::new(vec![], false)?;
        Self::NumericCategory(Self::lock_it(instance)).wrap()
    }

    pub fn new_logical() -> Result<Self> {
        Self::Logical(Arc::new(RwLock::new(Logical::new()))).wrap()
    }

    pub fn new_quantity() -> Result<Self> {
        Self::Quantity(Arc::new(RwLock::new(Quantity::default()))).wrap()
    }

    pub fn new_character() -> Result<Self> {
        Self::Character(Arc::new(RwLock::new(Character::default()))).wrap()
    }

    pub fn new_range_date() -> Result<Self> {
        Self::Date(Arc::new(RwLock::new(RangeDate::default()))).wrap()
    }

    pub fn new_range_time() -> Result<Self> {
        Self::Time(Arc::new(RwLock::new(RangeTime::default()))).wrap()
    }

    /// Count explicitly listed codes (excluding implicit ranges).
    pub fn num_values(&self) -> Result<usize> {
        match self {
            Self::NumericCategory(item) => Ok(item.read().values.len()),
            Self::TextCategory(item) => Ok(item.read().values.len()),
            Self::Quantity(item) => Ok(item.read().values.len()),
            _ => Ok(0),
        }
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        match self {
            Self::Character(item) => item.read().to_sss_classic(),
            Self::NumericCategory(item) => item.read().to_sss_classic(),
            Self::Quantity(item) => item.read().to_sss_classic(),
            Self::Logical(_) => Ok("TYPE LOGICAL\n".into()),
            Self::Date(_) | Self::Time(_) | Self::Note(_) | Self::TextCategory(_) => {
                let error = UnsupportedFeature {
                    current_version: "classic".into(),
                    feature: format!("{:?}", self),
                };
                Err(error.into())
            }
        }
    }

    pub fn is_categorical(&self) -> bool {
        matches!(
            self,
            Self::NumericCategory(_) | Self::Logical(_) | Self::TextCategory(_)
        )
    }

    pub fn is_scale(&self) -> bool {
        matches!(self, Self::Date(_) | Self::Time(_) | Self::Quantity(_))
    }

    pub fn is_open_end(&self) -> bool {
        matches!(
            self,
            Self::Quantity(_) | Self::Character(_) | Self::Time(_) | Self::Date(_)
        )
    }

    pub fn response_type(&self) -> &'static str {
        match self {
            Self::NumericCategory(_) => type_name::<NumericCategory>(),
            Self::TextCategory(_) => type_name::<TextCategory>(),
            Self::Quantity(_) => type_name::<Quantity>(),
            Self::Note(_) => type_name::<Note>(),
            Self::Logical(_) => type_name::<Logical>(),
            Self::Character(_) => type_name::<Character>(),
            Self::Date(_) => type_name::<RangeDate>(),
            Self::Time(_) => type_name::<RangeTime>(),
        }
    }
}

impl PartialEq for Response {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Logical(_), Self::Logical(_)) => true,
            (Self::Note(_), Self::Note(_)) => true,
            (Self::Character(self_item), Self::Character(other_item)) => {
                self_item.read().size == other_item.read().size
            }
            (Self::Date(self_item), Self::Date(other_item)) => {
                self_item.read().range == other_item.read().range
            }
            (Self::Time(self_item), Self::Time(other_item)) => {
                self_item.read().range == other_item.read().range
            }
            (Self::Quantity(self_item), Self::Quantity(other_item)) => {
                (self_item.read().range == other_item.read().range)
                    & (self_item.read().values == other_item.read().values)
            }
            (Self::NumericCategory(self_item), Self::NumericCategory(other_item)) => {
                self_item.read().eq(&other_item.read())
            }
            _ => false,
        }
    }
}

impl FromStr for Response {
    type Err = BadLine;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let to_match = s.trim().to_ascii_lowercase();
        match to_match.as_str() {
            "single" => Self::new_single_numerical().map_err(BadLine::from),
            "multiple" => Self::new_multiple().map_err(BadLine::from),
            "quantity" => Self::new_quantity().map_err(BadLine::from),
            "character" => Self::new_character().map_err(BadLine::from),
            "logical" => Self::new_logical().map_err(BadLine::from),
            "date" => Self::new_range_date().map_err(BadLine::from),
            "time" => Self::new_range_time().map_err(BadLine::from),
            _ => Err(BadLine::new(format!(
                "Unidentified response kind: '{}'",
                to_match
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Quantity {
    pub range: Range<f64>,
    pub values: Vec<Code<f64>>,
}

impl Quantity {
    pub fn values(&self) -> &[Code<f64>] {
        &self.values
    }

    /// Direct edits bypass validation; use `try_add_code` for checked
    /// insertion.
    pub fn values_mut(&mut self) -> &mut Vec<Code<f64>> {
        &mut self.values
    }

    pub fn try_add_code(&mut self, code: Code<f64>) -> Result<()> {
        self.values.push(code);
        Ok(())
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::new();
        append_line(&mut output, "TYPE QUANTITY");
        append_line(&mut output, "VALUES");
        append_line(
            &mut output,
            format_range(self.range.start, self.range.end, "TO"),
        );
        let choices = self
            .values
            .par_iter()
            .map(|item| {
                let mut clone = item.clone();
                clone.is_special = true;
                clone.to_sss_classic()
            })
            .collect::<Result<Vec<String>>>()?;

        for content in choices.iter() {
            append_line(&mut output, content);
        }

        append_line(&mut output, "END VALUES");
        Ok(output)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Character {
    pub size: u32,
}

impl Character {
    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::new();
        append_line(&mut output, "TYPE CHARACTER");
        append_line(&mut output, format!("SIZE {}", self.size));
        Ok(output)
    }
}

/// Introduced since SSS 2.0, `TextCategory` type is converted from variables of
/// type 'single' with `format` attribute 'literal'. An example would be:
///
/// ```text
/// <variable ident="11" type="single" format="literal">
///     <name>Q8</name>
///     <label>When is that most likely to be</label>
///     <position start="68" />
///     <filter>Q7</filter>
///     <!-- filter -->
///     <values>
///         <value code="A">Within 3 months</value>
///         <value code="B">Between 3 months and 1 year</value>
///         <value code="C">More than 1 years time</value>
///     ...
/// ```
///
/// This formatting is EXTREMELY difficult to work around in rust, even with
/// generics added in. Hence the they gets treated as their own type instead.
/// `TextCategory` type can ONLY be single-response and No `range` field
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextCategory {
    pub values: Vec<Code<String>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct NumericCategory {
    pub values: Vec<Code<u32>>,
    pub range: Option<Range<u32>>,
    pub is_single: bool,
    /// toggle `is_single` on/off won't overwrite existing value of `spread`,
    /// but hiding it from the user instead.
    pub spread: Option<Spread>,
}

impl NumericCategory {
    fn new(values: Vec<Code<u32>>, is_single: bool) -> Result<Self> {
        Ok(Self {
            values,
            is_single,
            range: None,
            spread: None,
        })
    }

    pub fn try_add_code(&mut self, code: Code<u32>) -> Result<()> {
        if let Some(found) = self.values.par_iter().find_first(|item| item.code == code.code) {
            Err(DuplicatedKey::new(found.code, &found.label.value).into())
        } else {
            self.values.push(code);
            Ok(())
        }
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = String::new();
        if self.is_single {
            append_line(&mut output, "TYPE SINGLE");
        } else {
            if let Some(spread) = self.spread.as_ref() {
                append_line(&mut output, spread.to_sss_classic());
            }
            append_line(&mut output, "TYPE MULTIPLE");
        }
        append_line(&mut output, "VALUES");

        if let Some(range) = &self.range {
            append_line(&mut output, format!("{} TO {}", range.start, range.end));
        }

        write_choices(&mut output, self.values.clone())?;
        Ok(output)
    }

    // return all entries with duplicated code value
    pub fn find_duplicated_entries(&self) -> Vec<&Code<u32>> {
        let mut groups: HashMap<u32, Vec<&Code<u32>>> = HashMap::new();

        for item in self.values.iter() {
            groups.entry(item.code).or_default().push(item);
        }

        groups.into_values().filter(|group| group.len() > 1).flatten().collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {}

impl Wrap for Note {}

impl Note {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Logical {}

impl Wrap for Logical {}

impl Logical {
    pub const ITEM: LazyLock<NumericCategory> = LazyLock::new(|| {
        NumericCategory::new(
            vec![
                Code::new(0_u64, "FALSE").unwrap(),
                Code::new(1_u64, "TRUE").unwrap(),
            ],
            true,
        )
        .unwrap()
    });

    pub fn new() -> Self {
        Self {}
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
pub struct Code<T: CodeType> {
    /// code value, this can be String, Float or u32
    pub code: T,

    /// text label
    pub label: Text,

    /// Response filter.
    pub filter: Option<String>,

    pub score: Option<f64>,

    /// (Optional) indicator for special values (e.g. missing data etc). Removed
    /// from SSS 1.2 onwards.
    pub is_special: bool,
}

impl<T: CodeType> Code<T> {
    /// create new instance with code & label whilest everything else set as
    /// their default value
    pub fn new<C>(code: C, label: impl Into<String>) -> Result<Self>
    where C: Into<T> {
        let code = code.into();
        code.validate()?;

        Ok(Self {
            code,
            label: Text::new(label),
            filter: None,
            score: None,
            is_special: false,
        })
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        if type_name::<T>() == type_name::<String>() {
            return Err(
                UnsupportedFeature::new("SSS Classic", "None numeric code values.").into(),
            );
        }
        let base = format!("{} {}", self.code, quote(self.label.to_sss_classic()));

        let content = if self.is_special {
            format!("{} SPECIAL", base)
        } else {
            base
        };

        Ok(content)
    }
}

/// Defines the part of the data record that is allocated to holding the value
/// of the variable.
#[derive(Debug, Clone, PartialEq, Default, Eq)]
pub struct Position {
    pub from: u32,
    pub to: u32,
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

#[derive(Debug, Clone, PartialEq, Default, Eq)]
pub struct Spread {
    /// The number of subfields within the overall field that is defined by the
    /// <position> element.
    pub subfield: u32,
    /// Denotes the width of each subfield.
    pub width: Option<u32>,
}

impl Spread {
    pub fn to_sss_classic(&self) -> String {
        match self.width.as_ref() {
            Some(width) => format!("SPREAD {} OF {}", self.subfield, width),
            None => format!("SPREAD {}", self.subfield),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Text {
    /// the default value
    pub value: Arc<RwLock<String>>,

    /// variants
    pub variant: Vec<TextVariant>,
}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.value.read().as_str() == other.value.read().as_str()
            && self.variant.len() == other.variant.len()
            && self.variant.iter().all(|variant| {
                other.variant.iter().any(|other| {
                    other.has_same_attribute(variant)
                        && other.value == variant.value
                })
            })
    }
}



impl From<String> for Text {
    fn from(value: String) -> Self {
        Self {
            value: lock_it_up(value),
            variant: Vec::new(),
        }
    }
}

impl From<std::sync::Arc<str>> for Text {
    fn from(value: std::sync::Arc<str>) -> Self {
        Self {
            value: lock_it_up(value.to_string()),
            variant: Vec::new(),
        }
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _variants = format!(", [{}] variants", self.variant.len());
        let variants = "";

        write!(
            f,
            r"Text(\n\tvalue='{}'\n\tvariant={}>",
            &self.value.read(), variants
        )
    }
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            value: lock_it_up(content.into()),
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
        self.value.read().clone()
    }

    pub fn contains_variant(&self, variant: &TextVariant) -> bool {
        self.variant
            .par_iter()
            .find_any(|item| item.has_same_attribute(variant))
            .is_some()
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextVariant {
    pub value: String,
    pub mode: Mode,
    language: Option<String>,
}

impl TextVariant {
    pub fn new<S>(value: S, mode: Mode, language: Option<S>) -> Result<Self>
    where S: Into<String> {
        let language = language
            .map(|lan| {
                let lan = lan.into();
                Self::validate_language_tag(&lan).map(|_| lan)
            })
            .transpose()?;

        Ok(Self {
            value: value.into(),
            mode,
            language: language,
        })
    }

    pub fn with_language(content: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            value: content.into(),
            language: Some(language.into()),
            mode: Mode::None,
        }
    }

    /// set `language` to `Language::Name(value)`
    pub fn set_language(&mut self, new: Option<&str>) -> Result<()> {
        if let Some(language) = new {
            Self::validate_language_tag(language)?;
            self.language = Some(language.into());
        } else {
            self.language = None
        };
        Ok(())
    }

    pub fn get_language(&self) -> Option<String> {
        self.language.clone()
    }

    /// reset `language` back to `Default`
    pub fn reset_language(&mut self) {
        self.language = None;
    }

    /// check if instance of `other` has the same attributes as `self`
    pub fn has_same_attribute(&self, other: &Self) -> bool {
        (self.mode == other.mode) & (self.language == other.language)
    }

    fn validate_language_tag(language: &str) -> Result<()> {
        let _ = oxilangtag::LanguageTag::parse(language).map_err(|err| {
            LanguageTagError(format!(
                "Error parsing language tag: {}. Detail: {}",
                language,
                err.to_string()
            ))
        })?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Default, Eq)]
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

impl Display for Language {
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
#[derive(Debug, PartialEq, Clone, EnumString, Eq, EnumIter)]
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
    pub fn try_as_str(&self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::Analysis => Some("analysis"),
            Self::Interview => Some("interview"),
        }
    }

    pub fn all_mode_options() -> Vec<&'static str> {
        Self::iter().filter_map(|mode| mode.try_as_str()).collect()
    }
}

// Helper functions
// ------------------------------------------------------------------------

#[inline]
fn quote(value: impl Display) -> String {
    format!(r#""{}""#, value)
}

#[inline]
fn append_line(output: &mut String, content: impl Display) {
    writeln!(output, "{content}").expect("writing to a String shouldn't fail");
}

#[inline]
fn write_choices<T: CodeType>(output: &mut String, choices: Vec<Code<T>>) -> Result<()> {
    for choice in choices {
        append_line(output, choice.to_sss_classic()?);
    }
    append_line(output, "END VALUES");
    Ok(())
}

#[inline]
fn format_range<T: Display>(from: T, to: T, delimiter: &str) -> String {
    if from.to_string() == to.to_string() {
        from.to_string()
    } else {
        format!("{from} {delimiter} {to}")
    }
}

#[inline]
fn is_sss_classic(text: &str) -> bool {
    !text.trim().starts_with('<')
}

#[inline]
pub fn lock_it_up<T>(item: T) -> Arc<RwLock<T>> {
    Arc::new(RwLock::new(item))
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
        let v1 = TextVariant::new("variant 1", Mode::None, Some("GB")).unwrap();
        let v2 = TextVariant::new("variant 2", Mode::None, Some("GB")).unwrap();

        instance.set_variant(v1);
        assert_eq!(instance.variant.len(), 1);
        instance.set_variant(v2);
        assert_eq!(instance.variant.len(), 1);
        assert_eq!(&instance.variant[0].value, "variant 2");

        let v3 = TextVariant::new("variant", Mode::None, Some("GB")).unwrap();
        instance.set_variant(v3);
        assert_eq!(instance.variant.len(), 1, "{:?}", instance.variant);

        let v4 = TextVariant::new("variant", Mode::Analysis, Some("GB")).unwrap();
        instance.set_variant(v4);
        assert_eq!(instance.variant.len(), 2, "{:?}", instance.variant)
    }

    #[test]
    fn test_duplicated_codes() {
        let mut response = NumericCategory::new(vec![], true).unwrap();
        let code1 = Code::new(1 as u32, "test").unwrap();
        let code2 = Code::new(2 as u32, "test").unwrap();
        let code3 = Code::new(3 as u32, "test").unwrap();

        response.values.push(code1.clone());
        response.values.push(code1.clone());
        response.values.push(code1.clone());
        response.values.push(code2.clone());
        response.values.push(code3.clone());
        response.values.push(code3.clone());

        let duplicated = response.find_duplicated_entries();
        assert_eq!(duplicated.len(), 5)
    }

    /// field `code` for struct `Code` can only be one of the:
    /// - None-NaN, None-infinity float
    /// - u32,
    /// - String
    #[test]
    fn test_allowed_code_types() {
        assert!(Code::<f64>::new(f64::NAN, "test").is_err());
        assert!(Code::<f64>::new(f64::INFINITY, "test").is_err());
        assert!(Code::<f64>::new(f64::NEG_INFINITY, "test").is_err());
    }

    // #[test]
    // fn test_create_question_from_rust() {
    //     todo!("This is indended to create an API for python side to create
    // new instance") }

    // #[test]
    // fn test_create_survey_from_rust() {
    //     todo!("This is indended to create an API for python side to create
    // new instance") }

    // #[test]
    // fn test_create_response_from_rust() {
    //     todo!("This is indended to create an API for python side to create
    // new instance") }

    // #[test]
    // fn test_convert_between_single_and_multiple() {
    //     // have some think about the current design of Response enum
    //     // should type <-> response being two separate things?
    //     // a master Response struct that contains everything needed, and
    // return     // thing according current type of response
    //     // provide a '.convert(resp_type)' method ?? to switch betwen types
    //     // and raises error when .convert betwen f64 <-> string
    //     //
    //     // OK ACTUAL IDEA:
    //     // Or: make current `Response` enum as a backend, rename to something
    //     // like `ResponseData` then createe a new `Response` struct that
    //     // exploses public API
    //     //
    //     // checkout: `TEMP`
    //     todo!()
    // }
}
