use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};
use chrono::{NaiveDate, NaiveTime};

use anyhow::{Context, Result};
use quick_xml::{
    encoding::DecodingReader,
    events::{BytesEnd, BytesStart, Event, attributes::Attributes},
    reader::Reader,
};
use rayon::prelude::*;

use crate::{
    error::{
        BadLine,
        InternalError,
        MissingAttributeError,
        OperationError,
        UnknownAttribute,
        UnsupportedFeature,
    },
    survey::{
        Format,
        Language,
        Location,
        Mode,
        Question,
        Response,
        Survey,
        Text,
        TextVariant,
        Version,
    },
};

pub fn read_sss_file_xml_file(sss_path: &str) -> Result<Survey> {
    let file = File::open(sss_path)?;
    let file = BufReader::new(file);
    let decoder = DecodingReader::new(file);
    let mut reader = Reader::from_reader(decoder);
    reader.config_mut().trim_text(true);

    parse_sss_xml_lines(reader)
}

#[allow(unused_variables, unused_mut)]
fn parse_sss_xml_lines<R>(mut reader: Reader<R>) -> Result<Survey>
where
    R: BufRead,
{
    let mut survey = Survey::empty_xml();
    let mut buf = Vec::new();
    let mut build_tasks = Vec::new();
    let mut index: usize = 0;
    loop {
        let event = reader.read_event_into(&mut buf)?;
        match event {
            Event::Decl(_) => (),
            Event::DocType(_) => (),
            Event::Comment(_) => (),
            Event::Start(ele) => {
                let name = ele.name();
                let tag = name.local_name().into_inner();
                match tag {
                    "xml" => (),
                    "sss" => set_sss_attributes(&mut survey, ele.attributes())?,
                    // keep adding new cases to parse here
                    "date" => {
                        let text = extract_text(&mut reader, ele)?;
                        survey.meta.date = Some(text);
                    }
                    "time" => {
                        let text = extract_text(&mut reader, ele)?;
                        survey.meta.time = Some(text);
                    }
                    "origin" => {
                        let text = extract_text(&mut reader, ele)?;
                        survey.meta.origin = Some(text);
                    }
                    "user" => {
                        let text = extract_text(&mut reader, ele)?;
                        survey.meta.user = Some(text);
                    }
                    "survey" => (),
                    "name" => {
                        let text = extract_text(&mut reader, ele)?;
                        survey.meta.name = Some(text);
                    }
                    "title" => {
                        let text = extract_text(&mut reader, ele)?;
                        survey.title = Some(text);
                    }
                    "record" => set_record_attributes(&mut survey, ele.attributes())?,
                    "variable" => {
                        let mut worker =
                            QuestionBuilder::init(index, ele.attributes().to_owned())?;
                        let end = ele.to_end();
                        consume_variable_data(&mut reader, &mut worker, end)?;
                        build_tasks.push(worker);
                        index += 1;
                    }
                    _ => (),
                };
            }

            Event::End(ele) => {
                // println!("{:?}", ele);
            }

            Event::Text(ele) => {
                // let text = ele.to_string();
                println!("{:?}", ele);
            }
            // Event::CData(ele) => { todo!() }
            // Event::Comment(ele) => { todo!() }
            // Event::Decl(ele) => { todo!() }
            // Event::PI(ele) => { todo!() }
            // Event::DocType(ele) => { println!("{:?}", ele) }
            Event::Empty(ele) => {
                println!("{:?}", ele);
            }
            // Event::GeneralRef(ele) => { println!("{:?}", ele) }
            Event::Eof => break,
            _ => {
                return Err(
                    OperationError::new(format!("Undefined event: '{:?}", event)).into(),
                );
            }
        }
    }

    let questions = build_tasks
        .par_drain(..)
        .map(|builder| builder.build())
        .collect::<Result<Vec<(usize, Question)>>>()?;

    // let pool = rayon::ThreadPoolBuilder::new().num_threads(8).build().unwrap();
    Ok(survey)
}

#[inline]
fn set_sss_attributes(survey: &mut Survey, attributes: Attributes<'_>) -> Result<()> {
    for attr_read in attributes {
        let attr = attr_read?;
        let attr_name = attr.key.into_inner();
        let value = attr.value.trim().to_string();
        match attr_name {
            "version" => {
                let version = Version::from_str(&value)?;
                match &version {
                    Version::V1_0 => {
                        return Err(UnsupportedFeature {
                            current_version: format!("{:?}", version),
                            feature: "XML output".into(),
                        }
                        .into());
                    }
                    _ => survey.meta.version = version,
                }
            }
            "modes" => {
                let mode_options = parse_mode_options(value)?;
                survey.meta.mode_options = mode_options;
            }
            "options" => {
                // deprecated after version 1.0
                if survey.meta.version > Version::V1_2 {
                    return Err(UnsupportedFeature {
                        current_version: format!("{:?}", survey.meta.version),
                        feature: "<options>".into(),
                    }
                    .into());
                }
            }
            "languages" => {
                survey.meta.language_options = value
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
            }
            _ => {
                return Err(UnknownAttribute::new(attr_name, attr.value))
                    .with_context(|| "Failed to parse survey meta.".to_string());
            }
        }
    }
    Ok(())
}

fn extract_attributes(attributes: Attributes<'_>) -> Result<HashMap<String, String>> {
    attributes
        .into_iter()
        .map(|item| {
            let item = item?;
            Ok((
                item.key.to_owned().0.to_string(),
                item.value.to_string(),
            ))
        })
        .collect::<Result<HashMap<String, String>>>()
}

#[derive(Debug)]
struct Element {
    pub tag: String,
    pub attribute: HashMap<String, String>,
    pub text: Option<String>,
    pub children: Vec<Box<Element>>,
}

impl Element {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            attribute: HashMap::new(),
            text: None,
            children: Vec::new(),
        }
    }

    pub fn with_attribute(tag: impl Into<String>, attributes: Attributes<'_>) -> Result<Self> {
        let attributes = attributes
            .into_iter()
            .map(|item| {
                let item = item?;
                Ok((
                    item.key.to_owned().0.to_string(),
                    item.value.to_string(),
                ))
            })
            .collect::<Result<HashMap<String, String>>>()?;

        Ok(Self {
            tag: tag.into(),
            attribute: attributes,
            text: None,
            children: Vec::new(),
        })
    }
}

// ------- for building variables -------
#[derive(Debug)]
struct QuestionBuilder {
    pub index: usize, // used for ordering/ reordering
    pub attributes: HashMap<String, String>,
    pub name: String,
    pub label: Element,
    pub filter: Option<String>,
    pub position: Option<BytesStart<'static>>,
    pub size: Option<BytesStart<'static>>, // for 'character' type of quesiton only
    pub range: Option<BytesStart<'static>>, // called 'range' from sss xml 2.0 onwards
    pub values: Vec<Element>,
}

impl QuestionBuilder {
    pub fn init(index: usize, attributes: Attributes<'_>) -> Result<Self> {
        let attributes = extract_attributes(attributes)?;
        Ok(Self {
            index: index,
            attributes: attributes,
            label: Element::new("label"),
            name: String::new(),
            filter: None,
            size: None,
            position: None,
            range: None,
            values: Vec::new(),
        })
    }

    fn build_text_variant(mut element: Element) -> Result<Text> {
        let mut text = Text::new(element.text.unwrap_or_default());
        text.variant = element
            .children
            .par_drain(..)
            .map(|mut element| {
                let language = element
                    .attribute
                    .remove("language")
                    .map_or(Language::None, Language::named);
                let mode = element
                    .attribute
                    .remove("mode")
                    .map_or(Ok(Mode::default()), |s| Mode::from_str(&s))?;
                Ok(TextVariant {
                    value: element.text.unwrap_or_default(),
                    language: language,
                    mode: mode,
                })
            })
            .collect::<Result<Vec<TextVariant>>>()?;
        Ok(text)
    }

    fn build_response(
        mut response: Response, 
        range: Option<BytesStart<'static>>,
        values: Vec<Element>,
    ) -> Result<Response> {
        match &mut response {
            Response::Logical => (),
            Response::Date { from, to } => {
                if let Some(date_range) = range {
                    let attrib = extract_attributes(date_range.attributes())?;
                    *from = {
                        let start = attrib.get("from")
                        .ok_or(MissingAttributeError::new("range", "from"))?;
                        Some(NaiveDate::parse_from_str(start, "%Y%m%d")?)
                    };
                    
                    *to = {
                        attrib.get("from")
                        .map(|s| NaiveDate::parse_from_str(s, "%Y%m%d"))
                        .transpose()?
                    }
                }
            }
            Response::Time { from, to } => {
                if let Some(date_range) = range {
                    let attrib = extract_attributes(date_range.attributes())?;
                    *from = {
                        let start = attrib.get("from")
                        .ok_or(MissingAttributeError::new("range", "from"))?;
                        Some(NaiveTime::parse_from_str(start, "%H%M%S")?)
                    };
                    *to = {
                        attrib.get("from")
                        .map(|s| NaiveTime::parse_from_str(s, "%H%M%S"))
                        .transpose()?
                    }
                }
            }
            _ => ()
        }
        // println!("Spread: {:?}\nvalues:{:?}", &spread, &values);
        Ok(response)
    }

    /// (question_index, question_instance)
    pub fn build(mut self) -> Result<(usize, Question)> {
        let text = Self::build_text_variant(self.label)?;
        let mut response = self
            .attributes
            .remove("type")
            .ok_or(MissingAttributeError::new("variable", "type"))
            .with_context(|| format!("\nTrying to read question '{}'", self.name))?
            .parse::<Response>()?;

        println!("Question: {}", self.name);
        let response = Self::build_response(response, self.range, self.values)?;

        // let date = NaiveDate::parse_from_str("20260918", "%Y%m%d").unwrap();
        // let time = NaiveTime::parse_from_str("124830", "%H%M%S").unwrap();

        let question = Question {
            index: self.index as u64,
            name: self.name.clone(),
            text: text,
            filter: None,
            location: Location::default(),
            response: response,
        };

        Ok((self.index, question))
    }
}

#[derive(Debug)]
enum Section {
    Start,
    QuestionMeta,
    LabelDefault,
    LabelVariant,
    CodeDefault,
    CodeVariant,
    Filter,
}

fn consume_variable_data<R>(
    reader: &mut Reader<R>,
    worker: &mut QuestionBuilder,
    end: BytesEnd,
) -> Result<()>
where
    R: BufRead,
{
    let mut buf = Vec::new();
    let mut current_position = Section::Start;
    loop {
        let event = reader.read_event_into(&mut buf)?;
        match event {
            // match reader.read_event_into(&mut buf)? {
            Event::Start(start) => {
                // let tag = start.local_name().into_inner();
                match start.local_name().into_inner() {
                    "name" => {
                        current_position = Section::QuestionMeta;
                        let end = start.to_end().into_owned();
                        let mut buff = Vec::new();
                        let name = reader.read_text_into(end.name(), &mut buff)?;
                        worker.name = name.to_string();
                    }
                    "label" => {
                        current_position = Section::LabelDefault;
                    }
                    "value" => {
                        current_position = Section::CodeDefault;
                        let element = Element::with_attribute("value", start.attributes())?;
                        worker.values.push(element);
                    }
                    "filter" => {
                        current_position = Section::Filter;
                    }
                    "text" => match current_position {
                        Section::LabelDefault | Section::LabelVariant => {
                            current_position = Section::LabelVariant;
                            let element = Element::with_attribute("text", start.attributes())?;
                            worker.label.children.push(Box::new(element));
                        }

                        Section::CodeDefault | Section::CodeVariant => {
                            current_position = Section::CodeVariant;
                            let element = Element::with_attribute("text", start.attributes())?;
                            worker.values.last_mut()
                                .map(|val| val.children.push(Box::new(element)) )
                                .ok_or(InternalError::new(format!("Parent note for <text> seems to be missing?? Current: {:#?}", worker)))?;
                        }
                        _ => {
                            return Err(InternalError::new(format!(
                                "Unexpected <text> tag in element: {:?}",
                                start.to_string()
                            ))
                            .into());
                        }
                    },
                    // _ => println!("{:?}", start),
                    _ => (),
                };
            }

            Event::Text(ele) => match current_position {
                Section::Filter => {
                    worker.filter = Some(ele.to_string());
                }

                Section::LabelDefault => {
                    worker.label.text = Some(ele.to_string());
                }

                Section::CodeDefault => {
                    let text = ele.into_inner().to_string();
                    worker
                        .values
                        .last_mut()
                        .map(|val| val.text = Some(text))
                        .ok_or(InternalError::new(format!(
                            "Parent note for <text> seems to be missing?? Current: {:#?}",
                            worker
                        )))?;
                }

                Section::LabelVariant => {
                    let text = ele.into_inner().to_string();
                    worker
                        .label
                        .children
                        .last_mut()
                        .map(|val| val.text = Some(text))
                        .ok_or(InternalError::new(format!(
                            "Parent note for <text> seems to be missing?? Current: {:#?}",
                            worker
                        )))?;
                }
                Section::CodeVariant => {
                    let text = ele.into_inner().to_string();
                    worker
                        .values
                        .last_mut()
                        .ok_or(InternalError::new(format!(
                            "No <value> was added but trying to amend text {:?}",
                            text
                        )))?
                        .children
                        .last_mut()
                        .map(|child| child.text = Some(text))
                        .ok_or(InternalError::new(format!(
                            "Parent note for <text> seems to be missing?? Current: {:#?}",
                            worker
                        )))?;
                }
                _ => (),
            },
            // self-closing elements
            Event::Empty(ele) => match ele.local_name().into_inner() {
                "position" => worker.position = Some(ele.into_owned()),
                "range" => worker.range = Some(ele.into_owned()),
                "spread" => worker.range = Some(ele.into_owned()),
                "size" => worker.size = Some(ele.into_owned()),
                _ => {
                    return Err(UnknownAttribute::new(
                        ele.local_name().into_inner(),
                        ele.to_string(),
                    )
                    .into());
                }
            },
            Event::End(close_tag) if close_tag == end => {
                break;
            }
            _ => (), // _ => println!("{:#?}", event)
        }
    }
    Ok(())
}

#[inline]
fn set_record_attributes(survey: &mut Survey, attributes: Attributes<'_>) -> Result<()> {
    for attr_read in attributes {
        let attr = attr_read?;
        let attr_name = attr.key.into_inner();
        let attr_value = attr.value.trim().to_string();
        match attr_name {
            "ident" => {
                if attr_value.len() != 1 {
                    return Err(
                        BadLine::new(format!(
                            "The 'ident' attribute of a <record> should be one single character. Parsed: {}", 
                            attr_value
                        )).into());
                } else {
                    let mut characters = attr_value.chars();
                    survey.record.id = characters
                        .next()
                        .ok_or(InternalError::new("Shoudn't happen"))?;
                }
            }
            "format" => {
                let option = Format::from_str(&attr_value)?;
                survey.record.format = Some(option);
            }
            "skip" => {
                survey.record.skip = Some(attr_value);
            }
            "href" => {
                survey.record.skip = Some(attr_value);
            }
            _ => {
                return Err(UnknownAttribute::new(attr_name, attr.value))
                    .with_context(|| "Failed to parse <record>.".to_string());
            }
        }
    }
    Ok(())
}

#[inline]
fn parse_mode_options(value: String) -> Result<Vec<Mode>> {
    value
        .split_whitespace()
        .map(|s| Mode::from_str(s.trim()))
        .collect::<Result<Vec<Mode>, _>>()
        .map_err(Into::into)
}

/// get content of the entire tag & export to string
#[inline]
fn extract_text<R>(reader: &mut Reader<R>, start: BytesStart) -> Result<String>
where
    R: BufRead,
{
    let mut buf = Vec::new();
    let end = start.to_end().into_owned();
    let result = reader.read_text_into(end.name(), &mut buf)?;
    Ok(result.to_string())
}
