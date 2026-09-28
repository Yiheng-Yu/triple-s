use anyhow::Result;
use parking_lot::{ArcRwLockReadGuard, RawRwLock, RwLockReadGuard, RwLockWriteGuard};
#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*, types::PyTuple};

use super::{ArcLock, DataSource, Question};
#[cfg(feature = "pyo3")]
use crate::utils::normalize_py_index;

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Record", skip_from_py_object)
)]
#[derive(Debug, Default, Clone)]
pub struct Record {
    id: char,

    // pub format: Option<CodeFormat>,
    /// (Optional) 'skip' attribute can be used to ignore one or more initial
    /// records in the Data File. This will be most useful for csv Data
    /// Files where the first line is often used as documentation (e.g.
    /// names for the columns/fields) for the succeeding values.
    skip: Option<String>,

    /// (Optional) Used to specify an explicit location for the Data
    /// File that is described by this Triple-S XML specification. Note that
    /// using an href attribute ties the specification to the Data File and
    /// may cause problems if the Metadata File and Data File are moved.
    href: Option<String>,

    data_source: DataSource,

    // NOTE: THIS KEPT AS ARC RWLOCK IS VERY IMPORTANT FOR PYTHON BINDING TO WORK!!!
    content: ArcLock<Vec<ArcLock<Question>>>,
}

impl Record {
    pub fn id(&self) -> &char {
        &self.id
    }

    pub fn id_mut(&mut self) -> &mut char {
        &mut self.id
    }

    pub fn skip(&self) -> &Option<String> {
        &self.skip
    }

    pub fn skip_mut(&mut self) -> &mut Option<String> {
        &mut self.skip
    }

    pub fn href(&self) -> &Option<String> {
        &self.href
    }

    pub fn href_mut(&mut self) -> &mut Option<String> {
        &mut self.href
    }

    pub fn data_source(&self) -> &DataSource {
        &self.data_source
    }

    pub fn data_source_mut(&mut self) -> &mut DataSource {
        &mut self.data_source
    }

    pub fn set_content(&self, value: Vec<ArcLock<Question>>) {
        *self.content.write() = value;
    }

    pub fn content(&self) -> RwLockReadGuard<'_, Vec<ArcLock<Question>>> {
        self.content.read()
    }

    pub fn content_mut(&mut self) -> RwLockWriteGuard<'_, Vec<ArcLock<Question>>> {
        self.content.write()
    }

    #[inline]
    fn get_question_(&self, question_name: &str) -> Option<ArcLock<Question>> {
        self.content
            .read()
            .iter()
            .find(|item| item.read().name().as_str() == question_name).cloned()
    }

    pub fn get_question(
        &self,
        question_name: &str,
    ) -> Option<ArcRwLockReadGuard<RawRwLock, Question>> {
        self.get_question_(question_name).map(|item| item.read_arc())
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = format!("RECORD {}\n", self.id);
        for question in self.content.read().iter() {
            output.push_str(&question.read().to_sss_classic()?);
        }
        output.push_str("END RECORD\n");
        Ok(output)
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Record {
    /// One RECORD keyword appears after SURVEY (or after TITLE if present).
    /// It is used to introduce the definition of the variables.
    /// The record_id is any single character A to Z (or a to z).
    /// The record_id can be used in conjunction with the variable_id (see the
    /// VARIABLE keyword later) to generate a unique variable name on
    /// import. For example: RECORD A
    #[getter]
    fn get_id(&self) -> PyResult<char> {
        Ok(self.id.clone())
    }

    /// Set the id.
    #[setter]
    fn set_id(&mut self, value: char) -> PyResult<()> {
        self.id = value;
        Ok(())
    }

    /// (Optional) Used to specify an explicit location for the Data
    /// File that is described by this Triple-S XML specification. Note that
    /// using an href attribute ties the specification to the Data File and
    /// may cause problems if the Metadata File and Data File are moved.
    #[getter]
    fn get_data_source(&self) -> PyResult<DataSource> {
        Ok(self.data_source.clone())
    }

    #[getter]
    fn get_href(&self) -> PyResult<Option<String>> {
        Ok(self.href().clone())
    }

    #[getter]
    fn get_content<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self.content.read().clone().into_iter().map(|item| item.read().clone());
        PyTuple::new(py, items)
    }

    #[pyo3(name="get_question", signature=(question_name:"str"))]
    pub fn py_get_question(&self, question_name: &str) -> Option<Question> {
        self.get_question_(question_name).map(|item| item.read().clone())
    }

    // todo: getter/ setter for href, datasource and skip

    fn __len__(&self) -> usize {
        self.content().len()
    }

    fn __getitem__<'py>(&self, index: &Bound<'py, PyAny>) -> PyResult<Question> {
        if let Ok(number) = index.extract::<isize>() {
            let total_length = self.__len__();
            let sanitised = normalize_py_index(number, total_length)
                .ok_or(exceptions::PyIndexError::new_err(number))?;

            self.content
                .read()
                .get(sanitised)
                .map(|item| item.read().clone())
                .ok_or(exceptions::PyIndexError::new_err(number))
        } else if let Ok(question_name) = index.extract::<String>() {
            self.py_get_question(&question_name).ok_or(exceptions::PyKeyError::new_err(
                format!("Unable to find question with name '{}'", question_name),
            ))
        } else {
            let type_name = index.get_type().name()?.extract::<String>()?;
            Err(exceptions::PyTypeError::new_err(format!(
                "`index` must be question index (as `int`) or question name (as `str`), found: {}",
                type_name
            )))
        }
    }
}
