use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use parking_lot::Mutex;
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3_polars::{PyDataFrame, error::PyPolarsErr};
use tempfile::TempPath;

use crate::error::{InvalidArg, NoDataError};
#[cfg(feature = "pyo3")]
use crate::utils::polars_to_pandas;

/// The backing file and its cleanup policy.
#[derive(Debug)]
enum CacheDir {
    /// An owned temporary file that is deleted on drop.
    Temporary(TempPath),

    /// An existing file that is preserved on drop.
    Persistent(PathBuf),
}

impl CacheDir {
    pub fn to_path_buf(&self) -> Result<PathBuf, InvalidArg> {
        match self {
            CacheDir::Temporary(path) => Ok(path.to_path_buf()),
            CacheDir::Persistent(path) => Ok(path.clone()),
        }
    }
}

/// Respondent data with a lazy query plan and an optional backing cache.
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "SurveyData", from_py_object)
)]
#[derive(Default)]
pub struct SurveyData {
    /// The query plan for reading data
    frame: Option<LazyFrame>,

    /// Cache that the `frame` sink into
    cache: Option<Arc<CacheDir>>,

    value_counts_cache: Arc<Mutex<HashMap<(String, bool, bool), DataFrame>>>,
}

impl Clone for SurveyData {
    fn clone(&self) -> Self {
        match self.frame.as_ref() {
            Some(lf) => {
                let new = lf.clone();
                Self {
                    frame: Some(new),
                    cache: self.cache.clone(),
                    value_counts_cache: Arc::clone(&self.value_counts_cache),
                }
            }
            None => Self::default(),
        }
    }
}

/// Fields
///
/// ---
impl SurveyData {
    /// Get the backing file and its cleanup policy, if present.
    pub fn cache(&self) -> Option<&str> {
        match self.cache.as_ref() {
            None => None,
            Some(cache) => match cache.as_ref() {
                CacheDir::Temporary(path) => Some(path.to_str()).flatten(),
                CacheDir::Persistent(path) => Some(path.to_str()).flatten(),
            },
        }
    }

    /// Write the current data to a new cache file, deleted on drop.
    pub fn set_cache(&mut self, cache_dest: impl AsRef<Path>) -> Result<()> {
        let cache_dest = std::path::absolute(cache_dest.as_ref())?;

        let parent = cache_dest.parent().unwrap_or(&cache_dest);
        let mut staged = tempfile::NamedTempFile::with_suffix_in(".parquet", parent)?;
        ParquetWriter::new(staged.as_file_mut()).finish(&mut self.to_polars()?)?;

        let mut cache = TempPath::try_from_path(&cache_dest)?;
        // Take ownership only after successfully publishing the new file.
        cache.disable_cleanup(true);
        drop(staged.persist_noclobber(&cache_dest)?);
        cache.disable_cleanup(false);

        *self = Self::from_cache(cache, None)?;
        Ok(())
    }

    /// Get the [`LazyFrame`] of the underlying survey data.
    #[inline]
    pub fn lazy(&self) -> Result<&LazyFrame, NoDataError> {
        self.frame.as_ref().ok_or(NoDataError {})
    }
}

impl SurveyData {
    /// Creates an empty [`SurveyData`] instance
    ///
    /// Example Creating new empty [`SurveyData`]:
    ///
    /// ```
    /// use triple_s::SurveyData;
    ///
    /// let data = SurveyData::empty();
    /// assert_eq!(data.has_data(), false);
    /// ```
    pub fn empty() -> Self {
        Self::default()
    }

    /// check if data loaded or not
    ///
    /// Example:
    ///
    /// ```
    /// use triple_s::SurveyData;
    ///
    /// let data = SurveyData::default();
    /// assert_eq!(data.has_data(), false);
    /// ```
    #[inline]
    pub fn has_data(&self) -> bool {
        self.frame.is_some()
    }

    /// Create new instance fro parquet file
    pub fn from_parquet(
        path_parquet: impl AsRef<Path>,
        schema: Option<Arc<Schema>>,
    ) -> Result<Self> {
        let cache = CacheDir::Persistent(std::path::absolute(path_parquet.as_ref())?);
        let args = ScanArgsParquet {
            schema,
            ..Default::default()
        };

        let path_buf = cache.to_path_buf()?;
        let path = PlRefPath::try_from_path(path_buf.as_path())?;
        let lazyframe = LazyFrame::scan_parquet(path, args)?;

        Ok(Self {
            frame: Some(lazyframe),
            cache: Some(Arc::new(cache)),
            value_counts_cache: Arc::default(),
        })
    }

    /// Create new instance from in-memory [`polars::prelude::LazyFrame`]
    pub fn from_lazyframe(frame: LazyFrame) -> PolarsResult<Self> {
        let cache = tempfile::NamedTempFile::with_suffix(".parquet")?.into_temp_path();

        // create parquet cache file
        let target = PlRefPath::try_from_path(cache.as_ref())?;
        let destination = SinkDestination::File {
            target: SinkTarget::Path(target.clone()),
        };
        let format = FileWriteFormat::Parquet(Arc::new(ParquetWriteOptions::default()));
        let sink = frame.sink(destination, format, UnifiedSinkArgs::default())?;
        sink.collect()?;

        // Create a new LazyFrame that uses the cached parquet as backend
        let frame = LazyFrame::scan_parquet(target, ScanArgsParquet::default())?;

        Ok(Self {
            frame: Some(frame),
            cache: Some(Arc::new(CacheDir::Temporary(cache))),
            value_counts_cache: Arc::default(),
        })
    }

    /// Create new instance from in-memory [`polars::prelude::DataFrame`]
    pub fn from_polars(df: DataFrame) -> PolarsResult<Self> {
        Self::from_lazyframe(df.lazy())
    }

    /// create new instance from cache
    pub(crate) fn from_cache(
        cache: TempPath,
        schema: Option<Arc<Schema>>,
    ) -> PolarsResult<Self> {
        let args = ScanArgsParquet {
            schema,
            ..Default::default()
        };

        let path = PlRefPath::try_from_path(cache.as_ref())?;
        let frame = LazyFrame::scan_parquet(path, args)?;

        Ok(Self {
            frame: Some(frame),
            cache: Some(Arc::new(CacheDir::Temporary(cache))),
            value_counts_cache: Arc::default(),
        })
    }

    /// export data to polars dataframe
    pub fn to_polars(&self) -> PolarsResult<DataFrame> {
        self.lazy()
            .map_err(|err| PolarsError::NoData(err.to_string().into()))?
            .clone()
            .collect()
    }

    pub(crate) fn cached_value_counts(
        &self,
        question: &str,
        count_nan: bool,
        multiple: bool,
    ) -> Option<DataFrame> {
        if matches!(self.cache.as_deref(), Some(CacheDir::Persistent(_))) {
            return None;
        }

        self.value_counts_cache
            .lock()
            .get(&(question.to_owned(), count_nan, multiple))
            .cloned()
    }

    pub(crate) fn cache_value_counts(
        &self,
        question: &str,
        count_nan: bool,
        multiple: bool,
        counts: DataFrame,
    ) {
        if matches!(self.cache.as_deref(), Some(CacheDir::Persistent(_))) {
            return;
        }

        self.value_counts_cache
            .lock()
            .insert((question.to_owned(), count_nan, multiple), counts);
    }

    /// Dump cached data to parquet datafile
    pub fn to_parquet(&self, parquet_path: impl AsRef<Path>) -> PolarsResult<()> {
        let path_sink = PlRefPath::try_from_path(parquet_path.as_ref())?;
        let path_sink = SinkDestination::File {
            target: SinkTarget::Path(path_sink),
        };

        let write_option = ParquetWriteOptions::default();
        let write_formt = FileWriteFormat::Parquet(Arc::new(write_option));

        self.lazy()?
            .clone()
            .sink(path_sink, write_formt, UnifiedSinkArgs::default())?
            .collect()?;

        Ok(())
    }

    /// Total number of rows
    ///
    /// Will default to `0` if data was not loaded
    ///
    /// ```
    /// use triple_s::SurveyData;
    ///
    /// let empty = SurveyData::empty();
    ///
    /// assert_eq!(empty.num_response()?, 0);
    ///
    /// Ok::<(), anyhow::Error>(())
    /// ```
    pub fn num_response(&self) -> PolarsResult<usize> {
        match self.frame.as_ref() {
            Some(lf) => Ok(lf.clone().collect()?.height()),
            None => Ok(0),
        }
    }

    /// List of question names in the current instance
    pub fn questions(&self) -> PolarsResult<Vec<String>> {
        match self.frame.as_ref() {
            None => Ok(Vec::new()),
            Some(lf) => Ok(lf
                .clone()
                .collect_schema()?
                .iter_names()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()),
        }
    }

    /// Get specific row in a survey
    pub fn get_row(&self, row_index: usize) -> PolarsResult<DataFrame> {
        self.lazy()?.clone().slice(row_index as i64, 1).collect()
    }

    /// Used by Survey::__repr__
    #[cfg(feature = "pyo3")]
    pub(crate) fn py_repr_content(&self) -> PolarsResult<Vec<(&str, String)>> {
        let mut info = Vec::new();

        const MAX_NUM_COLNAME_SHOW: usize = 3;
        let colnames = self.questions().ok().unwrap_or_default();
        let mut colname_repr = colnames[0..colnames.len().min(MAX_NUM_COLNAME_SHOW)]
            .iter()
            .map(|s| format!("{:?}", s))
            .collect::<Vec<String>>()
            .join(", ");
        if colnames.len() > MAX_NUM_COLNAME_SHOW {
            colname_repr.push_str(", ...");
        }
        let colname_repr = format!("[{}]", colname_repr);
        info.push(("questions", colname_repr));

        let num_response = self.num_response()?.to_string();
        info.push(("num_response", num_response));
        Ok(info)
    }

    #[cfg(feature = "pyo3")]
    #[inline]
    pub(crate) fn create_data_entry_iterator(&self) -> PyResult<DataEntryIterator> {
        let df = self.lazy()?.clone().collect().map_err(PyPolarsErr::Polars)?;
        Ok(DataEntryIterator::new(df))
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl SurveyData {
    /// List of question names in the current instance
    #[getter(questions)]
    fn py_get_questions(&self) -> Vec<String> {
        self.questions().ok().unwrap_or_default()
    }

    #[getter(num_response)]
    fn py_get_num_response(&self) -> PyResult<usize> {
        self.num_response().map_err(|err| PyPolarsErr::Polars(err).into())
    }

    fn __repr__(&self) -> PyResult<String> {
        let mut info = Vec::new();

        const MAX_NUM_COLNAME_SHOW: usize = 3;
        let colnames = self.questions().ok().unwrap_or_default();
        let mut colname_repr = colnames[0..colnames.len().min(MAX_NUM_COLNAME_SHOW)]
            .iter()
            .map(|s| format!("{:?}", s))
            .collect::<Vec<String>>()
            .join(", ");
        if colnames.len() > MAX_NUM_COLNAME_SHOW {
            colname_repr.push_str(", ...");
        }
        let colname_repr = format!("[{}]", colname_repr);
        info.push(("questions", colname_repr));

        let num_response = self.num_response().map_err(PyPolarsErr::Polars)?.to_string();
        info.push(("num_response", num_response));

        let formatted = info
            .into_iter()
            .map(|(field, value)| format!("{}: {}", field, value))
            .collect::<Vec<String>>()
            .join("\n  ");
        Ok(format!("SurveyData({{\n  {}\n}})", formatted))
    }

    pub(crate) fn __iter__(&self, py: Python<'_>) -> PyResult<Py<DataEntryIterator>> {
        let iterator = self.create_data_entry_iterator()?;
        Py::new(py, iterator)
    }

    /// Create new empty [`SurveyData`] instance
    ///
    /// Returns:
    ///     SurveyData: The constructed SurveyData instance.
    #[pyo3(name = "empty")]
    #[staticmethod]
    fn py_empty() -> Self {
        Self::default()
    }

    /// Create new [`SurveyData`] instance from `polars.DataFrame`
    ///
    /// Args:
    ///     frame (polars.DataFrame): Respondent data, with one column per question.
    ///
    /// Returns:
    ///     SurveyData: The constructed SurveyData instance.
    #[pyo3(name = "from_polars")]
    #[staticmethod]
    fn py_from_polars(frame: PyDataFrame) -> Self {
        Self {
            frame: Some(frame.0.lazy()),
            cache: None,
            value_counts_cache: Arc::default(),
        }
    }

    /// Export data as polars DataFrame
    ///
    /// Returns:
    ///     polars.DataFrame: The respondent data in Polars format.
    #[pyo3(name = "to_polars")]
    pub fn py_to_polars<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame =
            self.to_polars().map(PyDataFrame).map_err(|err| PyPolarsErr::Polars(err))?;
        frame.into_pyobject(py)
    }

    /// Export loaded data as a pandas DataFrame
    ///
    /// Returns:
    ///     pandas.DataFrame: The respondent data in pandas format.
    #[pyo3(name = "to_pandas")]
    pub fn py_to_pandas<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let df = self.to_polars().map_err(PyPolarsErr::Polars)?;
        polars_to_pandas(df, py)
    }

    /// Get a brief overview of each column of data, equivalent of `self.to_polars().describe()`
    ///
    /// Returns:
    ///     polars.DataFrame: Summary statistics for the data columns.
    #[pyo3(name = "describe")]
    pub fn py_describe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame = self.py_to_polars(py)?;
        frame.call_method("describe", (), None)
    }

    /// Check if data loaded or not
    ///
    /// Returns:
    ///     bool: Whether respondent data is loaded.
    #[pyo3(name = "has_data")]
    fn py_has_data(&self) -> bool {
        self.has_data()
    }
}

impl std::fmt::Debug for SurveyData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurveyData").field("frame", &"LazyFrame").finish()
    }
}

// ---------- Helpers -----------
/// Iterates through individual responses of a question
#[cfg(feature = "pyo3")]
#[pyclass]
pub(crate) struct DataEntryIterator {
    pub df: DataFrame,
    pub index: usize,
}

#[cfg(feature = "pyo3")]
impl DataEntryIterator {
    pub fn new(df: DataFrame) -> Self {
        Self { df, index: 0 }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl DataEntryIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__<'py>(&mut self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        if self.index >= self.df.height() {
            return Ok(None);
        }

        let row = self.df.slice(self.index as i64, 1);

        let row = PyDataFrame(row)
            .into_pyobject(py)?
            .call_method("to_dicts", (), None)? // convert to list of dictionaries
            .call_method("__getitem__", (0,).into_pyobject(py)?, None)?; // take the first row

        self.index += 1;
        Ok(Some(row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporary_cache_is_replaced_and_deleted_on_drop() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let destination = dir.path().join("cache.parquet");
        let expected = df!("value" => [1i64, 2])?;
        let mut data = SurveyData::from_polars(expected.clone())?;
        let original = PathBuf::from(data.cache().unwrap());
        assert!(original.is_file());
        assert!(data.to_polars()?.equals_missing(&expected));

        data.set_cache(&destination)?;
        assert_eq!(Path::new(data.cache().unwrap()), destination);
        assert!(!original.exists());
        assert!(destination.is_file());
        assert!(data.to_polars()?.equals_missing(&expected));
        drop(data);
        assert!(!destination.exists());
        Ok(())
    }

    #[test]
    fn test_persistent_source_survives_drop_and_cache_replacement() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let source = dir.path().join("source.parquet");
        let destination = dir.path().join("cache.parquet");
        let expected = df!("value" => [1i64, 2])?;
        ParquetWriter::new(std::fs::File::create(&source)?).finish(&mut expected.clone())?;
        let original = std::fs::read(&source)?;
        drop(SurveyData::from_parquet(&source, None)?);
        assert_eq!(std::fs::read(&source)?, original);

        let mut data = SurveyData::from_parquet(&source, None)?;
        data.set_cache(&destination)?;
        assert!(data.to_polars()?.equals_missing(&expected));
        drop(data);
        assert_eq!(std::fs::read(&source)?, original);
        assert!(!destination.exists());
        Ok(())
    }

    #[test]
    fn test_failed_set_cache_preserves_files_and_removes_staging_file() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let destination = dir.path().join("existing.parquet");
        std::fs::write(&destination, b"keep me")?;
        let expected = df!("value" => [1i64, 2])?;
        let mut data = SurveyData::from_polars(expected.clone())?;
        let original = PathBuf::from(data.cache().unwrap());

        assert!(data.set_cache(&destination).is_err());
        assert!(data.set_cache(&original).is_err());
        assert_eq!(std::fs::read(&destination)?, b"keep me");
        assert_eq!(Path::new(data.cache().unwrap()), original);
        assert!(data.to_polars()?.equals_missing(&expected));
        let mut empty = SurveyData::default();
        assert!(empty.set_cache(dir.path().join("empty.parquet")).is_err());
        assert!(empty.cache().is_none());
        assert_eq!(std::fs::read_dir(dir.path())?.count(), 1);
        Ok(())
    }
}
