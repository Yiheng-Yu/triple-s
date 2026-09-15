use std::{collections::BTreeMap, fmt::Write};

use regex::Regex;
use strum::EnumString;

use crate::{
    Choice,
    Location,
    Meta,
    Question,
    Record,
    Response,
    Spread,
    Survey,
    error::{self, BadLine, IoError},
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
        annotation: Option<String>,
    },
    /// field type to be determined by `Question` later on during parsing
    FloatRange {
        from: f64,
        to: f64,
        annotation: Option<String>,
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

#[inline]
fn quote(value: &str) -> String {
    format!(r#""{}""#, value)
}

#[inline]
fn line(output: &mut String, content: impl std::fmt::Display) {
    writeln!(output, "{content}").expect("writing to a String shouldn't fail");
}

fn range<T>(value: &str, delimiter: &str) -> Result<(T, T), IoError>
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
    pub fn parse_line(line: &str) -> Result<(Self, String), IoError> {
        let line = line.trim();
        if line.is_empty() {
            return Ok((Self::Empty, String::new()));
        }
        let mut fields = line.splitn(2, char::is_whitespace);
        let prefix = fields
            .next()
            .ok_or(IoError::from(BadLine::new(format!(
                "Unable to get prefix from line: '{}'",
                line
            ))))?
            .trim();

        // value can be empty
        let value = fields.next().unwrap_or_default().trim();

        // parse `Choice` or `Range` type, this bit of parsing is extremely annoying
        // NOTE: The order of these regex matches are important don't move them around becuase it mese things up
        let re_range_float = Regex::new(
            r"^(?P<range>[0-9\-]+\.[0-9]+ TO [0-9\-]+\.[0-9]+)(?P<suffix>.*)$",
        )
        .map_err(|err| {
            IoError::Internal(format!("Failed to create regex: {}", err))
        })?;
        if let Some(capture_float) = re_range_float.captures(line) {
            if let Some(float_range) = capture_float.name("range") {
                let (from, to) = range::<f64>(float_range.as_str(), "TO")?;
                let content = capture_float
                    .name("content")
                    .map_or("", |matched| matched.as_str())
                    .trim();
                let (text, annotation) = parse_annotation_line(content)?;
                return Ok((
                    Self::FloatRange {
                        from,
                        to,
                        annotation,
                    },
                    text.to_string(),
                ));
            } else {
                return Err(BadLine::new(format!(
                    "Unable to parse line due to regex error: '{}'",
                    line
                ))
                .into());
            }
        };

        let re_range_int = Regex::new(r"^(?P<range>[0-9\-]+ TO [0-9\-]+)(?P<content>.*)$")
            .map_err(|err| {
                IoError::Internal(format!("Failed to create regex: {}", err))
            })?;
        if let Some(capture_int) = re_range_int.captures(line) {
            if let Some(int_range) = capture_int.name("range") {
                let (from, to) = range::<i64>(int_range.as_str(), "TO")?;
                let content = capture_int
                    .name("content")
                    .map_or("", |matched| matched.as_str())
                    .trim();

                let (text, annotation) = parse_annotation_line(content)?;
                return Ok((
                    Self::IntRange {
                        from,
                        to,
                        annotation,
                    },
                    text.to_string(),
                ));
            } else {
                return Err(BadLine::new(format!(
                    "Unable to parse line due to regex error: '{}'",
                    line
                ))
                .into());
            }
        };

        let re_float_range_single = Regex::new(r"^(?P<range>[0-9\-]+\.[0-9]+)(?P<content>.*)$")
            .map_err(|err| {
                IoError::Internal(format!("Failed to create regex: {}", err))
            })?;
        if let Some(capture_float) = re_float_range_single.captures(line) {
            if let Some(float_range) = capture_float.name("range") {
                let from = float_range.as_str().parse::<f64>().map_err(|err| {
                    BadLine::new(format!(
                        "Error extracting range from line: `{}`, readon: {}",
                        line,
                        err
                    ))
                })?;
                let content = capture_float
                    .name("content")
                    .map_or("", |matched| matched.as_str())
                    .trim();
                let (text, annotation) = parse_annotation_line(content)?;
                return Ok((
                    Self::FloatRange {
                        from,
                        to: from,
                        annotation,
                    },
                    text.to_string(),
                ));
            } else {
                return Err(BadLine::new(format!(
                    "Unable to parse line due to regex error: '{}'",
                    line
                ))
                .into());
            }
        };

        let re_int_range_single = Regex::new(r"^(?P<range>[0-9\-]+)(?P<content>.*)$")
            .map_err(|err| {
                IoError::Internal(format!("Failed to create regex: {}", err))
            })?;
        if let Some(capture_int) = re_int_range_single.captures(line) {
            if let Some(int_range) = capture_int.name("range") {
                let from = int_range.as_str().parse::<i64>().map_err(|err| {
                    BadLine::new(format!(
                        "Error extracting code from line: `{}`, readon: {}",
                        line,
                        err
                    ))
                })?;

                let content = capture_int
                    .name("content")
                    .map_or("", |matched| matched.as_str())
                    .trim();
                let (text, annotation) = parse_annotation_line(content)?;
                return Ok((
                    Self::IntRange {
                        from,
                        to: from,
                        annotation,
                    },
                    text.to_string(),
                ));
            } else {
                return Err(BadLine::new(format!(
                    "Unable to parse line due to regex error: '{}'",
                    line
                ))
                .into());
            }
        };

        let value = unquote(value);
        let kind = match prefix.to_ascii_uppercase().as_str() {
            "SURVEY" => Self::SurveyStart,
            "RECORD" => Self::RecordStart,
            "VARIABLE" => Self::QuestionStart,
            "POSITION" => Self::Location,
            "TYPE" => Self::Kind,
            "VALUES" => Self::ResponseStart,
            "END" => Self::End(value.clone()),
            "SIZE" => Self::Size(value.parse().map_err(BadLine::new)?),
            _ => prefix
                .parse()
                .map_err(|e| BadLine::new(format!("line: '{line}', error: {e}")))?,
        };
        Ok((kind, value))
    }
}

fn response_kind(value: &str) -> Result<Response, IoError> {
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
                .map_or(Ok(0), |v| v.parse().map_err(BadLine::new))?,
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
                (
                    0_u64,
                    Choice {
                        code: 0,
                        label: "FALSE".to_string(),
                        filter: None,
                        annotation: None
                    },
                ),
                (
                    1_u64,
                    Choice {
                        code: 1,
                        label: "TRUE".to_string(),
                        filter: None,
                        annotation: None
                    },
                ),
            ]),
        }),
        "QUANTITY" => Ok(Response::Quantity { min: 0.0, max: 0.0 }),
        "NOTE" => Ok(Response::Note(String::new())),
        _ => Err(BadLine::new(format!("Unknown response type: '{value}'")).into()),
    }
}

fn question_mut<'a>(
    survey: &'a mut Survey,
    index: Option<usize>,
    line: &str,
) -> Result<&'a mut Question, IoError> {
    let index = index
        .ok_or_else(|| IoError::internal(format!("Line outside a variable: '{line}'")))?;
    survey
        .record
        .content
        .get_mut(index)
        .ok_or_else(|| IoError::internal("Invalid current question"))
}

pub fn read_sss_classic<I, E>(lines: I) -> Result<Survey, IoError>
where
    I: IntoIterator<Item = Result<String, E>>,
    IoError: From<E>,
{
    let mut survey = Survey::empty();
    let mut current = None;
    let mut in_values = false;
    for raw_line in lines {
        let line = raw_line.map_err(IoError::from)?;
        let (kind, value) = SSSClassicLineType::parse_line(&line)?;
        match kind {
            SSSClassicLineType::Empty | SSSClassicLineType::SurveyStart | SSSClassicLineType::BOF => {}
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
                survey.record.content.push(Question::default());
                current = Some(survey.record.content.len() - 1);
                in_values = false;
            }
            SSSClassicLineType::End(end) => match end.to_ascii_uppercase().as_str() {
                "VALUES" => in_values = false,
                "VARIABLE" => {
                    current = None;
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
            },
            SSSClassicLineType::ResponseStart => {
                current.ok_or_else(|| IoError::internal("VALUES outside a variable"))?;
                in_values = true;
            }
            SSSClassicLineType::Name => {
                question_mut(&mut survey, current, &line)?.name = value
            }
            SSSClassicLineType::Filter => {
                question_mut(&mut survey, current, &line)?.filter = Some(value)
            }
            SSSClassicLineType::Label => {
                question_mut(&mut survey, current, &line)?.text = value
            }
            SSSClassicLineType::Location => {
                if !in_values {
                    question_mut(&mut survey, current, &line)?.location = Location {
                        from: range(&value, "TO")?.0,
                        to: range(&value, "TO")?.1,
                    };
                }
            }
            SSSClassicLineType::Size(size) => {
                if let Response::Character(length) =
                    &mut question_mut(&mut survey, current, &line)?.response
                {
                    *length = size;
                }
            }
            SSSClassicLineType::Kind => {
                let question = question_mut(&mut survey, current, &line)?;
                question.response = response_kind(&value)?;
                if matches!(question.response, Response::Note(_)) {
                    question.response = Response::Note(question.text.clone());
                }
            }
            SSSClassicLineType::IntRange {
                from,
                to,
                annotation,
            } => {
                let question = question_mut(&mut survey, current, &line)?;
                match &mut question.response {
                    Response::Single { item } | Response::Multiple { item,.. } => {
                        if from >= 0 {
                            let choice = Choice {code: from as u64, label: value, filter: None, annotation};
                            item.insert( choice.code, choice);
                        } else {
                            return Err(BadLine::new(
                        format!("Code index cannot be negative. Line received: '{}', question: {}", line, question.name)
                            ).into())
                        }
                    }
                    Response::Quantity {..} | Response::QuantitySpecial {..} => {
                        match annotation {
                            Some(content) => {
                                question.response = Response::QuantitySpecial {
                                    min: from as f64, max: to as f64, special: format!(r#""{}" {}"#, value, content)
                                };
                            }
                            None => {
                                question.response = Response::Quantity {
                                    min: from as f64, max: to as f64
                                };
                            }
                        }
                    }
                    _ => return Err(BadLine::new(
                        format!("Wrong question type received for line `{value}`. Expect: Single/Multiple/Quantity. Actual: {:?}", question.response)
                    ).into())
                }
            }
            SSSClassicLineType::FloatRange {
                from,
                to,
                annotation,
            } => {
                let question = question_mut(&mut survey, current, &line)?;
                match &mut question.response {
                    Response::Quantity {..} | Response::QuantitySpecial {..} => {
                        match annotation {
                            Some(content) => {
                                question.response = Response::QuantitySpecial {
                                    min: from, max: to, special: format!(r#""{}" {}"#, value, content)
                                };
                            }
                            None => {
                                question.response = Response::Quantity {
                                    min: from, max: to
                                };
                            }
                        }
                    }
                    _ => return Err(BadLine::new(
                        format!("Wrong question type received for line `{value}`. Expect: Quantity. Actual: {:?}", question.response)
                    ).into())
                }
            }

            SSSClassicLineType::Spread => {
                match &mut question_mut(&mut survey, current, &line)?.response {
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
        }
    }
    Ok(survey)
}

impl Survey {
    pub fn to_sss_classic(&self) -> String {
        let mut output = String::from("SSS\n");
        output.push_str(&self.meta.to_sss_classic());
        line(&mut output, "SURVEY");
        if let Some(title) = self.title.as_ref() {
            line(&mut output, format!("TITLE {}", quote(title)));
        }
        output.push_str(&self.record.to_sss_classic());
        line(&mut output, "END SURVEY");
        line(&mut output, "END SSS");
        output
    }
}

impl Meta {
    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        if let Some(value) = &self.version {
            line(&mut output, format!("VERSION {}", value));
        }
        for (name, value) in [
            ("DATE", &self.date),
            ("TIME", &self.time),
            ("ORIGIN", &self.origin),
            ("USER", &self.user),
        ] {
            if let Some(value) = value {
                line(&mut output, format!("{name} {}", quote(value)));
            }
        }
        for value in &self.note {
            line(&mut output, format!("NOTE {}", quote(value)));
        }
        for (name, value) in [("STANDARD", &self.standard), ("STYLE", &self.style)] {
            if let Some(value) = value {
                line(&mut output, format!("{name} {}", quote(value)));
            }
        }
        output
    }
}

impl Record {
    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        line(&mut output, format!("RECORD {}", self.id));
        for (index, question) in self.content.iter().enumerate() {
            line(&mut output, format!("VARIABLE {}", index + 1));
            output.push_str(&question.to_sss_classic());
            line(&mut output, "END VARIABLE");
        }
        line(&mut output, "END RECORD");
        output
    }
}

impl Question {
    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        line(&mut output, format!("NAME {}", quote(&self.name)));
        line(&mut output, format!("LABEL {}", quote(&self.text)));
        line(&mut output, self.location.to_sss_classic());
        output.push_str(&self.response.to_sss_classic());
        output
    }
}

impl Location {
    pub fn to_sss_classic(&self) -> String {
        if self.from == self.to {
            format!("POSITION {}", self.from)
        } else {
            format!("POSITION {} TO {}", self.from, self.to)
        }
    }
}

impl Response {
    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        match self {
            Self::Quantity { min, max } => {
                line(&mut output, "TYPE QUANTITY");
                line(&mut output, "VALUES");
                line(&mut output, format_range(*min, *max, "TO"));
                line(&mut output, "END VALUES");
            }
            Self::QuantitySpecial { min, max, special } => {
                line(&mut output, "TYPE QUANTITY");
                line(&mut output, "VALUES");
                let range_string = format_range(*min, *max, "TO");
                let range_string = format!("{} {}", range_string, special);
                line(&mut output, range_string);
                line(&mut output, "END VALUES");
            }
            Self::Character(size) => {
                line(&mut output, "TYPE CHARACTER");
                line(&mut output, self.size_line(*size));
            }
            Self::Note(value) => {
                line(&mut output, "TYPE NOTE");
                line(&mut output, format!("LABEL {}", quote(value)));
            }
            Self::Single { item } => {
                line(&mut output, "TYPE SINGLE");
                write_choices(&mut output, item);
            }
            Self::Multiple { item, spread } => {
                line(&mut output, "TYPE MULTIPLE");
                line(&mut output, spread.to_sss_classic());
                write_choices(&mut output, item);
            }
        }
        output
    }

    fn size_line(&self, size: usize) -> String {
        format!("SIZE {size}")
    }
}

fn write_choices(output: &mut String, choices: &BTreeMap<u64, Choice>) {
    line(output, "VALUES");
    for choice in choices.values() {
        line(output, choice.to_sss_classic());
    }
    line(output, "END VALUES");
}

fn format_range<T: std::fmt::Display>(from: T, to: T, delimiter: &str) -> String {
    if from.to_string() == to.to_string() {
        from.to_string()
    } else {
        format!("{from} {delimiter} {to}")
    }
}

fn parse_annotation_line(content: &str) -> Result<(&str, Option<String>), IoError> {
    let (text, annotation) = if content.ends_with("SPECIAL") {
        let text = content
            .strip_suffix("SPECIAL")
            .map_or(content, |trimmed| trimmed.trim());
        (text, Some("SPECIAL".to_string()))
    } else if content.ends_with("WITH") {
        let text = content
            .strip_suffix("WITH")
            .map_or(content, |trimmed| trimmed.trim());
        println!("{}", text);
        (text, Some("WITH".to_string()))
    } else {
        (content, None)
    };

    let text = text.trim();
    if text.is_empty() {
        return Ok(("", annotation));
    }

    let stripped = text.strip_circumfix('"', '"').ok_or(BadLine::new(format!(
        "Unable to parse line: '{}'",
        content
    )))?;
    Ok((stripped, annotation))
}

impl Choice {
    pub fn to_sss_classic(&self) -> String {
        let suffix = self
            .annotation
            .as_ref()
            .map_or_default(|s| format!(" {}", s));
        format!("{} {}{}", self.code, quote(&self.label), suffix)
    }
}

impl Spread {
    pub fn to_sss_classic(&self) -> String {
        format!("SPREAD {} OF {}", self.num_subfields, self.width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// used to trim whitespaces for string comparison
    fn normalize(s: &str) -> String {
        s.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn put_into_record(content: &str) -> impl Iterator<Item = Result<String, IoError>> {
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
        .map(|line| Ok::<String, IoError>(line.to_string()))
        .collect::<Vec<_>>()
        .into_iter()
    }

    fn create_example_single_code_iterator() -> impl Iterator<Item = Result<String, IoError>> {
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
        let survey = read_sss_classic(lines).unwrap();
        assert_eq!(survey.record.content.len(), 1);

        let question = survey.get_question("AGE").unwrap();
        assert_eq!(question.name, "AGE");
        assert_eq!(question.text, "<p>What is your <strong>age</strong>?</p>");
        assert_eq!(question.response.is_categorical(), true);
    }

    #[test]
    fn test_single_code_question_write() {
        let lines = create_example_single_code_iterator();
        let survey = read_sss_classic(lines).unwrap();

        let question = survey.get_question("AGE").unwrap();
        
        let sss_output = question.to_sss_classic().trim().to_string();
        assert!(sss_output.starts_with(r#"NAME "AGE""#));
        assert!(sss_output.contains(r#"LABEL "<p>What is your <strong>age</strong>?</p>""#));
        assert_eq!(
            normalize(&question.location.to_sss_classic()), 
            normalize(r#"POSITION 7"#)
        );
        assert_eq!(
            normalize(&question.response.to_sss_classic()), 
            normalize(r#"TYPE SINGLE
            VALUES
            1 "Under 18"
            2 "18 to 24"
            3 "25 to 34"
            4 "35 to 44"
            5 "45 to 54"
            6 "55 to 64"
            7 "65 or older"
            8 "Prefer not to say" SPECIAL
            END VALUES"#)
        );
        assert!(sss_output.ends_with(r#"END VALUES"#));
    }

    // todo! finish this unit tests
}
