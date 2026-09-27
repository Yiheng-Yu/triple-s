extern crate triple_s;
use std::{str::FromStr, sync::Arc};

use parking_lot::{
    RawRwLock, RwLock, lock_api::{ArcRwLockReadGuard, ArcRwLockWriteGuard},
};
use pyo3::{
    exceptions,
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};
use triple_s::{
    self as sss, SurveyInfo, Mode, Position, Question, Record, Survey, Text, TextVariant, Version,
};

use crate::{
    repr,
    utils::{self, lock_it_up, normalize_py_index, to_backend_err},
};

/// Read a Triple-S file and return its survey.
#[pyfunction]
#[pyo3(signature = (path: "str") -> "Survey")]
pub fn read_file(path: &str) -> PyResult<PySurvey> {
    let survey = sss::read_datamap(path)
        .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))?;
    Ok(PySurvey::from(survey))
}

/// Survey
/// More detail can be found at: https://triple-s.org/documentation/manuals/
#[pyclass(module = "triple_s", name = "Survey")]
pub struct PySurvey {
    meta: Arc<RwLock<sss::SurveyInfo>>, // holy hell this works
    record: Arc<RwLock<sss::Record>>,
}

impl From<sss::Survey> for PySurvey {
    fn from(value: sss::Survey) -> Self {
        Self {
            meta: Arc::new(RwLock::new(value.meta)),
            record: Arc::new(RwLock::new(value.record)),
        }
    }
}

impl From<PySurvey> for sss::Survey {
    fn from(value: PySurvey) -> Self {
        Self {
            meta: value.meta.read().clone(),
            record: value.record.read().clone(),
        }
    }
}

impl PySurvey {
    fn to_backend_object(&self) -> Survey {
        let meta = self.meta.read().clone();
        let record = self.record.read().clone();
        Survey { meta, record }
    }
}

#[pymethods]
impl PySurvey {
    /// Export content back to sss classic
    fn to_sss_classic(&self) -> PyResult<String> {
        let instance: Survey = self.to_backend_object();
        instance.to_sss_classic().map_err(to_backend_err)
    }

    /// Return a shallow copy sharing the underlying data.
    fn __copy__(&self) -> Self {
        self.copy()
    }

    /// Return an independent deep copy.
    #[pyo3(signature = () -> "Survey")]
    fn __deepcopy__(&self) -> Self {
        self.deepcopy()
    }

    /// returns a shallow copy of current instance
    /// NOTE: updating the copied `Survey` instance would also mutate the original `Survey` instance too.
    /// To obtain a deep copy, use `copy.deepcopy` OR `deepcopy()` method
    fn copy(&self) -> Self {
        Self {
            meta: self.meta.clone(),
            record: self.record.clone(),
        }
    }

    /// returns a deep copy of current instance.
    /// updating the copied `Survey` instance would not affect the original `Survey` instance it copied from.
    fn deepcopy(&self) -> Self {
        Self {
            meta: Arc::new(RwLock::new(self.meta.read().clone())),
            record: Arc::new(RwLock::new(self.record.read().clone())),
        }
    }

    /// Survey metadata; changes are shared with this survey.
    #[getter]
    fn get_meta<'py>(&self) -> PyResult<PyMeta> {
        Ok(PyMeta::from_parent(&self))
    }

    /// Return the title.
    #[getter]
    fn get_title<'py>(&self) -> Option<String> {
        self.meta.read().title.clone()
    }

    /// Set the title.
    #[setter]
    fn set_title<'py>(&self, title: Option<String>) {
        self.meta.write().title = title;
    }

    /// Return the name.
    #[getter]
    fn get_name(&self) -> Option<String> {
        self.meta.read().name.clone()
    }

    /// Set the name.
    #[setter]
    fn set_name(&self, name: Option<String>) {
        self.meta.write().name = name;
    }

    /// Survey record; changes are shared with this survey.
    #[getter]
    fn get_record<'py>(&self) -> PyResult<PyRecord> {
        Ok(PyRecord::from_parent(&self))
    }

    /// Return the survey questions as a tuple.
    #[getter]
    fn get_content<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        self.get_record()?.get_content(py)
    }

    /// find question via question name, case insensitive
    #[pyo3(signature = (question_name: "str") -> "Question | None")]
    fn get_question(&self, question_name: &str) -> PyResult<Option<PyQuestion>> {
        self.get_record().map(|record| record.get(question_name))
    }

    //     fn __repr__(inner: &Bound<'_, Self>) -> PyResult<String> {
    //         let instance = inner.borrow();
    //         let py = inner.py();
    //         let title = instance.get_title().unwrap_or_default();
    //         let version = Self::get_meta(inner)?
    //             .get_version(py)?
    //             .to_string();

    //         Ok(format!(
    //             r#"Survey(title="{}", version={})"#,
    //             title, version
    //         ))
    //     }
}

/// Content of a survey
#[pyclass(module = "triple_s", name = "Record")]
pub struct PyRecord(Arc<RwLock<Record>>);

impl PyRecord {
    /// Create a view sharing the parent survey’s data.
    pub fn from_parent(survey: &PySurvey) -> Self {
        Self(survey.record.clone())
    }

    pub(crate) fn read(&self) -> ArcRwLockReadGuard<RawRwLock, Record> {
        self.0.read_arc()
    }

    pub(crate) fn write(&self) -> ArcRwLockWriteGuard<RawRwLock, Record> {
        self.0.write_arc()
    }
}

#[pymethods]
impl PyRecord {
    /// Export content back to sss classic
    fn to_sss_classic(&self) -> PyResult<String> {
        self.read().to_sss_classic().map_err(to_backend_err)
    }

    /// Return the number of questions.
    fn __len__(&self) -> usize {
        self.read().content.len()
    }

    /// One RECORD keyword appears after SURVEY (or after TITLE if present).
    /// It is used to introduce the definition of the variables.
    /// The record_id is any single character A to Z (or a to z).
    /// The record_id can be used in conjunction with the variable_id (see the VARIABLE keyword later) to
    /// generate a unique variable name on import.
    /// For example: RECORD A
    #[getter]
    fn get_id(&self) -> PyResult<char> {
        Ok(self.read().id.clone())
    }

    /// Set the id.
    #[setter]
    fn set_id(&mut self, value: char) -> PyResult<()> {
        self.write().id = value;
        Ok(())
    }

    /// List of questions/ variables that the current survey instance contains
    #[getter]
    fn get_content<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let guard = self.read();
        let content = guard.content.iter().map(PyQuestion::from_arc);
        PyTuple::new(py, content)
    }

    /// Find a question by name, ignoring case; return None if absent.
    #[pyo3(signature = (question_name: "str") -> "Question | None")]
    fn get(&self, question_name: &str) -> Option<PyQuestion> {
        self.read()
            .get_question(question_name)
            .map(PyQuestion::from_arc)
    }

    /// Return a question by index or name; raise IndexError if absent.
    fn __getitem__<'py>(&self, index: &Bound<'py, PyAny>) -> PyResult<PyQuestion> {
        if let Ok(number) = index.extract::<isize>() {
            let guard = self.read();
            let total_length = self.__len__();
            let sanitised = utils::normalize_py_index(number, total_length)
                .ok_or(exceptions::PyIndexError::new_err(number))?;
            guard
                .content
                .get(sanitised)
                .map(PyQuestion::from_arc)
                .ok_or(exceptions::PyIndexError::new_err(number))
        } else if let Ok(question_name) = index.extract::<String>() {
            self.get(&question_name)
                .ok_or(exceptions::PyIndexError::new_err(question_name))
        } else {
            let type_name = index.get_type().name()?.extract::<String>()?;
            Err(exceptions::PyTypeError::new_err(format!(
                "`index` must be question index (as `int`) or question name (as `str`), found: {}",
                type_name
            )))
        }
    }

    /// Append an independent copy of a question with a unique name.
    #[pyo3(signature = (question: "Question") -> "None")]
    fn append(&mut self, question: PyQuestion) -> PyResult<()> {
        let question_name = question.get_name()?;
        if !self.read().has_question(&question_name) {
            let instance = question.0.clone().read().clone();
            let instance = lock_it_up(instance);
            self.write().content.push(instance);
            Ok(())
        } else {
            Err(exceptions::PyValueError::new_err(format!(
                "Unable to append question '{}' because a question with same name already exists.",
                question_name,
            )))
        }
    }

    // // todo!!
    // fn extend(&mut self)

    //     fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
    //         // only show first n amount of members
    //         let content_length = self.content(py)?.len();
    //         let read_length = content_length.min(15);

    //         let mut question_string = Vec::new();
    //         for index in 0..read_length {
    //             let question = self.__getitem__(py, index as isize)?.__repr__(py)?;
    //             question_string.push(question);
    //         }
    //         if read_length < content_length {
    //             question_string.push("...".into())
    //         };
    //         let question_string = question_string.join(",\n    ");
    //         let current_id = self.get_id(py)?;
    //         Ok(format!(
    //             "Record(\n  id=\"{}\",\n  content=[\n    {}\n  ]\n)",
    //             current_id, question_string
    //         ))
    //     }
}

/// A survey question and its response definition.
#[pyclass(module = "triple_s", name = "Question", from_py_object)]
pub struct PyQuestion(Arc<RwLock<Question>>);

/// does a deep copy
impl Clone for PyQuestion {
    fn clone(&self) -> Self {
        let deepcopy = self.0.clone().read().clone();
        deepcopy.into()
    }
}

impl From<&Question> for PyQuestion {
    fn from(value: &Question) -> Self {
        let content = RwLock::from(value.clone());
        let content = Arc::new(content);
        Self(content)
    }
}

impl From<Question> for PyQuestion {
    fn from(value: Question) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }
}

impl PyQuestion {
    pub(crate) fn from_arc(question: &Arc<RwLock<Question>>) -> Self {
        Self(question.clone())
    }

    pub(crate) fn read(&self) -> ArcRwLockReadGuard<RawRwLock, Question> {
        self.0.read_arc()
    }

    pub(crate) fn write(&mut self) -> ArcRwLockWriteGuard<RawRwLock, Question> {
        self.0.write_arc()
    }
}

#[pymethods]
impl PyQuestion {
    /// Question construction is not implemented yet.
    #[new]
    fn new() -> Self {
        todo!()
    }

    /// Export fragments of `Question` instance from from SSS classic
    #[pyo3(signature = () -> "str")]
    fn to_sss_classic(&self) -> PyResult<String> {
        self.read()
            .to_sss_classic()
            .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    /// Return a string representation.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            r#"Question(name="{}")"#,
            self.read().name.clone()
        ))
    }

    /// Return the name.
    #[getter]
    fn get_name(&self) -> PyResult<String> {
        Ok(self.read().name.clone())
    }

    /// Set the name.
    #[setter]
    fn set_name(&mut self, value: &str) -> PyResult<()> {
        self.write().name = value.into();
        Ok(())
    }

    /// Return the question label and its text variants.
    #[getter]
    fn get_label(&self) -> PyResult<PyText> {
        let content = self.read().label.clone();
        let output = PyText::from(content);
        Ok(output)
    }

    /// Set the label from text or a Text instance.
    #[setter]
    fn set_label(&mut self, label: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(content) = label.extract::<String>() {
            if self.get_label()?.has_variants() {
                *self.write().label.value.write() = content;
                Ok(())
            } else {
                self.write().label = Text::new(content).into();
                Ok(())
            }
        } else if let Ok(content) = label.extract::<PyText>() {
            self.write().label = Text::try_from(content)?;
            Ok(())
        } else {
            todo!()
        }
    }

    /// Return the filter.
    #[getter]
    fn get_filter(&self) -> PyResult<Option<String>> {
        Ok(self.read().filter.clone())
    }

    /// Set the filter.
    #[setter]
    fn set_filter(&mut self, value: Option<String>) -> PyResult<()> {
        self.write().filter = value.clone();
        Ok(())
    }

    //     #[getter]
    //     fn get_response(question: &Bound<'_, Self>) -> PyResult<PyResponse> {
    //         Ok(PyResponse::from_inner(question))
    //     }

    #[getter]
    pub fn get_location(&self) -> PyResult<PyLocation> {
        let item = self.read().location.clone();
        let guard = RwLock::from(item);
        let content = Arc::from(guard);
        Ok(PyLocation(content))
    }

    //     #[setter]
    //     pub fn set_location(&self, py: Python<'_>, value: &Bound<'_, PyLocation>) -> PyResult<()> {
    //         let loc = value.borrow();
    //         let from = loc.get_from(py)?;
    //         let to = loc.get_to(py)?;

    //         let location = Position { from, to };
    //         self.with_question_write(py, |mut question| {
    //             question.location = location;
    //         })
    //     }

    //     #[getter]
    //     pub fn get_format(&self, py: Python<'_>) -> PyResult<PyFormat> {
    //         self.with_question_read(py, |inner| inner.format.clone().into())
    //     }

    //     #[setter]
    //     pub fn set_format(&mut self, format: &Bound<'_, PyAny>) -> PyResult<()> {
    //         if let Ok(arg) = format.extract::<&str>() {
    //             let format= sss::CodeFormat::from_str(arg)
    //                 .map_err(exceptions::PyValueError::new_err(format!(
    //                     "Unidentified format option: '{}'. Exepct to be one of: [{}]",
    //                     arg, sss::CodeFormat::all_format_names().collect::<Vec<&str>>()
    //                 )))?;
    //             todo!()
    //             // self.with_question_write(format.py(), |inner| inner)
    //         } else {
    //             todo!()
    //         }
    //     }
}

#[pyclass(module = "triple_s", name = "Location")]
pub struct PyLocation(Arc<RwLock<Position>>);

impl PyLocation {
    pub(crate) fn read(&self) -> ArcRwLockReadGuard<RawRwLock, Position> {
        self.0.read_arc()
    }

    pub(crate) fn write(&mut self) -> ArcRwLockWriteGuard<RawRwLock, Position> {
        self.0.write_arc()
    }
}

#[pymethods]
impl PyLocation {
    // #[new]
    // #[pyo3(signature = (from, to))]
    // fn new(from: u32, to: u32) -> Self {
    //     Self {
    //         inner: Storage::Owned(Arc::new(RwLock::new(Position { from, to }))),
    //     }
    // }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Location(from={}, to={})", self.read().from, self.read().to))
    }

    /// Start of a data location
    #[getter]
    fn get_start(&self) -> PyResult<u32> {
        Ok(self.read().from)  // `from` is a reserved python keyword
    }

    #[setter]
    fn set_start(&mut self, value: u32) -> PyResult<()> {
        self.write().from = value;
        Ok(())
    }

    #[getter]
    fn get_end(&self) -> PyResult<u32> {
        Ok(self.read().to)
    }

    #[setter]
    fn set_end(&mut self, value: u32) -> PyResult<()> {
        self.write().to = value;
        Ok(())
    }
}

/// meta information about the survey entity.
#[pyclass(dict, weakref, module = "triple_s", name = "SurveyInfo", from_py_object)]
pub struct PyMeta {
    inner: Arc<RwLock<SurveyInfo>>,
}

impl Clone for PyMeta {
    /// creates a deep copy of current instance
    fn clone(&self) -> Self {
        let instance = self.inner.read().clone();
        Self {
            inner: Arc::new(RwLock::new(instance)),
        }
    }
}

impl PyMeta {
    /// Create a view sharing the parent survey’s data.
    pub fn from_parent(survey: &PySurvey) -> Self {
        Self {
            inner: survey.meta.clone(),
        }
    }

    pub(crate) fn read(&self) -> ArcRwLockReadGuard<RawRwLock, SurveyInfo> {
        self.inner.read_arc()
    }

    pub(crate) fn write(&mut self) -> ArcRwLockWriteGuard<RawRwLock, SurveyInfo> {
        self.inner.write_arc()
    }
}

#[pymethods]
impl PyMeta {
    /// Create survey metadata for the given Triple-S version.
    /// Optional text fields default to None; languages default to [], modes to [None].
    /// Modes accept "analysis", "interview", or None. Invalid versions or modes raise ValueError.
    #[new]
    #[pyo3(signature = (
        version: "str",
        name: "str | None" = None,
        title: "str | None" = None,
        date: "str | None" = None,
        time: "str | None" = None,
        origin: "str | None" = None,
        user: "str | None" = None,
        style: "str | None" = None,
        language_options: "list[str] | None" = None,
        mode_options: "list[str | None] | None" = None,
        standard: "str | None" = None,
    ))]
    fn new(
        version: &str,
        name: Option<String>,
        title: Option<String>,
        date: Option<String>,
        time: Option<String>,
        origin: Option<String>,
        user: Option<String>,
        style: Option<String>,
        language_options: Option<Vec<String>>,
        mode_options: Option<Vec<Option<String>>>,
        standard: Option<String>,
    ) -> PyResult<Self> {
        let version = Version::from_str(version)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))?;
        let mode_options = mode_options
            .map(|options| {
                options
                    .into_iter()
                    .map(|mode| {
                        mode.as_deref()
                            .map(Mode::from_str)
                            .transpose()
                            .map(Option::unwrap_or_default)
                            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
                    })
                    .collect::<PyResult<Vec<Mode>>>()
            })
            .transpose()?
            .unwrap_or_else(|| vec![Mode::None]);

        Ok(Self {
            inner: lock_it_up(SurveyInfo {
                version,
                name,
                title,
                date,
                time,
                origin,
                user,
                style,
                language_options: language_options.unwrap_or_default(),
                mode_options,
                standard,
            }),
        })
    }

    /// ** Introduced since sss 1.2 **
    /// list of all available `mod` options
    #[getter]
    fn get_mode_options<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let options = self
            .read()
            .mode_options
            .iter()
            .map(Mode::try_as_str)
            .collect::<Vec<_>>();
        PyTuple::new(py, options)
    }

    //     #[new]
    //     #[pyo3(signature = (survey=None, version=None))]
    //     fn new(
    //         py: Python<'_>,
    //         survey: Option<&Bound<'_, PySurvey>>,
    //         version: Option<String>,
    //     ) -> PyResult<Self> {
    //     }

    /// make a shallow copy of current instance.
    #[pyo3(signature = () -> "SurveyInfo")]
    fn copy(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    /// make a deep copy of current instance
    #[pyo3(signature = () -> "SurveyInfo")]
    fn deepcopy(&self) -> Self {
        self.clone()
    }

    /// The version attribute is used to indicate the version of the Triple-S
    /// standard that applies to this specification.
    #[getter]
    fn get_version(&self) -> PyResult<&str> {
        Ok(self.read().version.as_str())
    }

    /// Set the version.
    #[setter]
    fn set_version(&mut self, value: String) -> PyResult<()> {
        let version = Version::from_str(&value)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))?;
        self.write().version = version;
        Ok(())
    }

    /// (Optional) name fo the survey
    #[getter]
    fn get_name(&self) -> PyResult<Option<String>> {
        Ok(self.read().name.clone())
    }

    /// Set the name.
    #[setter]
    fn set_name(&mut self, value: Option<String>) -> PyResult<()> {
        self.write().name = value;
        Ok(())
    }

    /// (Optional) The date the file was created.
    #[getter]
    fn get_date(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().date.clone())
    }

    /// Set the date.
    #[setter]
    fn set_date(&mut self, value: Option<String>) -> PyResult<()> {
        self.write().date = value;
        Ok(())
    }

    /// (Optional) The time the file was created.
    #[getter]
    fn get_time(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().time.clone())
    }

    /// Set the time.
    #[setter]
    fn set_time(&mut self, value: Option<String>) -> PyResult<()> {
        self.write().time = value;
        Ok(())
    }

    /// Return the title.
    #[getter]
    fn get_title(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().title.clone())
    }

    /// Set the title.
    #[setter]
    fn set_title(&mut self, value: Option<String>) -> PyResult<()> {
        self.write().title = value;
        Ok(())
    }

    /// (Optional) The originating system (program and operating system), i.e., `Windows 95`
    #[getter]
    fn get_origin(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().origin.clone())
    }

    /// Set the origin.
    #[setter]
    fn set_origin(&mut self, value: Option<String>) -> PyResult<()> {
        self.write().origin = value;
        Ok(())
    }

    /// (Optional) Name of the user who created this file
    #[getter]
    fn get_user(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().user.clone())
    }

    /// Set the user.
    #[setter]
    fn set_user(&mut self, value: Option<String>) -> PyResult<()> {
        self.write().user = value;
        Ok(())
    }

    /// Return the style.
    #[getter]
    fn get_style(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().style.clone())
    }

    /// Set the style.
    #[setter]
    fn set_style(&mut self, value: Option<String>) -> PyResult<()> {
        self.write().style = value;
        Ok(())
    }

    /// ** Deprecated after sss version 1.2 **
    /// (Optional) indicate that some aspect of the triple-s definition meets a
    /// defined standard that may reduce the amount of checking required by an
    /// import program. The only value that option_setting may have in
    /// version 1.2 is 'standardnames'.
    #[getter]
    fn get_standard(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().standard.clone())
    }

    /// Set the standard.
    #[setter]
    fn set_standard(&mut self, value: Option<String>) -> PyResult<()> {
        self.write().standard = value;
        Ok(())
    }

    /// ** Introduced since sss 1.2 **
    /// list of all available language options
    #[getter]
    fn get_language_options<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let options = self.read().language_options.clone();
        PyTuple::new(py, options)
    }

    /// Return version and populated name, title, date, time, origin, user, and standard fields.
    #[pyo3(signature = () -> "dict[str, str]")]
    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);

        dict.set_item("version", self.get_version()?)?;

        if let Some(value) = self.get_name()? {
            dict.set_item("name", value)?;
        };

        if let Some(value) = self.get_title()? {
            dict.set_item("title", value)?;
        }

        if let Some(value) = self.get_date()? {
            dict.set_item("date", value)?;
        };

        if let Some(value) = self.get_time()? {
            dict.set_item("time", value)?;
        };

        if let Some(value) = self.get_origin()? {
            dict.set_item("origin", value)?;
        };

        if let Some(value) = self.get_user()? {
            dict.set_item("user", value)?;
        };

        if let Some(value) = self.get_standard()? {
            dict.set_item("standard", value)?;
        };

        Ok(dict.unbind())
    }
}

// ============================= Enums/ Flags =============================
/// Default text with optional language and mode variants.
#[pyclass(name = "Text", module = "triple_s", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyText {
    pub(crate) value: Arc<RwLock<String>>,
    pub(crate) variants: Arc<RwLock<Vec<PyVariant>>>,
}

impl From<Text> for PyText {
    fn from(value: Text) -> Self {
        let variants = value.variant.into_iter().map(PyVariant::from).collect();
        Self {
            value: value.value.clone(),
            variants: Arc::new(RwLock::new(variants)),
        }
    }
}

impl From<String> for PyText {
    fn from(value: String) -> Self {
        Text::new(value).into()
    }
}

impl TryFrom<PyText> for Text {
    type Error = pyo3::PyErr;

    fn try_from(value: PyText) -> Result<Self, Self::Error> {
        Ok(Self {
            value: value.value.clone(),
            variant: value
                .variants
                .read()
                .clone()
                .into_iter()
                .map(TextVariant::try_from)
                .collect::<PyResult<Vec<_>>>()?,
        })
    }
}

impl PyText {
    fn try_insert_variant(&mut self, variant: PyVariant) -> PyResult<()> {
        let incoming = variant.0.read().clone();
        if self
            .variants
            .read()
            .iter()
            .any(|item| item.0.read().has_same_attribute(&incoming))
        {
            return Err(exceptions::PyValueError::new_err(
                "A variant with this mode and language already exists; use set_variant to overwrite it",
            ));
        }
        self.variants.write().push(variant);
        Ok(())
    }

    fn insert_or_overwrite_variant(&mut self, variant: PyVariant) {
        let incoming = variant.0.read().clone();
        for item in self.variants.read().iter() {
            let mut existing = item.0.write();
            if existing.has_same_attribute(&incoming) {
                existing.value = incoming.value;
                return;
            }
        }
        self.variants.write().push(variant);
    }
}

#[pymethods]
impl PyText {
    /// Create default text with optional Variant arguments.
    #[new]
    #[pyo3(signature = (content: "str", *variants: "Variant"))]
    fn new(content: String, variants: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let variants = variants.extract::<Vec<PyVariant>>()?;
        Ok(Self {
            value: lock_it_up(content),
            variants: lock_it_up(variants),
        })
    }

    /// Return an HTML representation for notebooks.
    fn _repr_html_(&self) -> PyResult<String> {
        repr::html_print_pytext(&self)
    }

    /// Return a shallow copy sharing the underlying data.
    fn __copy__(&self) -> Self {
        self.copy()
    }

    /// Return an independent deep copy.
    fn __deepcopy__(&self) -> Self {
        self.deepcopy()
    }

    /// Return a shallow copy sharing the underlying data.
    fn copy(&self) -> PyText {
        self.clone()
    }

    /// Return an independent deep copy.
    fn deepcopy(&self) -> PyText {
        let variant = self
            .variants
            .read()
            .iter()
            .map(|variant| PyVariant::from(variant.0.read().clone()))
            .collect::<Vec<PyVariant>>();

        Self {
            value: lock_it_up(self.value.read().clone()),
            variants: lock_it_up(variant),
        }
    }

    /// checks if there were any text variants
    fn has_variants(&self) -> bool {
        !self.variants.read().is_empty()
    }

    /// the default value
    #[getter]
    fn get_value(&self) -> PyResult<String> {
        Ok(self.value.read().clone())
    }

    /// Set the value.
    #[setter]
    fn set_value(&mut self, value: String) -> PyResult<()> {
        *self.value.write() = value;
        Ok(())
    }

    /// variants
    #[getter]
    fn get_variants(&self) -> PyResult<VariantList> {
        Ok(VariantList(self.variants.clone()))
    }

    /// Replace the text variants with the supplied list.
    #[setter]
    fn set_variants(&mut self, variants: &Bound<'_, PyList>) -> PyResult<()> {
        let variants = variants.extract::<Vec<PyVariant>>()?;
        self.variants = lock_it_up(variants);
        Ok(())
    }

    /// Add a text variant. Raises error of variant with same `mode` and `language` option already exists.
    ///
    /// Args:
    ///     variant (Variant | str): An existing variant or the text content of a
    ///         new variant.
    ///     mode (str | None): The mode of the new variant. Only valid when
    ///         `variant` is a string.
    ///     language (str | None): The language of the new variant. Only valid when
    ///         `variant` is a string.
    ///
    /// Examples:
    ///     Add an existing variant:
    ///
    ///         obj.add_variant(variant)
    ///
    ///     Create and add a variant from text:
    ///
    ///         obj.add_variant("Hello world", mode="interview", language="en")
    #[pyo3(signature = (variant: "Variant | str", mode: "str | None" = None, language: "str | None" = None) -> "None")]
    fn add_variant(
        &mut self,
        variant: &Bound<'_, PyAny>,
        mode: Option<String>,
        language: Option<String>,
    ) -> PyResult<()> {
        if let Ok(variant) = variant.extract::<PyVariant>() {
            self.try_insert_variant(variant)
        } else if let Ok(content) = variant.extract::<String>() {
            let variant = PyVariant::new(content, language, mode)?;
            self.try_insert_variant(variant)
        } else {
            Err(exceptions::PyValueError::new_err(
                "variant must be a Variant or str",
            ))
        }
    }

    /// Will modify existing variant if they have the same `mode` and `language`
    /// attribute or append to the current `self.variant` if not
    #[pyo3(signature = (variant: "Variant | str", mode: "str | None" = None, language: "str | None" = None) -> "None")]
    fn set_variant(
        &mut self,
        variant: &Bound<'_, PyAny>,
        mode: Option<String>,
        language: Option<String>,
    ) -> PyResult<()> {
        if let Ok(variant) = variant.extract::<PyVariant>() {
            self.insert_or_overwrite_variant(variant);
            Ok(())
        } else if let Ok(content) = variant.extract::<String>() {
            let variant = PyVariant::new(content, language, mode)?;
            self.insert_or_overwrite_variant(variant);
            Ok(())
        } else {
            Err(exceptions::PyValueError::new_err(
                "variant must be a Variant or str",
            ))
        }
    }
}

/// A mutable collection of text variants.
#[pyclass(name = "VariantList", module = "triple_s", from_py_object)]
pub struct VariantList(Arc<RwLock<Vec<PyVariant>>>);

impl Clone for VariantList {
    /// makes a DEEP COPY of variants
    fn clone(&self) -> Self {
        let interior = self
            .0
            .read()
            .clone()
            .into_iter()
            .map(|item| item.deepcopy())
            .collect::<Vec<PyVariant>>();
        Self(lock_it_up(interior))
    }
}

impl VariantList {
    /// Create a collection from the supplied variants.
    pub fn from_py_variants(variants: Vec<PyVariant>) -> Self {
        Self(lock_it_up(variants))
    }

    pub(crate) fn read(&self) -> ArcRwLockReadGuard<RawRwLock, Vec<PyVariant>> {
        self.0.read_arc()
    }

    pub(crate) fn write(&self) -> ArcRwLockWriteGuard<RawRwLock, Vec<PyVariant>> {
        self.0.write_arc()
    }

    fn try_insert_variant(&mut self, variant: PyVariant) -> PyResult<()> {
        let incoming = variant.0.read().clone();
        if self
            .read()
            .iter()
            .any(|item| item.0.read().has_same_attribute(&incoming))
        {
            return Err(exceptions::PyValueError::new_err(
                "A variant with this mode and language already exists; use set_variant to overwrite it",
            ));
        }
        self.write().push(variant);
        Ok(())
    }

    // fn insert_or_overwrite_variant(&mut self, variant: PyVariant) {
    //     let incoming = variant.0.read().clone();
    //     for item in self.read().iter() {
    //         let mut existing = item.0.write();
    //         if existing.has_same_attribute(&incoming) {
    //             existing.value = incoming.value;
    //             return;
    //         }
    //     }
    //     self.write().push(variant);
    // }
}

#[pymethods]
impl VariantList {
    /// Return a string representation.
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let collection = PyList::new(py, self.read().clone())?;
        py.import("pprint")?
            .getattr("pformat")?
            .call1((collection,))?
            .extract()
    }

    /// Return the number of variants.
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.read().len())
    }

    /// Return a variant by index; negative indices count from the end.
    fn __getitem__(&self, index: isize) -> PyResult<PyVariant> {
        let index = normalize_py_index(index, self.read().len())
            .ok_or(exceptions::PyIndexError::new_err(index))?;
        Ok(self
            .read()
            .get(index)
            .ok_or(exceptions::PyIndexError::new_err(index))?
            .clone())
    }

    /// Append a variant.
    #[pyo3(signature = (variant: "Variant") -> "None")]
    fn append(&mut self, variant: PyVariant) -> PyResult<()> {
        self.write().push(variant);
        Ok(())
    }

    /// Append variants; raise ValueError for duplicate mode and language pairs.
    #[pyo3(signature = (variants: "list[Variant]") -> "None")]
    fn extend(&mut self, variants: Vec<PyVariant>) -> PyResult<()> {
        for variant in variants {
            self.try_insert_variant(variant)?
        }
        Ok(())
    }

    /// remove first text variant in the list, and returns it.
    #[pyo3(signature = () -> "Variant | None")]
    fn popleft(&mut self) -> PyResult<Option<PyVariant>> {
        if self.read().is_empty() {
            Ok(None)
        } else {
            let removed = self.write().remove(0);
            let removed = removed.deepcopy();
            Ok(Some(removed))
        }
    }

    /// remove the last text variant in the list, and returns it.
    #[pyo3(signature = () -> "Variant | None")]
    fn pop(&mut self) -> PyResult<Option<PyVariant>> {
        let poped = self.write().pop().map(|item| item.deepcopy());
        Ok(poped)
    }

    /// Extend in place from a VariantList or list of variants.
    fn __iadd__(&mut self, other: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(v_list) = other.extract::<VariantList>() {
            let to_extend = v_list.read().clone();
            self.extend(to_extend)
        } else if let Ok(v_list) = other.extract::<Vec<PyVariant>>() {
            let v_list = v_list
                .into_iter()
                .map(|item| item.deepcopy())
                .collect::<Vec<PyVariant>>();
            self.extend(v_list)
        } else {
            let name = other.get_type().name()?.extract::<String>()?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect list of Variant instance, got: {}",
                name
            )))
        }
    }
}

/// From sss xml 2.0 onwards supports multi-language
///
/// For sss xml 2.0:
/// ```text
/// The text for a survey <title>, variable <label> or values
/// <value> element may contain any number of specialised text elements.
/// ```
///
/// Example sss xml 2.0 text, note that one entry can contain both `label` and
/// `language` attribute
/// ```xml
/// <label>Age
///     <text xml:lang="en-GB" mode="interview">How old are you?</text>
///     <text xml:lang="fr" mode="interview">Quel est votre âge?</text>
///     <text xml:lang="en-GB" mode="analysis">Age of respondent</text>
///     <text xml:lang="fr" mode="analysis">Âge de répondant</text>
/// </label>
/// ```
#[pyclass(name = "Variant", module = "triple_s", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyVariant(pub(crate) Arc<RwLock<TextVariant>>);

impl From<TextVariant> for PyVariant {
    fn from(value: TextVariant) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }
}

impl TryFrom<PyVariant> for TextVariant {
    type Error = pyo3::PyErr;

    fn try_from(value: PyVariant) -> Result<Self, Self::Error> {
        Ok(value.0.read().clone())
    }
}

#[pymethods]
impl PyVariant {
    /// Create a text variant; supply a language tag or mode (analysis or interview).
    #[new]
    #[pyo3(signature = (content: "str", language: "str | None" = None, mode: "str | None" = None))]
    fn new(content: String, language: Option<String>, mode: Option<String>) -> PyResult<Self> {
        if mode.is_none() & language.is_none() {
            return Err(exceptions::PyValueError::new_err(
                "At least one language or a mode must be provided",
            ));
        };

        let mode = mode
            .map(|s| Mode::from_str(&s))
            .transpose()
            .map_err(|err| to_backend_err(err.into()))?
            .unwrap_or_default();

        let instance = TextVariant::new(content, mode, language)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))?;

        Ok(instance.into())
    }

    /// Return a string representation.
    pub fn __repr__(&self) -> PyResult<String> {
        let mut repr = format!(r#"Variant(value='{}'"#, &self.0.read().value);
        repr = if let Some(mode) = self.get_mode() {
            format!("{}, mode='{}'", repr, mode)
        } else {
            repr
        };
        repr = if let Some(language) = self.get_language() {
            format!("{}, language='{}'", repr, language)
        } else {
            repr
        };
        let repr = format!("{})", repr);
        Ok(repr)
    }

    /// Return an HTML representation for notebooks.
    fn _repr_html_(&self) -> PyResult<String> {
        repr::html_print_pyvariant(&self)
    }

    /// Return a shallow copy sharing the underlying data.
    fn copy(&self) -> Self {
        Self(self.0.clone())
    }

    /// Return a shallow copy sharing the underlying data.
    fn __copy__(&self) -> Self {
        self.copy()
    }

    /// Return an independent deep copy.
    fn deepcopy(&self) -> Self {
        Self(lock_it_up(self.0.read().clone()))
    }

    /// Return an independent deep copy.
    fn __deepcopy__(&self) -> Self {
        self.deepcopy()
    }

    /// Return the variant text.
    #[getter]
    fn get_value(&self) -> String {
        self.0.read().value.clone()
    }

    /// Set the value.
    #[setter]
    fn set_value(&self, value: String) -> PyResult<()> {
        self.0.write().value = value;
        Ok(())
    }

    /// Return analysis, interview, or None for both modes.
    #[getter]
    pub fn get_mode(&self) -> Option<String> {
        self.0.read().mode.try_as_str().map(|s| s.to_string())
    }

    /// Set analysis, interview, or None; raise ValueError for invalid modes.
    #[setter]
    fn set_mode(&self, mode: Option<&str>) -> PyResult<()> {
        let mode = mode
            .map(Mode::from_str)
            .transpose()
            .map_err(|err| {
                exceptions::PyValueError::new_err(format!(
                    "{}, value should only be one of {:?}",
                    err.to_string(),
                    Mode::all_mode_options().as_slice()
                ))
            })?
            .unwrap_or_default();

        self.0.write().mode = mode;
        Ok(())
    }

    /// Return the language tag, or None if unspecified.
    #[getter]
    pub fn get_language(&self) -> Option<String> {
        self.0.read().get_language()
    }

    /// set `language` to `Language::Name(value)`
    #[setter]
    fn set_language(&self, language: Option<&str>) -> PyResult<()> {
        self.0
            .write()
            .set_language(language)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
    }
}

/// UnSigned or literal response code format.
#[pyclass(name = "Format", module = "config")]
pub struct PyFormat(pub sss::CodeFormat);

impl From<sss::CodeFormat> for PyFormat {
    fn from(value: sss::CodeFormat) -> Self {
        Self(value)
    }
}

impl From<PyFormat> for sss::CodeFormat {
    fn from(value: PyFormat) -> Self {
        value.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_new_variant() {
        const VALUE: &str = "variant";
        const LANGUAGE: &str = "gb";
        const MODE_STR: &str = "analysis";

        let instance = PyVariant::new(
            VALUE.into(),
            Some(LANGUAGE.into()),
            Some(MODE_STR.into()),
        )
        .unwrap();

        assert_eq!(instance.get_value().as_str(), VALUE);
        assert_eq!(instance.get_mode(), Some(MODE_STR.into()));
    }

    #[test]
    fn test_modify_text_variant() {
        panic!("TODO!")
    }
}
