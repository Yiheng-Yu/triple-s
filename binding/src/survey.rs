extern crate triple_s as tri;
use std::sync::Arc;

use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock};
use pyo3::{exceptions, prelude::*, types::PyDict};

#[pyfunction]
pub fn read_sss(path: &str) -> PyResult<PySurvey> {
    let survey = tri::read_sss(path)
        .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))?;
    Ok(PySurvey::from(survey))
}

/// Survey
/// More detail can be found at: https://triple-s.org/documentation/manuals/
#[pyclass(module = "triple_s", name = "Survey", from_py_object)]
pub struct PySurvey {
    // #[pyo3(get, set)]
    // pub title: Option<String>,
    // meta: Arc<RwLock<tri::Meta>>,
    // record: Arc<RwLock<tri::Record>>,
    inner: Arc<RwLock<tri::Survey>>,
}

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

    // pub(crate) fn read_content(self_: &Bound<'_, Self>) -> MappedRwLockReadGuard<'_, Vec<PyQuestion>> {
    //     parking_lot::RwLockReadGuard::map(self_.inner.read(), |survey| {
    //         &survey.record.content
    //     })
    // }
}

#[pymethods]
impl PySurvey {
    /// Export content back to sss classic
    fn to_sss_classic(&self) -> String {
        self.inner.read().to_sss_classic()
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
}

// /// Survey
// /// More detail can be found at: https://triple-s.org/documentation/manuals/
// #[pyclass(
//     dict,
//     weakref,
//     module = "triple_s",
//     name = "Survey",
//     from_py_object
// )]
// pub struct PySurvey {
//     inner: Arc<RwLock<tri::Survey>>,
// }

// impl Clone for PySurvey {
//     fn clone(&self) -> Self {
//         Self {
//             inner: Arc::clone(&self.inner)
//         }
//     }
// }

// #[pymethods]
// impl PySurvey {
//     #[getter]
//     fn meta(&self) -> PyResult<RwLockReadGuard<'_, tri::Meta>> {
//         self.read_meta()
//     }
// }

//     fn __repr__(&self) -> PyResult<String> {
//         let title = self.title()?.unwrap_or_default();
//         let version = self.read_inner()?
//             .meta
//             .version
//             .as_ref()
//             .map(|ver| ver.clone())
//             .unwrap_or_default();

//         Ok(format!(
//             r#"Survey(title="{}", version={})"#,
//             title,
//             version
//         ))
//     }

//     /// (Optional) Title of a survey.
//     #[getter]
//     fn title(&self) -> PyResult<Option<String>> {
//         Ok(self.read_inner()?.title.clone())
//     }

//     #[setter]
//     fn set_title(&mut self, value: Option<&str>) -> PyResult<()> {
//         self.write_inner()?.title = value.map(|val| val.to_string());
//         Ok(())
//     }

//     fn to_sss_classic(&self) -> PyResult<String> {
//         Ok(self.read_inner()?.to_sss_classic())
//     }

//     fn get_question(survey: &Bound<'_, Self>, question_name: &str) -> PyResult<PyQuestion> {
//         let query = question_name.to_uppercase();
//         survey.borrow().inner.record.content
//         .iter()
//         .enumerate()
//         .find(|(_, question)| question.name.to_uppercase() == query)
//         .map(|(index, _)| PyQuestion::from_inner(survey, index))
//         .ok_or(exceptions::PyKeyError::new_err(question_name.to_string()))?
//     }
// }

/// Content of a survey
#[pyclass(module = "triple_s", name = "Record")]
pub struct PyRecord {
    inner: Py<PySurvey>,
}

impl PyRecord {
    pub fn from_rust(survey: &Bound<'_, PySurvey>) -> Self {
        Self {
            inner: survey.clone().unbind(),
        }
    }

    pub fn len(&self, py: Python<'_>) -> usize {
        self.inner.borrow(py).read_record().content.len()
    }

    // /// Acquire the inner question for reading; surfaces lock poisoning as a
    // /// `PyException` instead of panicking.
    // pub(crate) fn get_question_by_index(&self, index: usize) -> PyResult<PyQuestion> {
    //     todo!()
    //     // parking_lot::RwLockReadGuard::map(
    //     //     self.inner.read(),
    //     //     |survey|
    //     //     &survey.record.content.get(index)
    //     //     .ok_or_else(|| exceptions::PyIndexError::new_err(index))
    //     // )
    // }
}

#[pymethods]
impl PyRecord {
    /// Export content back to sss classic
    fn to_sss_classic(&self, py: Python<'_>) -> String {
        self.inner.bind(py).borrow().read_record().to_sss_classic()
    }

    /// One RECORD keyword appears after SURVEY (or after TITLE if present).
    /// It is used to introduce the definition of the variables.
    /// The record_id is any single character A to Z (or a to z).
    /// The record_id can be used in conjunction with the variable_id (see the VARIABLE keyword later) to
    /// generate a unique variable name on import.
    /// For example: RECORD A
    #[getter]
    fn id(&self, py: Python<'_>) -> String {
        self.inner.bind(py).borrow().read_record().id.clone()
    }

    #[setter]
    fn set_id(&mut self, py: Python<'_>, value: String) -> PyResult<()> {
        self.inner.bind(py).borrow_mut().write_record().id = value;
        Ok(())
    }

    /// List of questions/ variables that the current survey instance contains
    #[getter]
    fn content(&self, py: Python<'_>) -> PyResult<Vec<PyQuestion>> {
        Ok((0..self.len(py))
            .map(|index| PyQuestion::from_inner(&self.inner, index))
            .collect())
    }

    //     fn __getitem__(&self, py: Python<'_>, index: isize) -> PyResult<PyQuestion> {
    //         let inner = self.inner.bind(py);
    //         let content_length = inner.borrow().inner.record.content.len();
    //         let index = normalize_index(index, content_length)?; // also checks if index really exists, raise IndexError if otherwise
    //         PyQuestion::from_inner(inner, index)
    //     }

    //     fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
    //         Ok(self.inner.bind(py).borrow().inner.record.content.len())
    //     }

    //     fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
    //         let inner = self.inner.bind(py);
    //         let record = inner.borrow();

    //         let content_length = record.inner.record.content.len();
    //         // only show first x amount of members
    //         let read_length = content_length.min(15);

    //         let mut question_string = Vec::new();
    //         for index in 0..read_length {
    //             let question = self.__getitem__(py, index as isize)?
    //                 .__repr__(py)?;
    //             question_string.push(question);
    //         };
    //         if read_length < content_length {
    //             question_string.push("...".into())
    //         };
    //         let question_string = question_string.join(",\n\t\t");
    //         let current_id = &record.inner.record.id;
    //         Ok(format!("Record(\n\tid=\"{}\",\n\tcontent=[\n\t\t{}\n\t]\n)", current_id, question_string))
    //     }
}

// #[pyclass(module = "triple_s", name = "Content")]
// pub struct PyContent {
//     inner: Vec<PyQuestion>
// }

#[pyclass(module = "triple_s", name = "Question")]
pub struct PyQuestion {
    inner: Py<PySurvey>,
    index: usize,
}

impl PyQuestion {
    pub fn from_inner(inner: &Py<PySurvey>, index: usize) -> Self {
        Self {
            inner: inner.clone(),
            index,
        }
    }

    fn with_question_read<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(MappedRwLockReadGuard<'_, tri::Question>) -> R,
    ) -> PyResult<R> {
        let inner = self.inner.bind(py).borrow();
        let question = inner.read_question(self.index)?;
        Ok(f(question))
    }

    fn with_question_write<R>(
        &self,
        py: Python<'_>,
        f: impl FnOnce(MappedRwLockWriteGuard<'_, tri::Question>) -> R,
    ) -> PyResult<R> {
        let inner = self.inner.bind(py).borrow_mut();
        let question = inner.write_question(self.index)?;
        Ok(f(question))
    }
}

#[pymethods]
impl PyQuestion {
    fn to_sss_classic(&self, py: Python<'_>) -> PyResult<String> {
        self.with_question_read(py, |question| question.to_sss_classic())
    }

    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        let kind = self.kind(py)?;
        let label = self.label(py)?;
        let label = if label.chars().count() > 20 {
            format!("{}…", label.chars().take(19).collect::<String>())
        } else {
            label
        };

        self.with_question_read(py, |question| {
            format!(
                r#"Question(name="{}", kind={}, label="{}")"#,
                &question.name, kind, label
            )
        })
    }

    #[getter]
    fn name(&self, py: Python<'_>) -> PyResult<String> {
        self.with_question_read(py, |question| question.name.clone())
    }

    #[setter]
    fn set_name(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        self.with_question_write(py, |mut question| question.name = value.to_string())
    }

    #[getter]
    fn label(&self, py: Python<'_>) -> PyResult<String> {
        self.with_question_read(py, |question| question.text.clone())
    }

    #[setter]
    fn set_label(&self, py: Python<'_>, value: &str) -> PyResult<()> {
        self.with_question_write(py, |mut question| question.text = value.to_string())
    }

    #[getter]
    fn filter(&self, py: Python<'_>) -> PyResult<Option<String>> {
        self.with_question_read(py, |question| question.filter.clone())
    }

    #[setter]
    fn set_filter(&self, py: Python<'_>, value: Option<&str>) -> PyResult<()> {
        let value = value.map(|s| s.to_string());
        self.with_question_write(py, |mut question| question.filter = value)
    }

    #[getter]
    fn kind(&self, py: Python<'_>) -> PyResult<Kind> {
        self.with_question_read(py, |question| Kind::from_response(&question.response))
    }

    //     #[getter]
    //     fn response(question: &Bound<'_, Self>) -> PyResult<PyResponse> {
    //         Ok(PyResponse::from_inner(question))
    //     }

    // #[getter]
    // pub fn location(&self, py: Python<'_>) -> PyResult<tri::Location> {
    //     self.with_question_read(py, |question| question.location.clone())
    // }

    //     // pub location: Location,
    //     // pub response: Response,
}

// #[pyclass(module = "triple_s", name = "Response")]
// pub struct PyResponse {
//     inner: Py<PyQuestion>,
// }

// impl PyResponse {
//     pub fn from_inner(question: &Bound<'_, PyQuestion>) -> Self {
//         Self {
//             inner: question.clone().unbind(),
//         }
//     }

//     fn with_response<R>(
//         &self,
//         py: Python<'_>,
//         f: impl FnOnce(&tri::Response) -> R,
//     ) -> PyResult<R> {
//         let question = self.inner.bind(py).borrow();

//         question.with_question(py, |question| f(&question.response))
//     }

//     fn with_response_mut<R>(
//         &self,
//         py: Python<'_>,
//         f: impl FnOnce(&mut tri::Response) -> R,
//     ) -> PyResult<R> {
//         let question = self.inner.bind(py).borrow();

//         question.with_question_mut(py, |question| f(&mut question.response))
//     }
// }

// #[pymethods]
// impl PyResponse {
//     fn __getitem__(
//         response: &Bound<'_, Self>,
//         py: Python<'_>,
//         index: isize,
//     ) -> PyResult<PyChoice> {
//         let question = response.borrow().inner.bind(py).borrow();
//         let survey = question.inner.bind(py).borrow();
//         let rust_question = survey
//             .inner
//             .record
//             .content
//             .get(question.index)
//             .ok_or_else(|| exceptions::PyIndexError::new_err("question index out of range"))?;

//         match &rust_question.response {
//             tri::Response::Single { item } | tri::Response::Multiple { item, .. } => {
//                 let keyname = index as u64;
//                 if item.contains_key(&keyname) {
//                     Ok(PyChoice::from_inner(response, keyname))
//                 } else {
//                     Err(exceptions::PyIndexError::new_err("index out of range"))
//                 }
//             }
//             _ => {
//                 let error_message =
//                     "Only available for `Single` and `Multiple` type of questions."
//                         .to_string();
//                 let error = exceptions::PyTypeError::new_err(error_message);
//                 Err(error)
//             }
//         }
//     }

//     fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
//         self.inner.bind(py).borrow().with_question(py, |question| {
//             let value = match &question.response {
//                 Response::Character(size) => format!("Character({})", size),
//                 Response::Note(_) => "Note".to_string(),
//                 Response::Quantity { min, max, na_value, na_label } => {
//                     match na_value.as_ref() {
//                         Some(value) => {
//                             format!(
//                                 r#"Quantity(min={}, max={}, na_value={}, na_label="{}")"#,
//                                 min, max, value, na_label.as_ref().map(|s| s.clone()).unwrap_or_default()
//                             )
//                         }
//                         None => format!("Quantity(min={}, max={})", min, max)
//                     }
//                 }
//                 Response::Single { item } => {
//                     format!("Single(item={})", format_response_items(item))
//                 }
//                 Response::Multiple { item, spread } => {
//                     format!(
//                         "Multiple(item={}, spread={:?})",
//                         format_response_items(item),
//                         spread
//                     )
//                 }
//             };
//             format!("Response.{}", value)
//         })
//     }

//     #[getter]
//     fn item(
//         response: &Bound<'_, Self>,
//         py: Python<'_>,
//     ) -> PyResult<std::collections::BTreeMap<u64, PyChoice>> {
//         response
//             .clone()
//             .borrow()
//             .with_response(py, |resp| match resp {
//                 Response::Single { item } | Response::Multiple { item, .. } => {
//                     let choices = item
//                         .keys()
//                         .map(|idx| (*idx, PyChoice::from_inner(response, *idx)));
//                     Ok(std::collections::BTreeMap::from_iter(choices))
//                 }
//                 _ => {
//                     let error_message =
//                         "Only available for `Single` and `Multiple` type of questions."
//                             .to_string();
//                     let error = exceptions::PyTypeError::new_err(error_message);
//                     Err(error)
//                 }
//             })?
//     }

//     /// checks if current instance was one of `Single` or `Multiple` type.
//     fn is_categorical(&self, py: Python<'_>) -> PyResult<bool> {
//         self.with_response(py, |response| {
//             matches!(
//                 response,
//                 Response::Single { .. } | Response::Multiple { .. }
//             )
//         })
//     }
// }

// /// A possible answer that a respondent can choose in a single-/ multi- response question
// #[pyclass(module = "triple_s", name = "Choice")]
// pub struct PyChoice {
//     inner: Py<PyResponse>,
//     index: u64,
// }

// impl PyChoice {
//     pub fn from_inner(response: &Bound<'_, PyResponse>, index: u64) -> Self {
//         Self {
//             inner: response.clone().unbind(),
//             index,
//         }
//     }

//     fn with_choice<R>(
//         &self,
//         py: Python<'_>,
//         f: impl FnOnce(&tri::Choice) -> R,
//     ) -> PyResult<R> {
//         // let response = self.inner.bind(py).borrow();
//         self.inner
//             .bind(py)
//             .borrow()
//             .with_response(py, |response| match response {
//                 Response::Single { item } | Response::Multiple { item, .. } => item
//                     .get(&self.index)
//                     .ok_or_else(|| exceptions::PyKeyError::new_err(self.index))
//                     .map(f),
//                 _ => Err(exceptions::PyTypeError::new_err(
//                     "Only available for `Single` and `Multiple` types of responses!",
//                 )),
//             })?
//     }

//     fn with_choice_mut<R>(
//         &self,
//         py: Python<'_>,
//         f: impl FnOnce(&mut tri::Choice) -> R,
//     ) -> PyResult<R> {
//         self.inner
//             .bind(py)
//             .borrow()
//             .with_response_mut(py, |response| match response {
//                 Response::Single { item } | Response::Multiple { item, .. } => item
//                     .get_mut(&self.index)
//                     .ok_or_else(|| exceptions::PyKeyError::new_err(self.index))
//                     .map(f),
//                 _ => Err(exceptions::PyTypeError::new_err(
//                     "Only available for `Single` and `Multiple` types of responses!",
//                 )),
//             })?
//     }
// }

// #[pymethods]
// impl PyChoice {
//     fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
//         self.with_choice(py, format_choice)
//     }

//     /// Code used to uniquely identify the current answer option.
//     #[getter]
//     fn code(&self, py: Python<'_>) -> PyResult<u64> {
//         self.with_choice(py, |choice| choice.code)
//     }

//     /// Text label of the current answer option
//     #[getter]
//     fn label(&self, py: Python<'_>) -> PyResult<String> {
//         self.with_choice(py, |choice| choice.label.clone())
//     }

//     #[setter]
//     fn set_label(&self, py: Python<'_>, value: &str) -> PyResult<()> {
//         self.with_choice_mut(py, |choice| choice.label = value.to_string())
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
//         self.with_choice(py, |choice| choice.filter.clone())
//     }

//     #[setter]
//     fn set_filter(&self, py: Python<'_>, value: Option<&str>) -> PyResult<()> {
//         self.with_choice_mut(py, |choice| {
//             choice.filter = value.map(|s| s.to_string())
//         })
//     }

//     /// (Optional) indicator for special values (e.g. mannotationissing data codes).
//     /// Removed from SSS 1.2 onwards.
//     #[getter]
//     fn is_special(&self, py: Python<'_>) -> PyResult<bool> {
//         self.with_choice(py, |choice| choice.is_special.clone())
//     }

//     #[setter]
//     fn set_is_special(&self, py: Python<'_>, value: bool) -> PyResult<()> {
//         self.with_choice_mut(py, |choice| {
//             choice.is_special = value
//         })
//     }
// }

/// Flag for indicating what response kind (i.e., single/ multiple etc.)
#[pyclass(eq, module = "triple_s", name = "Kind")]
#[derive(PartialEq, Debug)]
pub enum Kind {
    Single,
    Multiple,
    Quantity,
    Character,
    Note,
}

impl Kind {
    pub fn from_response(response: &tri::Response) -> Self {
        match response {
            tri::Response::Single { .. } => Kind::Single,
            tri::Response::Multiple { .. } => Kind::Multiple,
            tri::Response::Quantity { .. } => Kind::Quantity,
            tri::Response::Character(..) => Kind::Character,
            tri::Response::Note(..) => Kind::Note,
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single => write!(f, "Kind.Single"),
            Self::Multiple => write!(f, "Kind.Multiple"),
            Self::Quantity => write!(f, "Kind.Quantity"),
            Self::Character => write!(f, "Kind.Character"),
            Self::Note => write!(f, "Kind.Note"),
        }
    }
}

// /// Information about a survey instance, including creation date, user info etc.,
// #[pyclass(module = "triple_s", name = "Meta")]
// pub struct PyMeta {
//     inner: Py<PySurvey>,
// }

// impl PyMeta {
//     pub fn from_rust(survey: &Bound<'_, PySurvey>) -> Self {
//         Self {
//             inner: survey.clone().unbind(),
//         }
//     }
// }

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
    /// Export meta back to sss classic
    fn to_sss_classic(&self, py: Python<'_>) -> String {
        self.inner.bind(py).borrow().read_meta().to_sss_classic()
    }

    fn to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);

        if let Some(value) = self.version(py) {
            dict.set_item("version", value)?;
        };

        if let Some(value) = self.date(py) {
            dict.set_item("date", value)?;
        };

        if let Some(value) = self.time(py) {
            dict.set_item("time", value)?;
        };

        if let Some(value) = self.origin(py) {
            dict.set_item("origin", value)?;
        };

        if let Some(value) = self.user(py) {
            dict.set_item("user", value)?;
        };

        if let Some(value) = self.standard(py) {
            dict.set_item("standard", value)?;
        };

        Ok(dict.unbind())
    }

    /// SSS Version
    #[getter]
    fn version(&self, py: Python<'_>) -> Option<String> {
        self.inner.bind(py).borrow().read_meta().version.clone()
    }

    #[setter]
    fn set_version(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner.bind(py).borrow_mut().write_meta().version = value;
        Ok(())
    }

    /// (Optional) The date the file was created.
    #[getter]
    fn date(&self, py: Python<'_>) -> Option<String> {
        self.inner.bind(py).borrow().read_meta().date.clone()
    }

    #[setter]
    fn set_date(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner.bind(py).borrow_mut().write_meta().date = value;
        Ok(())
    }

    /// (Optional) The time the file was created.
    #[getter]
    fn time(&self, py: Python<'_>) -> Option<String> {
        self.inner.bind(py).borrow().read_meta().time.clone()
    }

    #[setter]
    fn set_time(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner.bind(py).borrow_mut().write_meta().time = value;
        Ok(())
    }

    /// (Optional) The originating system (program and operating system), i.e., `Windows 95`
    #[getter]
    fn origin(&self, py: Python<'_>) -> Option<String> {
        self.inner.bind(py).borrow().read_meta().origin.clone()
    }

    #[setter]
    fn set_origin(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner.bind(py).borrow_mut().write_meta().origin = value;
        Ok(())
    }

    /// (Optional) Name of the user who created this file
    #[getter]
    fn user(&self, py: Python<'_>) -> Option<String> {
        self.inner.bind(py).borrow().read_meta().user.clone()
    }

    #[setter]
    fn set_user(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner.bind(py).borrow_mut().write_meta().user = value;
        Ok(())
    }

    #[getter]
    fn style(&self, py: Python<'_>) -> Option<String> {
        self.inner.bind(py).borrow().read_meta().style.clone()
    }

    #[setter]
    fn set_style(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner.bind(py).borrow_mut().write_meta().style = value;
        Ok(())
    }

    /// (Optional) can appear any number of times in the definition file wherever a keyword is expected
    /// (including between values specified in a VALUES block).
    /// The importing program does not interpret the contents of the note_text.
    /// It is recommended that the NOTE form be used where comments are to be inserted into the SSS definition file by an export program.
    #[getter]
    fn note(&self, py: Python<'_>) -> Vec<String> {
        self.inner.bind(py).borrow().read_meta().note.clone()
    }

    #[setter]
    fn set_note(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner.bind(py).borrow_mut().write_meta().style = value;
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
    fn standard(&self, py: Python<'_>) -> Option<String> {
        self.inner.bind(py).borrow().read_meta().standard.clone()
    }

    #[setter]
    fn set_standard(&self, py: Python<'_>, value: Option<String>) -> PyResult<()> {
        self.inner.bind(py).borrow_mut().write_meta().standard = value;
        Ok(())
    }
}

// /// Just a dummy class used for creating a python parent class for inheritance
// #[pyclass(dict, module = "triple_s", name = "Proxy")]
// pub struct PyProxy {}

// /// helper functions that converts pyton index (can be negative) to rust index (usize)
// fn normalize_index(index: isize, total: usize) -> PyResult<usize> {
//     let index = if index < 0 {
//         total as isize + index
//     } else {
//         index
//     };

//     if index < 0 || index as usize >= total {
//         return Err(exceptions::PyIndexError::new_err("index out of range"));
//     }

//     Ok(index as usize)
// }

// fn format_response_items(item: &std::collections::BTreeMap<u64, tri::Choice>) -> String {
//     let content = item
//         .values()
//         .map(|choice| {
//             let formatted_string = format_choice(choice);
//             format!("{}: {}", choice.code, formatted_string)
//         })
//         .collect::<Vec<String>>();
//     let content = content.join(", ");
//     format!("{{{}}}", content)
// }

// #[inline]
// fn format_choice(choice: &tri::Choice) -> String {
//     let prefix = format!(
//         r#"Choice(code={}, label="{}""#,
//         choice.code, choice.label
//     );
//     match choice.filter.as_ref() {
//         Some(value) => format!(r#"{}, filter="{}")"#, prefix, value),
//         None => format!("{})", prefix),
//     }
// }
