use std::{collections::HashMap, io::BufRead, sync::Arc};

use anyhow::Result;
use quick_xml::{
    XmlVersion,
    events::{BytesEnd, BytesStart, BytesText, Event, attributes::Attributes},
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

/// Reads one complete `<variable>` element into a question builder.
pub fn collect_question_stream<R: BufRead>(mut reader: Reader<R>) -> Result<QuestionBuilder> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(start) if start.local_name().into_inner() == "variable" => {
                let attributes = extract_attributes(start.attributes())?
                    .map(|(k, v)| (k.clone(), Node::with_value(k, v)))
                    .collect();
                let mut builder = QuestionBuilder {
                    attributes,
                    responses: Node::new("response"),
                };
                consume_variable_data(&mut reader, &mut builder, start.to_end())?;
                return Ok(builder);
            }
            Event::Decl(_) | Event::Comment(_) => (),
            _ => {
                return Err(
                    InternalError::new("Expected a <variable> element".to_owned()).into(),
                );
            }
        }
        buf.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_question_stream() {
        let xml = r#"<variable ident="1" type="single"><name>Q1</name><label>Answer</label><values><value code="1">Yes</value></values></variable>"#;
        let builder = collect_question_stream(Reader::from_str(xml)).unwrap();
        assert_eq!(builder.attributes["ident"].value.as_deref(), Some("1"));
        assert_eq!(builder.attributes["name"].value.as_deref(), Some("Q1"));
        assert_eq!(builder.responses.children.len(), 1);
        assert_eq!(builder.responses.children[0].value.as_deref(), Some("Yes"));
    }

    #[test]
    fn test_collect_question_text_references() {
        let xml = r#"<variable ident="1" type="single">
            <label>Tea &amp; coffee<text mode="analysis">Drinks</text>
            </label>
            <values>
                <value code="1">&quot;Friar Tuck&quot; Restaurant</value>
                <value code="2">&quot;Maid Marion&quot; Cafe</value>
                <value code="3">Tea&#x20;&amp;&#32;coffee</value>
            </values>
        </variable>"#;
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let builder = collect_question_stream(reader).unwrap();
        assert_eq!(
            builder.attributes["label"].value.as_deref(),
            Some("Tea & coffee")
        );
        assert_eq!(
            builder.attributes["label"].children[0].value.as_deref(),
            Some("Drinks")
        );
        for (node, expected) in builder.responses.children.iter().zip([
            "\"Friar Tuck\" Restaurant",
            "\"Maid Marion\" Cafe",
            "Tea & coffee",
        ]) {
            assert_eq!(node.value.as_deref(), Some(expected));
        }
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
    let mut filter = Node::new("filter");
    let mut label = Node::new("label");
    let mut name = Node::new("name");
    let mut responses = Vec::new();
    let trim_start = reader.config().trim_text_start;
    let trim_end = reader.config().trim_text_end;
    reader.config_mut().trim_text(false);

    loop {
        let event = reader.read_event_into(&mut buf)?;
        let decoded; //
        let event = match event {
            Event::GeneralRef(reference) => {
                decoded = quick_xml::escape::unescape(&format!("&{};", reference.as_ref()))?
                    .into_owned();
                Event::Text(BytesText::from_escaped(&decoded))
            }
            event => event,
        };
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
                    "filter" => {
                        let filter_content = extract_text(reader, &start)?;
                        filter.value = Some(Arc::from(filter_content));
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
                    let mut value = label.value.as_deref().unwrap_or_default().to_owned();
                    value.push_str(ele.as_ref());
                    label.value = Some(Arc::from(value));
                }

                Section::CodeDefault => {
                    let text = ele.into_inner().to_string();
                    responses
                        .last_mut()
                        .map(|val| {
                            let mut value =
                                val.value.as_deref().unwrap_or_default().to_owned();
                            value.push_str(&text);
                            val.value = Some(Arc::from(value));
                        })
                        .ok_or(InternalError::new(format!(
                            "Parent note for <text> seems to be missing?? Current: {:#?}",
                            builder
                        )))?;
                }
                Section::LabelVariant => {
                    if ele.as_ref().trim().is_empty() {
                        continue;
                    }
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
                    if ele.as_ref().trim().is_empty() {
                        continue;
                    }
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
            Event::End(close_tag) => {
                if matches!(close_tag.local_name().into_inner(), "label" | "value") {
                    current_position = Section::Start;
                }
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

    reader.config_mut().trim_text_start = trim_start;
    reader.config_mut().trim_text_end = trim_end;
    for node in std::iter::once(&mut label).chain(responses.iter_mut()) {
        if let Some(value) = &node.value {
            node.value = Some(Arc::from(value.trim()));
        }
    }

    // and finally put things into the question builder
    builder.attributes.insert("filter".into(), filter);
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
            let decoded = attr.normalized_value(XmlVersion::Implicit1_0)?;
            // Literal codes may contain significant leading or trailing spaces.
            // UnSigned code parsing handles its own whitespace.
            let value: Arc<str> = if key.as_ref() == "code" {
                Arc::from(decoded.as_ref())
            } else {
                Arc::from(decoded.trim())
            };
            Ok((key, value))
        })
        .collect::<Result<_>>()?;

    Ok(attributes.into_iter())
}

/// Read text with XML references decoded once, preserving embedded markup.
#[inline]
fn extract_text<R>(reader: &mut Reader<R>, start: &BytesStart) -> Result<String>
where R: BufRead {
    let mut buf = Vec::new();
    let mut output = quick_xml::Writer::new(Vec::new());
    let mut depth = 0;
    let trim_start = reader.config().trim_text_start;
    let trim_end = reader.config().trim_text_end;
    reader.config_mut().trim_text(false);
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::End(end) if depth == 0 && end.name() == start.name() => break,
            Event::Start(tag) => {
                depth += 1;
                output.write_event(Event::Start(tag))?;
            }
            Event::End(tag) => {
                depth -= 1;
                output.write_event(Event::End(tag))?;
            }
            Event::GeneralRef(reference) => {
                let reference = format!("&{};", reference.as_ref());
                let decoded = quick_xml::escape::unescape(&reference)?;
                output.get_mut().extend_from_slice(decoded.as_bytes());
            }
            Event::CData(text) => {
                output.get_mut().extend_from_slice(text.as_ref().as_bytes());
            }
            Event::Eof => anyhow::bail!("Unexpected end of XML while reading text"),
            event => output.write_event(event)?,
        }
        buf.clear();
    }
    reader.config_mut().trim_text_start = trim_start;
    reader.config_mut().trim_text_end = trim_end;
    Ok(String::from_utf8(output.into_inner())?)
}
