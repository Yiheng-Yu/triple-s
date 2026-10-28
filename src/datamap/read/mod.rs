pub mod classic;
pub mod xml;
use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::{BufRead, BufReader, Cursor, Read},
    path::Path,
    range::Range,
    str::FromStr,
    sync::Arc,
};

use anyhow::{Context, Result};
use log::warn;
use quick_xml::{encoding::DecodingReader, reader::Reader as XmlReader};
use rayon::prelude::*;

use crate::{
    DataRange,
    error::{
        self,
        BadLine,
        DuplicatedKey,
        EventLog,
        MissingAttributeError,
        MissingValueError,
        UnknownAttribute,
        Warning,
    },
    datamap::{
        Code,
        CodeFormat,
        DataFormat,
        Location,
        Question,
        QuestionType,
        Response,
        Spread,
        Survey,
        Text,
        Usage,
        Value,
        Variant,
        Version,
    },
    prelude::*,
    utils::{lock_it_up, validate_date, validate_time},
};

#[derive(Debug, PartialEq)]
pub enum FileType {
    Classic,
    Xml,
}

impl FileType {
    pub fn parse(first_line: &str) -> Result<Self, error::BadLine> {
        match first_line.chars().next().unwrap_or_default() {
            'S' => Ok(Self::Classic),
            '<' => Ok(FileType::Xml),
            _ => Err(error::BadLine::new(format!(
                "Unable to identify sss version from the first line: '{first_line}'"
            ))),
        }
    }
}

/// Read triple-s datamap.
///
/// This functions auto detects triple-s version (classic/ xml), and currently
/// supports up till triple-s version 2.0 with non-hierarchial data. Triple-s
/// 3.0 stuff _should_be_ mostly okay, but fully tested (yet).
///
/// ## Example usage:
///
/// ```no_run
/// use triple_s::read_datamap;
///
/// let survey = read_datamap("../path/to/survey.sss").unwrap();
/// println!("{:?}", survey.title());
///
/// Ok::<(), Box<dyn std::error::Error>>(());
/// ```
pub fn read_datamap(sss_path: impl AsRef<Path>) -> Result<Survey> {
    let path = sss_path.as_ref();
    let input = File::open(path)
        .map_err(error::FileReadError::from)
        .with_context(|| format!("When opening '{}'", path.display()))?;

    let (survey, read_log) = read_from_reader(BufReader::new(input))
        .with_context(|| format!("When reading '{}'", path.display()))?;

    let context = format!("Reading datamap: '{:?}'", path.as_os_str());
    if let Some(summary) = read_log.warning_summary(context) {
        warn!("{}", summary);
    };

    Ok(survey)
}

/// create datamap instance from utf-8 string, auto detects file types (sss
/// classic/ xml)
pub fn read_from_string(data: impl AsRef<str>) -> Result<Survey> {
    let (survey, read_log) = read_from_reader(Cursor::new(data.as_ref().as_bytes()))?;

    if let Some(summary) = read_log.warning_summary("Reading from string") {
        warn!("{}", summary);
    };

    Ok(survey)
}

/// create datamap instance from a reader/ stream, auto detects file types (sss
/// classic/ xml)
pub fn read_from_reader<R: BufRead>(mut input: R) -> Result<(Survey, EventLog)> {
    let mut prefix = Vec::new();
    let mut first_line = true;
    loop {
        if input.read_until(b'\n', &mut prefix)? == 0 {
            return Err(error::FileReadError::new("file seems to be empty").into());
        }
        if first_line && prefix.starts_with(b"\xef\xbb\xbf") {
            prefix.drain(..3);
        }
        first_line = false;
        if let Some(start) = prefix.iter().position(|byte| !byte.is_ascii_whitespace()) {
            prefix.drain(..start);
            break;
        }
        prefix.clear();
    }
    let file_type = FileType::parse(std::str::from_utf8(&prefix[..1])?)?;
    let input = BufReader::new(Cursor::new(prefix).chain(input));
    let builder = match file_type {
        FileType::Classic => classic::create_survey_builder(input.lines())?,
        FileType::Xml => {
            let mut reader = XmlReader::from_reader(DecodingReader::new(input));
            reader.config_mut().trim_text(true);
            xml::create_survey_builder(reader)?
        }
    };
    builder.build()
}

pub trait ParseRange<T>
where
    T: FromStr + Clone,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    const DELIMITER: &str;
    fn from_text(text: &str) -> Result<Range<T>, anyhow::Error>;
}

impl<T> ParseRange<T> for Range<T>
where
    T: FromStr + Clone,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    const DELIMITER: &str = "TO";

    fn from_text(text: &str) -> Result<Range<T>, anyhow::Error> {
        let text = text.to_uppercase();

        let parts = text
            .split(Self::DELIMITER)
            .filter(|s| !s.is_empty())
            .map(|s| s.trim())
            .collect::<Vec<&str>>();

        if parts.len() > 2 {
            let error = BadLine::new(format!(
                "Unable to parse range from text '{}', there seems to be more than one delimiter ('TO') init.",
                text
            ));
            return Err(error.into());
        };

        let from = parts
            .first()
            .ok_or(BadLine::new(format!(
                "Unable to parse range from text: {}",
                text
            )))?
            .parse::<T>()
            .map_err(anyhow::Error::from)
            .with_context(|| format!("Text: {}", text))?;

        let to = parts
            .get(1)
            .map(|s| s.parse::<T>())
            .transpose()
            .map_err(anyhow::Error::from)
            .with_context(|| format!("Text: {}", text))?
            .unwrap_or(from.clone());

        let range = Range {
            start: from,
            end: to,
        };
        Ok(range)
    }
}

// ----------------- helpers ------------------
/// A xml-like node, used for parsing sss files of different structure
#[derive(Debug, Clone)]
pub struct Node {
    pub tag: Arc<str>,
    pub attributes: HashMap<Arc<str>, Arc<str>>,
    pub value: Option<Arc<str>>,
    pub children: Vec<Node>,
}

impl Node {
    /// Create a parser node with no attributes, value, or children.
    /// This constructor is internal to the crate.
    ///
    /// # Examples
    ///
    /// ```text
    /// let node = Node::new("variable");
    /// ```
    pub fn new(tag: impl Into<Arc<str>>) -> Self {
        Self {
            tag: tag.into(),
            attributes: HashMap::new(),
            value: None,
            children: Vec::new(),
        }
    }

    pub fn with_value(tag: impl Into<Arc<str>>, value: impl Into<Arc<str>>) -> Self {
        Self {
            tag: tag.into(),
            attributes: HashMap::new(),
            value: Some(value.into()),
            children: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct SurveyBuilder {
    is_classic: bool,
    pub attributes: HashMap<Arc<str>, Node>, // survey name, id etc.,
    pub questions: Vec<QuestionBuilder>,     // content of the questionnaire
}

impl SurveyBuilder {
    pub fn empty(is_classic: bool) -> Self {
        Self {
            is_classic,
            attributes: HashMap::new(),
            questions: Vec::new(),
        }
    }

    pub fn build(mut self) -> Result<(Survey, EventLog)> {
        let mut output = Survey::empty();
        let mut log = EventLog::new();

        output.info.title =
            self.attributes.remove("title").and_then(|val| val.value.map(unquote));

        output.info.name =
            self.attributes.remove("name").and_then(|val| val.value.map(unquote));

        if let Some(record) = self.attributes.remove("record") {
            if self.is_classic {
                *output.record.ident_mut() = record
                    .value
                    .and_then(|value| value.chars().next())
                    .ok_or(MissingValueError::new("survey", "record"))?;
            } else {
                *output.record.ident_mut() = record
                    .attributes
                    .get("ident")
                    .and_then(|value| value.chars().next())
                    .ok_or(MissingAttributeError::new("record", "ident"))?;

                *output.record.skip_mut() =
                    record.attributes.get("skip").map(ToString::to_string);
                *output.record.href_mut() =
                    record.attributes.get("href").map(ToString::to_string);

                *output.record.data_format_mut() = record
                    .attributes
                    .get("format")
                    .map(|value| DataFormat::from_str(value))
                    .transpose()
                    .context("When parsing record attribute 'format'")?;
            }
        }

        output.info.origin =
            self.attributes.remove("origin").and_then(|val| val.value.map(unquote));

        output.info.user =
            self.attributes.remove("user").and_then(|val| val.value.map(unquote));

        output.info.date =
            self.attributes.remove("date").and_then(|val| val.value.map(unquote));

        output.info.time =
            self.attributes.remove("time").and_then(|val| val.value.map(unquote));

        output.info.standard =
            self.attributes.remove("standard").and_then(|val| val.value.map(unquote));

        output.info.version = self
            .attributes
            .remove("version")
            .ok_or(MissingAttributeError::new("sss", "version"))?
            .value
            .map(|val| Version::from_str(val.as_ref()))
            .transpose()?
            .ok_or(MissingValueError::new("sss", "version"))?;

        let is_classic = self.is_classic;
        let results = self
            .questions
            .par_drain(..)
            .map(|item| item.build(is_classic))
            .collect::<Result<Vec<(Question, Vec<Warning>)>>>()?;

        for (question, warnings) in results {
            output.record.content_mut().push(lock_it_up(question));
            log.push_warnings(warnings);
        }
        output.record.content_mut().shrink_to_fit();
        Ok((output, log))
    }
}

#[derive(Debug)]
pub struct QuestionBuilder {
    pub attributes: HashMap<Arc<str>, Node>,
    pub responses: Node, // response codes
}

impl QuestionBuilder {
    pub fn empty() -> Self {
        Self {
            attributes: HashMap::new(),
            responses: Node::new("record"),
        }
    }

    fn parse_location_classic(node: &Node) -> Result<Location> {
        let value = node
            .value
            .as_ref()
            .ok_or(error::MissingValueError::new("variable", "Position"))?;

        let range = Range::<u32>::from_text(value)?;
        Ok(Location::new(range.start, Some(range.end))?)
    }

    fn parse_location_xml(node: &Node) -> Result<Location> {
        let from = node
            .attributes
            .get("start")
            .ok_or(error::MissingValueError::new("Position", "start"))?
            .parse::<u32>()?;

        let to = node
            .attributes
            .get("finish")
            .map(|s| s.parse::<u32>())
            .transpose()?
            .unwrap_or(from);

        Ok(Location::new(from, Some(to))?)
    }

    pub fn build(self, is_classic: bool) -> Result<(Question, Vec<Warning>)> {
        let name = self
            .attributes
            .get("name")
            .and_then(|node| node.value.as_deref())
            .unwrap_or("<unnamed>")
            .to_owned();
        let index = self
            .attributes
            .get(if is_classic { "variable" } else { "ident" })
            .and_then(|node| node.value.as_deref())
            .unwrap_or("<missing>")
            .to_owned();
        self.build_inner(is_classic)
            .with_context(|| format!("When parsing question {name} (index {index})"))
    }

    fn build_inner(mut self, is_classic: bool) -> Result<(Question, Vec<Warning>)> {
        let mut output = Question::default();
        let mut warnings = Vec::new();

        // mandatory: question index
        let attrib_name = if is_classic { "variable" } else { "ident" };
        *output.index_mut() = self
            .attributes
            .remove(attrib_name)
            .and_then(|val| val.value.map(|idx| idx.trim().parse::<u32>()))
            .ok_or(error::MissingAttributeError::new("variable", attrib_name))?
            .map_err(|err| BadLine::new(format!("Unable to parse variable index: {}", err)))?;

        // mandatory: question name
        *output.name_mut() = self
            .attributes
            .remove("name")
            .and_then(|val| val.value.map(unquote))
            .ok_or(error::MissingAttributeError::new("variable", "name"))?;

        let question_name = output.name().to_string();

        // sss classic does not have text variant
        let label = self
            .attributes
            .remove("label")
            .ok_or(error::MissingAttributeError::new("variable", "label"))
            .with_context(|| format!("When parsing question {}", question_name))?;
        let label = parse_text(label, is_classic)
            .with_context(|| format!("When parsing question {}", question_name))?;
        *output.label_mut() = label;

        let location = self
            .attributes
            .remove("position")
            .ok_or(error::MissingAttributeError::new("variable", "position"))
            .with_context(|| format!("When parsing question {}", question_name))?;

        *output.location_mut() = if is_classic {
            Self::parse_location_classic(&location)
                .with_context(|| format!("When parsing question {}", question_name))?
        } else {
            Self::parse_location_xml(&location)
                .with_context(|| format!("When parsing question {}", question_name))?
        };

        *output.format_mut() = self
            .attributes
            .remove("format")
            .and_then(|node| node.value.map(|s| CodeFormat::from_str(s.as_ref())))
            .transpose()
            .context("When parsing field 'format'")?;

        *output.usage_mut() = self
            .attributes
            .remove("use")
            .and_then(|node| node.value.map(|s| Usage::from_str(s.as_ref())))
            .transpose()
            .context("When parsing field 'use'")?;

        *output.filter_mut() = self
            .attributes
            .remove("filter")
            .and_then(|node| node.value.map(|s| s.to_string()));

        // build question responses
        let type_name = self
            .attributes
            .remove("type")
            .ok_or(error::MissingAttributeError::new("variable", "type"))
            .with_context(|| format!("When parsing question {}", question_name))?
            .value
            .ok_or(error::MissingValueError::new("variable", "type"))
            .with_context(|| format!("When parsing question {}", question_name))?;

        *output.question_type_mut() = QuestionType::from_str(type_name.as_ref())
            .with_context(|| {
                format!(
                    "Error identifying question type. Expect to be one of: '{:?}', got: '{}'",
                    QuestionType::available_options(),
                    type_name.as_ref()
                )
            })
            .with_context(|| format!("When parsing question {}", question_name))?;

        // update spread
        if let Some(mut node) = self.attributes.remove("spread") {
            let spread = if is_classic {
                let raw_spread =
                    node.value.ok_or(MissingAttributeError::new("spread", "subfields"))?;

                let splitted = raw_spread.trim().splitn(2, " OF ").collect::<Vec<&str>>();
                let subfields = splitted
                    .first()
                    .ok_or(MissingAttributeError::new("spread", "subfields"))?
                    .parse::<u32>()
                    .context("When parsing spread <subfield>")?;
                let width = splitted
                    .get(1)
                    .map(|value| value.parse::<u32>())
                    .transpose()
                    .context("When parsing spread <width>")?;

                Spread::new(subfields, width).context("When parsing spread")?
            } else {
                let subfields = node
                    .attributes
                    .remove("subfields")
                    .as_deref()
                    .map(|item| item.parse::<u32>())
                    .transpose()
                    .context("When parsing spread attributes")?
                    .ok_or(MissingAttributeError::new("spread", "subfields"))?;

                let width = node
                    .attributes
                    .remove("width")
                    .as_deref()
                    .map(|item| item.parse::<u32>())
                    .transpose()
                    .context("When parsing spread attributes")?;

                Spread::new(subfields, width).context("When parsing spread")?
            };
            *output.spread_mut() = Some(spread);
        };

        let response = ResponseBuilder::from_question_read(self.responses.children)
            .context("When parsing field 'values'")?;
        // ..then build respective entries based on the question type and format
        let question_type = output.question_type().clone();
        match question_type {
            QuestionType::Logical | QuestionType::None => (),
            QuestionType::Single => {
                if Some(&CodeFormat::Literal) == output.format().as_ref() {
                    let (response, resp_warnings) =
                        response.build_literal_category(&question_name, is_classic)?;

                    *output.response_mut() = response;
                    warnings.extend(resp_warnings);
                } else {
                    let (response, resp_warnings) =
                        response.build_numeric(ValueKind::UnSigned, is_classic)?;

                    *output.response_mut() = response;
                    warnings.extend(resp_warnings);
                }
            }
            QuestionType::Multiple => {
                let (response, resp_warnings) =
                    response.build_numeric(ValueKind::UnSigned, is_classic)?;
                *output.response_mut() = response;
                warnings.extend(resp_warnings);
            }
            QuestionType::Quantity => {
                let (response, resp_warnings) =
                    response.build_numeric(ValueKind::Float, is_classic)?;
                *output.response_mut() = response;
                warnings.extend(resp_warnings);
            }
            QuestionType::Date => {
                let (response, resp_warnings) =
                    response.build_numeric(ValueKind::Signed, is_classic)?;
                if let Some(range) = response.range().as_ref() {
                    validate_date(&range.start().to_string())?;
                    validate_date(&range.end().to_string())?;
                };
                *output.response_mut() = response;
                warnings.extend(resp_warnings);
            }
            QuestionType::Time => {
                let (response, resp_warnings) =
                    response.build_numeric(ValueKind::Signed, is_classic)?;
                if let Some(range) = response.range().as_ref() {
                    validate_time(&range.start().to_string())?;
                    validate_time(&range.end().to_string())?;
                };
                *output.response_mut() = response;
                warnings.extend(resp_warnings);
            }
            QuestionType::Character => {
                if let Some(size) = response.size {
                    let width = output.location().width()?;
                    let range = DataRange::new(
                        Value::UnSigned(1),
                        Value::UnSigned(size.saturating_add(1)),
                    )?;
                    let response = Response::new(Some(range), Vec::new());
                    *output.response_mut() = response;
                    if size > width {
                        warnings.push(Warning {
                            location: Some(question_name.clone()),
                            subfield: Some("value/size".to_string()),
                            kind: error::WarningKind::DataLocation,
                            message: format!(
                                "Field 'size' ({}) exceeds the width defined in question's data location ({}).",
                                size, width
                            )
                        })
                    }
                }
            }
        }

        for warning in &mut warnings {
            warning.location = Some(question_name.clone());
        }
        Ok((output, warnings))
    }
}

/// unquote texts but raises error if text was not quoted.
#[inline]
fn unquote(text: Arc<str>) -> String {
    let to_unquote = text.trim();
    if to_unquote.starts_with('"') || to_unquote.ends_with('"') {
        to_unquote[1..to_unquote.len() - 1].to_string()
    } else {
        to_unquote.to_string()
    }
}

fn parse_text(mut node: Node, is_classic: bool) -> Result<Text> {
    let value = node.value.take().unwrap_or_default();
    let value = if is_classic {
        unquote(value)
    } else {
        value.trim().to_string()
    };
    let mut text = Text::new(value.as_str());
    for variant in node.children {
        let value = variant
            .value
            .as_ref()
            .ok_or(MissingValueError::new("text", "value"))?
            .to_string();
        let language = variant
            .attributes
            .get("xml:lang")
            .or_else(|| variant.attributes.get("lang"))
            .map(|language| language.as_ref());

        let mode = variant.attributes.get("mode").map(|s| s.as_ref());

        let variant = Variant::new(value, mode, language)?;
        text.add_variant(variant)?;
    }
    Ok(text)
}

#[derive(Clone, Copy)]
pub enum ValueKind {
    Signed,
    UnSigned,
    Float,
    String,
}

#[derive(Debug)]
struct RawCode {
    value: Arc<str>,
    label: Node,
    score: Option<Arc<str>>,
    filter: Option<Arc<str>>,
}

// extract information from raw data first, build response later
#[derive(Debug, Default)]
struct ResponseBuilder {
    range_from: Option<Arc<str>>,
    range_to: Option<Arc<str>>,
    size: Option<u32>,
    entries: Vec<RawCode>,
}

impl ResponseBuilder {
    fn extract_value_entry(node: &mut Node) -> Result<RawCode> {
        let value = node
            .attributes
            .remove("code")
            .ok_or(MissingAttributeError::new("value", "code"))?;
        let score = node.attributes.remove("score");
        let filter = node.attributes.remove("filter");
        let mut label = Node::new("label");
        label.value = node.value.take();
        label.children = std::mem::take(&mut node.children);
        label.attributes = std::mem::take(&mut node.attributes);
        Ok(RawCode {
            value,
            label,
            score,
            filter,
        })
    }

    pub fn from_question_read(mut read: Vec<Node>) -> Result<Self> {
        let mut instance = Self::default();

        if let Some((first_entry, rest)) = read.split_first_mut() {
            // parse 'range' attribute
            if first_entry.tag.as_ref() == "range" {
                instance.range_from = first_entry.attributes.remove("from");
                instance.range_to = first_entry.attributes.remove("to");
            } else if first_entry.tag.as_ref() == "value" {
                instance.entries.push(Self::extract_value_entry(first_entry)?);
            } else if first_entry.tag.as_ref() == "size" {
                instance.size = first_entry
                    .value
                    .as_ref()
                    .map(|s| s.parse::<u32>())
                    .transpose()
                    .context("When parsing response field 'size'")?;
            } else {
                return Err(UnknownAttribute::new("values", first_entry.tag.as_ref()).into());
            };

            // collect code values
            let values = rest
                .par_iter_mut()
                .map(|item| {
                    debug_assert!(item.tag.as_ref() == "value");
                    Self::extract_value_entry(item)
                })
                .collect::<Result<Vec<RawCode>>>()?;
            instance.entries.extend(values);
        } else {
            return Ok(Self::default());
        }
        Ok(instance)
    }

    fn extract_range(&self, kind: ValueKind) -> Result<DataRange> {
        if let Some(from) = self.range_from.as_ref() {
            let from = match kind {
                ValueKind::Signed => {
                    let value = from.parse::<i32>()?;
                    Value::Signed(value)
                }
                ValueKind::UnSigned => {
                    let value = from.parse::<u32>()?;
                    Value::UnSigned(value)
                }
                ValueKind::Float => {
                    let value = from.parse::<f64>()?;
                    Value::Float(value)
                }
                _ => todo!(),
            };

            let to = self
                .range_to
                .as_ref()
                .map(|val| from.parse_like(val.as_ref()))
                .transpose()?
                .ok_or(MissingAttributeError::new("range", "to"))?;
            DataRange::new(from, to).map_err(|err| err.into())
        } else {
            Err(MissingAttributeError::new("range", "from").into())
        }
    }

    fn extract_code(raw: RawCode, format: ValueKind, is_classic: bool) -> Result<Code> {
        let code_name = raw.value.to_string();

        let label = parse_text(raw.label, is_classic).context(format!(
            "When parsing response label for code '{}'",
            code_name
        ))?;

        let score =
            raw.score.map(|value| value.parse::<f64>()).transpose().context(format!(
                "When parsing response attribute 'score' for code {}",
                code_name
            ))?;

        let value = match format {
            ValueKind::String => Value::String(raw.value.to_string()),
            ValueKind::UnSigned => {
                Value::UnSigned(raw.value.parse().context("Invalid unsigned code")?)
            }
            ValueKind::Signed => {
                Value::Signed(raw.value.parse().context("Invalid signed code")?)
            }
            ValueKind::Float => {
                Value::Float(raw.value.parse().context("Invalid floating-point code")?)
            }
        };
        let instance = Code {
            code: lock_it_up(value),
            label: lock_it_up(label),
            filter: lock_it_up(raw.filter.map(|s| s.to_string())),
            score: lock_it_up(score),
        };
        Ok(instance)
    }

    fn build_entries(
        self,
        range: Option<DataRange>,
        kind: ValueKind,
        is_classic: bool,
    ) -> Result<Response> {
        let values = self
            .entries
            .into_par_iter()
            .map(|raw| {
                let code = Self::extract_code(raw, kind, is_classic)?;
                Ok(code)
            })
            .collect::<Result<Vec<_>>>()?;

        // note: the 'special' flag is ignored for code value for now
        // TODO: MAKE IT NOT IGNORED!
        Ok(Response::new(range, values))
    }

    fn build_literal_category(
        self,
        question_name: &str,
        is_classic: bool,
    ) -> Result<(Response, Vec<Warning>)> {
        let range = if self.range_from.is_some() {
            Some(
                self.extract_range(ValueKind::UnSigned)
                    .context("When parsing response range")?,
            )
        } else {
            None
        };
        let has_range = range.is_some();
        let response = self.build_entries(range, ValueKind::String, is_classic)?;

        let warning = if has_range {
            vec![Warning {
                location: Some(question_name.into()),
                subfield: Some("values".into()),
                kind: error::WarningKind::UnexpectedAttribute,
                message: "Value range is not valid when format is 'literal'.".into(),
            }]
        } else {
            Vec::new()
        };

        Ok((response, warning))
    }

    fn build_numeric(
        self,
        kind: ValueKind,
        is_classic: bool,
    ) -> Result<(Response, Vec<Warning>)> {
        let range = if self.range_from.is_some() {
            Some(self.extract_range(kind).context("When parsing response range")?)
        } else {
            None
        };
        Ok((self.build_entries(range, kind, is_classic)?, Vec::new()))
    }
}

pub fn try_insert<K, V>(
    map: &mut std::collections::HashMap<K, V>,
    key: K,
    value: V,
) -> Result<()>
where
    K: std::fmt::Debug + Eq + Hash + Clone,
    V: std::fmt::Debug,
{
    let key_cloned = key.clone(); // for logging error
    if let Some(prev_value) = map.insert(key_cloned, value) {
        Err(DuplicatedKey::new(key, prev_value).into())
    } else {
        Ok(())
    }
}
