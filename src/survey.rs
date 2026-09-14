// survey entity
use std::{collections::BTreeMap, fs::File, io::BufReader};

use pyo3::prelude::*;
use strum::EnumString;

use crate::error::{self, BadLine, IoError};

#[derive(Debug, Clone, PartialEq)]
#[pyclass]
/// A Triple-S survey definition.
///
/// The definition file describes the file metadata, survey title, respondent
/// record, and the variables contained in that record.
pub struct Survey {
    /// The survey title from the optional `TITLE` statement.
    #[pyo3(get, set)]
    pub title: String,
    /// File-level metadata such as version, creation date, and origin.
    #[pyo3(get, set)]
    pub meta: Meta,
    /// The respondent record and its ordered variable definitions.
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

    pub fn to_sss_clasic(&self) -> String {
        [
            "SSS".to_string(),
            self.meta.to_sss_clasic(),
            "SURVEY".to_string(),
            format!("TITLE {}", self.title),
            self.record.to_sss_clasic(),
            "END SURVEY".to_string(),
        ].join("\n")
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

    fn __getitem__(
        slf: PyRef<'_, Self>,
        py: Python<'_>,
        index: &Bound<'_, PyAny>,
    ) -> PyResult<Py<QuestionProxy>> {
        let question_index = if let Ok(name) = index.extract::<String>() {
            slf.record
                .content
                .iter()
                .position(|question| question.name.to_uppercase() == name.to_uppercase())
                .ok_or_else(|| {
                    pyo3::exceptions::PyKeyError::new_err(format!("unknown question: {name}"))
                })?
        } else if let Ok(position) = index.extract::<usize>() {
            if position >= slf.record.content.len() {
                return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                    "question index out of range: {position}"
                )));
            }
            position
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "survey indices must be strings or integers",
            ));
        };

        Py::new(
            py,
            QuestionProxy {
                survey: slf.into(),
                question_index,
            },
        )
    }

    #[pyo3(signature = (classic=None) )]
    fn to_string(&self, classic: Option<bool>) -> PyResult<String> {
        let classic = classic.unwrap_or_default();
        if classic {
            Ok(self.to_sss_clasic())
        } else {
            Err(pyo3::exceptions::PyNotImplementedError::new_err("Not implemented yet"))
        }
    }
}

#[pyclass]
/// A live proxy to a question inside a [`Survey`] so that one can do:
///
/// survey['q1'].text = "new question text"
pub struct QuestionProxy {
    /// The survey that owns the question.
    survey: Py<Survey>,
    /// The zero-based position of the question in the record.
    question_index: usize,
}

impl QuestionProxy {
    fn with_question<T>(&self, f: impl FnOnce(&Question) -> T) -> PyResult<T> {
        Python::attach(|py| {
            let survey = self.survey.bind(py).borrow();
            let question =
                survey
                    .record
                    .content
                    .get(self.question_index)
                    .ok_or_else(|| {
                        pyo3::exceptions::PyReferenceError::new_err(
                            "question no longer exists",
                        )
                    })?;
            Ok(f(question))
        })
    }

    fn with_question_mut<T>(&self, f: impl FnOnce(&mut Question) -> T) -> PyResult<T> {
        Python::attach(|py| {
            let mut survey = self.survey.bind(py).borrow_mut();
            let question = survey
                .record
                .content
                .get_mut(self.question_index)
                .ok_or_else(|| {
                    pyo3::exceptions::PyReferenceError::new_err("question no longer exists")
                })?;
            Ok(f(question))
        })
    }
}

#[pymethods]
impl QuestionProxy {
    #[getter]
    fn name(&self) -> PyResult<String> {
        self.with_question(|question| question.name.clone())
    }

    #[setter]
    fn set_name(&self, value: String) -> PyResult<()> {
        self.with_question_mut(|question| question.name = value)
    }

    #[getter]
    fn text(&self) -> PyResult<String> {
        self.with_question(|question| question.text.clone())
    }

    #[setter]
    fn set_text(&self, value: String) -> PyResult<()> {
        self.with_question_mut(|question| question.text = value)
    }

    #[getter]
    fn location(&self) -> PyResult<Location> {
        self.with_question(|question| question.location.clone())
    }

    #[setter]
    fn set_location(&self, value: Location) -> PyResult<()> {
        self.with_question_mut(|question| question.location = value)
    }

    #[getter]
    fn response(&self, py: Python<'_>) -> PyResult<Py<ResponseProxy>> {
        self.with_question(|_| ())?;
        Py::new(
            py,
            ResponseProxy {
                survey: self.survey.clone_ref(py),
                question_index: self.question_index,
            },
        )
    }

    #[setter]
    fn set_response(&self, value: Response) -> PyResult<()> {
        self.with_question_mut(|question| question.response = value)
    }

    fn __repr__(&self) -> PyResult<String> {
        self.with_question(Question::__repr__)
    }
}

#[pyclass]
/// A live proxy to a question's [`Response`] definition to achieve this api:
///
/// survey['question'].response[3].label = "new label"
pub struct ResponseProxy {
    /// The survey that owns the response.
    survey: Py<Survey>,
    /// The zero-based position of the question in the record.
    question_index: usize,
}

impl ResponseProxy {
    fn with_response<T>(&self, f: impl FnOnce(&Response) -> T) -> PyResult<T> {
        Python::attach(|py| {
            let survey = self.survey.bind(py).borrow();
            let question =
                survey
                    .record
                    .content
                    .get(self.question_index)
                    .ok_or_else(|| {
                        pyo3::exceptions::PyReferenceError::new_err(
                            "question no longer exists",
                        )
                    })?;
            Ok(f(&question.response))
        })
    }
}

#[pymethods]
impl ResponseProxy {
    fn __getitem__(
        &self,
        py: Python<'_>,
        index: &Bound<'_, PyAny>,
    ) -> PyResult<Py<ChoiceProxy>> {
        let code = index.extract::<u64>().map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err("choice indices must be integers")
        })?;

        self.with_response(|response| match response {
            Response::Single { item } | Response::Multiple { item, .. } => {
                if !item.contains_key(&code) {
                    return Err(pyo3::exceptions::PyKeyError::new_err(code.to_string()));
                }
                Ok(())
            }
            _ => Err(pyo3::exceptions::PyTypeError::new_err(
                "only Single and Multiple responses contain choices",
            )),
        })??;

        Py::new(
            py,
            ChoiceProxy {
                survey: self.survey.clone_ref(py),
                question_index: self.question_index,
                code,
            },
        )
    }

    fn __repr__(&self) -> PyResult<String> {
        self.with_response(Response::__repr__)
    }
}

#[pyclass]
/// A live proxy to a coded choice in a `Single` or `Multiple` response.
///
/// Changing `code` moves the choice to the corresponding `BTreeMap` key.
pub struct ChoiceProxy {
    /// The survey that owns the choice.
    survey: Py<Survey>,
    /// The zero-based position of the question in the record.
    question_index: usize,
    /// The current code and map key for the choice.
    code: u64,
}

impl ChoiceProxy {
    fn with_choice<T>(&self, f: impl FnOnce(&Choice) -> T) -> PyResult<T> {
        Python::attach(|py| {
            let survey = self.survey.bind(py).borrow();
            let question =
                survey
                    .record
                    .content
                    .get(self.question_index)
                    .ok_or_else(|| {
                        pyo3::exceptions::PyReferenceError::new_err(
                            "question no longer exists",
                        )
                    })?;
            let item = match &question.response {
                Response::Single { item } | Response::Multiple { item, .. } => item,
                _ => {
                    return Err(pyo3::exceptions::PyTypeError::new_err(
                        "only Single and Multiple responses contain choices",
                    ));
                }
            };
            let choice = item.get(&self.code).ok_or_else(|| {
                pyo3::exceptions::PyReferenceError::new_err("choice no longer exists")
            })?;
            Ok(f(choice))
        })
    }

    fn with_choice_mut<T>(&self, f: impl FnOnce(&mut Choice) -> T) -> PyResult<T> {
        Python::attach(|py| {
            let mut survey = self.survey.bind(py).borrow_mut();
            let question = survey
                .record
                .content
                .get_mut(self.question_index)
                .ok_or_else(|| {
                    pyo3::exceptions::PyReferenceError::new_err("question no longer exists")
                })?;
            let item = match &mut question.response {
                Response::Single { item } | Response::Multiple { item, .. } => item,
                _ => {
                    return Err(pyo3::exceptions::PyTypeError::new_err(
                        "only Single and Multiple responses contain choices",
                    ));
                }
            };
            let choice = item.get_mut(&self.code).ok_or_else(|| {
                pyo3::exceptions::PyReferenceError::new_err("choice no longer exists")
            })?;
            Ok(f(choice))
        })
    }
}

#[pymethods]
impl ChoiceProxy {
    #[getter]
    fn code(&self) -> PyResult<u64> {
        self.with_choice(|choice| choice.code)
    }

    #[setter]
    fn set_code(&mut self, value: u64) -> PyResult<()> {
        if value == self.code {
            return Ok(());
        }

        Python::attach(|py| {
            let mut survey = self.survey.bind(py).borrow_mut();
            let question = survey
                .record
                .content
                .get_mut(self.question_index)
                .ok_or_else(|| {
                    pyo3::exceptions::PyReferenceError::new_err("question no longer exists")
                })?;
            let item = match &mut question.response {
                Response::Single { item } | Response::Multiple { item, .. } => item,
                _ => {
                    return Err(pyo3::exceptions::PyTypeError::new_err(
                        "only Single and Multiple responses contain choices",
                    ));
                }
            };
            if item.contains_key(&value) {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "a choice with that code already exists",
                ));
            }
            let mut choice = item.remove(&self.code).ok_or_else(|| {
                pyo3::exceptions::PyReferenceError::new_err("choice no longer exists")
            })?;
            choice.code = value;
            item.insert(value, choice);
            self.code = value;
            Ok(())
        })
    }

    #[getter]
    fn label(&self) -> PyResult<String> {
        self.with_choice(|choice| choice.label.clone())
    }

    #[setter]
    fn set_label(&self, value: String) -> PyResult<()> {
        self.with_choice_mut(|choice| choice.label = value)
    }

    #[getter]
    fn filter(&self) -> PyResult<Option<String>> {
        self.with_choice(|choice| choice.filter.clone())
    }

    #[setter]
    fn set_filter(&self, value: Option<String>) -> PyResult<()> {
        self.with_choice_mut(|choice| choice.filter = value)
    }

    fn __repr__(&self) -> PyResult<String> {
        self.with_choice(Choice::__repr__)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
#[pyclass(from_py_object)]
/// Optional metadata describing the definition file.
pub struct Meta {
    /// The Triple-S version used by the definition file, such as `1.1`.
    #[pyo3(get, set)]
    pub version: Option<String>,
    /// The date on which the definition file was created.
    #[pyo3(get, set)]
    pub date: Option<String>,
    /// The time at which the definition file was created.
    #[pyo3(get, set)]
    pub time: Option<String>,
    /// The originating program and operating system.
    #[pyo3(get, set)]
    pub origin: Option<String>,
    /// The user who created the definition file.
    #[pyo3(get, set)]
    pub user: Option<String>,
    /// An implementation-specific style value, when supplied.
    #[pyo3(get, set)]
    pub style: Option<String>,
    /// A note associated with the definition file.
    #[pyo3(get, set)]
    pub note: Vec<String>,
    /// A declared standard, such as `NAMES`, that the definition conforms to.
    #[pyo3(get, set)]
    pub standard: Option<String>,
}

impl Meta {
    pub fn to_sss_clasic(&self) -> String {
        let mut content = Vec::new();
        if let Some(value) = self.version.as_ref() {
            content.push(format!("VERSION {}", value))
        };

        if let Some(value) = self.date.as_ref() {
            content.push(format!(r#"DATE "{}""#, value.replace(r#"""#, r#"{"}"#)))
        };

        if let Some(value) = self.time.as_ref() {
            content.push(format!(r#"TIME "{}""#, value.replace(r#"""#, r#"{"}"#)))
        };

        if let Some(value) = self.origin.as_ref() {
            content.push(format!(r#"ORIGIN "{}""#, value.replace(r#"""#, r#"{"}"#)))
        };

        if let Some(value) = self.user.as_ref() {
            content.push(format!(r#"USER "{}""#, value.replace(r#"""#, r#"{"}"#)))
        };

        if let Some(value) = self.style.as_ref() {
            content.push(format!(r#"STYLE "{}""#, value.replace(r#"""#, r#"{"}"#)))
        };

        for value in &self.note {
            content.push(format!(r#"NOTE "{}""#, value.replace(r#"""#, r#"{"}"#)));
        }

        if let Some(value) = self.standard.as_ref() {
            content.push(format!(r#"STANDARD "{}""#, value.replace(r#"""#, r#"{"}"#)))
        };

        content.join("\n")
    }
}

#[pymethods]
impl Meta {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (*, version = None, date = None, time = None, origin = None, user = None, style = None, note = None, standard = None))]
    fn __new__(
        version: Option<String>,
        date: Option<String>,
        time: Option<String>,
        origin: Option<String>,
        user: Option<String>,
        style: Option<String>,
        note: Option<Vec<String>>,
        standard: Option<String>,
    ) -> Self {
        Self {
            version,
            date,
            time,
            origin,
            user,
            style,
            note: note.unwrap_or_default(),
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
        note: Option<Vec<String>>,
        standard: Option<String>,
    ) {
        *slf.borrow_mut() = Self {
            version,
            date,
            time,
            origin,
            user,
            style,
            note: note.unwrap_or_default(),
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
/// A respondent record definition containing an identifier and variables.
pub struct Record {
    /// The record identifier, a single alphabetic character in Triple-S 1.1.
    #[pyo3(get, set)]
    pub id: String,
    /// Variables in their definition-file order.
    #[pyo3(get, set)]
    pub content: Vec<Question>,
}

impl Record {
    pub fn to_sss_clasic(&self) -> String {
        [
            format!("RECORD {}", self.id),
            self.content.iter().enumerate()
                .map(|(idx, question)| question.to_sss_clasic(idx))
                .collect::<Vec<String>>()
                .join("\n"),
            "END RECORD".to_string()
        ].join("\n")
    }
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
/// A survey variable.
pub struct Question {
    /// Name of a question, used as question id (i.e., 'Q1', 'Q2')
    #[pyo3(get, set)]
    pub name: String,
    /// question text
    #[pyo3(get, set)]
    pub text: String,

    #[pyo3(get, set)]
    pub filter: Option<String>,

    /// data location
    #[pyo3(get, set)]
    pub location: Location,

    /// question responses
    #[pyo3(get, set)]
    pub response: Response,
}

impl Question {
    pub fn to_sss_clasic(&self, variabe_id: impl std::fmt::Display) -> String {
        let mut content = vec![format!("VARIABLE {}", variabe_id)];
        content.push(format!("NAME {}", self.name));
        content.push(format!("LABEL {}", self.text));
        content.push(self.response.sss_classic_type_name());
        content.push(self.location.to_sss_clasic());
        content.push(self.response.sss_classic_response_value());
        content.push("END VARIABLE".to_string());
        content.join("\n")
    }
}

#[pymethods]
impl Question {
    #[new]
    #[pyo3(signature = (name, *, location = None, response = None, text = "", filter = None))]
    fn __new__(
        name: &str,
        location: Option<Location>,
        response: Option<Response>,
        text: &str,
        filter: Option<&str>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            text: text.to_owned(),
            location: location.unwrap_or_default(),
            response: response.unwrap_or_default(),
            filter: filter.map(|s| s.to_string()),
        }
    }

    /// Initialize a question with a required name and optional definition fields.
    #[pyo3(signature = (name, *, location = None, response = None, text = "", filter = None))]
    fn __init__(
        slf: &Bound<'_, Self>,
        name: &str,
        location: Option<Location>,
        response: Option<Response>,
        text: &str,
        filter: Option<&str>,
    ) {
        *slf.borrow_mut() = Self {
            name: name.to_owned(),
            text: text.to_owned(),
            location: location.unwrap_or_default(),
            response: response.unwrap_or_default(),
            filter: filter.map(|s| s.to_string()),
        };
    }

    fn __repr__(&self) -> String {
        format!(
            r#"Question(name="{}", text="{}", location={}, response={:?})"#,
            self.name,
            self.text,
            self.location.__repr__(),
            self.response.__repr__()
        )
    }

    fn __eq__(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug, Clone, PartialEq)]
#[pyclass(from_py_object)]
/// The type-specific definition of a survey variable.
pub enum Response {
    /// A numeric variable with an inclusive legal range.
    Quantity {
        /// The smallest legal numeric value.
        min: f64,
        /// The largest legal numeric value.
        max: f64,
    },
    /// A free-form character variable with the maximum field length.
    Character(usize),
    /// A free-form note/text variable.
    Note(String),
    /// A categorical variable allowing one response.
    Single {
        /// Category choices keyed by their positive integer values.
        item: BTreeMap<u64, Choice>,
    },
    /// A categorical variable allowing multiple responses.
    Multiple {
        /// Category choices keyed by their positive integer values.
        item: BTreeMap<u64, Choice>,
        /// The optional subfield layout used by spread-format data.
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

    pub fn sss_classic_type_name(&self) -> String {
        let type_str = self.type_name().to_uppercase();
        format!("TYPE {}", type_str)
    }

    pub fn sss_classic_response_value(&self) -> String {
        match self {
            Self::Single { item } => {
                let content = item.values().map(|value| value.to_sss_clasic())
                .collect::<Vec<String>>()
                .join("\n");
            
                ["VALUES".to_string(), content, "END VALUES".to_string()].join("\n")
            }
            Self::Multiple { item, spread } => {
                let content = item.values().map(|value| value.to_sss_clasic())
                .collect::<Vec<String>>()
                .join("\n");
                let spread = spread.to_sss_clasic();
                [spread, "VALUES".to_string(), content, "END VALUES".to_string()].join("\n")
            }
            Self::Character(length) => {
                format!("SIZE {}", length)
            }
            Self::Quantity { min, max } => {
                [
                    "VALUES".to_string(), 
                    format!("{} TO {}", min, max), 
                    "END VALUES".to_string()
                ].join("\n")
            }
            // other response types are not in native SSS survey, but very commonly used by
            // places that implements their own variations of sss file.
            // hence we don't convert these.
            _ => "".into()
        }
    }
}

#[pymethods]
impl Response {
    #[getter]
    fn choices(&self) -> Vec<Choice> {
        match self {
            Self::Single { item } | Self::Multiple { item, .. } => {
                item.values().cloned().collect()
            }
            _ => Vec::new(),
        }
    }

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
/// A labelled category value in a `SINGLE` or `MULTIPLE` variable.
pub struct Choice {
    /// The positive integer category value.
    #[pyo3(get, set)]
    pub code: u64,
    /// The optional human-readable category label.
    #[pyo3(get, set)]
    pub label: String,
    /// The optional `SPECIAL` marker, such as for missing or unknown values.
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

    pub fn to_sss_clasic(&self) -> String {
        let filter = self
            .filter
            .as_deref()
            .map_or(String::new(), |filter| format!(" {}", filter));
        format!(
            r#"{} "{}"{}"#,
            self.code,
            self.label.replace(r#"""#, r#"{"}"#),
            filter
        )
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
/// A one-based inclusive field position in a fixed-length data record.
pub struct Location {
    /// The first character position occupied by the variable.
    #[pyo3(get, set)]
    pub from: u64,
    /// The last character position occupied by the variable.
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
        Ok(Self { from, to })
    }

    pub fn to_sss_clasic(&self) -> String {
        format!("POSITION {} TO {}", self.from, self.to)
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
/// The subfield layout for spread-format `MULTIPLE` responses.
pub struct Spread {
    /// The number of consecutive category subfields.
    #[pyo3(get, set)]
    pub num_subfields: u64,
    /// The width, in characters, of each subfield.
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

    pub fn to_sss_clasic(&self) -> String {
        format!("SPREAD {} OF {}", self.num_subfields, self.width)
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
            SSSClassicLineType::Note => survey.meta.note.push(value),
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
                question.text = value;
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
        path::PathBuf,
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

    fn parse_serialized_survey(serialized: &str) -> Survey {
        let path = PathBuf::from(std::env::temp_dir()).join(format!(
            "triple_s_round_trip_{}.sss",
            std::process::id()
        ));
        std::fs::write(&path, serialized).unwrap();
        let file = File::open(&path).unwrap();
        let mut lines = BufReader::new(file).lines();
        assert_eq!(lines.next().unwrap().unwrap(), "SSS");
        let survey = read_sss_classic(lines).unwrap();
        std::fs::remove_file(path).unwrap();
        survey
    }

    #[test]
    fn round_trips_classic_serializers() {
        let survey = Survey {
            title: "Round-trip survey".to_string(),
            meta: Meta {
                version: Some("1.1".to_string()),
                date: Some("14 September 2026".to_string()),
                time: Some("01:15".to_string()),
                origin: Some("triple-s".to_string()),
                user: Some("tester".to_string()),
                style: Some("standard".to_string()),
                note: vec!["first note".to_string(), "second note".to_string()],
                standard: Some("NAMES".to_string()),
            },
            record: Record {
                id: "A".to_string(),
                content: vec![
                    Question {
                        name: "AGE".to_string(),
                        text: "Respondent age".to_string(),
                        filter: None,
                        location: Location { from: 1, to: 3 },
                        response: Response::Quantity {
                            min: 0.0,
                            max: 120.0,
                        },
                    },
                    Question {
                        name: "COMMENT".to_string(),
                        text: "A comment".to_string(),
                        filter: None,
                        location: Location { from: 4, to: 103 },
                        response: Response::Character(100),
                    },
                    Question {
                        name: "SATISFACTION".to_string(),
                        text: "Satisfaction".to_string(),
                        filter: None,
                        location: Location { from: 104, to: 104 },
                        response: Response::Single {
                            item: BTreeMap::from([
                                (1, Choice::new(1, "Low", None::<String>)),
                                (
                                    9,
                                    Choice::new(9, "Unknown", Some("SPECIAL")),
                                ),
                            ]),
                        },
                    },
                    Question {
                        name: "FEATURES".to_string(),
                        text: "Selected features".to_string(),
                        filter: None,
                        location: Location { from: 105, to: 110 },
                        response: Response::Multiple {
                            item: BTreeMap::from([
                                (1, Choice::new(1, "Email", None::<String>)),
                                (2, Choice::new(2, "Phone", None::<String>)),
                            ]),
                            spread: Spread {
                                num_subfields: 2,
                                width: 3,
                            },
                        },
                    },
                ],
            },
        };

        let parsed = parse_serialized_survey(&survey.to_sss_clasic());
        assert_eq!(parsed, survey);

        println!("{}", survey.to_sss_clasic())
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
            survey.meta.note,
            vec!["HTML-rich Triple-S survey example".to_string(), "HTML markup is stored inside SSS text strings.".to_string(),]
        );
        assert_eq!(survey.meta.style, None);
        assert_eq!(survey.title, "Customer Experience Survey - HTML Test");
        assert_eq!(survey.record.id, "A");
        assert_eq!(survey.record.content.len(), 24);

        let quantity = &survey.record.content[0];
        assert_eq!(quantity.name, "RESPID");
        assert_eq!(quantity.text, "Respondent identifier");
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
            character.text,
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
