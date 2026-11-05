//! Configuration options for reading/ exporting/ intereacting with objects
#[cfg(feature="pyo3")]
use std::str::FromStr;

#[cfg(feature="pyo3")]
use crate::utils::get_type_name;

#[cfg(feature="pyo3")]
use pyo3::{exceptions, prelude::*};

use strum::{EnumString, EnumIter};

use crate::prelude::*;

/// General configurations
#[derive(Debug, Clone)]
pub struct Config {
    /// Specifies the DataFrame library used for Python outputs 
    /// default `Polars`.
    pub py_dataframe_format: DataFrameFormat,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            py_dataframe_format: DataFrameFormat::Polars
        }
    }
}

/// Specifies the default DataFrame library used for Python output, default `Polars`.
/// 
/// This can be overriden by individal functional calls.
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Default, Debug, Clone, EnumString, EnumIter)]
pub enum DataFrameFormat {
    /// Polars dataframe
    #[default]
    #[strum(serialize = "polars")]
    Polars,

    /// Pandas dataframe
    #[strum(serialize = "pandas")]
    Pandas
}

impl DataFrameFormat {
    #[cfg(feature="pyo3")]
    pub(crate) fn from_py_any<'py>(value: Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(value) = value.extract::<DataFrameFormat>() {
            Ok(value)
        } else if let Ok(value) = value.extract::<String>() {
            DataFrameFormat::from_str(&value)
                .map_err(|_| exceptions::PyValueError::new_err(format!(
                    "Expect input to be one of ['pandas', 'polars'] or DataFrameFormat instance, got: '{}'", value
                )))
        } else {
            let type_name = get_type_name(&value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect input to be one of ['pandas', 'polars'] or DataFrameFormat instance, got: '{}'", type_name
            )))
        }
    }
}

impl Flag for DataFrameFormat {
    fn to_string(&self) -> String {
        match self {
            Self::Polars => "polars".into(),
            Self::Pandas => "pandas".into(),
        }
    }
}
