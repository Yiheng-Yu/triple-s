use std::{collections::HashMap, io::BufRead, sync::Arc};

use anyhow::Result;
use quick_xml::{
    events::{BytesEnd, BytesStart, Event, attributes::Attributes},
    reader::Reader,
};

use super::{Node, QuestionBuilder, SurveyBuilder};
use crate::error::{InternalError, UnknownAttribute};

pub fn create_survey_builder<R>(mut reader: Reader<R>) -> Result<SurveyBuilder>
where R: BufRead {
    let mut survey_builder = SurveyBuilder::empty(false);
    let mut buf = Vec::new();
    let _index: usize = 0;

    // collect survey meta till the survey record tag
    let mut should_stop = false;
    loop {
        let event = reader.read_event_into(&mut buf)?;
        match event {
            Event::Decl(_) => (),
            Event::DocType(_) => (),
            Event::Comment(_) => (),
            Event::End(_) => (),
            Event::Text(_ele) => (),
            Event::Eof => should_stop = true,

            Event::Start(ele) => {
                let name = ele.name();
                let tag = name.local_name().into_inner();
                match tag {
                    "xml" => (),
                    "survey" => (),
                    "sss" => {
                        for (k, v) in extract_attributes(ele.attributes())? {
                            let node = Node::with_value(k.clone(), v.clone());
                            super::try_insert(&mut survey_builder.attributes, k, node)?
                        }
                    }
                    "date" | "time" | "origin" | "user" | "name" | "title" => {
                        let text = extract_text(&mut reader, &ele)?;
                        let node = Node::with_value(tag, text);
                        super::try_insert(
                            &mut survey_builder.attributes,
                            Arc::from(tag),
                            node,
                        )?
                    }
                    "record" => {
                        let node = with_attribute("record", ele.attributes())?;
                        survey_builder.attributes.insert("record".into(), node);
                    }
                    "variable" => {
                        let start = ele.clone();
                        let attributes = extract_attributes(start.attributes())?
                            .map(|(k, v)| (k.clone(), Node::with_value(k, v)))
                            .collect::<HashMap<_, _>>();

                        let mut question_builder = QuestionBuilder {
                            attributes,
                            responses: Node::new("response"),
                        };

                        let end = ele.to_end();
                        consume_variable_data(&mut reader, &mut question_builder, end)?;
                        survey_builder.questions.push(question_builder);
                    }
                    _ => {
                        return Err(InternalError::new(format!(
                            "Undefined xml tag: '{}'",
                            tag
                        ))
                        .into());
                    }
                }
            }

            _ => {
                return Err(
                    InternalError::new(format!("Undefined event: '{:?}", event)).into()
                );
            }
        }

        if should_stop {
            break;
        }
    }

    Ok(survey_builder)
}

#[derive(Debug)]
enum Section {
    Start,
    QuestionMeta,
    LabelDefault,
    LabelVariant,
    CodeDefault,
    CodeVariant,
    // Filter,
}

fn consume_variable_data<R>(
    reader: &mut Reader<R>,
    builder: &mut QuestionBuilder,
    end: BytesEnd,
) -> Result<()>
where
    R: BufRead,
{
    let mut buf = Vec::new();
    let mut current_position = Section::Start;

    // bunch of question attributes to be pushed later,,
    // let mut filter = Node::new("filter");
    let mut label = Node::new("label");
    let mut name = Node::new("name");
    let mut responses = Vec::new();

    loop {
        let event = reader.read_event_into(&mut buf)?;
        match event {
            Event::Start(start) => {
                match start.local_name().into_inner() {
                    "name" => {
                        current_position = Section::QuestionMeta;
                        let value = extract_text(reader, &start)?;
                        name.value = Some(Arc::from(value));
                    }
                    "label" => {
                        current_position = Section::LabelDefault;
                    }
                    // codeframe
                    "value" => {
                        current_position = Section::CodeDefault;
                        // here we don't collect content of the 'value' tag yet
                        // because calling extract_text would turn all text
                        // variants into raw string.
                        // cotents of '<value>' is handeled in 'Event::Text'
                        let mut value = Node::new("value");
                        value.attributes = extract_attributes(start.attributes())?.collect();
                        responses.push(value);
                    }

                    // <text> are ALL text variants
                    "text" => match current_position {
                        Section::LabelDefault | Section::LabelVariant => {
                            current_position = Section::LabelVariant;
                            let mut variant = with_attribute("text", start.attributes())?;
                            let text = extract_text(reader, &start)?;
                            variant.value = Some(Arc::from(text));
                            label.children.push(variant);
                        }
                        Section::CodeDefault | Section::CodeVariant => {
                            current_position = Section::CodeVariant;
                            let mut variant = with_attribute("text", start.attributes())?;
                            let text = extract_text(reader, &start)?;
                            variant.value = Some(Arc::from(text));
                            responses.last_mut()
                                .map(|val| val.children.push(variant))
                                .ok_or(InternalError::new(format!("Parent note for <text> seems to be missing?? Current: {:#?}", builder)))?;
                        }
                        _ => {
                            return Err(InternalError::new(format!(
                                "Unexpected <text> tag in element: {:?}",
                                start.to_string()
                            ))
                            .into());
                        }
                    },
                    "size" => {
                        let text = extract_text(reader, &start)?;
                        let node = Node::with_value("size", text);
                        responses.push(node);
                    }
                    _ => (),
                }
            }
            Event::Text(ele) => match current_position {
                // Section::Filter => {
                //     let value = ele.to_string();
                //     filter.value = Some(Arc::from(value));
                // }
                Section::LabelDefault => {
                    let value = ele.to_string();
                    label.value = Some(Arc::from(value));
                }

                Section::CodeDefault => {
                    let text = ele.into_inner().to_string();
                    responses.last_mut().map(|val| val.value = Some(Arc::from(text))).ok_or(
                        InternalError::new(format!(
                            "Parent note for <text> seems to be missing?? Current: {:#?}",
                            builder
                        )),
                    )?;
                }
                Section::LabelVariant => {
                    let text = ele.into_inner().to_string();
                    label
                        .children
                        .last_mut()
                        .map(|val| val.value = Some(Arc::from(text)))
                        .ok_or(InternalError::new(format!(
                            "Parent note for <text> seems to be missing?? Current: {:#?}",
                            builder
                        )))?;
                }
                Section::CodeVariant => {
                    let text = ele.into_inner().to_string();
                    responses
                        .last_mut()
                        .ok_or(InternalError::new(format!(
                            "No <value> was added but trying to amend text {:?}",
                            text
                        )))?
                        .children
                        .last_mut()
                        .map(|child| child.value = Some(Arc::from(text)))
                        .ok_or(InternalError::new(format!(
                            "Parent note for <text> seems to be missing?? Current: {:#?}",
                            builder
                        )))?;
                }
                _ => (),
            },
            Event::End(close_tag) if close_tag == end => {
                break;
            }

            // self-closing elements
            Event::Empty(ele) => {
                let tag = ele.local_name().into_inner();
                match tag {
                    "position" => {
                        let node = with_attribute("position", ele.attributes())?;
                        builder.attributes.insert("position".into(), node);
                    }
                    "range" => {
                        let node = with_attribute("range", ele.attributes())?;
                        responses.push(node);
                    }
                    "spread" => {
                        let node = with_attribute("spread", ele.attributes())?;
                        builder.attributes.insert("spread".into(), node);
                    }
                    _ => {
                        return Err(UnknownAttribute::new(
                            ele.local_name().into_inner(),
                            ele.to_string(),
                        )
                        .into());
                    }
                }
            }
            _ => (),
        }
    }

    // and finally put things into the question builder
    // builder.attributes.insert("filter".into(), filter);
    builder.attributes.insert("label".into(), label);
    builder.attributes.insert("name".into(), name);
    builder.responses.children = responses.into_iter().collect();

    Ok(())
}

fn with_attribute(tag: impl Into<Arc<str>>, attributes: Attributes<'_>) -> Result<Node> {
    let attributes = extract_attributes(attributes)?.collect();
    Ok(Node {
        tag: tag.into(),
        attributes,
        value: None,
        children: Vec::new(),
    })
}

#[inline]
fn extract_attributes(
    attributes: Attributes<'_>,
) -> Result<impl Iterator<Item = (Arc<str>, Arc<str>)>> {
    let attributes: Vec<_> = attributes
        .map(|attr| {
            let attr = attr?;
            let key: Arc<str> = Arc::from(attr.key.into_inner());
            let value: Arc<str> = Arc::from(attr.value.trim());
            Ok((key, value))
        })
        .collect::<Result<_>>()?;

    Ok(attributes.into_iter())
}

/// get content of the entire tag & export to string
#[inline]
fn extract_text<R>(reader: &mut Reader<R>, start: &BytesStart) -> Result<String>
where R: BufRead {
    let mut buf = Vec::new();
    let end = start.to_end().into_owned();
    let result = reader.read_text_into(end.name(), &mut buf)?;
    Ok(result.to_string())
}
