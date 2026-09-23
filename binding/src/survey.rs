extern crate triple_s;
use std::{str::FromStr, sync::Arc};

use parking_lot::{
    RawRwLock,
    RwLock,
    lock_api::{ArcRwLockReadGuard, ArcRwLockWriteGuard},
};
use pyo3::{
    exceptions,
    prelude::*,
    types::{PyList, PyTuple},
};
use rayon::prelude::*;
use triple_s::{
    self as sss,
    Language,
    Meta,
    Mode,
    Question,
    Record,
    Text,
    TextVariant,
    Version,
};

use crate::{
    error::{RustBackendError, to_backend_err},
};

#[pyfunction]
pub fn read_file(path: &str) -> PyResult<PySurvey> {
    let survey = sss::read_sss_file(path)
        .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))?;
    Ok(PySurvey::from(survey))
}

/// Survey
/// More detail can be found at: https://triple-s.org/documentation/manuals/
#[pyclass(module = "triple_s", name = "Survey", from_py_object)]
pub struct PySurvey {
    meta: Arc<RwLock<sss::Meta>>, // holy hell this works
    record: Arc<RwLock<sss::Record>>,
}

impl Clone for PySurvey {
    fn clone(&self) -> Self {
        Self {
            meta: Arc::new(RwLock::new(self.meta.read().clone())),
            record: Arc::new(RwLock::new(self.record.read().clone())),
        }
    }
}

impl From<sss::Survey> for PySurvey {
    fn from(value: sss::Survey) -> Self {
        Self {
            meta: value.meta,
            record: Arc::new(RwLock::new(value.record)),
        }
    }
}

// impl PySurvey {
//     /// Acquire the inner 'survey.meta' for reading; surfaces lock poisoning as a
//     /// `PyException` instead of panicking.
//     pub(crate) fn read_meta(&self) -> MappedRwLockReadGuard<'_, sss::Meta> {
//         parking_lot::RwLockReadGuard::map(self.inner.read(), |survey| &survey.meta)
//     }

//     /// Acquire the inner 'survey.meta' for writing; surfaces lock poisoning as a
//     /// `PyException` instead of panicking.
//     pub(crate) fn write_meta(&self) -> MappedRwLockWriteGuard<'_, sss::Meta> {
//         parking_lot::RwLockWriteGuard::map(self.inner.write(), |survey| &mut survey.meta)
//     }

//     /// Acquire the inner 'survey.record' for reading; surfaces lock poisoning as a
//     /// `PyException` instead of panicking.
//     pub(crate) fn read_record(&self) -> MappedRwLockReadGuard<'_, sss::Record> {
//         parking_lot::RwLockReadGuard::map(self.inner.read(), |survey| &survey.record)
//     }

//     /// Acquire the inner 'survey.record' for writing; surfaces lock poisoning as a
//     /// `PyException` instead of panicking.
//     pub(crate) fn write_record(&self) -> MappedRwLockWriteGuard<'_, sss::Record> {
//         parking_lot::RwLockWriteGuard::map(self.inner.write(), |survey| &mut survey.record)
//     }

//     /// Acquire the inner 'survey.question' for reading; surfaces lock poisoning as a
//     /// `PyException` instead of panicking.
//     pub(crate) fn read_question(
//         &self,
//         index: usize,
//     ) -> PyResult<MappedRwLockReadGuard<'_, sss::Question>> {
//         parking_lot::RwLockReadGuard::try_map(self.inner.read(), |survey| {
//             survey.record.content.get(index)
//         })
//         .map_err(|_| exceptions::PyIndexError::new_err(index))
//     }

//     /// Acquire the inner 'survey.question' for writing; surfaces lock poisoning as a
//     /// `PyException` instead of panicking.
//     pub(crate) fn write_question(
//         &self,
//         index: usize,
//     ) -> PyResult<MappedRwLockWriteGuard<'_, sss::Question>> {
//         parking_lot::RwLockWriteGuard::try_map(self.inner.write(), |survey| {
//             survey.record.content.get_mut(index)
//         })
//         .map_err(|_| exceptions::PyIndexError::new_err(index))
//     }
// }

#[pymethods]
impl PySurvey {
    //     /// Export content back to sss classic
    //     fn to_sss_classic(&self) -> PyResult<String> {
    //         self.inner
    //             .read()
    //             .to_sss_classic()
    //             .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))
    //     }

    /// return s a deep copy of current instance
    fn copy(&self) -> Self {
        self.clone()
    }

    #[getter]
    fn get_meta<'py>(&self) -> PyResult<PyMeta> {
        Ok(PyMeta::from_parent(&self))
    }

    // #[getter]
    // fn get_meta<'py>(inner: &Bound<'_, Self>) -> PyResult<PyMeta> {
    // Ok(PyMeta::from_rust(inner))
    // }

    #[getter]
    fn get_title<'py>(&self) -> Option<String> {
        self.meta.read().title.clone()
    }

    #[setter]
    fn set_title<'py>(&self, title: Option<String>) {
        self.meta.write().title = title;
    }

    #[getter]
    fn get_record<'py>(&self) -> PyResult<PyRecord> {
        Ok(PyRecord::from_parent(&self))
    }

    #[getter]
    fn get_content<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let content = self.record
            .read()
            .content
            .clone()
            .into_iter()
            .map(|question| PyQuestion::new(&question));
        PyTuple::new(py, content)
    }

    //     fn get_question(survey: &Bound<'_, Self>, question_name: &str) -> PyResult<PyQuestion> {
    //         let query = question_name.to_uppercase();
    //         survey
    //             .borrow()
    //             .read_record()
    //             .content
    //             .iter()
    //             .enumerate()
    //             .find(|(_, question)| question.name.to_uppercase() == query)
    //             .map(|(index, _)| PyQuestion::from_inner(&survey.clone().unbind(), index))
    //             .ok_or(exceptions::PyKeyError::new_err(
    //                 question_name.to_string(),
    //             ))
    //     }

    //     fn __len__(&self) -> PyResult<usize> {
    //         let length = self.read_record().content.len();
    //         Ok(length)
    //     }

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
pub struct PyRecord (Arc<RwLock<Record>>);

impl PyRecord {
    pub fn from_parent(survey: &PySurvey) -> Self {
        Self(survey.record.clone())
    }

    pub(crate) fn read(&self) -> ArcRwLockReadGuard<RawRwLock, Record> {
        self.0.read_arc()
    }

    pub(crate) fn write(&self) -> ArcRwLockWriteGuard<RawRwLock, Record> {
        self.0.write_arc()
    }

    //     pub fn len(&self, py: Python<'_>) -> usize {
    //         match &self.inner {
    //             Storage::Attached { inner, .. } => inner.borrow(py).read_record().content.len(),
    //             Storage::Owned(item) => item.read().content.len(),
    //         }
    //     }

    //     fn with_record_read<R>(
    //         &self,
    //         py: Python<'_>,
    //         f: impl FnOnce(MappedRwLockReadGuard<'_, sss::Record>) -> R,
    //     ) -> PyResult<R> {
    //         match &self.inner {
    //             Storage::Attached { inner, .. } => {
    //                 let item = inner.bind(py).borrow();
    //                 let item = item.read_record();
    //                 Ok(f(item))
    //             }
    //             Storage::Owned(item) => {
    //                 let item = item.read();
    //                 let item = parking_lot::RwLockReadGuard::map(item, |item| item);
    //                 Ok(f(item))
    //             }
    //         }
    // }

    //     fn with_record_write<R>(
    //         &self,
    //         py: Python<'_>,
    //         f: impl FnOnce(MappedRwLockWriteGuard<'_, sss::Record>) -> R,
    //     ) -> PyResult<R> {
    //         match &self.inner {
    //             Storage::Attached { inner, .. } => {
    //                 let item = inner.bind(py).borrow_mut();
    //                 let item = item.write_record();
    //                 Ok(f(item))
    //             }
    //             Storage::Owned(item) => {
    //                 let item = item.write();
    //                 let item = parking_lot::RwLockWriteGuard::map(item, |item| item);
    //                 Ok(f(item))
    //             }
    //         }
    //     }
}

#[pymethods]
impl PyRecord {
    //     /// Export content back to sss classic
    //     fn to_sss_classic(&self, py: Python<'_>) -> PyResult<String> {
    //         self.with_record_read(py, |record| record.to_sss_classic())?
    //             .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))
    //     }

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

    #[setter]
    fn set_id(&mut self, value: char) -> PyResult<()> {
        self.write().id = value;
        Ok(())
    }

    /// List of questions/ variables that the current survey instance contains
    #[getter]
    fn get_content(&self) -> Vec<PyQuestion> {
        todo!()
    }

    //     /// List of questions/ variables that the current survey instance contains
    //     #[getter]
    //     fn content(&self, py: Python<'_>) -> PyResult<Vec<PyQuestion>> {
    //         match &self.inner {
    //             Storage::Attached { inner, .. } => Ok((0..self.len(py))
    //                 .map(|index| PyQuestion::from_inner(&inner, index))
    //                 .collect()),
    //             Storage::Owned(item) => Ok(item
    //                 .read()
    //                 .content
    //                 .iter()
    //                 .map(PyQuestion::from_storage)
    //                 .collect()),
    //         }
    //     }

    //     fn __getitem__(&self, py: Python<'_>, index: isize) -> PyResult<PyQuestion> {
    //         let length = self.content(py)?.len();
    //         let lookup = normalize_index(index, length)?; // also checks if index really exists, raise IndexError if otherwise
    //         match &self.inner {
    //             Storage::Attached { inner, .. } => Ok(PyQuestion::from_inner(&inner, lookup)),
    //             Storage::Owned(item) => item
    //                 .read()
    //                 .content
    //                 .get(lookup)
    //                 .map(PyQuestion::from_storage)
    //                 .ok_or(exceptions::PyIndexError::new_err(index)),
    //         }
    //     }

    //     fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
    //         Ok(self.len(py))
    //     }

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

#[pyclass(module = "triple_s", name = "Question")]
pub struct PyQuestion {
    inner: Arc<RwLock<Question>>,
}

// #[pyclass(module = "triple_s", name = "Question")]
// pub struct PyQuestion {
//     inner: Storage<PySurvey, sss::Question>,
// }

impl PyQuestion {
    pub fn new(item: &Arc<RwLock<Question>>) -> Self {
        Self {
            inner: item.clone(),
        }
    }

    //     pub fn from_inner(inner: &Py<PySurvey>, index: usize) -> Self {
    //         Self {
    //             inner: Storage::Attached {
    //                 inner: inner.clone(),
    //                 index: index as u64,
    //             },
    //         }
    //     }

    //     fn with_question_read<R>(
    //         &self,
    //         py: Python<'_>,
    //         f: impl FnOnce(MappedRwLockReadGuard<'_, sss::Question>) -> R,
    //     ) -> PyResult<R> {
    //         match &self.inner {
    //             Storage::Attached { inner, index } => {
    //                 let inner = inner.bind(py).borrow();
    //                 let index = (*index).try_into().map_err(|_| {
    //                     PyErr::new::<exceptions::PyIndexError, _>("index is too large")
    //                 })?;
    //                 let question = inner.read_question(index)?;

    //                 Ok(f(question))
    //             }
    //             Storage::Owned(item) => {
    //                 let item = item.read();
    //                 let item = parking_lot::RwLockReadGuard::map(item, |item| item);
    //                 Ok(f(item))
    //             }
    //         }
    //     }

    //     fn with_question_write<R>(
    //         &self,
    //         py: Python<'_>,
    //         f: impl FnOnce(MappedRwLockWriteGuard<'_, sss::Question>) -> R,
    //     ) -> PyResult<R> {
    //         match &self.inner {
    //             Storage::Attached { inner, index } => {
    //                 let inner = inner.bind(py).borrow_mut();
    //                 let index = (*index).try_into().map_err(|_| {
    //                     PyErr::new::<exceptions::PyIndexError, _>("index is too large")
    //                 })?;
    //                 let question = inner.write_question(index)?;
    //                 Ok(f(question))
    //             }
    //             Storage::Owned(item) => {
    //                 let item = item.write();
    //                 let item = parking_lot::RwLockWriteGuard::map(item, |item| item);
    //                 Ok(f(item))
    //             }
    //         }
    //     }

    //     pub(crate) fn with_location_read<R>(
    //         &self,
    //         py: Python<'_>,
    //         f: impl FnOnce(&sss::Position) -> R,
    //     ) -> PyResult<R> {
    //         self.with_question_read(py, |question| f(&question.location))
    //     }

    //     pub(crate) fn with_location_write<R>(
    //         &self,
    //         py: Python<'_>,
    //         f: impl FnOnce(&mut sss::Position) -> R,
    //     ) -> PyResult<R> {
    //         self.with_question_write(py, |mut question| f(&mut question.location))
    //     }

    //     pub(crate) fn with_response<R>(
    //         &self,
    //         py: Python<'_>,
    //         f: impl FnOnce(&sss::Response) -> R,
    //     ) -> PyResult<R> {
    //         self.with_question_read(py, |question| f(&question.response))
    //     }

    //     pub(crate) fn with_response_mut<R>(
    //         &self,
    //         py: Python<'_>,
    //         f: impl FnOnce(&mut sss::Response) -> R,
    //     ) -> PyResult<R> {
    //         self.with_question_write(py, |mut question| f(&mut question.response))
    //     }
}

#[pymethods]
impl PyQuestion {
    fn to_sss_classic(&self) -> PyResult<String> {
        self.inner
            .read()
            .to_sss_classic()
            .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            r#"Question(name="{}")"#,
            self.inner.read().name
        ))
    }

    #[getter]
    fn get_name(&self) -> PyResult<String> {
        Ok(self.inner.read().name.clone())
    }

    #[setter]
    fn set_name(&self, value: &str) -> PyResult<()> {
        self.inner.write().name = value.into();
        Ok(())
    }

    #[getter]
    fn get_label(&self) -> PyResult<PyText> {
        let content = self.inner.read().label.clone();
        let output = PyText::from(content);
        Ok(output)
    }

    // #[setter]
    // fn set_label(&self, label: &Bound<'_, PyAny>) -> PyResult<()> {
    //     if let Ok(content) = label.extract::<String>() {
    //         if self.get_label()?.has_variants() {
    //             todo!()
    //         } else {
    //             todo!()
    //         }
    //     } else if let Ok(content) = label.extract::<PyText>() {
    //         todo!()
    //     }
    // }

    //     #[getter]
    //     fn get_filter(&self, py: Python<'_>) -> PyResult<Option<String>> {
    //         self.with_question_read(py, |question| question.filter.clone())
    //     }

    //     #[setter]
    //     fn set_filter(&self, py: Python<'_>, value: Option<&str>) -> PyResult<()> {
    //         let value = value.map(|s| s.to_string());
    //         self.with_question_write(py, |mut question| question.filter = value)
    //     }

    //     #[getter]
    //     fn get_response(question: &Bound<'_, Self>) -> PyResult<PyResponse> {
    //         Ok(PyResponse::from_inner(question))
    //     }

    //     #[getter]
    //     pub fn get_location(question: &Bound<'_, Self>) -> PyResult<PyLocation> {
    //         Ok(PyLocation::from_inner(question))
    //     }

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

// #[pyclass(module = "triple_s", name = "Response", subclass)]
// pub struct PyResponse {
//     inner: Storage<PyQuestion, sss::Response>,
// }

// impl PyResponse {
//     pub fn from_inner(question: &Bound<'_, PyQuestion>) -> Self {
//         Self {
//             inner: Storage::Attached { inner: question.clone().unbind(), index: 0}
//         }
//     }

//     fn with_response<R>(
//         &self,
//         py: Python<'_>,
//         f: impl FnOnce(&sss::Response) -> R,
//     ) -> PyResult<R> {
//         match &self.inner {
//             Storage::Attached {inner, ..} => {
//                 inner.bind(py).borrow().with_response(py, |response| f(&response))
//             }
//             Storage::Owned(inner) => {
//                 Ok(f(&inner.read()))
//             }
//         }
//     }

//     fn with_response_mut<R>(
//         &mut self,
//         py: Python<'_>,
//         f: impl FnOnce(&mut sss::Response) -> R,
//     ) -> PyResult<R> {
//         match &mut self.inner {
//             Storage::Attached {inner, ..} => {
//                 inner.bind(py).borrow_mut().with_response_mut(py, |mut response| f(&mut response))
//             }
//             Storage::Owned(inner) => {
//                 Ok(f(&mut inner.write()))
//             }
//         }
//     }
// }

// #[pymethods]
// impl PyResponse {
//     #[getter]
//     fn item(slf: &Bound<'_, Self>) -> PyResult<Option<Vec<PyCode>>> {
//         let py = slf.py();
//         slf.borrow().with_response(py, |response| {
//             match response {
//                 Response::NumericCategory(item) => {
//                     let items = (0..item.read().values.len())
//                         .map(|idx| PyCode::from_inner(slf, idx as u64))
//                         .collect::<Vec<PyCode>>();
//                     Some(items)
//                 }
//                 Response::TextCategory(item) => {
//                     let items = (0..item.read().values.len())
//                         .map(|idx| PyCode::from_inner(slf, idx as u64))
//                         .collect::<Vec<PyCode>>();
//                     Some(items)
//                 }
//                 _ => None
//             }
//         })
//     }

//     /// alias for `item`
//     #[getter]
//     fn content(slf: &Bound<'_, Self>) -> PyResult<Option<Vec<PyCode>>> {
//         Self::item(slf)
//     }
// }

// /// the attribute `code` in triple-s <value> tag can be u64, f64 (non-NAN and non-inf) & String
// /// hence the use of enum `CodeInterface` here to accomdate different datatypes
// enum CodeInterface {
//     Int(Code<u64>),
//     Float(Code<f64>),
//     String(Code<String>)
// }

// /// A possible answer that a respondent can choose in a single-/ multi- response question
// #[pyclass(module = "triple_s", name = "Choice")]
// pub struct PyCode {
//     storage: Storage<PyResponse, CodeInterface>,
// }

// impl PyCode {
//     pub(crate) fn with_integer_code<R>(
//         &self,
//         py: Python<'_>,
//         f: impl FnOnce(&Code<u64>) -> R,
//     ) -> PyResult<R> {
//         match &self.storage {
//             Storage::Attached { inner, index } => inner
//                 .bind(py)
//                 .try_borrow()?
//                 .with_response(py, |response| match response {
//                     Response::NumericCategory(member) => member
//                         .read()
//                         .values
//                         .get(*index as usize)
//                         .ok_or_else(|| exceptions::PyIndexError::new_err(*index))
//                         .map(f),
//                     _ => Err(exceptions::PyTypeError::new_err(
//                         "Expected a NumericCategory response",
//                     )),
//                 })?,
//             // Owned choices store their code at index zero.
//             Storage::Owned(item) => match &*item.read() {
//                 CodeInterface::Int(value) => Ok(f(value)),
//                 _ => Err(exceptions::PyTypeError::new_err("Expected integer codes")),
//             },
//         }
//     }

//     pub(crate) fn with_integer_code_mute<R>(
//         &mut self,
//         py: Python<'_>,
//         f: impl FnOnce(&mut Code<u64>) -> R,
//     ) -> PyResult<R> {
//         match &mut self.storage {
//             Storage::Attached { inner, index } => inner
//                 .bind(py)
//                 .try_borrow_mut()?
//                 .with_response_mut(py, |response| match response {
//                     Response::NumericCategory(member) => member
//                         .write()
//                         .values
//                         .get_mut(*index as usize)
//                         .ok_or_else(|| exceptions::PyIndexError::new_err(*index))
//                         .map(f),
//                     _ => Err(exceptions::PyTypeError::new_err(
//                         "Expected a NumericCategory response",
//                     )),
//                 })?,
//             // Owned choices store their code at index zero.
//             Storage::Owned(item) => match &mut *item.write() {
//                 CodeInterface::Int(value) => Ok(f(value)),
//                 _ => Err(exceptions::PyTypeError::new_err("Expected integer codes")),
//             },
//         }
//     }

//     pub(crate) fn with_string_code<R>(
//         &self,
//         py: Python<'_>,
//         f: impl FnOnce(&Code<String>) -> R,
//     ) -> PyResult<R> {
//         match &self.storage {
//             Storage::Attached { inner, index } => inner
//                 .bind(py)
//                 .try_borrow()?
//                 .with_response(py, |response| match response {
//                     Response::TextCategory(member) => member
//                         .read()
//                         .values
//                         .get(*index as usize)
//                         .ok_or_else(|| exceptions::PyIndexError::new_err(*index))
//                         .map(f),
//                     _ => Err(exceptions::PyTypeError::new_err(
//                         "Expected a TextCategory response",
//                     )),
//                 })?,
//             // Owned choices store their code at index zero.
//             Storage::Owned(item) => match &*item.read() {
//                 CodeInterface::String(value) => Ok(f(value)),
//                 _ => Err(exceptions::PyTypeError::new_err("Expected string codes")),
//             },
//         }
//     }

//     pub(crate) fn with_string_code_mute<R>(
//         &mut self,
//         py: Python<'_>,
//         f: impl FnOnce(&mut Code<String>) -> R,
//     ) -> PyResult<R> {
//         match &mut self.storage {
//             Storage::Attached { inner, index } => inner
//                 .bind(py)
//                 .try_borrow_mut()?
//                 .with_response_mut(py, |response| match response {
//                     Response::TextCategory(member) => member
//                         .write()
//                         .values
//                         .get_mut(*index as usize)
//                         .ok_or_else(|| exceptions::PyIndexError::new_err(*index))
//                         .map(f),
//                     _ => Err(exceptions::PyTypeError::new_err(
//                         "Expected a TextCategory response",
//                     )),
//                 })?,
//             // Owned choices store their code at index zero.
//             Storage::Owned(item) => match &mut *item.write() {
//                 CodeInterface::String(value) => Ok(f(value)),
//                 _ => Err(exceptions::PyTypeError::new_err("Expected string codes")),
//             },
//         }
//     }

//     pub fn from_inner(response: &Bound<'_, PyResponse>, index: u64) -> Self {
//         Self {
//             storage: Storage::Attached {
//                 inner: response.clone().unbind(),
//                 index,
//             },
//         }
//     }
// }

// #[pymethods]
// impl PyCode {
//     #[new]
//     #[pyo3(signature = (code, label))]
//     fn new(
//         code: &Bound<'_, PyAny>,
//         label: String,
//     ) -> PyResult<Self>{
//         let inner = if let Ok(code) = code.extract::<u64>() {
//             let item = Code::new(code, label)
//                 .map_err(|err| exceptions::PyRuntimeError::new_err(err.backtrace().to_string()))?;
//             CodeInterface::Int(item)
//         } else if let Ok(code) = code.extract::<String>() {
//             let item = Code::new(code, label)
//                 .map_err(|err| exceptions::PyRuntimeError::new_err(err.backtrace().to_string()))?;
//             CodeInterface::String(item)
//         } else if let Ok(code) = code.extract::<f64>() {
//             let item = Code::new(code, label)
//                 .map_err(|err| exceptions::PyRuntimeError::new_err(err.backtrace().to_string()))?;
//             CodeInterface::Float(item)
//         } else {
//             return Err(pyo3::exceptions::PyTypeError::new_err(
//                     "code must be an int, float or str",
//                 ))
//         };
//         Ok(Self { storage: Storage::from_owned(inner)} )
//     }

//     // todo! need to acoomdate different data types!!
//     fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
//         todo!()
//     }

//     /// Code used to uniquely identify the current answer option.
//     #[getter]
//     fn code(&self, py: Python<'_>) -> PyResult<u64> {
//         todo!()
//     }

//     /// Text label of the current answer option
//     #[getter]
//     fn label(&self, py: Python<'_>) -> PyResult<String> {
//         todo!()
//     }

//     #[setter]
//     fn set_label(&mut self, py: Python<'_>, value: &str) -> PyResult<()> {
//         todo!()
//     }

//     /// Response filter of the current answer option.
//     /// This attribute is not part of the .sss file itself, but can be very useful for extending current library to read other survey data maps modified from .sss xml (i.e., `.sif` from SnapSurveys, `.qps` from QPSMR etc).
//     /// An exmaple use case be setting up something like this:
//     /// Q1: How often do you drive?
//     /// choice 1: Never
//     /// choice 2: Always
//     ///
//     /// Q2: Do you enjoy driving?
//     /// choice 1: nah (ONLY shown for those who selected '2' in Q1)
//     /// choice 2: okay-ish (ONLY shown for those who selected '2' in Q1)
//     /// choice 3: very much
//     #[getter]
//     fn filter(&self, py: Python<'_>) -> PyResult<Option<String>> {
//         self.with_integer_code(py, |choice| choice.filter.clone())
//     }

//     #[setter]
//     fn set_filter(&mut self, py: Python<'_>, value: Option<&str>) -> PyResult<()> {
//         self.with_integer_code_mute(py, |choice| {
//             choice.filter = value.map(|s| s.to_string())
//         })
//     }

//     /// (Optional) indicator for special values (e.g. mannotationissing data codes).
//     /// Removed from SSS 1.2 onwards.
//     #[getter]
//     fn is_special(&self, py: Python<'_>) -> PyResult<bool> {
//         self.with_integer_code(py, |choice| choice.is_special)
//     }

//     #[setter]
//     fn set_is_special(&mut self, py: Python<'_>, value: bool) -> PyResult<()> {
//         self.with_integer_code_mute(py, |choice| choice.is_special = value)
//     }
// }

// #[pyclass(module = "triple_s", name = "Location")]
// pub struct PyLocation {
//     inner: Storage<PyQuestion, Position>,
// }

// impl PyLocation {
//     pub(crate) fn from_inner(question: &Bound<'_, PyQuestion>) -> Self {
//         Self {
//             inner: Storage::Attached {
//                 inner: question.clone().unbind(),
//                 index: 0,
//             },
//         }
//     }

//     fn with_location_read<R>(
//         &self,
//         py: Python<'_>,
//         f: impl FnOnce(&sss::Position) -> R,
//     ) -> PyResult<R> {
//         match &self.inner {
//             Storage::Owned(location) => {
//                 let location = location.read();
//                 Ok(f(&location))
//             }

//             Storage::Attached { inner, .. } => {
//                 inner.bind(py).borrow().with_location_read(py, f)
//             }
//         }
//     }

//     fn with_location_write<R>(
//         &self,
//         py: Python<'_>,
//         f: impl FnOnce(&mut sss::Position) -> R,
//     ) -> PyResult<R> {
//         match &self.inner {
//             Storage::Owned(location) => {
//                 let mut location = location.write();
//                 Ok(f(&mut *location))
//             }

//             Storage::Attached { inner, .. } => {
//                 inner.bind(py).borrow_mut().with_location_write(py, f)
//             }
//         }
//     }
// }

// #[pymethods]
// impl PyLocation {
//     #[new]
//     #[pyo3(signature = (from, to))]
//     fn new(from: u64, to: u64) -> Self {
//         Self {
//             inner: Storage::Owned(Arc::new(RwLock::new(Position { from, to }))),
//         }
//     }

//     fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
//         self.with_location_read(py, |loc| {
//             format!("Location(from={}, to={})", loc.from, loc.to)
//         })
//     }

//     #[getter]
//     fn get_from(&self, py: Python<'_>) -> PyResult<u64> {
//         self.with_location_read(py, |loc| loc.from.clone())
//     }

//     #[setter]
//     fn set_from(&self, py: Python<'_>, value: u64) -> PyResult<()> {
//         self.with_location_write(py, |loc| loc.from = value)
//     }

//     #[getter]
//     fn get_to(&self, py: Python<'_>) -> PyResult<u64> {
//         self.with_location_read(py, |loc| loc.to.clone())
//     }

//     #[setter]
//     fn set_to(&self, py: Python<'_>, value: u64) -> PyResult<()> {
//         self.with_location_write(py, |loc| loc.to = value)
//     }
// }

#[pyclass(dict, weakref, module = "triple_s", name = "Meta", from_py_object)]
pub struct PyMeta {
    inner: Arc<RwLock<Meta>>,
}

impl Clone for PyMeta {
    fn clone(&self) -> Self {
        let instance = self.inner.read().clone();
        Self {
            inner: Arc::new(RwLock::new(instance)),
        }
    }
}

impl PyMeta {
    pub fn from_parent(survey: &PySurvey) -> Self {
        Self {
            inner: survey.meta.clone(),
        }
    }
}

#[pymethods]
impl PyMeta {
    //     #[new]
    //     #[pyo3(signature = (survey=None, version=None))]
    //     fn new(
    //         py: Python<'_>,
    //         survey: Option<&Bound<'_, PySurvey>>,
    //         version: Option<String>,
    //     ) -> PyResult<Self> {
    //         todo!()
    //         // let version = version
    //         //     .as_deref()
    //         //     .map(Version::from_str)
    //         //     .transpose()
    //         //     .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))?;

    //         // if let Some(survey) = survey {
    //         //     if let Some(version) = version {
    //         //         survey.borrow_mut().write_meta().version = version;
    //         //     }
    //         //     return Ok(Self::from_rust(survey));
    //         // }

    //         // let mut survey = sss::Survey::empty();
    //         // survey.meta = sss::Meta::with_version(version.unwrap_or(Version::V1_1));
    //         // Ok(Self {
    //         //     inner: Py::new(py, PySurvey::from(survey))?,
    //         // })
    //     }

    //     /// Export meta back to sss classic
    //     fn to_sss_classic(&self, py: Python<'_>) -> String {
    //         self.inner.bind(py).borrow().read_meta().to_sss_classic()
    //     }

    //     fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
    //         let dict = PyDict::new(py);

    //         dict.set_item("version", self.get_version(py)?)?;

    //         if let Some(value) = self.get_date(py)? {
    //             dict.set_item("date", value)?;
    //         };

    //         if let Some(value) = self.get_time(py)? {
    //             dict.set_item("time", value)?;
    //         };

    //         if let Some(value) = self.get_origin(py)? {
    //             dict.set_item("origin", value)?;
    //         };

    //         if let Some(value) = self.get_user(py)? {
    //             dict.set_item("user", value)?;
    //         };

    //         if let Some(value) = self.get_standard(py)? {
    //             dict.set_item("standard", value)?;
    //         };

    //         Ok(dict.unbind())
    //     }

    /// make a deep copy of current instance
    fn copy(&self) -> Self {
        self.clone()
    }

    /// SSS Version
    #[getter]
    fn get_version(&self) -> PyResult<&str> {
        Ok(self.inner.read().version.as_str())
    }

    #[setter]
    fn set_version(&self, value: String) -> PyResult<()> {
        let version = Version::from_str(&value)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))?;
        self.inner.write().version = version;
        Ok(())
    }

    /// (Optional) name fo the survey
    #[getter]
    fn get_name(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().name.clone())
    }

    #[setter]
    fn set_name(&self, value: Option<String>) -> PyResult<()> {
        self.inner.write().name = value;
        Ok(())
    }

    /// (Optional) The date the file was created.
    #[getter]
    fn get_date(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().date.clone())
    }

    #[setter]
    fn set_date(&self, value: Option<String>) -> PyResult<()> {
        self.inner.write().date = value;
        Ok(())
    }

    /// (Optional) The time the file was created.
    #[getter]
    fn get_time(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().time.clone())
    }

    #[setter]
    fn set_time(&self, value: Option<String>) -> PyResult<()> {
        self.inner.write().time = value;
        Ok(())
    }

    #[getter]
    fn get_title(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().title.clone())
    }

    #[setter]
    fn set_title(&self, value: Option<String>) -> PyResult<()> {
        self.inner.write().title = value;
        Ok(())
    }

    /// (Optional) The originating system (program and operating system), i.e., `Windows 95`
    #[getter]
    fn get_origin(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().origin.clone())
    }

    #[setter]
    fn set_origin(&self, value: Option<String>) -> PyResult<()> {
        self.inner.write().origin = value;
        Ok(())
    }

    /// (Optional) Name of the user who created this file
    #[getter]
    fn get_user(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().user.clone())
    }

    #[setter]
    fn set_user(&self, value: Option<String>) -> PyResult<()> {
        self.inner.write().user = value;
        Ok(())
    }

    #[getter]
    fn get_style(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().style.clone())
    }

    #[setter]
    fn set_style(&self, value: Option<String>) -> PyResult<()> {
        self.inner.write().style = value;
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
    fn get_standard(&self) -> PyResult<Option<String>> {
        Ok(self.inner.read().standard.clone())
    }

    #[setter]
    fn set_standard(&self, value: Option<String>) -> PyResult<()> {
        self.inner.write().standard = value;
        Ok(())
    }
}

// ============================= Enums/ Flags =============================
#[pyclass(name = "Text", module = "triple_s")]
pub struct PyText(Arc<RwLock<Text>>);

impl From<Text> for PyText {
    fn from(value: Text) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }
}

impl From<String> for PyText {
    fn from(value: String) -> Self {
        let content = Text::new(value);
        content.into()
    }
}

impl TryFrom<PyText> for Text {
    type Error = pyo3::PyErr;

    fn try_from(value: PyText) -> Result<Self, Self::Error> {
        let inside = Arc::into_inner(value.0)
            .ok_or(RustBackendError::new_err("Unable to move instance of `Text` out of `PyText` because there are some other threads are still accessing it."))?;
        Ok(inside.into_inner())
    }
}

impl PyText {
    pub fn from_text_instance(value: Arc<RwLock<Text>>) -> Self {
        Self(value)
    }

    pub fn read(&self) -> ArcRwLockReadGuard<RawRwLock, Text> {
        self.0.read_arc()
    }

    pub fn write(&mut self) -> ArcRwLockWriteGuard<RawRwLock, Text> {
        self.0.write_arc()
    }

    fn try_insert_variant(&mut self, variant: TextVariant) -> PyResult<()> {
        if !self.read().contains_variant(&variant) {
            self.write().set_variant(variant);
            Ok(())
        } else {
            let msg_mode = match variant.mode {
                Mode::None => "".into(),
                Mode::Analysis => "mode 'analysis'",
                Mode::Interview => "mode 'interview'",
            };
            let msg_lan = variant
                .language
                .as_ref()
                .map_or_default(|lan| format!("language '{}'", lan.to_string()));
            let delimiter = if (variant.mode != Mode::None) & (variant.language.as_ref().is_some()) {
                " and "
            } else {
                ""
            };
            let message = format!(
                "Unable to add text variant because variant with {}{}{} already exists! Call `set_variant` to overwrite existing text variant.",
                msg_mode, delimiter, msg_lan
            );
            Err(exceptions::PyValueError::new_err(message))
        }
    }

    fn insert_or_overwrite_variant(&mut self, variant: TextVariant) {
        self.write().set_variant(variant);
    }
}

#[pymethods]
impl PyText {
    #[new]
    #[pyo3(signature=(content, *variants:"Variant"))]
    fn new(content: String, variants: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let mut instance: Self = Text::new(content).into();
        instance.write().variant = variants
            .extract::<Vec<PyVariant>>()?
            .into_par_iter()
            .map(TextVariant::try_from)
            .collect::<PyResult<Vec<TextVariant>>>()?;

        Ok(instance)
    }

    /// checks if there were any text variants
    fn has_variants(&self) -> bool {
        self.0.read().variant.len() > 0
    }

    #[getter]
    fn get_variants<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let variants = self
            .read()
            .variant
            .par_iter()
            .map(|item| PyVariant::from(item.clone()))
            .collect::<Vec<PyVariant>>();

        PyTuple::new(py, variants)
    }

    #[setter]
    fn set_variants(&mut self, variants: &Bound<'_, PyList>) -> PyResult<()> {
        self.write().variant = variants
            .extract::<Vec<PyVariant>>()?
            .into_par_iter()
            .map(TextVariant::try_from)
            .collect::<PyResult<Vec<TextVariant>>>()?;

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
    #[pyo3(signature=(variant:"Variant|str", mode=None, language=None))]
    fn add_variant(
        &mut self,
        variant: &Bound<'_, PyAny>,
        mode: Option<String>,
        language: Option<String>,
    ) -> PyResult<()> {
        if let Ok(variant) = variant.extract::<PyVariant>() {
            let variant = TextVariant::try_from(variant)?;
            self.try_insert_variant(variant)
        } else if let Ok(content) = variant.extract::<String>() {
            let pyvariant = PyVariant::new(content, language, mode)?;
            let variant = TextVariant::try_from(pyvariant)?;
            self.try_insert_variant(variant)
        } else {
            Err(exceptions::PyValueError::new_err(
                "variant must be a Variant or str",
            ))
        }
    }

    /// Add a text variant, or overwrite it if variant with same `mode` or `language` exists.
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
    ///         obj.set_variant(variant)
    ///
    ///     Create and add a variant from text:
    ///
    ///         obj.set_variant("Hello world", mode="interview", language="en")
    #[pyo3(signature=(variant:"Variant|str", mode=None, language=None))]
    fn set_variant(
        &mut self,
        variant: &Bound<'_, PyAny>,
        mode: Option<String>,
        language: Option<String>,
    ) -> PyResult<()> {
        if let Ok(variant) = variant.extract::<PyVariant>() {
            let variant = TextVariant::try_from(variant)?;
            self.insert_or_overwrite_variant(variant);
            Ok(())
        } else if let Ok(content) = variant.extract::<String>() {
            let pyvariant = PyVariant::new(content, language, mode)?;
            let variant = TextVariant::try_from(pyvariant)?;
            self.insert_or_overwrite_variant(variant);
            Ok(())
        } else {
            Err(exceptions::PyValueError::new_err(
                "variant must be a Variant or str",
            ))
        }
    }
}

#[pyclass(name = "Variant", module = "triple_s", from_py_object)]
#[derive(Debug, Clone)]
pub struct PyVariant(Arc<RwLock<TextVariant>>);

impl From<TextVariant> for PyVariant {
    fn from(value: TextVariant) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }
}

impl TryFrom<PyVariant> for TextVariant {
    type Error = pyo3::PyErr;

    fn try_from(value: PyVariant) -> Result<Self, Self::Error> {
        let inside = Arc::into_inner(value.0)
            .ok_or(exceptions::PyRuntimeError::new_err("Unable to move instance of `TextVariant` out of `PyVariant` because there are some other threads are still accessing it."))?;
        Ok(inside.into_inner())
    }
}

#[pymethods]
impl PyVariant {
    #[new]
    #[pyo3(signature=(content, language=None, mode=None))]
    fn new(content: String, language: Option<String>, mode: Option<String>) -> PyResult<Self> {
        if mode.is_none() & language.is_none() {
            return Err(exceptions::PyValueError::new_err(
                "At least one language or a mode must be provided",
            ));
        };

        let language = language.map(Language::new);
        let mode = mode
            .map(|s| Mode::from_str(&s))
            .transpose()
            .map_err(|err| to_backend_err(err.into()))?
            .unwrap_or_default();

        let instance = TextVariant {
            value: content,
            language: language,
            mode: mode,
        };

        Ok(instance.into())
    }
}
