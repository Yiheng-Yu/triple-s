use std::collections::BTreeMap;

use anyhow::{Context, Result};
use regex::Regex;
use strum::EnumString;

use crate::{
    Choice,
    Location,
    Question,
    Response,
    Spread,
    Survey,
    error::{self, BadLine, FileReadError, InternalError},
};

#[derive(Debug, PartialEq)]
pub enum SSSType {
    Classic,
    XML,
}

impl SSSType {
    pub fn parse(first_line: &str) -> Result<Self, BadLine> {
        match first_line.chars().next().unwrap_or_default() {
            'S' => Ok(Self::Classic),
            '<' => Ok(Self::XML),
            _ => Err(BadLine::new(format!(
                "Unable to identify sss version from the first line: '{first_line}'"
            ))),
        }
    }
}

#[derive(Debug)]
pub enum LineKind {
    Classic(SSSClassicLineType),
}

#[derive(Debug, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum SSSClassicLineType {
    #[strum(serialize = "")]
    Empty,
    Version,
    Date,
    Time,
    Origin,
    User,
    Note,
    Standard,
    Style,
    Label,
    #[strum(serialize = "SSS")]
    BOF,
    #[strum(serialize = "SURVEY")]
    SurveyStart,
    Title,
    #[strum(serialize = "RECORD")]
    RecordStart,
    #[strum(serialize = "VARIABLE")]
    QuestionStart,
    Name,
    Filter,
    #[strum(serialize = "POSITION")]
    Location,
    #[strum(serialize = "TYPE")]
    Kind,
    #[strum(serialize = "VALUES")]
    ResponseStart,
    End(String),

    /// field type to be determined by `Question` later on during parsing
    IntRange {
        from: i64,
        to: i64,
        has_na_value: bool,
    },
    /// field type to be determined by `Question` later on during parsing
    FloatRange {
        from: f64,
        to: f64,
        /// to handle case like: `-10.0 TO 10.0 WITH`
        has_na_value: bool,
    },
    /// line that looks like '9.9 "default value" SPECIAL
    FloatLabelled {
        value: f64,
        label: String,
        is_special: bool,
    },

    /// line that looks like '9 "default value" SPECIAL
    IntLabelled {
        value: i64,
        label: String,
        is_special: bool,
    },

    Spread,
    Size(usize),
}

fn unquote(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
        .to_string()
}

fn range<T>(value: &str, delimiter: &str) -> Result<(T, T)>
where
    T: std::str::FromStr + Clone,
    T::Err: std::fmt::Display,
{
    let parts: Vec<_> = value.split_whitespace().collect();
    if parts.len() == 1 {
        let item: T = parts[0].parse().map_err(BadLine::new)?;
        return Ok((item.clone(), item));
    }
    if parts.len() != 3 || !parts[1].eq_ignore_ascii_case(delimiter) {
        return Err(BadLine::new(format!("Invalid range: '{value}'")).into());
    }
    Ok((
        parts[0].parse().map_err(BadLine::new)?,
        parts[2].parse().map_err(BadLine::new)?,
    ))
}

impl SSSClassicLineType {
    pub fn parse_line(line: &str) -> Result<(Self, String)> {
        let line = line.trim();
        if line.is_empty() {
            return Ok((Self::Empty, String::new()));
        }

        let mut fields = line.splitn(2, char::is_whitespace);
        let prefix = fields
            .next()
            .ok_or(BadLine::new(format!(
                "Unable to get prefix from line: '{}'",
                line
            )))
            .with_context(|| format!("When parsing line: '{}'", line))?
            .trim();
        // value can be empty
        let value = fields.next().unwrap_or_default().trim();

        // parse `Choice` or `Range` type, this bit of parsing is extremely annoying
        if prefix.starts_with(['.', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-']) {
            if !(line.ends_with("SPECIAL")
                || line.ends_with("WITH")
                || line.ends_with('"')
                || line.ends_with(|c: char| c.is_ascii_digit())
                || line.ends_with('.'))
            {
                let error = BadLine::new(format!("Unidentified line kind: '{}'", line));
                return Err(error.into());
            };

            let pattern_range = Regex::new(r"^(?P<from>[0-9\.\-]+) TO (?P<to>[0-9\.\-]+)")?;
            let range_match = pattern_range.captures(line);
            match range_match {
                // example text: `10 TO 15`
                Some(captured) => {
                    let from = captured
                        .name("from")
                        .ok_or(BadLine::new(format!(
                            "Unable to get minimun quantity value from line: '{}'",
                            line
                        )))?
                        .as_str();
                    let to = captured
                        .name("to")
                        .ok_or(BadLine::new(format!(
                            "Unable to get maximum quantity value from line: '{}'",
                            line
                        )))?
                        .as_str();

                    let has_na_value = contains_na_value(line)
                        .with_context(|| format!("When parsing line: '{}'", line))?;

                    let parsed = if is_float(from) | is_float(to) {
                        let from = parse_float_string(from)
                            .with_context(|| format!("When parsing line: '{}'", line))?;
                        let to = parse_float_string(to)
                            .with_context(|| format!("When parsing line: '{}'", line))?;
                        Self::FloatRange {
                            from,
                            to,
                            has_na_value,
                        }
                    } else {
                        let from = parse_int_string(from)
                            .with_context(|| format!("When parsing line: '{}'", line))?;
                        let to = parse_int_string(to)
                            .with_context(|| format!("When parsing line: '{}'", line))?;
                        Self::IntRange {
                            from,
                            to,
                            has_na_value,
                        }
                    };

                    return Ok((parsed, "".into()));
                }
                None => {
                    // can be Choice/ Quantity with special value
                    let is_special = ends_with_suffix_special(value)
                        .with_context(|| format!("When parsing line: '{}'", line))?;

                    let sanitised = value.strip_suffix(" SPECIAL").unwrap_or(value);
                    let content = sanitised
                        .strip_circumfix('"', '"')
                        .map_or(sanitised.to_string(), |s| s.to_string());

                    let parsed = if is_float(prefix) {
                        let amount = parse_float_string(prefix)
                            .with_context(|| format!("When parsing line: '{}'", line))?;
                        Self::FloatLabelled {
                            value: amount,
                            label: content,
                            is_special,
                        }
                    } else {
                        let amount = parse_int_string(prefix)
                            .with_context(|| format!("When parsing line: '{}'", line))?;
                        Self::IntLabelled {
                            value: amount,
                            label: content,
                            is_special,
                        }
                    };
                    return Ok((parsed, "".into()));
                }
            }
        };
        let value = unquote(value);

        match prefix.to_ascii_uppercase().as_str() {
            "SURVEY" => Ok((Self::SurveyStart, value)),
            "RECORD" => Ok((Self::RecordStart, value)),
            "VARIABLE" => Ok((Self::QuestionStart, value)),
            "POSITION" => Ok((Self::Location, value)),
            "TYPE" => Ok((Self::Kind, value)),
            "VALUES" => Ok((Self::ResponseStart, value)),
            "SIZE" => Ok((Self::Size(value.parse().map_err(BadLine::new)?), value)),
            "END" => Ok((Self::End(value.to_uppercase()), "".to_string())),
            _ => {
                let parsed = prefix
                    .parse::<Self>()
                    .with_context(|| format!("When parsing line: '{}'", line))?;
                Ok((parsed, value))
            }
        }
    }
}

fn response_kind(value: &str) -> Result<Response> {
    match value
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase()
        .as_str()
    {
        "CHARACTER" => Ok(Response::Character(
            value
                .split_whitespace()
                .nth(1)
                .map_or(Ok(0), |v| v.parse().map_err(BadLine::new))
                .with_context(|| format!("When parsing text: '{}'", value))?,
        )),
        "SINGLE" => Ok(Response::Single {
            item: BTreeMap::new(),
        }),
        "MULTIPLE" => Ok(Response::Multiple {
            item: BTreeMap::new(),
            spread: Spread::default(),
        }),
        "LOGICAL" => Ok(Response::Single {
            item: BTreeMap::from([
                (0, Choice::new(0, "FALSE", None)),
                (1, Choice::new(1, "TRUE", None)),
            ]),
        }),
        "QUANTITY" => Ok(Response::Quantity {
            min: 0.0,
            max: 0.0,
            na_label: None,
            na_value: None,
        }),
        "NOTE" => Ok(Response::Note(String::new())),
        _ => Err(BadLine::new(format!("Unknown response type: '{value}'")).into()),
    }
}

fn get_question_mut<'a>(
    survey: &'a mut Survey,
    index: Option<usize>,
    line: &str,
) -> Result<&'a mut Question> {
    let index = index
        .ok_or_else(|| InternalError::new(format!("Line outside a variable: '{line}'")))?;
    let question = survey
        .record
        .content
        .get_mut(index)
        .ok_or_else(|| InternalError::new("Invalid current question"))?;
    Ok(question)
}

pub fn read_sss_file_classic<I, E>(lines: I) -> Result<Survey>
where
    I: IntoIterator<Item = Result<String, E>>,
    E: std::error::Error,
{
    let mut survey = Survey::empty();
    let mut current = None;
    let mut in_values = false;

    for (line_number, raw_line) in lines.into_iter().enumerate() {
        let line = raw_line
            .map_err(|err| FileReadError::new(err.to_string()))
            .with_context(|| format!("Line number: {}", line_number))?;

        let (kind, value) = SSSClassicLineType::parse_line(&line)
            .with_context(|| format!("Line number: {}", line_number))?;
        match kind {
            SSSClassicLineType::Empty
            | SSSClassicLineType::SurveyStart
            | SSSClassicLineType::BOF => {}
            SSSClassicLineType::Version => survey.meta.version = Some(value),
            SSSClassicLineType::Date => survey.meta.date = Some(value),
            SSSClassicLineType::Time => survey.meta.time = Some(value),
            SSSClassicLineType::Origin => survey.meta.origin = Some(value),
            SSSClassicLineType::User => survey.meta.user = Some(value),
            SSSClassicLineType::Note => survey.meta.note.push(value),
            SSSClassicLineType::Standard => survey.meta.standard = Some(value),
            SSSClassicLineType::Style => survey.meta.style = Some(value),
            SSSClassicLineType::Title => survey.title = Some(value),
            SSSClassicLineType::RecordStart => survey.record.id = value,
            SSSClassicLineType::QuestionStart => {
                let question_index = value
                    .parse::<u64>()
                    .with_context(|| format!("Line number: {}", line_number))?;
                survey.record.content.push(Question {
                    index: question_index,
                    name: "".into(),
                    text: "".into(),
                    filter: None,
                    location: Location::default(),
                    response: Response::Single {
                        item: BTreeMap::default(),
                    },
                });

                current = Some(survey.record.content.len() - 1);
                in_values = false;
            }
            SSSClassicLineType::End(end) => {
                let to_match = end
                    .to_ascii_uppercase()
                    .split_whitespace()
                    .next()
                    .ok_or(BadLine::new("Unable to parse line"))
                    .with_context(|| format!("Line number: {}", line_number))?
                    .to_ascii_uppercase();

                match to_match.as_str() {
                    "VALUES" => in_values = false,
                    "VARIABLE" => {
                        in_values = false;
                    }
                    "RECORD" | "SURVEY" | "SSS" => break,
                    _ => {
                        return Err(error::UnexpectedLineKind::new(
                            LineKind::Classic(SSSClassicLineType::End(end)),
                            line,
                        )
                        .into());
                    }
                }
            }
            SSSClassicLineType::ResponseStart => {
                current
                    .ok_or_else(|| InternalError::new("VALUES outside a variable"))
                    .with_context(|| {
                        format!("When parsing line {}: '{}'", line_number, line)
                    })?;
                in_values = true;
            }
            SSSClassicLineType::Name => {
                get_question_mut(&mut survey, current, &line)
                    .with_context(|| format!("When parsing line {}: '{}'", line_number, line))?
                    .name = value
            }
            SSSClassicLineType::Filter => {
                get_question_mut(&mut survey, current, &line)
                    .with_context(|| format!("When parsing line {}: '{}'", line_number, line))?
                    .filter = Some(value)
            }
            SSSClassicLineType::Label => {
                get_question_mut(&mut survey, current, &line)
                    .with_context(|| format!("When parsing line {}: '{}'", line_number, line))?
                    .text = value
            }
            SSSClassicLineType::Location => {
                if !in_values {
                    get_question_mut(&mut survey, current, &line)
                        .with_context(|| {
                            format!("When parsing line {}: '{}'", line_number, line)
                        })?
                        .location = Location {
                        from: range(&value, "TO")
                            .with_context(|| {
                                format!("When parsing line {}: '{}'", line_number, line)
                            })?
                            .0,
                        to: range(&value, "TO")
                            .with_context(|| {
                                format!("When parsing line {}: '{}'", line_number, line)
                            })?
                            .1,
                    };
                }
            }
            SSSClassicLineType::Size(size) => {
                if let Response::Character(length) =
                    &mut get_question_mut(&mut survey, current, &line)
                        .with_context(|| {
                            format!("When parsing line {}: '{}'", line_number, line)
                        })?
                        .response
                {
                    *length = size;
                }
            }
            SSSClassicLineType::Kind => {
                let question =
                    get_question_mut(&mut survey, current, &line).with_context(|| {
                        format!("When parsing line {}: '{}'", line_number, line)
                    })?;
                question.response = response_kind(&value).with_context(|| {
                    format!("When parsing line {}: '{}'", line_number, line)
                })?;
                if matches!(question.response, Response::Note(_)) {
                    question.response = Response::Note(question.text.clone());
                }
            }

            SSSClassicLineType::Spread => {
                let question =
                    get_question_mut(&mut survey, current, &line).with_context(|| {
                        format!("When parsing line {}: '{}'", line_number, line)
                    })?;
                match &mut question.response {
                    Response::Multiple { spread, .. } => {
                        let (num_subfields, width) = range(&value, "OF")?;
                        *spread = Spread {
                            num_subfields,
                            width,
                        };
                    }
                    _ => return Err(BadLine::new("SPREAD requires MULTIPLE").into()),
                }
            }

            SSSClassicLineType::IntRange {
                from,
                to,
                has_na_value: _,
            } => {
                let question =
                    get_question_mut(&mut survey, current, &line).with_context(|| {
                        format!("When parsing line {}: '{}'", line_number, line)
                    })?;
                match &mut question.response {
                    Response::Quantity {
                        min,
                        max,
                        na_value: _,
                        na_label: _,
                    } => {
                        *min = from as f64;
                        *max = to as f64;
                    }
                    _ => {
                        let message = format!(
                            "Wrong question type received for line `{value}`. Expect: Quantity. Actual: {:?}",
                            question.response
                        );
                        let error = InternalError::new(message);
                        return Err(error)
                            .with_context(|| format!("When parsing line: {}", line));
                    }
                }
            }

            SSSClassicLineType::FloatRange {
                from,
                to,
                has_na_value: _,
            } => {
                let question =
                    get_question_mut(&mut survey, current, &line).with_context(|| {
                        format!("When parsing line {}: '{}'", line_number, line)
                    })?;
                match &mut question.response {
                    Response::Quantity {
                        min,
                        max,
                        na_value: _,
                        na_label: _,
                    } => {
                        *min = from;
                        *max = to;
                    }
                    _ => {
                        let message = format!(
                            "Wrong question type received for line `{value}`. Expect: Quantity. Actual: {:?}",
                            question.response
                        );
                        let error = InternalError::new(message);
                        return Err(error)
                            .with_context(|| format!("When parsing line: {}", line));
                    }
                }
            }

            SSSClassicLineType::IntLabelled {
                value,
                label,
                is_special,
            } => {
                let question =
                    get_question_mut(&mut survey, current, &line).with_context(|| {
                        format!("When parsing line {}: '{}'", line_number, line)
                    })?;
                match &mut question.response {
                    Response::Single { item } | Response::Multiple { item, .. } => {
                        let code = if value >= 0 {
                            value as u64
                        } else {
                            let error = BadLine::new("Code number cannot be negative");
                            return Err(error).with_context(|| {
                                format!(
                                    "When parsing line {} ('{}') for question {}",
                                    line_number, line, question.name
                                )
                            });
                        };
                        let choice = Choice::new(code, label, None)._set_special(is_special);
                        item.insert(choice.code, choice);
                    }
                    Response::Quantity {
                        min: _,
                        max: _,
                        na_value,
                        na_label,
                    } => {
                        if is_special {
                            *na_value = Some(value as f64);
                            *na_label = Some(label);
                        } else {
                            let message = format!("Undefined line: '{}'", line);
                            let error = InternalError::new(message);
                            return Err(error).with_context(|| {
                                format!(
                                    "When parsing line {} ('{}') for question {}",
                                    line_number, line, question.name
                                )
                            });
                        }
                    }
                    _ => {
                        let message = format!(
                            "Wrong question type received for line `{value}`. Expect: Single/Multiple/Quantity. Actual: {:?}",
                            question.response
                        );
                        let error = InternalError::new(message);
                        return Err(error).with_context(|| {
                            format!("When parsing line {}: '{}'", line_number, line)
                        });
                    }
                }
            }
            SSSClassicLineType::FloatLabelled {
                value,
                label,
                is_special,
            } => {
                let question =
                    get_question_mut(&mut survey, current, &line).with_context(|| {
                        format!("When parsing line {}: '{}'", line_number, line)
                    })?;
                match &mut question.response {
                    Response::Quantity {
                        min: _,
                        max: _,
                        na_value,
                        na_label,
                    } => {
                        if is_special {
                            *na_value = Some(value);
                            *na_label = Some(label);
                        } else {
                            let message = format!("Undefined line: '{}'", line);
                            let error = InternalError::new(message);
                            return Err(error).with_context(|| {
                                format!(
                                    "When parsing line '{}' for question {}",
                                    line, question.name
                                )
                            });
                        }
                    }
                    _ => {
                        let message = format!(
                            "Wrong question type received for line {} ('{}'). Expect: Single/Multiple/Quantity. Actual: {:?}",
                            line_number, value, question.response
                        );
                        let error = InternalError::new(message);
                        return Err(error).with_context(|| {
                            format!("When parsing line {}: '{}'", line_number, line)
                        });
                    }
                }
            }
        }
    }
    Ok(survey)
}

#[inline]
fn is_float(text: &str) -> bool {
    text.contains(".")
}

#[inline]
fn contains_na_value(text: &str) -> Result<bool> {
    if text.ends_with(" WITH") {
        Ok(true)
    } else if text.ends_with(|c: char| c.is_ascii_digit()) {
        Ok(false)
    } else {
        Err(BadLine::new("Unidentified line suffix").into())
    }
}

#[inline]
fn ends_with_suffix_special(text: &str) -> Result<bool> {
    if text.ends_with(" SPECIAL") {
        Ok(true)
    } else if text.ends_with('"') {
        Ok(false)
    } else {
        Err(BadLine::new("Unidentified line suffix").into())
    }
}

#[inline]
fn parse_float_string(text: &str) -> Result<f64, std::num::ParseFloatError> {
    if text.starts_with('.') {
        format!("0{}", text).trim().parse::<f64>()
    } else {
        text.trim().parse::<f64>()
    }
}

#[inline]
fn parse_int_string(text: &str) -> Result<i64, std::num::ParseIntError> {
    text.trim().parse::<i64>()
}

#[cfg(test)]
mod tests {

    use super::*;

    fn assert_parsed_quantity_float_range(
        parsed: SSSClassicLineType,
        correct_from: f64,
        correct_to: f64,
        correct_na_flag: bool,
    ) {
        match parsed {
            SSSClassicLineType::FloatRange {
                from,
                to,
                has_na_value,
            } => {
                assert_eq!(from, correct_from);
                assert_eq!(to, correct_to);
                assert_eq!(has_na_value, correct_na_flag);
            }
            _ => panic!("Parsed into wrong kind of line: {:?}", parsed),
        }
    }

    #[test]
    fn test_parse_quantity_float_line() {
        let line = ".0 TO 999.9";
        let (parsed, text) = SSSClassicLineType::parse_line(line).unwrap();
        assert_eq!(text, "");
        assert_parsed_quantity_float_range(parsed, 0.0, 999.9, false);

        let line = "0 TO 999.9";
        let (parsed, text) = SSSClassicLineType::parse_line(line).unwrap();
        assert_eq!(text, "");
        assert_parsed_quantity_float_range(parsed, 0.0, 999.9, false);

        let line = "-0 TO 999.9";
        let (parsed, text) = SSSClassicLineType::parse_line(line).unwrap();
        assert_eq!(text, "");
        assert_parsed_quantity_float_range(parsed, 0.0, 999.9, false);

        let line = "    -99 TO 999.9 WITH ";
        let (parsed, text) = SSSClassicLineType::parse_line(line).unwrap();
        assert_eq!(text, "");
        assert_parsed_quantity_float_range(parsed, -99.0, 999.9, true);
    }

    #[test]
    fn test_parse_quantity_float_error() {
        let line = "+0 TO 999.9";
        let should_be_error = SSSClassicLineType::parse_line(line);
        assert!(
            should_be_error.is_err(),
            "Error failed to raise when parsing line '{}', result: {:?}",
            line,
            should_be_error
        );

        let line = "0 TO 999.9 WHAT";
        let should_be_error = SSSClassicLineType::parse_line(line);
        assert!(
            should_be_error.is_err(),
            "Error failed to raise when parsing line '{}', result: {:?}",
            line,
            should_be_error
        );

        let line = "0 TO 999.9 SPECIAL";
        let should_be_error = SSSClassicLineType::parse_line(line);
        assert!(
            should_be_error.is_err(),
            "Error failed to raise when parsing line '{}', result: {:?}",
            line,
            should_be_error
        );

        let line = "0 TO 999";
        let (parsed, _) = SSSClassicLineType::parse_line(line).unwrap();
        match &parsed {
            SSSClassicLineType::FloatRange { .. } => {
                panic!("Expect to be IntRange but FloatRange was parsed!")
            }
            SSSClassicLineType::IntRange { .. } => (),
            _ => panic!("Parsed into wrong kind of line: {:?}", parsed),
        }
    }

    fn assert_parsed_quantity_int_range(
        parsed: SSSClassicLineType,
        correct_from: i64,
        correct_to: i64,
        correct_na_flag: bool,
    ) {
        match parsed {
            SSSClassicLineType::IntRange {
                from,
                to,
                has_na_value,
            } => {
                assert_eq!(from, correct_from);
                assert_eq!(to, correct_to);
                assert_eq!(has_na_value, correct_na_flag);
            }
            _ => panic!("Parsed into wrong kind of line: {:?}", parsed),
        }
    }

    #[test]
    fn test_parse_quantity_int_error() {
        let line = "0 TO 999 WHAT";
        let should_be_error = SSSClassicLineType::parse_line(line);
        assert!(
            should_be_error.is_err(),
            "Error failed to raise when parsing line '{}', result: {:?}",
            line,
            should_be_error
        );

        let line = "0 TO 999 SPECIAL";
        let should_be_error = SSSClassicLineType::parse_line(line);
        assert!(
            should_be_error.is_err(),
            "Error failed to raise when parsing line '{}', result: {:?}",
            line,
            should_be_error
        );

        let line = "0";
        let should_be_error = SSSClassicLineType::parse_line(line);
        assert!(
            should_be_error.is_err(),
            "Error failed to raise when parsing line '{}', result: {:?}",
            line,
            should_be_error
        );

        let line = "0 TO 999";
        let (parsed, _) = SSSClassicLineType::parse_line(line).unwrap();
        match &parsed {
            SSSClassicLineType::IntRange { .. } => (),
            SSSClassicLineType::FloatRange { .. } => {
                panic!("Expect to be FloatRange but IntRange was parsed!")
            }
            _ => panic!("Parsed into wrong kind of line: {:?}", parsed),
        }
    }

    #[test]
    fn test_parse_quantity_int_line() {
        let line = "0 TO 999";
        let (parsed, text) = SSSClassicLineType::parse_line(line).unwrap();
        assert_eq!(text, "");
        assert_parsed_quantity_int_range(parsed, 0, 999, false);

        let line = "-0 TO 999";
        let (parsed, text) = SSSClassicLineType::parse_line(line).unwrap();
        assert_eq!(text, "");
        assert_parsed_quantity_int_range(parsed, 0, 999, false);

        let line = "-3 TO 999 WITH";
        let (parsed, text) = SSSClassicLineType::parse_line(line).unwrap();
        assert_eq!(text, "");
        assert_parsed_quantity_int_range(parsed, -3, 999, true);
    }

    #[test]
    fn test_parse_labelled_line() {
        assert_eq!(
            SSSClassicLineType::parse_line(r#"1 "Hii how are you""#)
                .unwrap()
                .0,
            SSSClassicLineType::IntLabelled {
                value: 1,
                label: "Hii how are you".to_string(),
                is_special: false
            }
        );
        assert_eq!(
            SSSClassicLineType::parse_line(r#"1 """#).unwrap().0,
            SSSClassicLineType::IntLabelled {
                value: 1,
                label: "".to_string(),
                is_special: false
            }
        );

        assert_eq!(
            SSSClassicLineType::parse_line(r#"1 "" SPECIAL"#).unwrap().0,
            SSSClassicLineType::IntLabelled {
                value: 1,
                label: "".to_string(),
                is_special: true
            }
        );

        assert_eq!(
            SSSClassicLineType::parse_line(r#"-1 "" SPECIAL"#)
                .unwrap()
                .0,
            SSSClassicLineType::IntLabelled {
                value: -1,
                label: "".to_string(),
                is_special: true
            }
        );

        assert_eq!(
            SSSClassicLineType::parse_line(r#"-1.6 "TEST TEXT" SPECIAL"#)
                .unwrap()
                .0,
            SSSClassicLineType::FloatLabelled {
                value: -1.6,
                label: "TEST TEXT".to_string(),
                is_special: true
            }
        );
    }

    #[test]
    fn test_parse_labelled_line_error() {
        let line = r#"-1.6 SPECIAL"#;
        let result = SSSClassicLineType::parse_line(line);
        assert!(
            result.is_err(),
            "Error failed to raise when parsing line '{}', result: {:?}",
            line,
            result
        );

        let line = r#"-1.6 "TEST" WITH"#;
        let result = SSSClassicLineType::parse_line(line);
        assert!(
            result.is_err(),
            "Error failed to raise when parsing line '{}', result: {:?}",
            line,
            result
        );

        let line = r#"-1.6 "#;
        let result = SSSClassicLineType::parse_line(line);
        assert!(
            result.is_err(),
            "Error failed to raise when parsing line '{}', result: {:?}",
            line,
            result
        );
    }

    #[test]
    fn test_parse_start_n_end() {
        let (parsed, text) = SSSClassicLineType::parse_line("SSS").unwrap();
        assert_eq!(parsed, SSSClassicLineType::BOF);
        assert_eq!(text, "");

        let (parsed, text) = SSSClassicLineType::parse_line("SURVEY").unwrap();
        assert_eq!(parsed, SSSClassicLineType::SurveyStart);
        assert_eq!(text, "");

        let (parsed, text) = SSSClassicLineType::parse_line("VARIABLE").unwrap();
        assert_eq!(parsed, SSSClassicLineType::QuestionStart);
        assert_eq!(text, "");

        let (parsed, text) = SSSClassicLineType::parse_line("record").unwrap();
        assert_eq!(parsed, SSSClassicLineType::RecordStart);
        assert_eq!(text, "");

        let (parsed, text) = SSSClassicLineType::parse_line("end record").unwrap();
        assert_eq!(parsed, SSSClassicLineType::End("RECORD".into()));
        assert_eq!(text, "");

        let (parsed, text) = SSSClassicLineType::parse_line("end survey").unwrap();
        assert_eq!(parsed, SSSClassicLineType::End("SURVEY".into()));
        assert_eq!(text, "");

        let (parsed, text) = SSSClassicLineType::parse_line("end variable").unwrap();
        assert_eq!(parsed, SSSClassicLineType::End("VARIABLE".into()));
        assert_eq!(text, "");
    }

    // used for trimming whitespaces for string comparison
    fn normalize(s: &str) -> String {
        s.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn put_into_record(content: &str) -> impl Iterator<Item = Result<String, BadLine>> {
        let content = format!(
            r#"
        SSS
        VERSION 1.0
        STANDARD NAMES
        SURVEY
        TITLE "Customer Experience Survey - HTML Test"
        RECORD A
        {}
        END RECORD
        END SURVEY
        END SSS
        "#,
            content
        );
        content
            .lines()
            .map(|line| Ok::<String, BadLine>(line.to_string()))
            .collect::<Vec<Result<String, BadLine>>>()
            .into_iter()
    }

    fn create_example_single_code_iterator() -> impl Iterator<Item = Result<String, BadLine>> {
        let content = r#"
        VARIABLE 1
        NAME "AGE"
        LABEL "<p>What is your <strong>age</strong>?</p>"
        TYPE SINGLE
        POSITION 7
        VALUES
        1 "Under 18"
        2 "18 to 24"
        3 "25 to 34"
        4 "35 to 44"
        5 "45 to 54"
        6 "55 to 64"
        7 "65 or older"
        8 "Prefer not to say" SPECIAL
        END VALUES
        END VARIABLE"#;
        put_into_record(content)
    }

    #[test]
    fn test_single_code_question_read() {
        let lines = create_example_single_code_iterator();
        let survey = read_sss_file_classic(lines).unwrap();
        assert_eq!(survey.record.content.len(), 1);

        let question = survey.get_question("AGE").unwrap();
        assert_eq!(question.name, "AGE");
        assert_eq!(
            question.text,
            "<p>What is your <strong>age</strong>?</p>"
        );
        assert_eq!(question.response.is_categorical(), true);
    }

    #[test]
    fn test_single_code_question_write() {
        let lines = create_example_single_code_iterator();
        let survey = read_sss_file_classic(lines).unwrap();
        let question = survey.get_question("AGE").unwrap();

        let sss_output = question.to_sss_classic().trim().to_string();
        assert!(sss_output.contains(r#"NAME "AGE""#));
        assert!(sss_output.contains(r#"LABEL "<p>What is your <strong>age</strong>?</p>""#));
        assert_eq!(
            normalize(&question.location.to_sss_classic()),
            normalize(r#"POSITION 7"#)
        );
        assert_eq!(
            normalize(&question.response.to_sss_classic()),
            normalize(
                r#"TYPE SINGLE
            VALUES
            1 "Under 18"
            2 "18 to 24"
            3 "25 to 34"
            4 "35 to 44"
            5 "45 to 54"
            6 "55 to 64"
            7 "65 or older"
            8 "Prefer not to say" SPECIAL
            END VALUES"#
            )
        );
        assert!(sss_output.ends_with(r#"END VALUES"#));
    }

    // todo! finish this unit tests
    fn create_example_multi_code_iterator() -> impl Iterator<Item = Result<String, BadLine>> {
        let content = r#"
        VARIABLE 1
        NAME "K2"
        LABEL "I AM A QUESTION LABEL"
        TYPE MULTIPLE
        POSITION 1147 TO 1155
        SPREAD 7 OF 2
        VALUES
        1 "Under 18"
        2 "18 to 24"
        3 "25 to 34"
        4 "35 to 44"
        5 "45 to 54"
        6 "55 to 64"
        7 "65 or older"
        8 "Prefer not to say" SPECIAL
        END VALUES
        END VARIABLE"#;
        put_into_record(content)
    }

    impl Response {
        pub fn get_spread(&self) -> Option<&Spread> {
            match self {
                Self::Multiple { item: _, spread } => Some(spread),
                _ => None,
            }
        }
    }

    #[test]
    fn test_multi_code_question_read() {
        let lines = create_example_multi_code_iterator();
        let survey = read_sss_file_classic(lines).unwrap();
        assert_eq!(survey.record.content.len(), 1);

        let question = survey.get_question("K2").unwrap();
        assert_eq!(question.name, "K2");
        assert_eq!(question.text, "I AM A QUESTION LABEL");
        assert_eq!(question.response.is_categorical(), true);
    }

    #[test]
    fn test_multi_code_question_write() {
        let lines = create_example_multi_code_iterator();
        let survey = read_sss_file_classic(lines).unwrap();
        assert_eq!(survey.record.content.len(), 1);

        let question = survey.get_question("K2").unwrap();
        let sss_output = question.to_sss_classic().trim().to_string();
        assert!(sss_output.contains(r#"NAME "K2""#));
        assert!(sss_output.contains(r#"LABEL "I AM A QUESTION LABEL""#));
        assert_eq!(
            question.location.to_sss_classic(),
            "POSITION 1147 TO 1155"
        );
        assert_eq!(
            question.response.get_spread().unwrap().to_sss_classic(),
            "SPREAD 7 OF 2"
        );
    }

    #[test]
    fn test_sss_write_spread() {
        let spread = Spread {
            num_subfields: 5,
            width: 3,
        };
        assert_eq!(spread.to_sss_classic(), "SPREAD 5 OF 3")
    }

    fn create_example_float_quantity_iterator() -> impl Iterator<Item = Result<String, BadLine>>
    {
        let content = r#"
        VARIABLE 1
        NAME "QUANTITY"
        LABEL "I AM A QUESTION LABEL"
        TYPE QUANTITY
        POSITION 1147 TO 1155
        POSITION 2399 TO 2408
            VALUES
                0.00 TO 99.00
            END VALUES
        END VARIABLE"#;
        put_into_record(content)
    }

    #[test]
    fn test_quantity_question_float_read() {
        let lines = create_example_float_quantity_iterator();
        let survey = read_sss_file_classic(lines).unwrap();
        assert_eq!(survey.record.content.len(), 1);

        let question = survey.get_question("QUANTITY").unwrap();
        assert_eq!(question.response.is_quantity(), true);

        match &question.response {
            Response::Quantity {
                min,
                max,
                na_value,
                na_label,
            } => {
                assert_eq!(min.clone(), 0.00);
                assert_eq!(max.clone(), 99.00);
                assert_eq!(na_value.as_ref(), None);
                assert_eq!(na_label.as_ref(), None);
            }
            _ => panic!("Wrong kind of response received!"),
        }
    }

    fn create_example_float_quantity_with_special_iterator()
    -> impl Iterator<Item = Result<String, BadLine>> {
        let content = r#"
        VARIABLE 1
        NAME "QUANTITY"
        LABEL "I AM A QUESTION LABEL"
        TYPE QUANTITY
        POSITION 1147 TO 1155
        POSITION 2399 TO 2408
            VALUES
                0.00 TO 99.00 WITH
                -999999.99 "Don't Know" SPECIAL
            END VALUES
        END VARIABLE"#;
        put_into_record(content)
    }

    #[test]
    fn test_quantity_question_float_special_read() {
        let lines = create_example_float_quantity_with_special_iterator();
        let survey = read_sss_file_classic(lines).unwrap();
        assert_eq!(survey.record.content.len(), 1);

        let question = survey.get_question("QUANTITY").unwrap();
        assert_eq!(question.response.is_quantity(), true);

        match &question.response {
            Response::Quantity {
                min,
                max,
                na_value,
                na_label,
            } => {
                assert_eq!(min.clone(), 0.00);
                assert_eq!(max.clone(), 99.00);
                assert_eq!(na_value.unwrap(), -999999.99);
                assert_eq!(
                    na_label.as_ref().map(|s| s.clone()),
                    Some("Don't Know".to_string())
                );
            }
            _ => panic!("Wrong kind of response received!"),
        }
    }

    fn create_character_question_iterator() -> impl Iterator<Item = Result<String, BadLine>> {
        let content = r#"
        VARIABLE 12
            NAME "PRODUCTINFO"
            LABEL "This is a label"
            TYPE CHARACTER
            POSITION 1243 TO 3242
            SIZE 2000
        END VARIABLE"#;
        put_into_record(content)
    }

    #[test]
    fn test_character_question_read_success() {
        let lines = create_character_question_iterator();
        let survey = read_sss_file_classic(lines).unwrap();
        assert_eq!(survey.record.content.len(), 1);
        let question = survey.get_question("PRODUCTINFO").unwrap();
        assert_eq!(question.response, Response::Character(2000))
    }

    #[test]
    fn test_character_question_read_fail() {
        let content = r#"
        VARIABLE 12
            NAME "PRODUCTINFO"
            LABEL "This is a label"
            TYPE CHARACTER
            POSITION 1243 TO 3242
            SIZE -2000
        END VARIABLE"#;
        let result = read_sss_file_classic(put_into_record(content));
        match result {
            Ok(instance) => panic!(
                "Error not raised when it should, instance parsed:\n{:#?}",
                instance
            ),
            Err(err) => {
                println!("Error successfully raised: '{:?}'", err);
            }
        }
    }
}
