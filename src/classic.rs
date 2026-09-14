use std::{collections::BTreeMap, fs::File, io::BufReader};
use strum::EnumString;
use crate::{error::{self, BadLine, IoError}, Choice, Location, NumericSpecial, Question, Response, Spread, Survey};

#[derive(Debug, PartialEq)]
pub enum SSSType { Classic, XML }

impl SSSType {
    pub fn parse(first_line: &str) -> Result<Self, BadLine> {
        match first_line.chars().next().unwrap_or_default() {
            'S' => Ok(Self::Classic), '<' => Ok(Self::XML),
            _ => Err(BadLine::new(format!("Unable to identify sss version from the first line: '{first_line}'"))),
        }
    }
}

#[derive(Debug)]
pub enum LineKind { Classic(SSSClassicLineType) }

#[derive(Debug, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum SSSClassicLineType {
    #[strum(serialize = "")] Empty,
    Version, Date, Time, Origin, User, Note, Standard, Style, Label,
    #[strum(serialize = "SURVEY")] SurveyStart,
    Title,
    #[strum(serialize = "RECORD")] RecordStart,
    #[strum(serialize = "VARIABLE")] QuestionStart,
    Name,
    #[strum(serialize = "POSITION")] Location,
    #[strum(serialize = "TYPE")] Kind,
    #[strum(serialize = "VALUES")] ResponseStart,
    End(String), Choice(u64),
    Range { from: f64, to: f64 },
    Special { value: f64, label: String, filter: Option<String> },
    Spread, Size(usize),
}

fn unquote(value: &str) -> String {
    value.strip_prefix('"').and_then(|value| value.strip_suffix('"')).unwrap_or(value).to_string()
}

fn quoted(value: &str) -> Result<(String, &str), IoError> {
    let end = value[1..].find('"').ok_or_else(|| BadLine::new("Unclosed quoted value"))? + 1;
    Ok((value[1..end].to_string(), value[end + 1..].trim()))
}

fn range<T>(value: &str, delimiter: &str) -> Result<(T, T), IoError>
where T: std::str::FromStr + Clone, T::Err: std::fmt::Display {
    let parts: Vec<_> = value.split_whitespace().collect();
    if parts.len() == 1 { let item: T = parts[0].parse().map_err(|e| BadLine::new(e))?; return Ok((item.clone(), item)); }
    if parts.len() != 3 || !parts[1].eq_ignore_ascii_case(delimiter) { return Err(BadLine::new(format!("Invalid range: '{value}'")).into()); }
    Ok((parts[0].parse().map_err(|e| BadLine::new(e))?, parts[2].parse().map_err(|e| BadLine::new(e))?))
}

fn range_without_annotation(value: &str) -> &str {
    [" WITH", " SPECIAL"].iter().find_map(|suffix| {
        value.len().checked_sub(suffix.len()).filter(|&start| value[start..].eq_ignore_ascii_case(suffix)).map(|start| &value[..start])
    }).unwrap_or(value)
}

impl SSSClassicLineType {
    pub fn parse_line(line: &str) -> Result<(Self, String), IoError> {
        let line = line.trim();
        if line.is_empty() { return Ok((Self::Empty, String::new())); }
        let mut fields = line.splitn(2, char::is_whitespace);
        let prefix = fields.next().unwrap_or_default();
        let value = fields.next().unwrap_or_default().trim();
        if prefix.starts_with(&['0','1','2','3','4','5','6','7','8','9','-']) {
            let (label, suffix) = if value.starts_with('"') { quoted(value)? } else { (String::new(), "") };
            if !label.is_empty() {
                if let Ok(code) = prefix.parse::<u64>() { return Ok((Self::Choice(code), label)); }
                let filter = suffix.eq_ignore_ascii_case("SPECIAL").then(|| "SPECIAL".to_string());
                if !suffix.is_empty() && filter.is_none() { return Err(BadLine::new(format!("Invalid numeric special value: '{line}'")).into()); }
                return Ok((Self::Special { value: prefix.parse().map_err(|e| BadLine::new(e))?, label, filter }, String::new()));
            }
            let (from, to) = range(range_without_annotation(line), "TO")?;
            return Ok((Self::Range { from, to }, String::new()));
        }
        let value = unquote(value);
        let kind = match prefix.to_ascii_uppercase().as_str() {
            "SURVEY" => Self::SurveyStart, "RECORD" => Self::RecordStart, "VARIABLE" => Self::QuestionStart,
            "POSITION" => Self::Location, "TYPE" => Self::Kind, "VALUES" => Self::ResponseStart,
            "END" => Self::End(value.clone()), "SIZE" => Self::Size(value.parse().map_err(|e| BadLine::new(e))?),
            _ => prefix.parse().map_err(|e| BadLine::new(format!("line: '{line}', error: {e}")))?,
        };
        Ok((kind, value))
    }
}

fn response_kind(value: &str) -> Result<Response, IoError> {
    match value.split_whitespace().next().unwrap_or_default().to_ascii_uppercase().as_str() {
        "CHARACTER" => Ok(Response::Character(value.split_whitespace().nth(1).map_or(Ok(0), |v| v.parse().map_err(|e| BadLine::new(e)))?)),
        "SINGLE" | "LOGIC" | "LOGICAL" => Ok(Response::Single { item: BTreeMap::new() }),
        "MULTIPLE" => Ok(Response::Multiple { item: BTreeMap::new(), spread: Spread::default() }),
        "QUANTITY" => Ok(Response::Quantity { min: 0.0, max: 0.0 }),
        "NOTE" => Ok(Response::Note(String::new())),
        _ => Err(BadLine::new(format!("Unknown response type: '{value}'")).into()),
    }
}

fn question_mut<'a>(survey: &'a mut Survey, index: Option<usize>, line: &str) -> Result<&'a mut Question, IoError> {
    let index = index.ok_or_else(|| IoError::internal(format!("Line outside a variable: '{line}'")))?;
    survey.record.content.get_mut(index).ok_or_else(|| IoError::internal("Invalid current question"))
}

pub fn read_sss_classic(lines: std::io::Lines<BufReader<File>>) -> Result<Survey, IoError> {
    let mut survey = Survey::empty();
    let mut current = None;
    let mut in_values = false;
    for raw_line in lines {
        let line = raw_line.map_err(IoError::from)?;
        let (kind, value) = SSSClassicLineType::parse_line(&line)?;
        match kind {
            SSSClassicLineType::Empty | SSSClassicLineType::SurveyStart => {},
            SSSClassicLineType::Version => survey.meta.version = Some(value), SSSClassicLineType::Date => survey.meta.date = Some(value),
            SSSClassicLineType::Time => survey.meta.time = Some(value), SSSClassicLineType::Origin => survey.meta.origin = Some(value),
            SSSClassicLineType::User => survey.meta.user = Some(value), SSSClassicLineType::Note => survey.meta.note.push(value),
            SSSClassicLineType::Standard => survey.meta.standard = Some(value), SSSClassicLineType::Style => survey.meta.style = Some(value),
            SSSClassicLineType::Title => survey.title = value, SSSClassicLineType::RecordStart => survey.record.id = value,
            SSSClassicLineType::QuestionStart => { survey.record.content.push(Question::default()); current = Some(survey.record.content.len() - 1); in_values = false; },
            SSSClassicLineType::End(end) => match end.to_ascii_uppercase().as_str() { "VALUES" => in_values = false, "VARIABLE" => { current = None; in_values = false; }, "RECORD" | "SURVEY" | "SSS" => break, _ => return Err(error::UnexpectedLineKind::new(LineKind::Classic(SSSClassicLineType::End(end)), line).into()) },
            SSSClassicLineType::ResponseStart => { current.ok_or_else(|| IoError::internal("VALUES outside a variable"))?; in_values = true; },
            SSSClassicLineType::Name => question_mut(&mut survey, current, &line)?.name = value,
            SSSClassicLineType::Label => question_mut(&mut survey, current, &line)?.text = value,
            SSSClassicLineType::Location => if !in_values { question_mut(&mut survey, current, &line)?.location = Location { from: range(&value, "TO")?.0, to: range(&value, "TO")?.1 }; },
            SSSClassicLineType::Size(size) => if let Response::Character(length) = &mut question_mut(&mut survey, current, &line)?.response { *length = size; },
            SSSClassicLineType::Kind => question_mut(&mut survey, current, &line)?.response = response_kind(&value)?,
            SSSClassicLineType::Choice(code) => match &mut question_mut(&mut survey, current, &line)?.response { Response::Single { item } | Response::Multiple { item, .. } => { item.insert(code, Choice::new(code, value, line.to_ascii_uppercase().ends_with(" SPECIAL").then(|| "SPECIAL"))); }, _ => return Err(BadLine::new("Choices require SINGLE or MULTIPLE").into()) },
            SSSClassicLineType::Spread => match &mut question_mut(&mut survey, current, &line)?.response { Response::Multiple { spread, .. } => { let (num_subfields, width) = range(&value, "OF")?; *spread = Spread { num_subfields, width }; }, _ => return Err(BadLine::new("SPREAD requires MULTIPLE").into()) },
            SSSClassicLineType::Range { from, to } => match &mut question_mut(&mut survey, current, &line)?.response { Response::Quantity { min, max } | Response::QuantitySpecial { min, max, .. } => { *min = from; *max = to; }, _ => return Err(BadLine::new("Range requires QUANTITY").into()) },
            SSSClassicLineType::Special { value, label, filter } => { let question = question_mut(&mut survey, current, &line)?; let special = NumericSpecial { value, label, filter }; question.response = match std::mem::replace(&mut question.response, Response::Quantity { min: 0.0, max: 0.0 }) { Response::Quantity { min, max } => Response::QuantitySpecial { min, max, special: vec![special] }, Response::QuantitySpecial { min, max, special: mut values } => { values.push(special); Response::QuantitySpecial { min, max, special: values } }, response => response }; }
        }
    }
    Ok(survey)
}
