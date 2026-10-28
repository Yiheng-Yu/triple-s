//! Corresponds to the `<record>` node and its contents.
//! Uused for string questions inside the survey.
#[cfg(feature = "pyo3")]
use std::str::FromStr;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result};
use parking_lot::{ArcRwLockReadGuard, RawRwLock, RwLockReadGuard, RwLockWriteGuard};
use polars::{frame::row::Row, prelude::*};
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
use rayon::prelude::*;
use strum::{EnumIter, EnumString};

use super::{ArcLock, Question};
#[cfg(feature = "pyo3")]
use crate::{
    Survey,
    utils::{extract_indicies, get_type_name, normalize_py_index, py_value_error},
};
use crate::{
    SurveyData,
    error::{self, InvalidArg},
    prelude::{DeepCopy, Flag},
    utils::TextReader,
};

const BATCH_SIZE: usize = 500; // no. rows when writing to paquet cache

/// Flags that collects all allowed values of `format` attribute of a `<record>`
/// element. This is used to indicate types of the data the current datamap file
/// binds to.
///
/// Introduced in triple-s 2.0
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Debug, EnumString, PartialEq, Default, Clone, Eq, EnumIter)]
pub enum DataFormat {
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

impl Flag for DataFormat {
    /// format flag into string,,,
    fn to_string(&self) -> String {
        match self {
            Self::Csv => "csv".into(),
            Self::Fixed => "fixed".into(),
        }
    }
}

impl DeepCopy for DataFormat {
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
    data_format: Option<DataFormat>,
    content: ArcLock<Vec<ArcLock<Question>>>,
    data: ArcLock<SurveyData>,
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
    pub fn data_format(&self) -> Option<&DataFormat> {
        self.data_format.as_ref()
    }

    /// A read-write borrow of the current `data_format` field
    pub fn data_format_mut(&mut self) -> &mut Option<DataFormat> {
        &mut self.data_format
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

    /// A read only copy of the survey data.
    pub fn data(&self) -> RwLockReadGuard<'_, SurveyData> {
        self.data.read()
    }

    /// A read-write borrow of the survey data.
    pub fn data_mut(&self) -> RwLockWriteGuard<'_, SurveyData> {
        self.data.write()
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
        if self.id == '\0' {
            return Err(InvalidArg::new("Record ident must be set before XML export").into());
        }
        let ident = self.id.to_string();
        let mut element =
            writer.create_element("record").with_attribute(("ident", ident.as_str()));
        for (name, value) in [("skip", &self.skip), ("href", &self.href)] {
            if let Some(value) = value {
                element = element.with_attribute((name, value.as_str()));
            }
        }
        if let Some(value) = &self.data_format {
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
    /// Supports both UTF-8 and BOM-marked UTF-16LE/BE encodings.
    ///
    /// This `load_data` function will stream data into a temporary parquet cache file (`.parquet`) to be
    /// used later. To get the cache, call [`Record::to_parquet()`] method afterwards:
    ///
    /// ```no_run
    /// use triple_s::read_triple_s;
    ///
    /// let mut survey =
    ///     read_triple_s("path/to/datamap.sss", Some("path/to/database.asc"), false)?;
    ///
    /// let mut record = survey.record_mut();
    /// record.load_data("new/data/path.asc")?; // loads data
    ///
    /// let df = record.to_polars()?; // get data as polars dataframe
    /// println!("{:?}", df);
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn load_data(&mut self, path_data: impl AsRef<Path>) -> Result<()> {
        let (parquet_file, cache) =
            tempfile::NamedTempFile::with_suffix(".parquet")?.into_parts();

        let path_data = path_data.as_ref();
        self.set_data_source(path_data, false)
            .with_context(|| format!("When trying to load data from '{}'", path_data.display()))?;

        let schema =
            Schema::from_iter(self.content().iter().map(|item| item.read().polars_field()));
        let mut writer = ParquetWriter::new(parquet_file).batched(&schema)?;
        let mut batch_buffer = Vec::new();
        for row in self.scan_data_file()? {
            batch_buffer.push(row.with_context(|| {
                format!("When trying to load data from '{}'", path_data.display())
            })?);
            if batch_buffer.len() == BATCH_SIZE {
                Self::write_batch(&mut writer, &schema, &mut batch_buffer)?;
            }
        }
        Self::write_batch(&mut writer, &schema, &mut batch_buffer)?;
        writer.finish()?;

        let data = SurveyData::from_cache(cache, Some(Arc::new(schema)))?;
        *self.data.write() = data;
        Ok(())
    }

    fn write_batch(
        writer: &mut polars::io::parquet::write::BatchedWriter<File>,
        schema: &Schema,
        batch_buffer: &mut Vec<Row<'static>>,
    ) -> Result<()> {
        let df = DataFrame::from_rows_and_schema(batch_buffer, schema)?;
        writer.write_batch(&df)?;
        batch_buffer.clear();
        Ok(())
    }

    fn scan_data_file(&self) -> Result<Box<dyn Iterator<Item = Result<Row<'static>>> + '_>> {
        let path_data = self
            .href()
            .as_ref()
            .ok_or(error::InternalError::new("field `href` not defined"))?;

        let file = File::open(path_data)
            .map_err(error::FileReadError::from)
            .with_context(|| format!("When opening '{}'", path_data))?;
        let reader = TextReader::new(BufReader::new(file));
        let mut reader = BufReader::new(reader);

        // first apply skip...
        let skip = self
            .skip
            .as_deref()
            .unwrap_or("0")
            .parse::<usize>()
            .context("Record skip must be a non-negative integer")?;
        let mut discarded = Vec::new();
        for _ in 0..skip {
            discarded.clear();
            if reader.read_until(b'\n', &mut discarded)? == 0 {
                break;
            }
        }

        // then do iteration
        let data_format = self.data_format().cloned().unwrap_or(DataFormat::Fixed); // default DataFormat
        match data_format {
            DataFormat::Fixed => {
                let width = self
                    .content()
                    .iter()
                    .map(|question| *question.read().location().finish() as usize)
                    .max()
                    .context("Cannot determine record width: no questions")?;
                Ok(Box::new(std::iter::from_fn(
                    move || match Self::read_one_record(&mut reader, width) {
                        Ok(row) if row.is_empty() => None,
                        Ok(row) => Some(
                            self.content()
                                .iter()
                                .map(|question| {
                                    let question = question.read();
                                    let field = question.locate_data_asc(&row)?;
                                    Ok(question.parse_question_value(field)?.into_static())
                                })
                                .collect::<Result<Vec<_>>>()
                                .map(Row),
                        ),
                        Err(error) => Some(Err(error)),
                    },
                )))
            }
            DataFormat::Csv => {
                let rows = csv::ReaderBuilder::new()
                    .has_headers(false)
                    .flexible(true)
                    .from_reader(reader)
                    .into_records()
                    .enumerate()
                    .map(move |(index, row)| {
                        let row = row.with_context(|| {
                            format!("When reading CSV record {}", index + 1)
                        })?;
                        self.content()
                            .iter()
                            .map(|question| {
                                let question = question.read();
                                let field =
                                    question.locate_data_csv(&row).with_context(|| {
                                        format!(
                                            "When trying to extract data for question `{}`",
                                            question.name()
                                        )
                                    })?;
                                Ok(question.parse_question_value(field)?.into_static())
                            })
                            .collect::<Result<Vec<_>>>()
                            .map(Row)
                    });
                Ok(Box::new(rows))
            }
        }
    }

    /// Update data source after validation.
    ///
    /// This method will:
    ///     - check the file extension type and if the `path` actually exists.
    ///
    /// if input argument `validate` was `true`:
    ///     - Scan the data row by row to check if there were any errors, and raises error
    ///       should there were any.
    ///
    /// And finally, this method will *overwrite* these fields:
    /// 1) the 'href' field with the provided 'path' attribute
    /// 2) `data_format` field according to the file extension of the `path` provided ([`CodeFormat::Fixed`] or [`CodeFormat::Csv`])
    ///
    /// CSV positions are one-based field numbers.
    /// The record's `skip` attribute controls how many initial records to ignore in either
    /// format.
    pub fn set_data_source(&mut self, path: impl AsRef<Path>, validate: bool) -> Result<()> {
        let path_ref = path.as_ref();
        let path_string = path_ref.to_string_lossy().into_owned();

        // parse and update data source
        let extension = path_ref
            .extension()
            .ok_or(InvalidArg::new(format!(
                "Invalid file type: Expect '.csv' or '.asc' file, got: {}",
                path_string
            )))?
            .to_string_lossy()
            .into_owned();
        let new_data_format = match extension.as_str() {
            "asc" => DataFormat::Fixed,
            "csv" => DataFormat::Csv,
            _ => {
                return Err(InvalidArg::new(format!(
                    "Invalid file type: Expect '.csv' or '.asc' file, got: {}",
                    path_string
                ))
                .into());
            }
        };
        *self.data_format_mut() = Some(new_data_format);

        // read & validate file
        let file = File::open(path_ref)
            .map_err(error::FileReadError::from)
            .with_context(|| format!("When opening '{}'", path_string))?;
        let skip = self
            .skip
            .as_deref()
            .unwrap_or("0")
            .parse::<usize>()
            .context("Record skip must be a non-negative integer")?;

        // do an initial scan of the data source, checking if there were any errors for the data file
        if validate {
            self.validate_data_format(file, skip)
                .with_context(|| format!("When opening '{}'", path_string))?;
        }

        *self.href_mut() = Some(path_string);
        Ok(())
    }

    /// Scan data source as (provided in `path`) to validate data from each respondent,
    /// raises error if there were anything wrong
    /// This method would not load any data into the memory.
    fn validate_data_format(&self, file: File, skip: usize) -> Result<()> {
        let file = TextReader::new(BufReader::new(file));
        let data_format = self.data_format().unwrap_or(&DataFormat::Fixed);
        let content = self.content();

        match data_format {
            DataFormat::Csv => {
                let mut reader = csv::ReaderBuilder::new()
                    .has_headers(false)
                    .flexible(true)
                    .from_reader(file);

                for (index, row) in reader.records().enumerate().skip(skip) {
                    let row =
                        row.with_context(|| format!("When reading CSV record {}", index + 1))?;

                    content
                        .par_iter()
                        .try_for_each(|item| item.read().validate_csv_entry(&row))
                        .with_context(|| {
                            format!("When trying to extract data from line {}", index)
                        })?;
                }
                Ok(())
            }
            DataFormat::Fixed => {
                let lines = BufReader::new(file).lines(); // each line is treated as data from one respondent

                for (index, line) in lines.enumerate().skip(skip) {
                    let line = line?;
                    content
                        .par_iter()
                        .try_for_each(|item| item.read().validate_asc_entry(&line))
                        .with_context(|| {
                            format!("When trying to extract data from line {}", index)
                        })?;
                }

                Ok(())
            }
        }
    }

    /// extract one row of data from a fixed width data read
    fn read_one_record<R: BufRead>(reader: &mut R, record_width: usize) -> Result<String> {
        let mut result = String::with_capacity(record_width);

        for _ in 0..record_width {
            let mut first = [0u8; 1];

            // EOF
            if reader.read(&mut first)? == 0 {
                break;
            }
            let len = match first[0] {
                0x00..=0x7f => 1,
                0xc2..=0xdf => 2,
                0xe0..=0xef => 3,
                0xf0..=0xf4 => 4,
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid UTF-8",
                    )
                    .into());
                }
            };
            let mut bytes = [0u8; 4];
            bytes[0] = first[0];
            reader.read_exact(&mut bytes[1..len])?;

            let character = std::str::from_utf8(&bytes[..len])
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            result.push_str(character);
        }

        // Record separators are not part of the fixed-width data. Peek so
        // separator-free records and significant field spaces stay intact.
        if reader.fill_buf()?.first() == Some(&b'\r') {
            reader.consume(1);
        }
        if reader.fill_buf()?.first() == Some(&b'\n') {
            reader.consume(1);
        }

        Ok(result)
    }
}

/// Export data
///
/// ---
impl Record {
    /// Convert question data to polars dataframe
    pub fn to_polars(&self) -> PolarsResult<DataFrame> {
        self.data().to_polars()
    }

    /// Dump cached data to parquet datafile
    pub fn to_parquet(&self, parquet_path: impl AsRef<Path>) -> PolarsResult<()> {
        let path_sink = PlRefPath::try_from_path(parquet_path.as_ref())?;
        let path_sink = SinkDestination::File {
            target: SinkTarget::Path(path_sink),
        };

        let write_option = ParquetWriteOptions::default();
        let write_formt = FileWriteFormat::Parquet(Arc::new(write_option));

        self.data()
            .lazy()?
            .clone()
            .sink(path_sink, write_formt, UnifiedSinkArgs::default())?
            .collect()?;

        Ok(())
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
    #[getter(data_format)]
    fn py_get_data_format(&self) -> PyResult<Option<DataFormat>> {
        Ok(self.data_format.clone())
    }

    #[setter(data_format)]
    fn py_set_data_format<'py>(
        &mut self,
        data_format: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<()> {
        self.data_format = match data_format {
            None => None,
            Some(bound) => {
                let value = if let Ok(value) = bound.extract::<DataFormat>() {
                    value
                } else if let Ok(value) = bound.extract::<&str>() {
                    value
                        .parse::<DataFormat>()
                        .with_context(||
                            format!(
                                "Expect to triple_s.DataFormat instance or one of {:?}, got: '{}'.",
                                DataFormat::available_options(),
                                value
                            )
                        )
                        .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))?
                } else {
                    let type_name = get_type_name(bound)?;
                    return Err(exceptions::PyTypeError::new_err(format!(
                        "Expect to triple_s.DataFormat instance or one of {:?}, got: '{}'.",
                        DataFormat::available_options(),
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

    /// (Optional) corresponds to the 'skip' attribute of the `<record>` xml element.
    ///
    /// Used to ignore one or more initial records in the Data File.
    /// This will be most useful for csv Data Files where the first line is often used as
    /// documentation (e.g. names for the columns/fields) for the succeeding values.
    #[getter(skip)]
    fn py_get_skip(&self) -> PyResult<Option<String>> {
        Ok(self.skip.clone())
    }

    #[setter(skip)]
    fn py_set_skip(&mut self, skip: Option<String>) -> PyResult<()> {
        self.skip = skip;
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

    /// Load fixed-width or CSV data using the record's format and skip
    /// metadata.
    #[pyo3(name="load_data", signature=(path:"str"))]
    pub fn py_load_data(&mut self, path: &str) -> PyResult<()> {
        self.load_data(path)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{:?}", err)))
    }

    /// Export loaded data into polars dataframe
    #[pyo3(name = "to_polars")]
    pub fn py_to_polars<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.data().py_to_polars(py)
    }

    /// Export loaded data as a pandas DataFrame in question order.
    #[pyo3(name = "to_pandas")]
    pub fn py_to_pandas<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame = self.py_to_polars(py)?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("use_pyarrow_extension_array", true)?;
        frame.call_method("to_pandas", (), Some(&kwargs))
    }

    /// Dump loaded data to parquet file
    #[pyo3(name = "to_parquet")]
    pub fn py_to_parquet(&self, parquet_path: &str) -> PyResult<()> {
        self.to_parquet(parquet_path).map_err(PyPolarsErr::Polars)?;
        Ok(())
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

    /// Create new instance from dictionary with the following keys:
    ///
    /// 'ident' (str|None, *optional):
    ///     id of the `Record` instance, default 'A'.
    ///
    /// 'href' (str|None, *optional*):
    ///     href,
    ///     Default to be 'None'
    ///
    /// 'data_format' (str|DataFormat|None, *optional*):
    ///     data source, default 'None'
    ///
    /// 'content' (List[Question|dict], *optional*):
    ///     list of [`Question`] instance or dictionaries to pass to
    ///     Question.from_dict,
    ///     Default to be an empty list (`[]`)
    ///
    /// 'data' (SurveyData|None, *optional*):
    ///     data of the corresponding survey.
    ///     Default SurveyData.empty()
    #[pyo3(name = "from_dict")]
    #[staticmethod]
    #[allow(unused_variables)]
    pub(crate) fn py_from_dict(content: Bound<'_, PyDict>) -> PyResult<Self> {
        let ident = content
            .get_item("ident")?
            .map(|item| item.extract::<Option<char>>())
            .transpose()?
            .flatten()
            .unwrap_or('A');

        let href = content
            .get_item("href")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();

        let skip = content
            .get_item("skip")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();

        let questions = content
            .get_item("content")?
            .map(|item| item.extract::<Vec<Bound<'_, PyAny>>>())
            .transpose()?
            .unwrap_or_default()
            .into_iter()
            .map(|item| {
                let question = Question::from_py_any(item)?;
                Ok(lock_it_up(question))
            })
            .collect::<PyResult<Vec<ArcLock<Question>>>>()?;

        let data_format = content
            .get_item("data_format")?
            .map(|item| {
                if let Ok(item) = item.extract::<Option<String>>() {
                    item.map(|val|
                        DataFormat::from_str(&val)
                            .with_context(||
                                format!(
                                    "Expect to be one of: {:?}",
                                    DataFormat::available_options()
                                )
                            )
                            .map_err(|err| exceptions::PyValueError::new_err(format!("{:?}", err)))
                    ).transpose()
                } else if let Ok(val) = item.extract::<DataFormat>() {
                    Ok(Some(val))
                } else {
                    let type_name = get_type_name(&item)?;
                    Err(
                        exceptions::PyTypeError::new_err(
                            format!(
                                "Expect value of 'data_format' key to be one of {:?}, NoneType or `DataFormat` instance, got '{}'",
                                DataFormat::available_options(),
                                type_name
                            )
                        )
                    )
                }
            })
            .transpose()?
            .flatten();

        let data = content
            .get_item("data")?
            .map(|item| {
                if let Ok(value) = item.extract::<SurveyData>() {
                    Ok(value)
                } else {
                    let type_name = get_type_name(&item)?;
                    Err(exceptions::PyTypeError::new_err(type_name))
                }
            })
            .transpose()?
            .unwrap_or(SurveyData::default());

        Ok(Self {
            id: ident,
            href: href,
            skip: skip,
            data_format: data_format,
            content: lock_it_up(questions),
            data: lock_it_up(data),
        })
    }

    /// Update path to the data file ('href' attribute) after validation.
    ///
    /// This method will:
    ///     - check the file extension type and update the corresponding `data_format` field
    ///       accordingly.
    ///     - Scan the data row by row to check if there were any errors, and raises error
    ///       should there were any.
    ///     - If no error was raised during the scan, this method will *overwrite* the 'href'
    ///       field with the provided 'path' attribute.
    ///
    /// CSV positions are one-based field numbers.
    /// The record's `skip` attribute controls how many initial records to ignore in either
    /// format.
    #[pyo3(name = "update_data_source")]
    fn py_update_data_source(&mut self, path: Option<&str>) -> PyResult<()> {
        match path {
            None => {
                *self.href_mut() = None;
                Ok(())
            }
            Some(path) => self.set_data_source(path, true).map_err(py_value_error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn unicode_survey(csv: bool) -> super::super::Survey {
        let positions = if csv {
            ("1", "1", "2", "2")
        } else {
            ("1", "3", "4", "5")
        };
        crate
            ::read_from_string(
                &format!(
                    r#"<sss version="2.0"><survey><record ident="V" format="{}">
            <variable ident="1" type="character"><name>TEXT</name>
                <position start="{}" finish="{}"/><size>3</size></variable>
            <variable ident="2" type="quantity"><name>NUMBER</name>
                <position start="{}" finish="{}"/><values><range from="0" to="99"/></values></variable>
            </record></survey></sss>"#,
                    if csv {
                        "csv"
                    } else {
                        "fixed"
                    },
                    positions.0,
                    positions.1,
                    positions.2,
                    positions.3
                )
            )
            .unwrap()
    }

    #[test]
    fn unicode_data_loads_and_validates_in_both_formats() {
        for csv in [false, true] {
            let text = if csv {
                "中文😀,17\r\n北京é,23"
            } else {
                "中文😀17\r\n北京é23"
            };
            let utf16: Vec<u8> = std::iter::once(0xfeff)
                .chain(text.encode_utf16())
                .flat_map(u16::to_le_bytes)
                .collect();
            for bytes in [text.as_bytes(), utf16.as_slice()] {
                let mut file = tempfile::Builder::new()
                    .suffix(if csv { ".csv" } else { ".asc" })
                    .tempfile()
                    .unwrap();
                file.write_all(bytes).unwrap();
                let mut survey = unicode_survey(csv);
                survey.record_mut().set_data_source(file.path(), true).unwrap();
                survey.load_data(file.path()).unwrap();
                let frame = survey.record().to_polars().unwrap();
                assert_eq!(frame.shape(), (2, 2));
                assert_eq!(
                    frame.column("TEXT").unwrap().str().unwrap().get(0),
                    Some("中文😀")
                );
                assert_eq!(
                    frame.column("TEXT").unwrap().str().unwrap().get(1),
                    Some("北京é")
                );
                assert_eq!(
                    frame.column("NUMBER").unwrap().f64().unwrap().get(0),
                    Some(17.0)
                );
                assert_eq!(
                    frame.column("NUMBER").unwrap().f64().unwrap().get(1),
                    Some(23.0)
                );
            }
        }
    }

    #[test]
    fn fixed_width_record_separators_preserve_fields() {
        for separator in ["\n", "\r\n", ""] {
            for trailing_separator in [false, true] {
                let mut input = ["a   ", "  b ", "    "].join(separator);
                if trailing_separator {
                    input.push_str(separator);
                }
                // A tiny buffer also exercises CRLF across buffer boundaries.
                let mut reader = BufReader::with_capacity(1, input.as_bytes());
                for expected in ["a   ", "  b ", "    ", ""] {
                    assert_eq!(Record::read_one_record(&mut reader, 4).unwrap(), expected);
                }
            }
        }
    }
}
