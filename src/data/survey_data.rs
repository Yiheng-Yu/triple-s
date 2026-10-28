use std::{path::Path, sync::Arc};
use crate::error::NoDataError;

use anyhow::Result;
use polars::prelude::*;
use tempfile::TempPath;

#[cfg(feature="pyo3")]
use pyo3::{prelude::*, types::PyDict};

#[cfg(feature = "pyo3")]
use pyo3_polars::{PyDataFrame, error::PyPolarsErr};

/// Survey data
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "SurveyData", skip_from_py_object)
)]
pub struct SurveyData {
    frame: Option<LazyFrame>,
    cache: Option<Arc<TempPath>>,
}

impl Default for SurveyData {
    fn default() -> Self {
        Self {
            frame: None,
            cache: None,
        }
    }
}


impl SurveyData {

    /// Get the [`LazyFrame`] of the underlying survey data.
    #[inline]
    pub fn lazy(&self) -> Result<&LazyFrame, NoDataError> {
        self.frame
            .as_ref()
            .ok_or(NoDataError {})
    }

    /// Get the mutable reference of the [`LazyFrame`] for the underlying survey data.
    #[inline]
    pub fn lazy_mut(&mut self) -> Result<&mut LazyFrame, NoDataError> {
        self.frame
            .as_mut()
            .ok_or(NoDataError {})
    }

    /// Create new instance fro parquet file
    pub fn from_parquet(
        path_parquet: impl AsRef<Path>,
        schema: Option<Arc<Schema>>,
    ) -> Result<Self> {
        let mut args = ScanArgsParquet::default();
        args.schema = schema;

        let path_parquet = PlRefPath::try_from_path(path_parquet.as_ref())?;
        let lazyframe = LazyFrame::scan_parquet(path_parquet, args)?;
        
        Ok(Self { 
            frame: Some(lazyframe),
            cache: None,
        })
    }

    /// create new instance from cache
    #[allow(dead_code)]
    pub(crate) fn from_cache(
        cache: Arc<TempPath>,
        schema: Option<Arc<Schema>>,
    ) -> Result<Self> {
        let mut args = ScanArgsParquet::default();
        args.schema = schema;

        let path = PlRefPath::try_from_path(cache.as_ref().as_ref())?;
        let frame = LazyFrame::scan_parquet(path, args)?;

        Ok(Self { 
            frame: Some(frame), 
            cache: Some(cache) })
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
    /// Create new [`SurveyData`] instance from `polars.DataFrame`
    #[pyo3(name="from_polars")]
    #[staticmethod]
    fn py_from_polars(frame: PyDataFrame) -> Self {
        Self {
            frame: Some(frame.0.lazy()),
            cache: None
        }
    }

    #[pyo3(name="to_polars")]
    fn py_to_polars<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame = self.to_polars()
            .map(PyDataFrame)
            .map_err(|err| PyPolarsErr::Polars(err))?;
        frame.into_pyobject(py)
    }

    /// Export loaded data as a pandas DataFrame
    #[pyo3(name="to_pandas")]
    pub fn py_to_pandas<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame = self.py_to_polars(py)?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("use_pyarrow_extension_array", true)?;
        frame.call_method("to_pandas", (), Some(&kwargs))
    }

    /// Get a brief overview of each column of data, equivalent of `self.to_polars().describe()`
    #[pyo3(name="describe")]
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
