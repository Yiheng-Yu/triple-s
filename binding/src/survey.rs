extern crate triple_s as tri;
use std::{str::FromStr, sync::Arc};

use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock};
use pyo3::{exceptions, prelude::*, types::{PyDict, PyTuple}};
use tri::{Code, Position, Response, Text, Version, prelude::*};

#[pyfunction]
pub fn read_file(path: &str) -> PyResult<PySurvey> {
    let survey = tri::read_sss_file(path)
        .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))?;
    Ok(PySurvey::from(survey))
}

enum Storage<P, T> {
    Attached { inner: Py<P>, index: u64 },
    Owned(Arc<RwLock<T>>),
}

impl<P, T> Storage<P, T> {
    pub fn from_owned(item: T) -> Self {
        Self::Owned(Arc::new(RwLock::new(item)))
    }
}

/// Survey
/// More detail can be found at: https://triple-s.org/documentation/manuals/
#[pyclass(module = "triple_s", name = "Survey", from_py_object)]
pub struct PySurvey {
    // #[pyo3(get, set)]
    // pub title: Option<String>,
    // meta: Arc<RwLock<tri::Classic>>,
    // record: Arc<RwLock<tri::Record>>,
    inner: Arc<RwLock<tri::Survey>>,
}

impl Wrap for PySurvey {}

impl Clone for PySurvey {
    fn clone(&self) -> Self {
        Self {
            // title: self.title.clone(),
            // meta: Arc::clone(&self.meta),
            // record: Arc::clone(&self.record)
            inner: Arc::clone(&self.inner),
        }
    }
}

impl From<tri::Survey> for PySurvey {
    fn from(value: tri::Survey) -> Self {
        Self {
            // title: value.title,
            // meta: Arc::new(RwLock::new(value.meta)),
            // record: Arc::new(RwLock::new(value.record))
            inner: Arc::new(RwLock::new(value)),
        }
    }
}

impl PySurvey {
    /// Acquire the inner 'survey.meta' for reading; surfaces lock poisoning as a
    /// `PyException` instead of panicking.
    pub(crate) fn read_meta(&self) -> MappedRwLockReadGuard<'_, tri::Meta> {
        parking_lot::RwLockReadGuard::map(self.inner.read(), |survey| &survey.meta)
    }

    /// Acquire the inner 'survey.meta' for writing; surfaces lock poisoning as a
    /// `PyException` instead of panicking.
    pub(crate) fn write_meta(&self) -> MappedRwLockWriteGuard<'_, tri::Meta> {
        parking_lot::RwLockWriteGuard::map(self.inner.write(), |survey| &mut survey.meta)
    }

    /// Acquire the inner 'survey.record' for reading; surfaces lock poisoning as a
    /// `PyException` instead of panicking.
    pub(crate) fn read_record(&self) -> MappedRwLockReadGuard<'_, tri::Record> {
        parking_lot::RwLockReadGuard::map(self.inner.read(), |survey| &survey.record)
    }

    /// Acquire the inner 'survey.record' for writing; surfaces lock poisoning as a
    /// `PyException` instead of panicking.
    pub(crate) fn write_record(&self) -> MappedRwLockWriteGuard<'_, tri::Record> {
        parking_lot::RwLockWriteGuard::map(self.inner.write(), |survey| &mut survey.record)
    }

    /// Acquire the inner 'survey.question' for reading; surfaces lock poisoning as a
    /// `PyException` instead of panicking.
    pub(crate) fn read_question(
        &self,
        index: usize,
    ) -> PyResult<MappedRwLockReadGuard<'_, tri::Question>> {
        parking_lot::RwLockReadGuard::try_map(self.inner.read(), |survey| {
            survey.record.content.get(index)
        })
        .map_err(|_| exceptions::PyIndexError::new_err(index))
    }

    /// Acquire the inner 'survey.question' for writing; surfaces lock poisoning as a
    /// `PyException` instead of panicking.
    pub(crate) fn write_question(
        &self,
        index: usize,
    ) -> PyResult<MappedRwLockWriteGuard<'_, tri::Question>> {
        parking_lot::RwLockWriteGuard::try_map(self.inner.write(), |survey| {
            survey.record.content.get_mut(index)
        })
        .map_err(|_| exceptions::PyIndexError::new_err(index))
    }
}

#[pymethods]
impl PySurvey {
    #[new]
    #[pyo3(signature = (title=None))]
    fn new(title: Option<String>) -> Self {
        todo!()
        // let mut survey = tri::Survey::empty();
        // survey.title = title;
        // Self::from(survey)
    }

    /// Export content back to sss classic
    fn to_sss_classic(&self) -> PyResult<String> {
        self.inner
            .read()
            .to_sss_classic()
            .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    #[getter]
    fn get_meta<'py>(inner: &Bound<'_, Self>) -> PyResult<PyMeta> {
        Ok(PyMeta::from_rust(inner))
    }

    #[getter]
    fn get_title<'py>(&self) -> Option<String> {
        self.inner.read().title.clone()
    }

    #[setter]
    fn set_title<'py>(&self, title: Option<String>) {
        self.inner.write().title = title;
    }

    #[getter]
    fn get_record<'py>(inner: &Bound<'_, Self>) -> PyResult<PyRecord> {
        Ok(PyRecord::from_rust(inner))
    }

    #[getter]
    fn get_content<'py>(inner: &Bound<'_, Self>) -> PyResult<Vec<PyQuestion>> {
        PyRecord::from_rust(inner).content(inner.py())
    }

    fn get_question(survey: &Bound<'_, Self>, question_name: &str) -> PyResult<PyQuestion> {
        let query = question_name.to_uppercase();
        survey
            .borrow()
            .read_record()
            .content
            .iter()
            .enumerate()
            .find(|(_, question)| question.name.to_uppercase() == query)
            .map(|(index, _)| PyQuestion::from_inner(&survey.clone().unbind(), index))
            .ok_or(exceptions::PyKeyError::new_err(
                question_name.to_string(),
            ))
    }

    fn __len__(&self) -> PyResult<usize> {
        let length = self.read_record().content.len();
        Ok(length)
    }

    fn __repr__(inner: &Bound<'_, Self>) -> PyResult<String> {
        let instance = inner.borrow();
        let py = inner.py();
        let title = instance.get_title().unwrap_or_default();
        let version = Self::get_meta(inner)?
            .get_version(py)?
            .to_string();

        Ok(format!(
            r#"Survey(title="{}", version={})"#,
            title, version
        ))
    }
}

/// Content of a survey
#[pyclass(module = "triple_s", name = "Record")]
pub struct PyRecord {
    inner: Storage<PySurvey, tri::Record>,
}

impl PyRecord {
    pub fn from_rust(survey: &Bound<'_, PySurvey>) -> Self {
        Self {
            inner: Storage::Attached {
                inner: survey.clone().unbind(),
                index: 0,
            },
        }
    }

    pub fn len(&self, py: Python<'_>) -> usize {
        match &self.inner {
            Storage::Attached { inner, .. } => inner.borrow(py).read_record().content.len(),
            Storage::Owned(item) => item.read().content.len(),
        }
    }

    fn with_record_read<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(MappedRwLockReadGuard<'_, tri::Record>) -> R,
    ) -> PyResult<R> {
        match &self.inner {
            Storage::Attached { inner, .. } => {
                let item = inner.bind(py).borrow();
                let item = item.read_record();
                Ok(f(item))
            }
            Storage::Owned(item) => {
                let item = item.read();
                let item = parking_lot::RwLockReadGuard::map(item, |item| item);
                Ok(f(item))
            }
        }
    }

    fn with_record_write<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(MappedRwLockWriteGuard<'_, tri::Record>) -> R,
    ) -> PyResult<R> {
        match &self.inner {
            Storage::Attached { inner, .. } => {
                let item = inner.bind(py).borrow_mut();
                let item = item.write_record();
                Ok(f(item))
            }
            Storage::Owned(item) => {
                let item = item.write();
                let item = parking_lot::RwLockWriteGuard::map(item, |item| item);
                Ok(f(item))
            }
        }
    }
}

#[pymethods]
impl PyRecord {
    #[new]
    #[pyo3(signature = (id='A'))]
    fn new(id: char) -> Self {
        todo!()
        // Self {
        //     inner: Storage::from_owned(tri::Record {
        //         id,
        //         ..Default::default()
        //     }),
        // }
    }

    /// Export content back to sss classic
    fn to_sss_classic(&self, py: Python<'_>) -> PyResult<String> {
        self.with_record_read(py, |record| record.to_sss_classic())?
            .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    /// One RECORD keyword appears after SURVEY (or after TITLE if present).
    /// It is used to introduce the definition of the variables.
    /// The record_id is any single character A to Z (or a to z).
    /// The record_id can be used in conjunction with the variable_id (see the VARIABLE keyword later) to
    /// generate a unique variable name on import.
    /// For example: RECORD A
    #[getter]
    fn get_id(&self, py: Python<'_>) -> PyResult<char> {
        self.with_record_read(py, |record| record.id.clone())
    }

    #[setter]
    fn set_id(&mut self, py: Python<'_>, value: char) -> PyResult<()> {
        self.with_record_write(py, |mut record| record.id = value)
    }

    /// List of questions/ variables that the current survey instance contains
    #[getter]
    fn content(&self, py: Python<'_>) -> PyResult<Vec<PyQuestion>> {
        match &self.inner {
            Storage::Attached { inner, .. } => Ok((0..self.len(py))
                .map(|index| PyQuestion::from_inner(&inner, index))
                .collect()),
            Storage::Owned(item) => Ok(item
                .read()
                .content
                .iter()
                .map(PyQuestion::from_storage)
                .collect()),
        }
    }

    fn __getitem__(&self, py: Python<'_>, index: isize) -> PyResult<PyQuestion> {
        let length = self.content(py)?.len();
        let lookup = normalize_index(index, length)?; // also checks if index really exists, raise IndexError if otherwise
        match &self.inner {
            Storage::Attached { inner, .. } => Ok(PyQuestion::from_inner(&inner, lookup)),
            Storage::Owned(item) => item
                .read()
                .content
                .get(lookup)
                .map(PyQuestion::from_storage)
                .ok_or(exceptions::PyIndexError::new_err(index)),
        }
    }

    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        Ok(self.len(py))
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        // only show first n amount of members
        let content_length = self.content(py)?.len();
        let read_length = content_length.min(15);

        let mut question_string = Vec::new();
        for index in 0..read_length {
            let question = self.__getitem__(py, index as isize)?.__repr__(py)?;
            question_string.push(question);
        }
        if read_length < content_length {
            question_string.push("...".into())
        };
        let question_string = question_string.join(",\n    ");
        let current_id = self.get_id(py)?;
        Ok(format!(
            "Record(\n  id=\"{}\",\n  content=[\n    {}\n  ]\n)",
            current_id, question_string
        ))
    }
}

#[pyclass(module = "triple_s", name = "Question")]
pub struct PyQuestion {
    inner: Storage<PySurvey, tri::Question>,
}

impl PyQuestion {
    pub fn from_storage(question: &tri::Question) -> Self {
        Self {
            inner: Storage::from_owned(question.clone()),
        }
    }

    pub fn from_inner(inner: &Py<PySurvey>, index: usize) -> Self {
        Self {
            inner: Storage::Attached {
                inner: inner.clone(),
                index: index as u64,
            },
        }
    }

    fn with_question_read<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(MappedRwLockReadGuard<'_, tri::Question>) -> R,
    ) -> PyResult<R> {
        match &self.inner {
            Storage::Attached { inner, index } => {
                let inner = inner.bind(py).borrow();
                let index = (*index).try_into().map_err(|_| {
                    PyErr::new::<exceptions::PyIndexError, _>("index is too large")
                })?;
                let question = inner.read_question(index)?;

                Ok(f(question))
            }
            Storage::Owned(item) => {
                let item = item.read();
                let item = parking_lot::RwLockReadGuard::map(item, |item| item);
                Ok(f(item))
            }
        }
    }

    fn with_question_write<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(MappedRwLockWriteGuard<'_, tri::Question>) -> R,
    ) -> PyResult<R> {
        match &self.inner {
            Storage::Attached { inner, index } => {
                let inner = inner.bind(py).borrow_mut();
                let index = (*index).try_into().map_err(|_| {
                    PyErr::new::<exceptions::PyIndexError, _>("index is too large")
                })?;
                let question = inner.write_question(index)?;
                Ok(f(question))
            }
            Storage::Owned(item) => {
                let item = item.write();
                let item = parking_lot::RwLockWriteGuard::map(item, |item| item);
                Ok(f(item))
            }
        }
    }

    pub(crate) fn with_location_read<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(&tri::Position) -> R,
    ) -> PyResult<R> {
        self.with_question_read(py, |question| f(&question.location))
    }

    pub(crate) fn with_location_write<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(&mut tri::Position) -> R,
    ) -> PyResult<R> {
        self.with_question_write(py, |mut question| f(&mut question.location))
    }

    pub(crate) fn with_response<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(&tri::Response) -> R,
    ) -> PyResult<R> {
        self.with_question_read(py, |question| f(&question.response))
    }

    pub(crate) fn with_response_mut<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(&mut tri::Response) -> R,
    ) -> PyResult<R> {
        self.with_question_write(py, |mut question| f(&mut question.response))
    }
}

#[pymethods]
impl PyQuestion {
    #[new]
    #[pyo3(signature = (name="", label="", index=0))]
    fn new(name: &str, label: &str, index: u64) -> Self {
        todo!()
        // Self {
        //     inner: Storage::from_owned(tri::Question {
        //         index,
        //         name: name.to_string(),
        //         label: Text::new(label),
        //         ..Default::default()
        //     }),
        // }
    }

    fn to_sss_classic(&self, py: Python<'_>) -> PyResult<String> {
        self.with_question_read(py, |question| question.to_sss_classic())?
            .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        self.with_question_read(py, |question| {
            format!(
                r#"Question(name="{}")"#,
                &question.name
            )
        })
    }

    #[getter]
    fn get_name(&self, py: Python<'_>) -> PyResult<String> {
        self.with_question_read(py, |question| question.name.clone())
    }

    #[setter]
    fn set_name(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        self.with_question_write(py, |mut question| question.name = value.to_string())
    }

    #[getter]
    fn get_label(&self, py: Python<'_>) -> PyResult<String> {
        self.with_question_read(py, |question| question.label.value.clone())
    }

    #[setter]
    fn set_label(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        self.with_question_write(py, |mut question| question.label = Text::new(value))
    }

    #[getter]
    fn get_filter(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.with_question_read(py, |question| question.filter.clone())
    }

    #[setter]
    fn set_filter(&self, py: Python<'_>, value: Option<&str>) -> PyResult<()> {
        let value = value.map(|s| s.to_string());
        self.with_question_write(py, |mut question| question.filter = value)
    }

    #[getter]
    fn get_response(question: &Bound<'_, Self>) -> PyResult<PyResponse> {
        Ok(PyResponse::from_inner(question))
    }

    #[getter]
    pub fn get_location(question: &Bound<'_, Self>) -> PyResult<PyLocation> {
        Ok(PyLocation::from_inner(question))
    }

    #[setter]
    pub fn set_location(&self, py: Python<'_>, value: &Bound<'_, PyLocation>) -> PyResult<()> {
        let loc = value.borrow();
        let from = loc.get_from(py)?;
        let to = loc.get_to(py)?;

        let location = Position { from, to };
        self.with_question_write(py, |mut question| {
            question.location = location;
        })
    }
}

#[pyclass(module = "triple_s", name = "Response", subclass)]
pub struct PyResponse {
    inner: Storage<PyQuestion, tri::Response>,
}

impl PyResponse {
    pub fn from_inner(question: &Bound<'_, PyQuestion>) -> Self {
        Self {
            inner: Storage::Attached { inner: question.clone().unbind(), index: 0}
        }
    }

    fn with_response<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(&tri::Response) -> R,
    ) -> PyResult<R> {
        match &self.inner {
            Storage::Attached {inner, ..} => {
                inner.bind(py).borrow().with_response(py, |response| f(&response))
            }
            Storage::Owned(inner) => {
                Ok(f(&inner.read()))
            }
        }
    }

    fn with_response_mut<R>(
        &mut self,
        py: Python<'_>,
        f: impl FnOnce(&mut tri::Response) -> R,
    ) -> PyResult<R> {
        match &mut self.inner {
            Storage::Attached {inner, ..} => {
                inner.bind(py).borrow_mut().with_response_mut(py, |mut response| f(&mut response))
            }
            Storage::Owned(inner) => {
                Ok(f(&mut inner.write()))
            }
        }
    }
}

#[pymethods]
impl PyResponse {
    #[getter]
    fn item(slf: &Bound<'_, Self>) -> PyResult<Option<Vec<PyCode>>> {
        let py = slf.py();
        slf.borrow().with_response(py, |response| {
            match response {
                Response::NumericCategory(item) => {
                    let items = (0..item.read().values.len())
                        .map(|idx| PyCode::from_inner(slf, idx as u64))
                        .collect::<Vec<PyCode>>();
                    Some(items)
                }
                Response::TextCategory(item) => {
                    let items = (0..item.read().values.len())
                        .map(|idx| PyCode::from_inner(slf, idx as u64))
                        .collect::<Vec<PyCode>>();
                    Some(items)
                }
                _ => None
            }
        })
    }

    /// alias for `item`
    #[getter]
    fn content(slf: &Bound<'_, Self>) -> PyResult<Option<Vec<PyCode>>> {
        Self::item(slf)
    }
}

enum CodeInterface {
    Int(Code<u64>),
    Float(Code<f64>),
    String(Code<String>)
}

/// A possible answer that a respondent can choose in a single-/ multi- response question
#[pyclass(module = "triple_s", name = "Choice")]
pub struct PyCode {
    storage: Storage<PyResponse, CodeInterface>,
}

impl PyCode {
    pub(crate) fn with_integer_code<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(&Code<u64>) -> R,
    ) -> PyResult<R> {
        match &self.storage {
            Storage::Attached { inner, index } => inner
                .bind(py)
                .try_borrow()?
                .with_response(py, |response| match response {
                    Response::NumericCategory(member) => member
                        .read()
                        .values
                        .get(*index as usize)
                        .ok_or_else(|| exceptions::PyIndexError::new_err(*index))
                        .map(f),
                    _ => Err(exceptions::PyTypeError::new_err(
                        "Expected a NumericCategory response",
                    )),
                })?,
            // Owned choices store their code at index zero.
            Storage::Owned(item) => match &*item.read() {
                CodeInterface::Int(value) => Ok(f(value)),
                _ => Err(exceptions::PyTypeError::new_err("Expected integer codes")),
            },
        }
    }

    pub(crate) fn with_integer_code_mute<R>(
        &mut self,
        py: Python<'_>,
        f: impl FnOnce(&mut Code<u64>) -> R,
    ) -> PyResult<R> {
        match &mut self.storage {
            Storage::Attached { inner, index } => inner
                .bind(py)
                .try_borrow_mut()?
                .with_response_mut(py, |response| match response {
                    Response::NumericCategory(member) => member
                        .write()
                        .values
                        .get_mut(*index as usize)
                        .ok_or_else(|| exceptions::PyIndexError::new_err(*index))
                        .map(f),
                    _ => Err(exceptions::PyTypeError::new_err(
                        "Expected a NumericCategory response",
                    )),
                })?,
            // Owned choices store their code at index zero.
            Storage::Owned(item) => match &mut *item.write() {
                CodeInterface::Int(value) => Ok(f(value)),
                _ => Err(exceptions::PyTypeError::new_err("Expected integer codes")),
            },
        }
    }

    pub(crate) fn with_string_code<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(&Code<String>) -> R,
    ) -> PyResult<R> {
        match &self.storage {
            Storage::Attached { inner, index } => inner
                .bind(py)
                .try_borrow()?
                .with_response(py, |response| match response {
                    Response::TextCategory(member) => member
                        .read()
                        .values
                        .get(*index as usize)
                        .ok_or_else(|| exceptions::PyIndexError::new_err(*index))
                        .map(f),
                    _ => Err(exceptions::PyTypeError::new_err(
                        "Expected a TextCategory response",
                    )),
                })?,
            // Owned choices store their code at index zero.
            Storage::Owned(item) => match &*item.read() {
                CodeInterface::String(value) => Ok(f(value)),
                _ => Err(exceptions::PyTypeError::new_err("Expected string codes")),
            },
        }
    }

    pub(crate) fn with_string_code_mute<R>(
        &mut self,
        py: Python<'_>,
        f: impl FnOnce(&mut Code<String>) -> R,
    ) -> PyResult<R> {
        match &mut self.storage {
            Storage::Attached { inner, index } => inner
                .bind(py)
                .try_borrow_mut()?
                .with_response_mut(py, |response| match response {
                    Response::TextCategory(member) => member
                        .write()
                        .values
                        .get_mut(*index as usize)
                        .ok_or_else(|| exceptions::PyIndexError::new_err(*index))
                        .map(f),
                    _ => Err(exceptions::PyTypeError::new_err(
                        "Expected a TextCategory response",
                    )),
                })?,
            // Owned choices store their code at index zero.
            Storage::Owned(item) => match &mut *item.write() {
                CodeInterface::String(value) => Ok(f(value)),
                _ => Err(exceptions::PyTypeError::new_err("Expected string codes")),
            },
        }
    }

    pub fn from_inner(response: &Bound<'_, PyResponse>, index: u64) -> Self {
        Self {
            storage: Storage::Attached {
                inner: response.clone().unbind(),
                index,
            },
        }
    }
}


#[pymethods]
impl PyCode {
    #[new]
    #[pyo3(signature = (code, label))]
    fn new(
        code: &Bound<'_, PyAny>,
        label: String,
    ) -> PyResult<Self>{
        let inner = if let Ok(code) = code.extract::<u64>() {
            let item = Code::new(code, label)
                .map_err(|err| exceptions::PyRuntimeError::new_err(err.backtrace().to_string()))?;
            CodeInterface::Int(item)
        } else if let Ok(code) = code.extract::<String>() {
            let item = Code::new(code, label)
                .map_err(|err| exceptions::PyRuntimeError::new_err(err.backtrace().to_string()))?;
            CodeInterface::String(item)
        } else if let Ok(code) = code.extract::<f64>() {
            let item = Code::new(code, label)
                .map_err(|err| exceptions::PyRuntimeError::new_err(err.backtrace().to_string()))?;
            CodeInterface::Float(item)
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                    "code must be an int, float or str",
                ))
        };
        Ok(Self { storage: Storage::from_owned(inner)} )
    }

    // todo! need to acoomdate different data types!!
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        todo!()
    }

    /// Code used to uniquely identify the current answer option.
    #[getter]
    fn code(&self, py: Python<'_>) -> PyResult<u64> {
        todo!()
    }

    /// Text label of the current answer option
    #[getter]
    fn label(&self, py: Python<'_>) -> PyResult<String> {
        todo!()
    }

    #[setter]
    fn set_label(&mut self, py: Python<'_>, value: &str) -> PyResult<()> {
        todo!()
    }

    /// Response filter of the current answer option.
    /// This attribute is not part of the .sss file itself, but can be very useful for extending current library to read other survey data maps modified from .sss xml (i.e., `.sif` from SnapSurveys, `.qps` from QPSMR etc).
    /// An exmaple use case be setting up something like this:
    /// Q1: How often do you drive?
    /// choice 1: Never
    /// choice 2: Always
    ///
    /// Q2: Do you enjoy driving?
    /// choice 1: nah (ONLY shown for those who selected '2' in Q1)
    /// choice 2: okay-ish (ONLY shown for those who selected '2' in Q1)
    /// choice 3: very much
    #[getter]
    fn filter(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.with_integer_code(py, |choice| choice.filter.clone())
    }

    #[setter]
    fn set_filter(&mut self, py: Python<'_>, value: Option<&str>) -> PyResult<()> {
        self.with_integer_code_mute(py, |choice| {
            choice.filter = value.map(|s| s.to_string())
        })
    }

    /// (Optional) indicator for special values (e.g. mannotationissing data codes).
    /// Removed from SSS 1.2 onwards.
    #[getter]
    fn is_special(&self, py: Python<'_>) -> PyResult<bool> {
        self.with_integer_code(py, |choice| choice.is_special)
    }

    #[setter]
    fn set_is_special(&mut self, py: Python<'_>, value: bool) -> PyResult<()> {
        self.with_integer_code_mute(py, |choice| choice.is_special = value)
    }
}


#[pyclass(module = "triple_s", name = "Location")]
pub struct PyLocation {
    inner: Storage<PyQuestion, Position>,
}

impl PyLocation {
    pub(crate) fn from_inner(question: &Bound<'_, PyQuestion>) -> Self {
        Self {
            inner: Storage::Attached {
                inner: question.clone().unbind(),
                index: 0,
            },
        }
    }

    fn with_location_read<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(&tri::Position) -> R,
    ) -> PyResult<R> {
        match &self.inner {
            Storage::Owned(location) => {
                let location = location.read();
                Ok(f(&location))
            }

            Storage::Attached { inner, .. } => {
                inner.bind(py).borrow().with_location_read(py, f)
            }
        }
    }

    fn with_location_write<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(&mut tri::Position) -> R,
    ) -> PyResult<R> {
        match &self.inner {
            Storage::Owned(location) => {
                let mut location = location.write();
                Ok(f(&mut *location))
            }

            Storage::Attached { inner, .. } => {
                inner.bind(py).borrow_mut().with_location_write(py, f)
            }
        }
    }
}

#[pymethods]
impl PyLocation {
    #[new]
    #[pyo3(signature = (from, to))]
    fn new(from: u64, to: u64) -> Self {
        Self {
            inner: Storage::Owned(Arc::new(RwLock::new(Position { from, to }))),
        }
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        self.with_location_read(py, |loc| {
            format!("Location(from={}, to={})", loc.from, loc.to)
        })
    }

    #[getter]
    fn get_from(&self, py: Python<'_>) -> PyResult<u64> {
        self.with_location_read(py, |loc| loc.from.clone())
    }

    #[setter]
    fn set_from(&self, py: Python<'_>, value: u64) -> PyResult<()> {
        self.with_location_write(py, |loc| loc.from = value)
    }

    #[getter]
    fn get_to(&self, py: Python<'_>) -> PyResult<u64> {
        self.with_location_read(py, |loc| loc.to.clone())
    }

    #[setter]
    fn set_to(&self, py: Python<'_>, value: u64) -> PyResult<()> {
        self.with_location_write(py, |loc| loc.to = value)
    }
}

#[pyclass(dict, weakref, module = "triple_s", name = "Meta", from_py_object)]
pub struct PyMeta {
    inner: Py<PySurvey>,
}

impl PyMeta {
    fn from_rust(inner: &Bound<'_, PySurvey>) -> Self {
        Self {
            inner: inner.clone().unbind(),
        }
    }
}

impl Clone for PyMeta {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[pymethods]
impl PyMeta {
    #[new]
    #[pyo3(signature = (survey=None, version=None))]
    fn new(
        py: Python<'_>,
        survey: Option<&Bound<'_, PySurvey>>,
        version: Option<String>,
    ) -> PyResult<Self> {
        todo!()
        // let version = version
        //     .as_deref()
        //     .map(Version::from_str)
        //     .transpose()
        //     .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))?;

        // if let Some(survey) = survey {
        //     if let Some(version) = version {
        //         survey.borrow_mut().write_meta().version = version;
        //     }
        //     return Ok(Self::from_rust(survey));
        // }

        // let mut survey = tri::Survey::empty();
        // survey.meta = tri::Meta::with_version(version.unwrap_or(Version::V1_1));
        // Ok(Self {
        //     inner: Py::new(py, PySurvey::from(survey))?,
        // })
    }

    /// Export meta back to sss classic
    fn to_sss_classic(&self, py: Python<'_>) -> String {
        self.inner.bind(py).borrow().read_meta().to_sss_classic()
    }

    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);

        dict.set_item("version", self.get_version(py)?)?;

        if let Some(value) = self.get_date(py)? {
            dict.set_item("date", value)?;
        };

        if let Some(value) = self.get_time(py)? {
            dict.set_item("time", value)?;
        };

        if let Some(value) = self.get_origin(py)? {
            dict.set_item("origin", value)?;
        };

        if let Some(value) = self.get_user(py)? {
            dict.set_item("user", value)?;
        };

        if let Some(value) = self.get_standard(py)? {
            dict.set_item("standard", value)?;
        };

        Ok(dict.unbind())
    }

    /// SSS Version
    #[getter]
    fn get_version(&self, py: Python<'_>) -> PyResult<&str> {
        Ok(self.inner
            .bind(py)
            .borrow()
            .read_meta()
            .version
            .as_str())
    }

    #[setter]
    fn set_version(&self, py: Python<'_>, value: String) -> PyResult<()> {
        let version = Version::from_str(&value)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))?;
        self.inner
            .bind(py)
            .borrow_mut()
            .write_meta()
            .version = version;
        Ok(())
    }

    /// (Optional) The date the file was created.
    #[getter]
    fn get_date(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.inner
            .bind(py)
            .borrow()
            .read_meta()
            .date.clone())
    }
    #[setter]
    fn set_date(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner
            .bind(py)
            .borrow_mut()
            .write_meta()
            .date = value;
        Ok(())

    }

    /// (Optional) The time the file was created.
    #[getter]
    fn get_time(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.inner
            .bind(py)
            .borrow()
            .read_meta()
            .time
            .clone())
    }

    #[setter]
    fn set_time(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner
            .bind(py)
            .borrow_mut()
            .write_meta()
            .time = value;
        Ok(())
    }

    /// (Optional) The originating system (program and operating system), i.e., `Windows 95`
    #[getter]
    fn get_origin(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.inner
            .bind(py)
            .borrow()
            .read_meta()
            .origin.clone())
    }

    #[setter]
    fn set_origin(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner
            .bind(py)
            .borrow_mut()
            .write_meta()
            .origin = value;
        Ok(())
    }

    /// (Optional) Name of the user who created this file
    #[getter]
    fn get_user(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.inner
            .bind(py)
            .borrow()
            .read_meta()
            .user
            .clone())
    }

    #[setter]
    fn set_user(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner
            .bind(py)
            .borrow_mut()
            .write_meta()
            .user = value;
        Ok(())
    }

    #[getter]
    fn get_style(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.inner
            .bind(py)
            .borrow()
            .read_meta()
            .style.clone())
    }

    #[setter]
    fn set_style(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner
            .bind(py)
            .borrow_mut()
            .write_meta()
            .style = value;
        Ok(())
    }

    /// This statement is used to indicate that some aspect of the SSS definition meets a defined standard
    /// that may reduce the amount of checking required by an import program.
    /// The only standard_option provided in version 1.1 is NAMES. The definition of SSS standard names is:
    /// - consist of 1-8 characters
    /// - consist of only the characters A-Z (and a-z) and 0-9
    /// - start with a letter (A-Z or a-z)
    /// - case insensitive (i.e. upper and lower case are equivalent)
    /// - unique within the survey (e.g. Q1 and Q001 are different, q1 and Q1 are the same)
    /// - appear as the "name_text" without leading, trailing, or embedded blanks
    /// If all variable names in the SSS definition file conform to this definition then the statement STANDARD
    /// NAMES should appear between the VERSION and SURVEY statements.
    #[getter]
    fn get_standard(&self, py: Python<'_>) -> PyResult<Option<String>> {
        Ok(self.inner
            .bind(py)
            .borrow()
            .read_meta()
            .standard.clone())
    }

    #[setter]
    fn set_standard(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner
            .bind(py)
            .borrow_mut()
            .write_meta()
            .standard = value;
        Ok(())
    }
}

// // TODO!
// #[pyclass(dict, module = "triple_s", name = "Mode")]
// pub struct PyMode {
//     inner: Py<PySurvey>
// }

// /// Just a dummy class used for creating a python parent class for inheritance
// #[pyclass(dict, module = "triple_s", name = "Proxy")]
// pub struct PyProxy {}

/// helper functions that converts pyton index (can be negative) to rust index (usize)
fn normalize_index(index: isize, total: usize) -> PyResult<usize> {
    let index = if index < 0 {
        total as isize + index
    } else {
        index
    };

    if index < 0 || index as usize >= total {
        return Err(exceptions::PyIndexError::new_err("index out of range"));
    }

    Ok(index as usize)
}

fn format_response_items(item: &[Code<u64>]) -> String {
    let content = item
        .iter()
        .map(|choice| {
            let formatted_string = format_choice(choice);
            format!("{}: {}", choice.code, formatted_string)
        })
        .collect::<Vec<String>>();
    let content = content.join(", ");
    format!("{{{}}}", content)
}

#[inline]
fn format_choice(choice: &Code<u64>) -> String {
    let prefix = format!(
        r#"Choice(code={}, label="{:#?}""#,
        choice.code, choice.label
    );
    match choice.filter.as_ref() {
        Some(value) => format!(r#"{}, filter="{}")"#, prefix, value),
        None => format!("{})", prefix),
    }
}
