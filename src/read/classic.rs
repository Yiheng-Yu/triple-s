use std::sync::{Arc, LazyLock};

use anyhow::{Context, Result};
use regex::Regex;

pub const SPECIAL: &str = "SPECIAL";
pub const WITH: &str = "WITH";

use crate::{
    error::{BadLine, InternalError},
    read::{Node, QuestionBuilder, SurveyBuilder},
};

static PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    let number = r"((-)?[.](\d+))|((-)?(\d+)(?:[.]?(\d*))?)";
    let text = r"(?P<text>[A-Z]+)";

    let range_start = format!(r"(?P<range_start>{number})");
    let range_end = format!(r"\s+TO\s+(?P<range_end>{number})");
    let range = format!(r"{range_start}(?:{range_end})?");

    let prefix = format!("(?P<prefix>{text}|{range})");
    let value = r#"(?P<value>.*)"#;

    let pattern = format!(r#"^({prefix})(?: {value})?$"#);
    Regex::new(&pattern).unwrap()
});

impl TryFrom<&str> for Node {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        let capture = PATTERN.captures(trimmed).ok_or(BadLine::new(format!(
            r#"Failed to parse line: '{}'"#,
            value
        )))?;

        if capture.name("prefix").is_none() {
            let error = BadLine::new(format!(
                "Unable to identify line type from input: {}",
                value
            ));
            return Err(error.into());
        }

        let tag: Arc<str> = capture
            .name("text")
            .map(|s| s.as_str().to_ascii_lowercase())
            .map_or("range".into(), Arc::from);

        let mut node = Node::new(tag);

        if let Some(content) = capture.name("value").map(|s| s.as_str()) {
            let content = content.trim();

            // split flag suffix from the actual text
            let (value, flag) = match content.rsplit_once(' ') {
                Some((value, "SPECIAL")) => (Some(value), Some(SPECIAL)),
                Some((value, "WITH")) => (Some(value), Some(WITH)),
                _ => (Some(content), None),
            };

            if let Some(value) = value {
                node.value = Some(Arc::from(value));
            }

            if let Some(flag) = flag {
                node.attributes.insert("flag".into(), flag.into());
            }
        }

        if let Some(value) = capture.name("flag") {
            node.attributes.insert("flag".into(), value.as_str().into());
        }

        if let Some(value) = capture.name("range_start") {
            node.attributes.insert("from".into(), value.as_str().into());
        }

        if let Some(value) = capture.name("range_end") {
            node.attributes.insert("to".into(), value.as_str().into());
        }

        Ok(node)
    }
}

pub fn create_survey_builder<I, E>(lines: I) -> Result<SurveyBuilder>
where
    I: IntoIterator<Item = Result<String, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut survey_builder = SurveyBuilder::empty(true);
    let mut line_number = 0;
    let mut iterator = lines.into_iter();

    // collect survey meta information
    loop {
        line_number += 1;
        if let Some(line_read) = iterator.next() {
            let line = line_read
                .with_context(|| format!("Unable to read line {}", line_number))?
                .trim()
                .to_string();

            if !line.is_empty() {
                let node = Node::try_from(line.as_str()).with_context(|| {
                    format!(
                        "Error raised when reading line {} (text: '{}')",
                        line_number, line
                    )
                })?;
                let tag = node.tag.clone();
                survey_builder.attributes.insert(tag.clone(), node);
                if tag.as_ref() == "record" {
                    break;
                }
            }
        } else {
            break;
        }
    }

    // collect question
    let question_builder = QuestionBuilder::empty();
    survey_builder.questions.push(question_builder);
    let mut current_index = survey_builder.questions.len() - 1;
    loop {
        line_number += 1;
        if let Some(line_read) = iterator.next() {
            let line = line_read
                .with_context(|| format!("Unable to read line {}", line_number))?
                .trim()
                .to_string();

            if !line.is_empty() {
                if (&line == "END RECORD") | (&line == "END SURVEY") {
                    // pop the last builder which was added by the very final
                    // 'END VARIABLE'
                    survey_builder.questions.remove(current_index);
                    break;
                }

                let mut node = Node::try_from(line.as_str()).with_context(|| {
                    format!(
                        "Error raised when reading line {} (text: '{}')",
                        line_number, line
                    )
                })?;
                match node.tag.as_ref() {
                    "end" => {
                        if let Some(tag_ended) = node.value {
                            if tag_ended
                                .split_whitespace()
                                .next()
                                .is_some_and(|tag| tag == "VARIABLE")
                            {
                                // create a new question builder
                                let question_builder = QuestionBuilder::empty();
                                survey_builder.questions.push(question_builder);
                                current_index = survey_builder.questions.len() - 1;
                            }
                        } else {
                            let error = BadLine::new(format!(
                                r#"Unable to element name from: '{}'"#,
                                line
                            ));
                            return Err(error).with_context(|| {
                                format!("Error raised when reading line {}", line_number)
                            });
                        }
                    }
                    // "position" => {
                    //     println!("{:#?}", node);
                    // }
                    "range" => {
                        // in sss classics, question response values ALWAYS
                        // starts with number, hence this 'range' tag
                        let builder =
                            get_question_mut(&mut survey_builder, current_index, line_number)?;

                        // <range> can only be the first element,
                        // and everything else should be tagged as <value>
                        let is_empty = builder.responses.children.is_empty();
                        if is_empty {
                            if !node.attributes.contains_key("to") {
                                node.tag = "value".into();
                                if let Some(from) = node.attributes.remove("from") {
                                    node.attributes.insert("code".into(), from);
                                }
                            }
                        } else {
                            node.tag = "value".into();
                            if let Some(from) = node.attributes.remove("from") {
                                node.attributes.insert("code".into(), from);
                            }
                        }
                        builder.responses.children.push(node);
                    }

                    "size" => {
                        get_question_mut(&mut survey_builder, current_index, line_number)?
                            .responses
                            .children
                            .push(node);
                    }
                    _ => {
                        let builder =
                            get_question_mut(&mut survey_builder, current_index, line_number)?;
                        let res = builder.attributes.insert(node.tag.clone(), node);
                        if let Some(old) = res {
                            let error = BadLine::new(format!(
                                "Found duplicate entry: {{tag: '{}', value: '{:?}'}}",
                                old.tag.clone(),
                                old.value
                            ));
                            return Err(error).with_context(|| {
                                format!("Error raised when reading line {}", line_number)
                            });
                        }
                    }
                }
            }
        } else {
            break;
        }
    }
    Ok(survey_builder)
}

/// used internally
/// start_line_idx: used only for providing context for printing error
pub fn collect_question_stream<I, E>(
    mut stream: I,
    start_line_idx: usize,
) -> Result<QuestionBuilder>
where
    I: Iterator<Item = Result<String, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    // TODO: Make this a msvc??? so let this runs things and tell survey_builder
    // TODO: Put this into the create_survey_builder!
    let mut question_builder = QuestionBuilder::empty();
    let mut line_number = start_line_idx;
    let mut should_break = false;

    const END_MARKERS: &[&str] = &["END RECORD", "END SURVEY", "END VARIABLE"];
    loop {
        line_number += 1;
        let current_read = stream
            .next()
            .transpose()
            .with_context(|| format!("Unable to read line {}", line_number))?
            .map(|s| s.trim().to_owned());

        if let Some(line) = current_read {
            if line.is_empty() {
                break;
            }

            if END_MARKERS.iter().any(|prefix| line.starts_with(prefix)) {
                break;
            }

            let mut node = Node::try_from(line.as_str()).with_context(|| {
                format!(
                    "Error raised when reading line {} (text: '{}')",
                    line_number, line
                )
            })?;

            match node.tag.as_ref() {
                "end" => {
                    if let Some(tag_ended) = node.value {
                        if tag_ended
                            .split_whitespace()
                            .next()
                            .is_some_and(|tag| tag == "VARIABLE")
                        {
                            // just in case of future stupid me,,,
                            panic!("should not expect 'variable' end tag because it should already be checked earlier on!")
                        }
                    } else {
                        let error = BadLine::new(format!(
                            r#"Unable to element name from: '{}'"#,
                            line
                        ));
                        return Err(error).with_context(|| {
                            format!("Error raised when reading line {}", line_number)
                        });
                    }
                }
                // "position" => {
                //     println!("{:#?}", node);
                // }
                "range" => {
                    // <range> can only be the first element,
                    // and everything else should be tagged as <value>
                    let is_empty = question_builder.responses.children.is_empty();
                    if is_empty {
                        if !node.attributes.contains_key("to") {
                            node.tag = "value".into();
                            if let Some(from) = node.attributes.remove("from") {
                                node.attributes.insert("code".into(), from);
                            }
                        }
                    } else {
                        node.tag = "value".into();
                        if let Some(from) = node.attributes.remove("from") {
                            node.attributes.insert("code".into(), from);
                        }
                    }
                    question_builder.responses.children.push(node);
                }

                "size" => {  // only used for 'character' type
                    question_builder
                        .responses
                        .children
                        .push(node);
                }
                _ => { // variable attributes
                    let res = question_builder.attributes.insert(node.tag.clone(), node);
                    if let Some(old) = res {
                        let error = BadLine::new(format!(
                            "Found duplicate entry: {{tag: '{}', value: '{:?}'}}",
                            old.tag.clone(),
                            old.value
                        ));
                        return Err(error).with_context(|| {
                            format!("Error raised when reading line {}", line_number)
                        });
                    }
                }
            }
        } else {
            should_break = true;
        };

        if should_break {
            break;
        }
    }
    Ok(question_builder)
}

// ---------------- helper function ----------------
fn get_question_mut(
    survey_builder: &mut SurveyBuilder,
    current_index: usize,
    current_line_number: usize,
) -> Result<&mut QuestionBuilder> {
    let total_len = survey_builder.questions.len();
    let question_builder = survey_builder.questions
        .get_mut(current_index)
        .ok_or(
            InternalError::new(
                format!(
                    "Index mismatch. Trying to access question [{}] but current survey builder has total of [{}] questions",
                    current_index,
                    total_len
                )
            )
        )
        .with_context(|| format!("Error raised when reading line {}", current_line_number))?;
    Ok(question_builder)
}

// fn raise_index_mismatch_error(
//     current_index: usize,
//     total_len: usize,
//     line_number: usize,
// ) -> Result<()> {
//     let error = InternalError::new(format!(
//         "Index mismatch. Trying to access question [{}] but current survey
// builder has total of [{}] questions",         current_index, total_len
//     ));

//     Err(error).with_context(|| format!("Error raised when reading line {}",
// line_number)) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_start_success() {
        let example = [
            (r#".32 TO 64 "HELLOW 55 TO 32""#, ".32", "64"),
            (r#"-.32 TO 64 "HELLOW 55 TO 32""#, "-.32", "64"),
            (r#"32"#, "32", ""),
            (r#"43"#, "43", ""),
            (r#"-43"#, "-43", ""),
            (r#"3421.5000"#, "3421.5000", ""),
            (r#"555 SPECIAL"#, "555", ""),
        ];
        for (text, start, end) in example.into_iter() {
            let captured = PATTERN
                .captures(text)
                .expect(format!("Failed to match with text: '{}'", text).as_str());
            let _ = captured
                .name("prefix")
                .expect(format!("Failed to get prefix from text: '{}'", text).as_str());

            let cap_start = captured
                .name("range_start")
                .expect(format!("Failed to get `range_start` from text: '{}'", text).as_str())
                .as_str();

            let cap_end = captured.name("range_end").map(|s| s.as_str()).unwrap_or_default();

            assert_eq!(cap_start, start);
            assert_eq!(cap_end, end);
        }
    }
    #[test]
    fn test_prefix_capture() {
        let example_success = [
            (r#"TEST "" SPECIAL"#, "TEST"),
            (r"LOCATION WITH", "LOCATION"),
            (r#".32 TO 64 "HELLOW 55 TO 32""#, ".32 TO 64"),
            (r#"-.32 TO 64 "HELLOW 55 TO 32""#, "-.32 TO 64"),
            (r#"32"#, "32"),
            (r#"43"#, "43"),
            (r#"-43"#, "-43"),
            (r#"3421.5000"#, "3421.5000"),
            (r#"555 SPECIAL"#, "555"),
        ];

        for (text, corr) in example_success.into_iter() {
            let cap = PATTERN
                .captures(text)
                .expect(format!("Failed to match with text: '{}'", text).as_str());
            let prefix = cap
                .name("prefix")
                .expect(format!("Failed to get prefix from text: '{}'", text).as_str());
            assert_eq!(prefix.as_str(), corr);
        }
    }

    #[test]
    fn test_text_capture() {
        let example_success = [
            (r#"TEST "" SPECIAL"#, "TEST"),
            (r"LOCATION WITH", "LOCATION"),
        ];
        let name = "text";
        for (text, correct) in example_success.into_iter() {
            let captured = PATTERN
                .captures(text)
                .expect(format!("Failed to match with text: '{}'", text).as_str());

            // let captured_tags = captured
            let value = captured
                .name(name)
                .expect(format!("Failed to match with text: '{}'", text).as_str())
                .as_str();
            assert_eq!(value, correct);
        }

        let example_non_capture = ["32.21", "-3.2"];
        for text in example_non_capture.into_iter() {
            let captured = PATTERN
                .captures(text)
                .expect(format!("Failed to match with text: '{}'", text).as_str());
            assert!(
                captured.name(name).is_none(),
                "Text '{}' should not contain matched <{}>. Matched: {:?}",
                text,
                name,
                captured.name(name).unwrap().as_str()
            );
        }
    }

    #[test]
    fn test_value_capture() {
        let examples = [
            (r#"LABEL "A question""#, r#""A question""#),
            (
                r#"8 "Prefer not to say" SPECIAL"#,
                r#""Prefer not to say" SPECIAL"#,
            ),
            (r#"TEST "" SPECIAL"#, r#""" SPECIAL"#),
            (r"VARIABLE 3", "3"),
            (r"END VARIABLE", "VARIABLE"),
        ];

        for (text, correct) in examples {
            let captured = PATTERN
                .captures(text)
                .expect(format!("Failed to match with text: '{}'", text).as_str());
            let value = captured
                .name("value")
                .expect(format!("Failed to match <value>, text: '{}'", text).as_str())
                .as_str();

            assert_eq!(value, correct);
        }
    }

    #[test]
    fn test_regex_fail() {
        let example = [
            "hi 66 SPECIAL",
            r#" "random" SPECIAL"#,
            r#" TEXT "random" SPECIAL"#,
        ];
        for text in example.into_iter() {
            let captured = PATTERN.captures(text);
            assert!(
                captured.is_none(),
                "Matching for text '{}' should fail but didn't. Matched: {:?}",
                text,
                captured.unwrap()
            );
        }
    }
}
