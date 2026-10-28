use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, types::PyDict};
#[cfg(feature = "pyo3")]
use pyo3_polars::{PyDataFrame, error::PyPolarsErr};
use tempfile::TempPath;

use crate::error::{InvalidArg, NoDataError};

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
            // CacheDir::Memory => Err(InvalidArg::new("Memory cache has no filesystem path")),
        }
    }
}

/// Survey data
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "SurveyData", from_py_object)
)]
#[derive(Default)]
pub struct SurveyData {
    /// The query plan for reading data
    frame: Option<LazyFrame>,

    /// Cache that the `frame` sink into
    cache: Option<CacheDir>,
}

impl Clone for SurveyData {
    fn clone(&self) -> Self {
        match self.frame.as_ref() {
            Some(lf) => {
                let new = lf.clone();
                Self {
                    frame: Some(new),
                    cache: None,
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
            Some(cache) => match cache {
                CacheDir::Temporary(path) => Some(path.to_str()).flatten(),
                CacheDir::Persistent(path) => Some(path.to_str()).flatten(),
            },
        }
        // self.cache.as_ref()
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
    /// Create new instance fro parquet file
    pub fn from_parquet(
        path_parquet: impl AsRef<Path>,
        schema: Option<Arc<Schema>>,
    ) -> Result<Self> {
        let cache = CacheDir::Persistent(std::path::absolute(path_parquet.as_ref())?);
        let args = ScanArgsParquet { schema, ..Default::default() };

        let path_buf = cache.to_path_buf()?;
        let path = PlRefPath::try_from_path(path_buf.as_path())?;
        let lazyframe = LazyFrame::scan_parquet(path, args)?;

        Ok(Self {
            frame: Some(lazyframe),
            cache: Some(cache),
        })
    }

    /// Create new instance from in-memory [`polars::LazyFrame`]
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
            cache: Some(CacheDir::Temporary(cache)),
        })
    }

    /// Create new instance from in-memory [`polars::DataFrame`]
    pub fn from_polars(df: DataFrame) -> PolarsResult<Self> {
        Self::from_lazyframe(df.lazy())
    }

    /// create new instance from cache
    pub(crate) fn from_cache(
        cache: TempPath,
        schema: Option<Arc<Schema>>,
    ) -> Result<Self> {
        let args = ScanArgsParquet { schema, ..Default::default()};

        let path = PlRefPath::try_from_path(cache.as_ref())?;
        let frame = LazyFrame::scan_parquet(path, args)?;

        Ok(Self {
            frame: Some(frame),
            cache: Some(CacheDir::Temporary(cache)),
        })
    }

    /// export data to polars dataframe
    pub fn to_polars(&self) -> PolarsResult<DataFrame> {
        self.lazy()
            .map_err(|err| PolarsError::NoData(err.to_string().into()))?
            .clone()
            .collect()
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl SurveyData {
    fn __repr__(&self) -> String {
        let name = "SurveyData";
        let content = "";
        format!("{} {{{}}}", name, content)
    }

    /// Create new empty [`SurveyData`] instance
    #[pyo3(name = "empty")]
    #[staticmethod]
    fn py_empty() -> Self {
        Self::default()
    }

    /// Create new [`SurveyData`] instance from `polars.DataFrame`
    #[pyo3(name = "from_polars")]
    #[staticmethod]
    fn py_from_polars(frame: PyDataFrame) -> Self {
        Self {
            frame: Some(frame.0.lazy()),
            cache: None,
        }
    }

    /// Export data as polars DataFrame
    #[pyo3(name = "to_polars")]
    pub fn py_to_polars<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame =
            self.to_polars().map(PyDataFrame).map_err(|err| PyPolarsErr::Polars(err))?;
        frame.into_pyobject(py)
    }

    /// Export loaded data as a pandas DataFrame
    #[pyo3(name = "to_pandas")]
    pub fn py_to_pandas<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame = self.py_to_polars(py)?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("use_pyarrow_extension_array", true)?;
        frame.call_method("to_pandas", (), Some(&kwargs))
    }

    /// Get a brief overview of each column of data, equivalent of `self.to_polars().describe()`
    #[pyo3(name = "describe")]
    pub fn py_describe<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame = self.py_to_polars(py)?;
        frame.call_method("describe", (), None)
    }
}

impl std::fmt::Debug for SurveyData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurveyData").field("frame", &"LazyFrame").finish()
    }
}
