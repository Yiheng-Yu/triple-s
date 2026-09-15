use std::{collections::BTreeMap, fmt::Write, fs::File, io::BufReader};

use strum::EnumString;

use crate::{
    Choice,
    Location,
    Meta,
    NumericSpecial,
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
    Choice(u64),
    Range {
        from: f64,
        to: f64,
    },
    Special {
        value: f64,
        label: String,
        filter: Option<String>,
    },
    Spread,
    Size(usize),
}

fn unquote(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
        .replace("{22}", "\"")
        .to_string()
}

fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "{22}"))
}

fn line(output: &mut String, content: impl std::fmt::Display) {
    writeln!(output, "{content}").expect("writing to a String cannot fail");
}

fn quoted(value: &str) -> Result<(String, &str), IoError> {
    let end = value[1..]
        .find('"')
        .ok_or_else(|| BadLine::new("Unclosed quoted value"))?
        + 1;
    Ok((value[1..end].to_string(), value[end + 1..].trim()))
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

fn range_without_annotation(value: &str) -> &str {
    [" WITH", " SPECIAL"]
        .iter()
        .find_map(|suffix| {
            value
                .len()
                .checked_sub(suffix.len())
                .filter(|&start| value[start..].eq_ignore_ascii_case(suffix))
                .map(|start| &value[..start])
        })
        .unwrap_or(value)
}

impl SSSClassicLineType {
    pub fn parse_line(line: &str) -> Result<(Self, String), IoError> {
        let line = line.trim();
        if line.is_empty() {
            return Ok((Self::Empty, String::new()));
        }
        let mut fields = line.splitn(2, char::is_whitespace);
        let prefix = fields.next().unwrap_or_default();
        let value = fields.next().unwrap_or_default().trim();
        if prefix.starts_with(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-']) {
            let (label, suffix) = if value.starts_with('"') {
                quoted(value)?
            } else {
                (String::new(), "")
            };
            if !label.is_empty() {
                if let Ok(code) = prefix.parse::<u64>() {
                    return Ok((Self::Choice(code), label));
                }
                let filter = suffix
                    .eq_ignore_ascii_case("SPECIAL")
                    .then(|| "SPECIAL".to_string());
                if !suffix.is_empty() && filter.is_none() {
                    return Err(BadLine::new(format!(
                        "Invalid numeric special value: '{line}'"
                    ))
                    .into());
                }
                return Ok((
                    Self::Special {
                        value: prefix.parse().map_err(BadLine::new)?,
                        label,
                        filter,
                    },
                    String::new(),
                ));
            }
            let (from, to) = range(range_without_annotation(line), "TO")?;
            return Ok((Self::Range { from, to }, String::new()));
        }
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
                    },
                ),
                (
                    1_u64,
                    Choice {
                        code: 1,
                        label: "TRUE".to_string(),
                        filter: None,
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

pub fn read_sss_classic(lines: std::io::Lines<BufReader<File>>) -> Result<Survey, IoError> {
    let mut survey = Survey::empty();
    let mut current = None;
    let mut in_values = false;
    for raw_line in lines {
        let line = raw_line.map_err(IoError::from)?;
        let (kind, value) = SSSClassicLineType::parse_line(&line)?;
        match kind {
            SSSClassicLineType::Empty | SSSClassicLineType::SurveyStart => {}
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
            SSSClassicLineType::Choice(code) => {
                match &mut question_mut(&mut survey, current, &line)?.response {
                    Response::Single { item } | Response::Multiple { item, .. } => {
                        item.insert(
                            code,
                            Choice::new(
                                code,
                                value,
                                line.to_ascii_uppercase()
                                    .ends_with(" SPECIAL")
                                    .then_some("SPECIAL"),
                            ),
                        );
                    }
                    _ => return Err(BadLine::new("Choices require SINGLE or MULTIPLE").into()),
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
            SSSClassicLineType::Range { from, to } => {
                match &mut question_mut(&mut survey, current, &line)?.response {
                    Response::Quantity { min, max }
                    | Response::QuantitySpecial { min, max, .. } => {
                        *min = from;
                        *max = to;
                    }
                    _ => return Err(BadLine::new("Range requires QUANTITY").into()),
                }
            }
            SSSClassicLineType::Special {
                value,
                label,
                filter,
            } => {
                let question = question_mut(&mut survey, current, &line)?;
                let special = NumericSpecial {
                    value,
                    label,
                    filter,
                };
                question.response = match std::mem::replace(
                    &mut question.response,
                    Response::Quantity { min: 0.0, max: 0.0 },
                ) {
                    Response::Quantity { min, max } => Response::QuantitySpecial {
                        min,
                        max,
                        special: vec![special],
                    },
                    Response::QuantitySpecial {
                        min,
                        max,
                        special: mut values,
                    } => {
                        values.push(special);
                        Response::QuantitySpecial {
                            min,
                            max,
                            special: values,
                        }
                    }
                    response => response,
                };
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
            line(&mut output, format!("VERSION {}", quote(value)));
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
        if let Some(filter) = &self.filter {
            line(&mut output, format!("FILTER {}", quote(filter)));
        }
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
                line(&mut output, format_range(*min, *max, "TO"));
                for value in special {
                    line(&mut output, value.to_sss_classic());
                }
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

impl Choice {
    pub fn to_sss_classic(&self) -> String {
        let suffix = self
            .filter
            .as_deref()
            .filter(|value| value.eq_ignore_ascii_case("SPECIAL"))
            .map_or(String::new(), |value| format!(" {value}"));
        format!("{} {}{}", self.code, quote(&self.label), suffix)
    }
}

impl NumericSpecial {
    pub fn to_sss_classic(&self) -> String {
        let suffix = self
            .filter
            .as_deref()
            .filter(|value| value.eq_ignore_ascii_case("SPECIAL"))
            .map_or(String::new(), |value| format!(" {value}"));
        format!("{} {}{}", self.value, quote(&self.label), suffix)
    }
}

impl Spread {
    pub fn to_sss_classic(&self) -> String {
        format!("SPREAD {} OF {}", self.num_subfields, self.width)
    }

    #[cfg(test)]
    fn sample() -> Self {
        Self {
            num_subfields: 3,
            width: 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_all_classic_members() {
        let choice = Choice::new(1, "A \"choice\"", Some("SPECIAL"));
        let special = NumericSpecial {
            value: 99.0,
            label: "Unknown".into(),
            filter: Some("SPECIAL".into()),
        };
        let survey = Survey {
            title: Some("Title".into()),
            meta: Meta {
                version: Some("1.1".into()),
                date: Some("today".into()),
                time: Some("now".into()),
                origin: Some("origin".into()),
                user: Some("user".into()),
                style: Some("style".into()),
                note: vec!["note".into()],
                standard: Some("NAMES".into()),
            },
            record: Record {
                id: "A".into(),
                content: vec![
                    Question {
                        name: "Q1".into(),
                        text: "Question".into(),
                        filter: None,
                        location: Location { from: 1, to: 2 },
                        response: Response::Single {
                            item: [(1, choice)].into_iter().collect(),
                        },
                    },
                    Question {
                        name: "Q2".into(),
                        text: "Note".into(),
                        filter: None,
                        location: Location::default(),
                        response: Response::QuantitySpecial {
                            min: 0.0,
                            max: 10.0,
                            special: vec![special],
                        },
                    },
                ],
            },
        };
        let text = survey.to_sss_classic();
        assert!(text.contains("1 \"A {22}choice{22}\" SPECIAL"));
        assert!(text.contains("99 \"Unknown\" SPECIAL"));
        assert!(text.contains("POSITION 1 TO 2"));
        assert!(text.contains("END SSS"));
    }

    #[test]
    fn serializes_response_variants() {
        assert!(Response::Character(10).to_sss_classic().contains("SIZE 10"));
        assert!(
            Response::Note("memo".into())
                .to_sss_classic()
                .contains("TYPE NOTE")
        );
        assert!(
            Response::Multiple {
                item: BTreeMap::new(),
                spread: Spread::sample(),
            }
            .to_sss_classic()
            .contains("SPREAD 3 OF 4")
        );
    }

    #[test]
    fn serializes_each_classic_model_member() {
        assert_eq!(
            Location { from: 2, to: 2 }.to_sss_classic(),
            "POSITION 2"
        );
        assert_eq!(Spread::sample().to_sss_classic(), "SPREAD 3 OF 4");
        assert_eq!(
            Choice::new(4, "Yes", None::<String>).to_sss_classic(),
            "4 \"Yes\""
        );
        assert_eq!(
            NumericSpecial {
                value: -1.0,
                label: "Missing".into(),
                filter: Some("SPECIAL".into()),
            }
            .to_sss_classic(),
            "-1 \"Missing\" SPECIAL"
        );
        assert!(Meta::default().to_sss_classic().is_empty());
        assert!(Record::default().to_sss_classic().contains("END RECORD"));
        assert!(Question::default().to_sss_classic().contains("NAME \"\""));
        assert!(Response::default().to_sss_classic().contains("TYPE SINGLE"));
        assert!(Survey::empty().to_sss_classic().contains("END SURVEY"));
    }
}
