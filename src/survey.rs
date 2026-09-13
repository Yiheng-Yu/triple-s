// survey entity
use std::{collections::BTreeMap, fs::File, io::BufReader};

use pyo3::prelude::*;
use strum::EnumString;

use crate::error::{self, BadLine, IoError};

#[derive(Debug, Clone, PartialEq)]
#[pyclass]
/// A Triple-S survey containing metadata, a title, and its record.
pub struct Survey {
    #[pyo3(get, set)]
    pub title: String,
    #[pyo3(get, set)]
    pub meta: Meta,
    #[pyo3(get, set)]
    pub record: Record,
}

impl Survey {
    pub fn empty() -> Self {
        Self {
            meta: Meta::default(),
            title: "".into(),
            record: Record::default(),
        }
    }
}

#[pymethods]
impl Survey {
    #[new]
    #[pyo3(signature = (title, meta=None, record=None))]
    fn __new__(title: &str, meta: Option<Meta>, record: Option<Record>) -> Self {
        Self {
            title: title.to_string(),
            meta: meta.unwrap_or_default(),
            record: record.unwrap_or_default(),
        }
    }

    /// Initialize a survey from its required metadata and record.
    #[pyo3(signature = (title, meta=None, record=None))]
    fn __init__(
        slf: &Bound<'_, Self>,
        title: &str,
        meta: Option<Meta>,
        record: Option<Record>,
    ) {
        *slf.borrow_mut() = Self {
            title: title.to_string(),
            meta: meta.unwrap_or_default(),
            record: record.unwrap_or_default(),
        };
    }

    fn __repr__(&self) -> String {
        format!(
            "Survey(title={:?}, meta={}, record={})",
            self.title,
            self.meta.__repr__(),
            self.record.__repr__()
        )
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }

}

#[derive(Debug, Clone, Default, PartialEq)]
#[pyclass(from_py_object)]
/// Optional metadata associated with a Triple-S survey.
pub struct Meta {
    #[pyo3(get, set)]
    pub version: Option<String>,
    #[pyo3(get, set)]
    pub date: Option<String>,
    #[pyo3(get, set)]
    pub time: Option<String>,
    #[pyo3(get, set)]
    pub origin: Option<String>,
    #[pyo3(get, set)]
    pub user: Option<String>,
    #[pyo3(get, set)]
    pub style: Option<String>,
    #[pyo3(get, set)]
    pub note: Option<String>,
    // Optional between VERSION and SURVEY. This statement is used to indicate that some aspect of the
    // SSS definition meets a defined standard that may reduce the amount of checking required by an
    // import program
    #[pyo3(get, set)]
    pub standard: Option<String>,
}

#[pymethods]
impl Meta {
    #[new]
    #[pyo3(signature = (*, version = None, date = None, time = None, origin = None, user = None, style = None, note = None, standard = None))]
    fn __new__(
        version: Option<String>,
        date: Option<String>,
        time: Option<String>,
        origin: Option<String>,
        user: Option<String>,
        style: Option<String>,
        note: Option<String>,
        standard: Option<String>,
    ) -> Self {
        Self {
            version,
            date,
            time,
            origin,
            user,
            style,
            note,
            standard,
        }
    }

    /// Initialize optional survey metadata.
    #[pyo3(signature = (*, version = None, date = None, time = None, origin = None, user = None, style = None, note = None, standard = None))]
    fn __init__(
        slf: &Bound<'_, Self>,
        version: Option<String>,
        date: Option<String>,
        time: Option<String>,
        origin: Option<String>,
        user: Option<String>,
        style: Option<String>,
        note: Option<String>,
        standard: Option<String>,
    ) {
        *slf.borrow_mut() = Self {
            version,
            date,
            time,
            origin,
            user,
            style,
            note,
            standard,
        };
    }

    fn __repr__(&self) -> String {
        format!(
            "Meta(version={:?}, date={:?}, time={:?}, origin={:?}, user={:?}, style={:?}, note={:?}, standard={:?})",
            self.version,
            self.date,
            self.time,
            self.origin,
            self.user,
            self.style,
            self.note,
            self.standard
        )
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[pyclass(from_py_object)]
/// A survey record containing an identifier and ordered questions.
pub struct Record {
    #[pyo3(get, set)]
    pub id: String,
    #[pyo3(get, set)]
    pub content: Vec<Question>,
}

#[pymethods]
impl Record {
    #[new]
    #[pyo3(signature = (content, *, id = ""))]
    fn __new__(content: Vec<Question>, id: &str) -> Self {
        Self {
            id: id.to_owned(),
            content,
        }
    }

    /// Initialize a record with required questions and an optional identifier.
    #[pyo3(signature = (content, *, id = ""))]
    fn __init__(slf: &Bound<'_, Self>, content: Vec<Question>, id: &str) {
        *slf.borrow_mut() = Self {
            id: id.to_owned(),
            content,
        };
    }

    fn __repr__(&self) -> String {
        const MAX_QUESTIONS: usize = 5;
        let content = if self.content.len() > MAX_QUESTIONS {
            let questions = self.content[..MAX_QUESTIONS]
                .iter()
                .map(Question::__repr__)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}, ...]", questions)
        } else {
            let questions = self
                .content
                .iter()
                .map(Question::__repr__)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", questions)
        };
        format!("Record(id={:?}, content={})", self.id, content)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
#[pyclass(from_py_object)]
/// A survey question with its location and response definition.
pub struct Question {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub title: String,
    #[pyo3(get, set)]
    pub location: Location,
    #[pyo3(get, set)]
    pub response: Response,
}

#[pymethods]
impl Question {
    #[new]
    #[pyo3(signature = (location, response, *, name = "", title = ""))]
    fn __new__(location: Location, response: Response, name: &str, title: &str) -> Self {
        Self {
            name: name.to_owned(),
            title: title.to_owned(),
            location,
            response,
        }
    }

    /// Initialize a question with its required location and response.
    #[pyo3(signature = (location, response, *, name = "", title = ""))]
    fn __init__(
        slf: &Bound<'_, Self>,
        location: Location,
        response: Response,
        name: &str,
        title: &str,
    ) {
        *slf.borrow_mut() = Self {
            name: name.to_owned(),
            title: title.to_owned(),
            location,
            response,
        };
    }

    fn __repr__(&self) -> String {
        format!(
            r#"Question(name="{}", title="{}", location={}, response={:?})"#,
            self.name, self.title, self.location.__repr__(), self.response.__repr__()
        )
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug, Clone, PartialEq)]
#[pyclass(from_py_object)]
/// A response definition for a survey question.
pub enum Response {
    Quantity {
        min: f64,
        max: f64,
    },
    Character(usize),
    Note(String),
    Single {
        item: BTreeMap<u64, Choice>,
    },
    Multiple {
        item: BTreeMap<u64, Choice>,
        spread: Spread,
    },
}

impl std::default::Default for Response {
    fn default() -> Self {
        Self::Single {
            item: BTreeMap::new(),
        }
    }
}

impl Response {
    pub fn type_name(&self) -> &str {
        match self {
            Self::Character(_) => "Character",
            Self::Note(_) => "Note",
            Self::Quantity { .. } => "Quantity",
            Self::Single { .. } => "Single",
            Self::Multiple { .. } => "Multiple",
        }
    }
}

#[pymethods]
impl Response {
    fn __repr__(&self) -> String {
        match self {
            Self::Quantity { min, max } => {
                format!("Response.Quantity(min={}, max={})", min, max)
            }
            Self::Character(length) => format!("Response.Character({})", length),
            Self::Note(note) => format!("Response.Note({:?})", note),
            Self::Single { item } => {
                format!("Response.Single(item={})", format_choices(item))
            }
            Self::Multiple { item, spread } => format!(
                "Response.Multiple(item={}, spread={:?})",
                format_choices(item),
                spread
            ),
        }
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
}

fn format_choices(item: &BTreeMap<u64, Choice>) -> String {
    const MAX_CHOICES: usize = 5;
    if item.len() > MAX_CHOICES {
        let choices = item
            .values()
            .take(MAX_CHOICES)
            .map(Choice::__repr__)
            .collect::<Vec<_>>()
            .join(", ");
        format!("({}, ...)", choices)
    } else {
        let choices = item
            .values()
            .map(Choice::__repr__)
            .collect::<Vec<_>>()
            .join(", ");
        format!("({})", choices)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[pyclass(from_py_object)]
/// A coded answer choice.
pub struct Choice {
    #[pyo3(get, set)]
    pub code: u64,
    #[pyo3(get, set)]
    pub label: String,
    #[pyo3(get, set)]
    pub filter: Option<String>,
}

impl Choice {
    pub fn new<T>(code: u64, label: impl Into<String>, filter: Option<T>) -> Self
    where
        T: Into<String>,
    {
        Self {
            code,
            label: label.into(),
            filter: filter.map(|val| val.into()),
        }
    }
}

#[pymethods]
impl Choice {
    #[new]
    #[pyo3(signature = (code, *, label = "", filter = None))]
    fn __new__(code: u64, label: &str, filter: Option<String>) -> Self {
        Self {
            code,
            label: label.to_owned(),
            filter,
        }
    }

    /// Initialize a coded answer choice.
    #[pyo3(signature = (code, *, label = "", filter = None))]
    fn __init__(slf: &Bound<'_, Self>, code: u64, label: &str, filter: Option<String>) {
        *slf.borrow_mut() = Self {
            code,
            label: label.to_owned(),
            filter,
        };
    }

    fn __repr__(&self) -> String {
        format!(
            "Choice(code={}, label={:?}, filter={:?})",
            self.code, self.label, self.filter
        )
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
}

fn parse_classic_response_kind(line: &str) -> Result<Response, IoError> {
    let line = line.trim();
    let kind = line
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    let instance = match kind.as_str() {
        "CHARACTER" => {
            let length = line.split_whitespace().nth(1).map_or(Ok(0), |length| {
                length
                    .parse::<usize>()
                    .map_err(|err| IoError::from(BadLine::new(err)))
            })?;
            Response::Character(length)
        }
        "SINGLE" => Response::Single {
            item: BTreeMap::new(),
        },
        "MULTIPLE" => Response::Multiple {
            item: BTreeMap::new(),
            spread: Spread::default(),
        },
        "LOGIC" | "LOGICAL" => Response::Single {
            item: BTreeMap::new(),
        },
        "QUANTITY" => Response::Quantity { min: 0.0, max: 0.0 },
        "NOTE" => Response::Note("".into()),
        _ => {
            return Err(BadLine::new(format!(
                "Unable to identify variable response type: '{}'",
                line
            ))
            .into());
        }
    };
    Ok(instance)
}

#[derive(Debug, Clone, PartialEq)]
#[pyclass(from_py_object)]
/// A one-based inclusive field position in a data record.
pub struct Location {
    #[pyo3(get, set)]
    pub from: u64,
    #[pyo3(get, set)]
    pub to: u64,
}

impl std::default::Default for Location {
    fn default() -> Self {
        Location { from: 1, to: 1 }
    }
}

impl Location {
    pub fn parse_line(line: &str) -> Result<Self, IoError> {
        let (from, to) = parse_range_string::<u64>(line, "to")?;
        Ok(Self { from: from, to: to })
    }
}

#[pymethods]
impl Location {
    #[new]
    fn __new__(from: u64, to: u64) -> Self {
        Self { from, to }
    }

    /// Initialize a field position.
    fn __init__(slf: &Bound<'_, Self>, from: u64, to: u64) {
        *slf.borrow_mut() = Self { from, to };
    }

    fn __repr__(&self) -> String {
        format!("Location(from={}, to={})", self.from, self.to)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
}

/// Official documentation:
/// (from https://triple-s.org/wp-content/uploads/Triple-S-XML-3.0.pdf)
/// The num_subfields attribute must be a positive integer, and denotes the
/// number of subfields within the overall field that is defined by the <position>
/// element. The subfield_width is also a positive integer and denotes the
/// width of each subfield. For fixed Data Files, the <position> element must
/// define a width of at least (num_subfields * subfield_width). The
/// subfield_width must be large enough to hold the largest category value
/// specified for the multiple.
///  For example: <spread subfields="5" width="3"/>
/// indicates 5 subfields each of width 3 characters
/// The width attribute may be omitted when used in conjunction with fixed Data
/// Files if the num_subfields exactly fills the area defined by the <position>
/// element. In this case the subfield_width is determined by dividing the
/// width derived from the <position> element by num_subfields. Note that
/// for csv Data Files the width attribute must always be specified as it cannot
/// be determined from the overall width defined in the <position> element.
#[derive(Debug, Clone, PartialEq)]
#[pyclass(from_py_object)]
/// The number and width of subfields used by a multiple response.
pub struct Spread {
    #[pyo3(get, set)]
    pub num_subfields: u64,
    #[pyo3(get, set)]
    pub width: u64,
}

impl std::default::Default for Spread {
    fn default() -> Self {
        Spread {
            num_subfields: 999,
            width: 999,
        }
    }
}

impl Spread {
    pub fn parse_line(line: &str) -> Result<Self, IoError> {
        let (from, to) = parse_range_string::<u64>(line, "of")?;
        Ok(Self {
            num_subfields: from,
            width: to,
        })
    }
}

#[pymethods]
impl Spread {
    #[new]
    fn __new__(num_subfields: u64, width: u64) -> Self {
        Self {
            num_subfields,
            width,
        }
    }

    /// Initialize a multiple-response spread.
    fn __init__(slf: &Bound<'_, Self>, num_subfields: u64, width: u64) {
        *slf.borrow_mut() = Self {
            num_subfields,
            width,
        };
    }

    fn __repr__(&self) -> String {
        format!(
            "Spread(num_subfields={}, width={})",
            self.num_subfields, self.width
        )
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
}

fn parse_range_string<T>(line: &str, delimiter: &str) -> Result<(T, T), IoError>
where
    T: std::str::FromStr + Clone,
    T::Err: std::fmt::Display,
{
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() || (parts.len() != 1 && parts.len() != 3) {
        return Err(BadLine::new(format!("Invalid range: '{}'", line)).into());
    }
    let from = parts[0]
        .parse::<T>()
        .map_err(|err| IoError::from(BadLine::new(err)))?;
    if parts.len() == 1 {
        return Ok((from.clone(), from));
    }
    if !parts[1].eq_ignore_ascii_case(delimiter) {
        return Err(BadLine::new(format!("Invalid range: '{}'", line)).into());
    }
    let to = parts[2]
        .parse::<T>()
        .map_err(|err| IoError::from(BadLine::new(err)))?;
    Ok((from, to))
}

// ===================== parse sss 1.0 =====================
#[track_caller]
pub fn read_sss_classic(lines: std::io::Lines<BufReader<File>>) -> Result<Survey, IoError> {
    let mut survey = Survey::empty();
    let mut current_question: Option<usize> = None;
    let mut in_values = false;

    for line in lines {
        let line = line.map_err(IoError::from)?;
        let (line_type, value) = SSSClassicLineType::parse_line(&line)?;
        match line_type {
            SSSClassicLineType::Empty => {}
            SSSClassicLineType::SurveyStart => {}
            SSSClassicLineType::Date => survey.meta.date = Some(value),
            SSSClassicLineType::Time => survey.meta.time = Some(value),
            SSSClassicLineType::Origin => survey.meta.origin = Some(value),
            SSSClassicLineType::User => survey.meta.user = Some(value),
            SSSClassicLineType::Version => survey.meta.version = Some(value),
            SSSClassicLineType::Standard => survey.meta.standard = Some(value),
            SSSClassicLineType::Note => survey.meta.note = Some(value),
            SSSClassicLineType::Style => survey.meta.style = Some(value),
            SSSClassicLineType::Title => survey.title = value,
            SSSClassicLineType::RecordStart => survey.record.id = value,
            SSSClassicLineType::QuestionStart => {
                survey.record.content.push(Question::default());
                current_question = Some(survey.record.content.len() - 1);
                in_values = false;
            }
            SSSClassicLineType::End(end_kind) => {
                match end_kind.to_ascii_uppercase().as_str() {
                    "VALUES" => in_values = false,
                    "VARIABLE" => {
                        current_question = None;
                        in_values = false;
                    }
                    "RECORD" | "SURVEY" | "SSS" => break,
                    _ => {
                        return Err(unexpected_classic_line(
                            &line,
                            SSSClassicLineType::End(end_kind),
                        ));
                    }
                }
            }
            SSSClassicLineType::ResponseStart => {
                require_question(current_question, &line)?;
                in_values = true;
            }
            SSSClassicLineType::Name => {
                let question = question_mut(&mut survey, current_question, &line)?;
                question.name = value;
            }
            SSSClassicLineType::Label => {
                let question = question_mut(&mut survey, current_question, &line)?;
                question.title = value;
            }
            SSSClassicLineType::Location => {
                if !in_values {
                    let question = question_mut(&mut survey, current_question, &line)?;
                    question.location = Location::parse_line(&value)?;
                }
            }
            SSSClassicLineType::Size(size) => {
                let question = question_mut(&mut survey, current_question, &line)?;
                if let Response::Character(length) = &mut question.response {
                    *length = size;
                }
            }
            SSSClassicLineType::Kind => {
                let question = question_mut(&mut survey, current_question, &line)?;
                question.response = parse_classic_response_kind(&value)?;
            }
            SSSClassicLineType::Choice(number) => {
                let question = question_mut(&mut survey, current_question, &line)?;
                match &mut question.response {
                    Response::Single { item } | Response::Multiple { item, .. } => {
                        let filter = line
                            .to_ascii_uppercase()
                            .ends_with(" SPECIAL")
                            .then(|| "SPECIAL".to_string());
                        item.insert(number, Choice::new(number, value, filter));
                    }
                    response => {
                        return Err(BadLine::new(format!(
                            "Question '{}' cannot contain choices for response type {}",
                            question.name,
                            response.type_name()
                        ))
                        .into());
                    }
                }
            }
            SSSClassicLineType::Spread => {
                let question = question_mut(&mut survey, current_question, &line)?;
                match &mut question.response {
                    Response::Multiple { spread, .. } => *spread = Spread::parse_line(&value)?,
                    response => {
                        return Err(BadLine::new(format!(
                            "Question '{}' cannot contain a spread for response type {}",
                            question.name,
                            response.type_name()
                        ))
                        .into());
                    }
                }
            }
            SSSClassicLineType::Range { from, to } => {
                let question = question_mut(&mut survey, current_question, &line)?;
                match &mut question.response {
                    Response::Quantity { min, max } => {
                        *min = from;
                        *max = to;
                    }
                    response => {
                        return Err(BadLine::new(format!(
                            "Question '{}' cannot contain a range for response type {}",
                            question.name,
                            response.type_name()
                        ))
                        .into());
                    }
                }
            }
        }
    }
    Ok(survey)
}

fn require_question(question: Option<usize>, line: &str) -> Result<(), IoError> {
    question
        .ok_or_else(|| IoError::internal(format!("Line outside a variable: '{}'", line)))?;
    Ok(())
}

fn question_mut<'a>(
    survey: &'a mut Survey,
    index: Option<usize>,
    line: &str,
) -> Result<&'a mut Question, IoError> {
    let index = index
        .ok_or_else(|| IoError::internal(format!("Line outside a variable: '{}'", line)))?;
    survey
        .record
        .content
        .get_mut(index)
        .ok_or_else(|| IoError::internal("Invalid current question"))
}

fn unexpected_classic_line(line: &str, line_type: SSSClassicLineType) -> IoError {
    error::UnexpectedLineKind::new(LineKind::Classic(line_type), line).into()
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
    Spread,
    Size(usize),
}

impl SSSClassicLineType {
    pub fn parse_line(line: &str) -> Result<(Self, String), IoError> {
        let line = line.trim();
        if line.is_empty() {
            return Ok((Self::Empty, "".into()));
        }
        let mut fields = line.splitn(2, char::is_whitespace);
        let code = fields.next().unwrap_or_default();
        let value = fields.next().unwrap_or_default().trim();
        if code.contains('.') {
            let number = code
                .parse::<f64>()
                .map_err(|err| IoError::from(BadLine::new(err)))?;
            if value.starts_with('"') {
                let (label, suffix) = parse_quoted_value(value)?;
                if suffix.is_empty()
                    || suffix.eq_ignore_ascii_case("SPECIAL")
                    || suffix.eq_ignore_ascii_case("WITH")
                {
                    return Ok((
                        Self::Range {
                            from: number,
                            to: number,
                        },
                        label,
                    ));
                }
            }
        }
        if let Ok(number) = code.parse::<u64>()
            && value.starts_with('"')
        {
            let (label, suffix) = parse_quoted_value(value)?;
            if suffix.is_empty()
                || suffix.eq_ignore_ascii_case("SPECIAL")
                || suffix.eq_ignore_ascii_case("WITH")
            {
                return Ok((Self::Choice(number), label));
            }
        }
        let range_line = line
            .strip_suffix(" WITH")
            .or_else(|| line.strip_suffix(" SPECIAL"))
            .unwrap_or(line);
        if let Ok((from, to)) = parse_range_string::<f64>(range_line, "TO") {
            return Ok((Self::Range { from, to }, String::new()));
        }
        let code = code.to_ascii_uppercase();
        let value = unquote(value);
        let line_type = match code.as_str() {
            "SURVEY" => Self::SurveyStart,
            "RECORD" => Self::RecordStart,
            "VARIABLE" => Self::QuestionStart,
            "POSITION" => Self::Location,
            "TYPE" => Self::Kind,
            "VALUES" => Self::ResponseStart,
            "END" => Self::End(value.clone()),
            "SIZE" => Self::Size(
                value
                    .parse()
                    .map_err(|err| IoError::from(BadLine::new(err)))?,
            ),
            _ => code.parse::<Self>().map_err(|err| {
                IoError::from(BadLine::new(format!(
                    "line: '{}', error: {}",
                    line, err
                )))
            })?,
        };
        Ok((line_type, value))
    }
}

fn unquote(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
        .to_string()
}

fn parse_quoted_value(value: &str) -> Result<(String, &str), IoError> {
    if !value.starts_with('"') {
        return Err(BadLine::new(format!("Expected quoted value: '{}'", value)).into());
    }
    let end = value[1..]
        .find('"')
        .ok_or_else(|| BadLine::new(format!("Unclosed quoted value: '{}'", value)))?
        + 1;
    Ok((value[1..end].to_string(), value[end + 1..].trim()))
}

/// sss file type
/// classic: https://triple-s.org/wp-content/uploads/SSS-1.1.pdf
/// xml: sss 1.1 onwards: https://triple-s.org/documentation/manuals/
#[derive(Debug, PartialEq)]
pub enum SSSType {
    Classic,
    XML,
}

impl SSSType {
    pub fn parse(first_line: &str) -> Result<Self, BadLine> {
        match first_line.chars().next().unwrap_or_default() {
            '<' => Ok(Self::XML),
            'S' => Ok(Self::Classic),
            _ => {
                let message = format!(
                    "Unable to indetify sss version from the first line: '{}'",
                    first_line
                );
                Err(BadLine::new(message))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs::File,
        io::{BufRead, BufReader},
    };

    use super::*;

    fn read_classic_fixture() -> Survey {
        let file = File::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/sss_classic.sss"
        ))
        .unwrap();
        let mut lines = BufReader::new(file).lines();
        assert_eq!(lines.next().unwrap().unwrap(), "SSS");
        read_sss_classic(lines).unwrap()
    }

    #[test]
    fn test_read_sss_version() {
        assert_eq!(SSSType::parse("SSS").unwrap(), SSSType::Classic);
        assert_eq!(SSSType::parse("<").unwrap(), SSSType::XML);
    }

    #[test]
    fn reads_classic_survey_fields() {
        let survey = read_classic_fixture();

        assert_eq!(survey.meta.version.as_deref(), Some("1.1"));
        assert_eq!(
            survey.meta.date.as_deref(),
            Some("13th September 2026")
        );
        assert_eq!(survey.meta.time.as_deref(), Some("12:30"));
        assert_eq!(
            survey.meta.origin.as_deref(),
            Some("Example Survey Exporter v4.2")
        );
        assert_eq!(
            survey.meta.user.as_deref(),
            Some("Survey Administrator")
        );
        assert_eq!(survey.meta.standard.as_deref(), Some("NAMES"));
        assert_eq!(
            survey.meta.note.as_deref(),
            Some("HTML markup is stored inside SSS text strings.")
        );
        assert_eq!(survey.meta.style, None);
        assert_eq!(survey.title, "Customer Experience Survey - HTML Test");
        assert_eq!(survey.record.id, "A");
        assert_eq!(survey.record.content.len(), 24);

        let quantity = &survey.record.content[0];
        assert_eq!(quantity.name, "RESPID");
        assert_eq!(quantity.title, "Respondent identifier");
        assert_eq!((quantity.location.from, quantity.location.to), (1, 6));
        assert!(matches!(
            quantity.response,
            Response::Quantity {
                min: 1.0,
                max: 999999.0
            }
        ));

        let single = &survey.record.content[1];
        assert_eq!(single.name, "AGE");
        assert_eq!(single.location.from, 7);
        assert!(matches!(
            &single.response,
            Response::Single { item }
                if item.len() == 8
                    && item.get(&8).is_some_and(|choice| {
                        choice.code == 8
                            && choice.label == "Prefer not to say"
                            && choice.filter.as_deref() == Some("SPECIAL")
                    })
        ));

        let multiple = &survey.record.content[4];
        assert_eq!(multiple.name, "FEATURES2");
        assert_eq!(
            (multiple.location.from, multiple.location.to),
            (15, 20)
        );
        assert!(matches!(
            &multiple.response,
            Response::Multiple { item, spread }
                if item.len() == 6
                    && spread.num_subfields == 3
                    && spread.width == 3
        ));

        let character = &survey.record.content[8];
        assert_eq!(character.name, "COMMENTS");
        assert_eq!(
            character.title,
            "<p>Please tell us about your experience.</p><p>You can include <strong>any comments</strong> that may help us improve.</p>"
        );
        assert_eq!(
            (character.location.from, character.location.to),
            (42, 1041)
        );
        assert!(matches!(character.response, Response::Character(1000)));

        let logical = &survey.record.content[12];
        assert_eq!(logical.name, "HELP");
        assert_eq!(logical.location.from, 3243);
        assert!(matches!(
            logical.response,
            Response::Single { ref item } if item.is_empty()
        ));
    }

    #[test]
    fn parses_classic_line_values() {
        assert_eq!(
            SSSClassicLineType::parse_line(r#"LABEL "<p>Choose one</p>""#)
                .unwrap()
                .1,
            "<p>Choose one</p>"
        );
        assert!(matches!(
            SSSClassicLineType::parse_line("POSITION 42").unwrap().0,
            SSSClassicLineType::Location
        ));
        assert!(matches!(
            SSSClassicLineType::parse_line("-999.99 TO 9999.99")
                .unwrap()
                .0,
            SSSClassicLineType::Range {
                from: -999.99,
                to: 9999.99
            }
        ));
    }

    #[test]
    fn parses_response_kinds_and_ranges() {
        assert!(matches!(
            parse_classic_response_kind("CHARACTER").unwrap(),
            Response::Character(0)
        ));
        assert!(matches!(
            parse_classic_response_kind("LOGICAL").unwrap(),
            Response::Single { .. }
        ));
        assert_eq!(Location::parse_line("17").unwrap().from, 17);
        assert_eq!(Location::parse_line("17").unwrap().to, 17);
        let spread = Spread::parse_line("3 OF 4").unwrap();
        assert_eq!((spread.num_subfields, spread.width), (3, 4));
    }

    #[test]
    fn preserves_special_choice_filter() {
        let (kind, label) =
            SSSClassicLineType::parse_line(r#"9 "Prefer not to say" SPECIAL"#).unwrap();
        assert_eq!(kind, SSSClassicLineType::Choice(9));
        assert_eq!(label, "Prefer not to say");
    }
}
