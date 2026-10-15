//! Corresponds to the `<record>` node and its contents.
//! Uused for string questions inside the survey.
use std::{fs::File, io::{BufRead, BufReader}, path::Path};

use anyhow::{Context, Result};
use parking_lot::{ArcRwLockReadGuard, RawRwLock, RwLockReadGuard, RwLockWriteGuard};
#[cfg(feature = "pyo3")]
use pyo3::{
    IntoPyObjectExt,
    exceptions,
    prelude::*,
    types::{PyDict, PySlice, PyTuple},
};
#[cfg(feature = "pyo3")]
use pyo3_polars::{PyDataFrame, error::PyPolarsErr};
use quick_xml::Writer;
use strum::{EnumIter, EnumString};

use super::{ArcLock, Question};
use crate::{error, prelude::{DeepCopy, Flag}, utils::lock_it_up};
use polars::prelude::*;

#[cfg(feature = "pyo3")]
use crate::utils::{extract_indicies, get_type_name, normalize_py_index, py_value_error};

/// Flags that collects all allowed values of `format` attribute of a `<record>` element.
/// This is used to indicate types of the data the current datamap file binds to.
/// 
/// Introduced in triple-s 2.0
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Debug, EnumString, PartialEq, Default, Clone, Eq, EnumIter)]
pub enum DataSource {
    /// Supported from sss 2.0 onwards.
    /// the data representation is comma separated values, using one field
    /// for each variable, with data values similar to the fixed format. Note
    /// that for data in csv format the position element refers to the
    /// field number.
    #[strum(serialize = "csv")]
    Csv,

    /// The data representation is fixed format fields.
    /// Basically `.asc` text file that formatted bit like csv but without any
    /// column header.
    #[default]
    #[strum(serialize = "fixed")]
    Fixed,
}

impl Flag for DataSource {
    /// format flag into string,,,
    fn to_string(&self) -> String {
        match self {
            Self::Csv => "csv".into(),
            Self::Fixed => "fixed".into(),
        }
    }
}

impl DeepCopy for DataSource {
    #[doc = include_str!("../doc/rs_deep_copy_impl.md")]
    fn deepcopy(&self) -> Self {
        self.clone()
    }
}

/// Content of a datamap
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Record", skip_from_py_object)
)]
#[derive(Debug, Default, Clone)]
pub struct Record {
    id: char,
    skip: Option<String>,
    href: Option<String>,
    data_source: Option<DataSource>,
    content: ArcLock<Vec<ArcLock<Question>>>,
}

/// # Fields
///
/// ---
impl Record {
    /// Corresponds to the `ident` attribute of a `<record>` element.
    /// 
    /// Basically it's a single-character from A-Z/ a-z for id.
    pub fn ident(&self) -> &char {
        &self.id
    }

    /// a read-write view of the `ident` field.e
    pub fn ident_mut(&mut self) -> &mut char {
        &mut self.id
    }

    /// Corresponds with the `skip` attribute of a `<record>` elememnt
    ///
    /// From the official documentation:
    ///
    /// (Optional) 'skip' attribute can be used to ignore one or more initial
    /// records in the Data File. This will be most useful for csv Data
    /// Files where the first line is often used as documentation (e.g.
    /// names for the columns/fields) for the succeeding values.
    pub fn skip(&self) -> &Option<String> {
        &self.skip
    }

    /// a read-write borrow of the `skip` field
    pub fn skip_mut(&mut self) -> &mut Option<String> {
        &mut self.skip
    }

    /// (Optional) path to the data file the current datamap ties to.
    /// This corresponds to the `href` attribute of a `<record>` element
    ///
    /// Note that using an href attribute ties the specification to the Data
    /// File and may cause problems if the Metadata File and Data File are
    /// moved.
    pub fn href(&self) -> &Option<String> {
        &self.href
    }

    /// returns a read-write borrow of the `href` field
    pub fn href_mut(&mut self) -> &mut Option<String> {
        &mut self.href
    }

    /// Indicating types of the data the current datamap file binds to
    /// This corresponds to the `format` attribute of a `<record>` element.
    /// 
    /// Introduced in triple-s 2.0
    pub fn data_source(&self) -> Option<&DataSource> {
        self.data_source.as_ref()
    }

    /// A read-write borrow of the current `data_source` field
    pub fn data_source_mut(&mut self) -> &mut Option<DataSource> {
        &mut self.data_source
    }

    /// Collections of `Question` instances of a survey, this corresponds to the
    /// lists of `<variable>` child nodes
    pub fn content(&self) -> RwLockReadGuard<'_, Vec<ArcLock<Question>>> {
        self.content.read()
    }

    /// A read-write borrow of contents in a survey
    pub fn content_mut(&mut self) -> RwLockWriteGuard<'_, Vec<ArcLock<Question>>> {
        self.content.write()
    }
}

impl Record {
    #[inline]
    fn get_question_(&self, question_name: &str) -> Option<ArcLock<Question>> {
        self.content
            .read()
            .iter()
            .find(|item| item.read().name().to_uppercase() == question_name.to_uppercase())
            .cloned()
    }

    /// get a question by its question name, name matching is case insensitive.
    pub fn get_question(
        &self,
        question_name: &str,
    ) -> Option<ArcRwLockReadGuard<RawRwLock, Question>> {
        self.get_question_(question_name).map(|item| item.read_arc())
    }

    /// export to triple-s classic.
    pub fn to_classic(&self) -> Result<String> {
        let mut output = format!("RECORD {}\n", self.id);
        for question in self.content.read().iter() {
            output.push_str(&question.read().to_classic()?);
        }
        output.push_str("END RECORD\n");
        Ok(output)
    }

    /// Whether the current datamap has associated .asc/ .csv data file or not.
    pub fn has_data(&self) -> bool {
        self.href().is_some()
    }

    /// Serialises the record and its questions as a Triple-S XML `<record>`
    /// element.
    pub fn to_xml(&self) -> Result<String> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
        self.write_xml(&mut writer)?;
        String::from_utf8(writer.into_inner()).map_err(anyhow::Error::from)
    }

    /// Writes the record to the supplied writer, propagating question and I/O
    /// errors.
    pub fn write_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        anyhow::ensure!(
            self.id != '\0',
            "Record ident must be set before XML export"
        );
        let ident = self.id.to_string();
        let mut element =
            writer.create_element("record").with_attribute(("ident", ident.as_str()));
        for (name, value) in [("skip", &self.skip), ("href", &self.href)] {
            if let Some(value) = value {
                element = element.with_attribute((name, value.as_str()));
            }
        }
        if let Some(value) = &self.data_source {
            element = element.with_attribute(("format", value.to_string().as_str()));
        }
        element.write_inner_content(|children| {
            for question in self.content.read().iter() {
                let question = question.read();
                question
                    .write_xml(children)
                    .with_context(|| format!("When exporting question '{}'", question.name()))
                    .map_err(std::io::Error::other)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

/// Read data file
/// 
/// ---
impl Record {
    /// Load fixed-width or CSV data according to the record's `format`.
    /// CSV positions are one-based field numbers. The record's `skip`
    /// attribute controls how many initial records to ignore in either format.
    pub fn load_data(&mut self, path: impl AsRef<Path>) -> Result<()> {
        // reader for the data file provided
        let path = path.as_ref();
        let file = File::open(path)
            .map_err(error::FileReadError::from)
            .with_context(|| format!("When opening '{}'", path.display()))?;
        let skip = self.skip.as_deref().unwrap_or("0")
            .parse::<usize>()
            .context("Record skip must be a non-negative integer")?;
        
        if self.data_source() == Some(&DataSource::Csv) {
            return self.load_csv_data(file, skip)
                .with_context(|| format!("When reading CSV data from '{}'", path.display()));
        }
        let lines = BufReader::new(file).lines();  // each line is treated as data from one respondent
        
        for (index, line) in lines.enumerate() {
            let data = line?;
            if index < skip {
                continue;
            }
            for question in self.content_mut().iter_mut() {
                let entry = question.read().parse_row(&data)
                    .with_context(|| format!("When trying to extract data for question {}", question.read().name()))?;
                question.write().data_mut().push(lock_it_up(entry));
            }
        }

        Ok(())
    }

    fn load_csv_data(&mut self, file: File, skip: usize) -> Result<()> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(file);
        for (index, row) in reader.records().enumerate() {
            let row = row.with_context(|| format!("When reading CSV record {}", index + 1))?;
            if index < skip {
                continue;
            }
            let questions = self.content();
            let entries = questions.iter()
                .map(|question| {
                    let question = question.read();
                    let position = *question.location().start() as usize;
                    let field = position.checked_sub(1)
                        .and_then(|column| row.get(column))
                        .with_context(|| format!(
                            "CSV record {} has {} fields; question '{}' requires field {}",
                            index + 1, row.len(), question.name(), position,
                        ))?;
                    question.parse_field(field).with_context(|| format!(
                        "When parsing CSV record {}, field {} for question '{}'",
                        index + 1, position, question.name(),
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            for (question, entry) in questions.iter().zip(entries) {
                question.write().data_mut().push(lock_it_up(entry));
            }
        }
        Ok(())
    }

    /// convert data into polars dataframe
    pub fn to_polars(&self) -> Result<DataFrame, PolarsError> {
        let columns = self.content().iter()
                .map(|item| item.read().to_polars().map(Column::from))
            .collect::<Result<Vec<Column>, PolarsError>>()?;

        DataFrame::new_infer_height(columns)
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Record {
    #[pyo3(name = "to_xml")]
    fn py_to_xml(&self) -> PyResult<String> {
        self.to_xml().map_err(py_value_error)
    }

    /// Corresponds to the `ident` attribute of a `<record>` element.
    /// 
    /// Basically it's a single-character from A-Z/ a-z for id.
    #[getter(ident)]
    fn py_get_ident(&self) -> PyResult<char> {
        Ok(self.id.clone())
    }

    /// Set the id.
    #[setter(ident)]
    fn py_set_ident(&mut self, value: char) -> PyResult<()> {
        self.id = value;
        Ok(())
    }

    /// Indicating types of the data the current datamap file binds to
    /// This corresponds to the `format` attribute of a `<record>` element.
    /// 
    /// Introduced in triple-s 2.0
    #[getter(data_source)]
    fn py_get_data_source(&self) -> PyResult<Option<DataSource>> {
        Ok(self.data_source.clone())
    }

    #[setter(data_source)]
    fn py_set_data_source<'py>(
        &mut self,
        data_source: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<()> {
        self.data_source = match data_source {
            None => None,
            Some(bound) => {
                let value = if let Ok(value) = bound.extract::<DataSource>() {
                    value
                } else if let Ok(value) = bound.extract::<&str>() {
                    value.parse::<DataSource>()
                    .with_context(|| format!(
                        "Expect to triple_s.DataSource instance or one of {:?}, got: '{}'.",
                        DataSource::available_options(), value
                    ))
                    .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))?
                } else {
                    let type_name = get_type_name(bound)?;
                    return Err(exceptions::PyTypeError::new_err(format!(
                        "Expect to triple_s.DataSource instance or one of {:?}, got: '{}'.",
                        DataSource::available_options(),
                        type_name
                    )));
                };
                Some(value)
            }
        };

        Ok(())
    }

    /// (Optional) path to the data file the current datamap ties to. 
    /// 
    /// This corresponds to the href attribute of a <record> element.
    #[getter(href)]
    fn py_get_href(&self) -> PyResult<Option<String>> {
        Ok(self.href().clone())
    }

    #[setter(href)]
    fn py_set_href(&mut self, href: Option<String>) -> PyResult<()> {
        self.href = href;
        Ok(())
    }

    /// Collections of `Question` instances of a survey, this corresponds to the
    /// lists of `<variable>` child nodes
    #[getter(content)]
    fn py_get_content<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self.content.read().clone().into_iter().map(|item| item.read().clone());
        PyTuple::new(py, items)
    }

    /// Get question by question name, returns 'None' if it doesn't exist.
    /// 
    /// Name matching is not case sensitive.
    #[pyo3(name="get_question", signature=(question_name:"str"))]
    pub fn py_get_question(&self, question_name: &str) -> Option<Question> {
        self.get_question_(question_name).map(|item| item.read().clone())
    }

    /// Load fixed-width or CSV data using the record's format and skip metadata.
    #[pyo3(name="load_data", signature=(path:"str"))]
    pub fn py_load_data(&mut self, path: &str) -> PyResult<()> {
        self.load_data(path)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{:?}", err)))
    }

    /// Export loaded data into polars dataframe
    #[pyo3(name="to_polars")]
    pub fn py_to_polars(&self) -> PyResult<PyDataFrame> {
        self.to_polars()
            .map(PyDataFrame)
            .map_err(|err| PyPolarsErr::from(err).into())
    }

    /// Export loaded data as a pandas DataFrame in question order.
    #[pyo3(name = "to_pandas")]
    pub fn py_to_pandas<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let columns = PyDict::new(py);
        for question in self.content().iter() {
            let question = question.read();
            columns.set_item(question.name(), question.py_to_pandas(py)?)?;
        }
        py.import("pandas")?.getattr("DataFrame")?.call1((columns,))
    }

    fn __len__(&self) -> usize {
        self.content().len()
    }

    fn __getitem__<'py>(&self, index: &Bound<'py, PyAny>) -> PyResult<Py<PyAny>> {
        let py = index.py();

        if let Ok(number) = index.extract::<isize>() {
            let total_length = self.__len__();
            let sanitised = normalize_py_index(number, total_length)
                .ok_or(exceptions::PyIndexError::new_err(number))?;
            let item = self
                .content
                .read()
                .get(sanitised)
                .map(|item| item.read().clone())
                .ok_or(exceptions::PyIndexError::new_err(number))?;
            item.into_py_any(py)
        } else if let Ok(question_name) = index.extract::<String>() {
            self.py_get_question(&question_name)
                .ok_or(exceptions::PyKeyError::new_err(format!(
                    "Unable to find question with name '{}'",
                    question_name
                )))?
                .into_py_any(py)
        } else if let Ok(indicies) = index.cast::<PySlice>() {
            let total = self.__len__() as isize;
            let content_read = self.content.read();
            let indicies = extract_indicies(indicies, total)?;
            let result = indicies
                .into_iter()
                .map(|idx| {
                    let result = content_read
                        .get(idx as usize)
                        .ok_or(exceptions::PyIndexError::new_err(idx))?
                        .read()
                        .clone();
                    Ok(result)
                })
                .collect::<PyResult<Vec<Question>>>()?;
            result.into_py_any(py)
        } else {
            let type_name = index.get_type().name()?.extract::<String>()?;
            Err(exceptions::PyTypeError::new_err(format!(
                "`index` must be question index (as `int`) or question name (as `str`), found: {}",
                type_name
            )))
        }
    }
}
