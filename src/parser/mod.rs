mod classic;
mod xml;

use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::{BufRead, BufReader, Cursor},
    range::Range,
    str::FromStr,
    sync::{Arc, LazyLock},
};

use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveTime};
use parking_lot::RwLockWriteGuard;
use quick_xml::{encoding::DecodingReader, reader::Reader as XmlReader};
use rayon::prelude::*;

use crate::{
    entities::{
        Code,
        Format,
        Language,
        Mode,
        Position,
        Question,
        Response,
        Spread,
        Survey,
        Text,
        TextVariant,
        Version,
    },
    error::{
        self,
        BadLine,
        DuplicatedKey,
        InternalError,
        MissingAttributeError,
        MissingValueError,
    },
};

/// tag names
const RANGE: LazyLock<Arc<str>> = LazyLock::new(|| Arc::from("range"));
const FLAG: LazyLock<Arc<str>> = LazyLock::new(|| Arc::from("flag"));

/// attribute names
const FROM: LazyLock<Arc<str>> = LazyLock::new(|| Arc::from("from"));
const TO: LazyLock<Arc<str>> = LazyLock::new(|| Arc::from("to"));

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

pub fn read_from_string(data: String) -> Result<Survey> {
    let data = data.trim().to_string();
    let first_line =
        data.lines().next().ok_or(error::FileReadError::new("file seems to be empty"))?;

    let sss_version = FileType::parse(first_line)?;
    let builder = match sss_version {
        FileType::Classic => {
            let lines = BufReader::new(Cursor::new(data.into_bytes())).lines();
            classic::create_survey_builder(lines)?
        }

        FileType::Xml => {
            let file = BufReader::new(Cursor::new(data.into_bytes()));
            let decoder = DecodingReader::new(file);
            let mut reader = XmlReader::from_reader(decoder);
            reader.config_mut().trim_text(true);
            xml::create_survey_builder(reader)?
        }
    };

    builder.build()
}

pub fn read_sss_file(sss_path: &str) -> Result<Survey> {
    let input: File = File::open(sss_path).map_err(error::FileReadError::from)?;

    // identify sss version
    let mut lines = BufReader::new(input).lines();
    let first_line = lines
        .next()
        .ok_or(error::FileReadError::new("file seems to be empty"))?
        .with_context(|| format!("When trying to read from: '{}'", sss_path))?;

    let sss_version = FileType::parse(&first_line)?;
    let builder = match sss_version {
        FileType::Classic => {
            let input = File::open(sss_path).map_err(error::FileReadError::from)?;
            let lines = BufReader::new(input).lines();
            classic::create_survey_builder(lines)?
        }

        FileType::Xml => {
            let file = File::open(sss_path)?;
            let file = BufReader::new(file);
            let decoder = DecodingReader::new(file);
            let mut reader = XmlReader::from_reader(decoder);
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
            .first()
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

// -------------- helpers ---------------
/// A xml-like node, used for parsing sss files of different structure
#[derive(Debug, Clone)]
pub struct Node {
    pub tag: Arc<str>,
    pub attributes: HashMap<Arc<str>, Arc<str>>,
    pub value: Option<Arc<str>>,
    pub children: Vec<Box<Node>>,
}

impl Node {
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

    pub fn build(mut self) -> Result<Survey> {
        let mut output = Survey::empty();

        output.title = self
            .attributes
            .remove("title")
            .and_then(|val| val.value.map(|v| unquote(v, self.is_classic)))
            .transpose()?;

        output.name = self
            .attributes
            .remove("name")
            .and_then(|val| val.value.map(|v| unquote(v, self.is_classic)))
            .transpose()?
            .unwrap_or_default();

        if let Some(record) = self.attributes.remove("record") {
            if self.is_classic {
                output.record.id = record
                    .value
                    .and_then(|value| value.chars().next())
                    .ok_or(MissingValueError::new("survey", "record"))?;
            } else {
                output.record.id = record
                    .attributes
                    .get("ident")
                    .and_then(|value| value.chars().next())
                    .ok_or(MissingAttributeError::new("record", "ident"))?;
                output.record.skip = record.attributes.get("skip").map(ToString::to_string);
                output.record.href = record.attributes.get("href").map(ToString::to_string);
            }
        }

        output.meta.origin = self
            .attributes
            .remove("origin")
            .and_then(|val| val.value.map(|v| unquote(v, self.is_classic)))
            .transpose()?;

        output.meta.user = self
            .attributes
            .remove("user")
            .and_then(|val| val.value.map(|v| unquote(v, self.is_classic)))
            .transpose()?;

        output.meta.date = self
            .attributes
            .remove("date")
            .and_then(|val| val.value.map(|v| unquote(v, self.is_classic)))
            .transpose()?;

        output.meta.time = self
            .attributes
            .remove("time")
            .and_then(|val| val.value.map(|v| unquote(v, self.is_classic)))
            .transpose()?;

        output.meta.standard = self
            .attributes
            .remove("standard")
            .and_then(|val| val.value.map(|v| unquote(v, false)))
            .transpose()?;

        output.meta.version = self
            .attributes
            .remove("version")
            .ok_or(MissingAttributeError::new("sss", "version"))?
            .value
            .map(|val| Version::from_str(val.as_ref()))
            .transpose()?
            .ok_or(MissingValueError::new("sss", "version"))?;

        let is_classic = self.is_classic;
        output.record.content = self
            .questions
            .par_drain(..)
            .map(|item| item.build(is_classic))
            .collect::<Result<Vec<Question>>>()?;

        Ok(output)
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

    pub fn build_response(
        mut response: Response,
        mut data: Vec<Box<Node>>,
        is_classic: bool,
    ) -> Result<Response> {
        match &mut response {
            Response::Logical => (),
            Response::Character(item) => {
                let size = data
                    .into_par_iter()
                    .find_first(|node| node.tag.as_ref() == "size")
                    .ok_or(MissingAttributeError::new("response", "size"))?
                    .value
                    .ok_or(error::MissingValueError::new("variable", "size"))?;
                let range = Range::<u64>::from_text(size.as_ref())?;

                let mut item = RwLockWriteGuard::map(item.write(), |resp| resp);
                item.size = range.start;
            }

            Response::Quantity(item) => {
                let mut item = RwLockWriteGuard::map(item.write(), |resp| resp);
                let mut iterator = data.into_iter();

                // quantity should always start with range node.
                let range_node =
                    iterator.next().ok_or(MissingAttributeError::new("range", "variable"))?;

                let (from, to) = get_range::<f64>(&range_node, true)?;
                item.range.start = from;
                item.range.end = to.ok_or(MissingAttributeError::new("range", "to"))?;

                // then add special values for the rest
                item.values = iterator
                    .map(|node| parse_into_choice::<f64>(node, is_classic))
                    .collect::<Result<Vec<Code<f64>>>>()?;
            }
            Response::Multiple(item) => {
                let mut item = RwLockWriteGuard::map(item.write(), |resp| resp);
                let first_entry =
                    data.first().ok_or(MissingValueError::new("values", "value"))?;

                if first_entry.tag.as_ref() == "range" {
                    let (from, to) = get_range::<u64>(first_entry, true)?;
                    let end = to.ok_or(MissingAttributeError::new("range", "to"))?;
                    item.range = Some(Range { start: from, end });
                };

                item.values = data
                    .par_drain(..)
                    .filter(|val| val.tag.as_ref() != "range")
                    .map(|node| parse_into_choice::<u64>(node, is_classic))
                    .collect::<Result<Vec<Code<u64>>>>()?;
            }

            Response::Single(item) => {
                let mut item = RwLockWriteGuard::map(item.write(), |resp| resp);
                let first_entry =
                    data.first().ok_or(MissingValueError::new("values", "value"))?;

                if first_entry.tag.as_ref() == "range" {
                    let (from, to) = get_range::<u64>(first_entry, true)?;
                    let end = to.ok_or(MissingAttributeError::new("range", "to"))?;
                    item.range = Some(Range { start: from, end });
                };

                item.values = data
                    .par_drain(..)
                    .filter(|val| val.tag.as_ref() != "range")
                    .map(|node| parse_into_choice::<u64>(node, is_classic))
                    .collect::<Result<Vec<Code<u64>>>>()?;
            }

            Response::SingleLiteral(item) => {
                let mut item = RwLockWriteGuard::map(item.write(), |resp| resp);
                item.values = data
                    .par_drain(..)
                    .map(|node| parse_into_choice::<String>(node, is_classic))
                    .collect::<Result<Vec<Code<String>>>>()?;
            }

            Response::Date(item) => {
                if let Some(node) = data.into_iter().next() {
                    let start = node
                        .attributes
                        .get(FROM.as_ref())
                        .map(|val| NaiveDate::parse_from_str(val.as_ref(), "%Y%m%d"))
                        .transpose()?;

                    if let Some(start) = start {
                        let end = node
                            .attributes
                            .get(TO.as_ref())
                            .map(|val| NaiveDate::parse_from_str(val.as_ref(), "%Y%m%d"))
                            .transpose()?
                            .unwrap_or(NaiveDate::MAX);

                        let mut item = RwLockWriteGuard::map(item.write(), |resp| resp);
                        item.range = Some(Range { start, end })
                    }
                }
            }
            Response::Time(item) => {
                if let Some(node) = data.into_iter().next() {
                    let start = node
                        .attributes
                        .get(FROM.as_ref())
                        .map(|val| NaiveTime::parse_from_str(val.as_ref(), "%H%M%S"))
                        .transpose()?;

                    if let Some(start) = start {
                        let end = node
                            .attributes
                            .get(TO.as_ref())
                            .map(|val| NaiveTime::parse_from_str(val.as_ref(), "%H%M%S"))
                            .transpose()?
                            .unwrap_or(
                                NaiveTime::from_hms_nano_opt(23, 59, 59, 999_999_999).unwrap(),
                            );

                        let mut item = RwLockWriteGuard::map(item.write(), |resp| resp);
                        item.range = Some(Range { start, end })
                    }
                }
            }

            Response::Note => {
                return Err(InternalError::new(
                    "Customised response type 'Note' should not exist".to_string(),
                )
                .into());
            }
        }
        Ok(response)
    }

    fn parse_location_classic(node: &Node) -> Result<Position> {
        let value = node
            .value
            .as_ref()
            .ok_or(error::MissingValueError::new("variable", "position"))?;

        let range = Range::<u64>::from_text(value)?;
        Ok(Position {
            from: range.start,
            to: range.end,
        })
    }

    fn parse_location_xml(node: &Node) -> Result<Position> {
        let from = node
            .attributes
            .get("start")
            .ok_or(error::MissingValueError::new("position", "start"))?
            .parse::<u64>()?;

        let to = node
            .attributes
            .get("finish")
            .map(|s| s.parse::<u64>())
            .transpose()?
            .unwrap_or(from);

        Ok(Position { from, to })
    }

    pub fn build(mut self, is_classic: bool) -> Result<Question> {
        let mut output = Question::default();

        // mandatory: question index
        let attrib_name = if is_classic { "variable" } else { "ident" };
        output.index = self
            .attributes
            .remove(attrib_name)
            .and_then(|val| val.value.map(|idx| idx.trim().parse::<u64>()))
            .ok_or(error::MissingAttributeError::new("variable", attrib_name))?
            .map_err(|err| BadLine::new(format!("Unable to parse variable index: {}", err)))?;

        // mandatory: question name
        output.name = self
            .attributes
            .remove("name")
            .and_then(|val| val.value.map(|name| unquote(name, is_classic)))
            .transpose()?
            .ok_or(error::MissingAttributeError::new("variable", "name"))?;

        let question_name = output.name.to_string();

        // sss classic does not have text variant
        let label = self
            .attributes
            .remove("label")
            .ok_or(error::MissingAttributeError::new("variable", "label"))
            .with_context(|| format!("When parsing question {}", question_name))?;
        output.label = parse_text(label, is_classic)
            .with_context(|| format!("When parsing question {}", question_name))?;

        let position = self
            .attributes
            .remove("position")
            .ok_or(error::MissingAttributeError::new("variable", "position"))
            .with_context(|| format!("When parsing question {}", question_name))?;

        output.location = if is_classic {
            Self::parse_location_classic(&position)
                .with_context(|| format!("When parsing question {}", question_name))?
        } else {
            Self::parse_location_xml(&position)
                .with_context(|| format!("When parsing question {}", question_name))?
        };

        output.format = self
            .attributes
            .remove("format")
            .and_then(|node| node.value.map(|s| Format::from_str(s.as_ref())))
            .transpose()?
            .unwrap_or_default();

        // build question responses
        let response = self
            .attributes
            .remove("type")
            .ok_or(error::MissingAttributeError::new("variable", "type"))
            .with_context(|| format!("When parsing question {}", question_name))?
            .value
            .ok_or(error::MissingValueError::new("variable", "type"))
            .with_context(|| format!("When parsing question {}", question_name))?;

        let response = Response::from_str(&response)?;
        let response = match &output.format {
            Format::Literal => match &response {
                Response::Single(_) => Response::new_literal_single(),
                _ => response,
            },
            _ => response,
        };

        let response = Self::build_response(response, self.responses.children, is_classic)
            .with_context(|| format!("When parsing question {}", question_name))?;

        if let Some(mut node) = self.attributes.remove("spread")
            && let Response::Multiple(item) = &response
        {
            let spread = if is_classic {
                let text = node
                    .value
                    .ok_or(MissingValueError::new("variable", "spread"))
                    .with_context(|| format!("When parsing question {}", question_name))?
                    .to_uppercase();

                let text = text
                    .splitn(2, "OF")
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<&str>>();

                let subfield = text
                    .first()
                    .ok_or(MissingValueError::new("variable", "spread"))
                    .with_context(|| format!("When parsing question {}", question_name))?
                    .trim()
                    .parse::<u64>()
                    .with_context(|| format!("When parsing question {}", question_name))?;

                let width = text.get(1).map(|s| s.parse::<u64>()).transpose()?;

                Spread { subfield, width }
            } else {
                let subfield = node
                    .attributes
                    .remove("subfields")
                    .ok_or(error::MissingAttributeError::new("variable", "subfields"))
                    .with_context(|| format!("When parsing question {}", question_name))?
                    .as_ref()
                    .parse::<u64>()
                    .with_context(|| format!("When parsing question {}", question_name))?;

                let width = node
                    .attributes
                    .remove("width")
                    .map(|s| s.as_ref().trim().parse::<u64>())
                    .transpose()
                    .with_context(|| format!("When parsing question {}", question_name))?;

                Spread { subfield, width }
            };

            let mut guard = item.write();
            guard.spread = Some(spread);
        };

        output.response = response;
        Ok(output)
    }
}

/// unquote texts but raises error if text was not quoted.
#[inline]
fn unquote(text: Arc<str>, raise_error: bool) -> Result<String> {
    let to_unquote = text.trim();
    let result = to_unquote.strip_circumfix('"', '"').map(|s| s.to_string());
    if raise_error {
        result.ok_or(
            error::BadLine::new(format!("Not double-quoted: '{}'", text.as_ref())).into(),
        )
    } else {
        Ok(result.unwrap_or(to_unquote.to_string()))
    }
}

fn parse_text(mut node: Node, is_classic: bool) -> Result<Text> {
    let value = node
        .value
        .take()
        .map(|value| unquote(value, is_classic))
        .transpose()?
        .unwrap_or_default();
    let mut text = Text::from(value);

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
            .map(|language| Language::from_string(language))
            .transpose()?;
        let mode = variant
            .attributes
            .get("mode")
            .map(|mode| Mode::from_str(mode))
            .transpose()?
            .unwrap_or_default();
        text.set_variant(TextVariant {
            value,
            language,
            mode,
        });
    }
    Ok(text)
}

#[inline]
fn parse_into_choice<T>(node: Box<Node>, is_classic: bool) -> Result<Code<T>>
where
    T: Clone + FromStr + std::fmt::Display,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    if node.tag.as_ref() != "value" {
        return Err(InternalError::new(format!(
            "Exepct 'range' node, received: '{}'",
            node.tag
        ))
        .into());
    };

    let code = node
        .attributes
        .get("code")
        .ok_or(MissingAttributeError::new(node.tag.as_ref(), "code"))?
        .parse::<T>()?;

    let is_special = node.attributes.contains_key(FLAG.as_ref());
    let text = parse_text(*node, is_classic)?;
    let choice = Code {
        code,
        label: text,
        filter: None,
        is_special,
        score: None,
    };
    Ok(choice)
}

/// from_to == true:
/// 'from' 'to'
/// from_to == false:
/// 'start'
#[inline]
fn get_range<T>(node: &Box<Node>, from_to: bool) -> Result<(T, Option<T>)>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let (attr_from, attr_to) = if from_to {
        ("from", "to")
    } else {
        ("start", "finish")
    };

    let from = node
        .attributes
        .get(attr_from)
        .ok_or(MissingAttributeError::new(node.tag.as_ref(), attr_from))?
        .parse::<T>()?;
    let to = node.attributes.get(attr_to).map(|s| s.as_ref().parse::<T>()).transpose()?;
    Ok((from, to))
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
