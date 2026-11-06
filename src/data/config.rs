//! Configuration options for reading/ exporting/ intereacting with objects
#[cfg(feature = "pyo3")]
use std::str::FromStr;

#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*};
use strum::{EnumIter, EnumString};

use crate::prelude::*;
#[cfg(feature = "pyo3")]
use crate::utils::get_type_name;

/// General configurations
#[derive(Debug, Clone)]
pub struct Config {
    /// Specifies the DataFrame library used for Python outputs
    ///
    /// Default `Polars`.
    pub py_dataframe_format: DataFrameFormat,

    /// Flag for indicating how the raw data for single and multi-choices questions should be displayed.
    pub show: Show
}

impl Default for Config {
    fn default() -> Self {
        Self {
            py_dataframe_format: DataFrameFormat::default(),
            show: Show::default()
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
    Pandas,
}

impl DataFrameFormat {
    #[cfg(feature = "pyo3")]
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
                "Expect input to be one of ['pandas', 'polars'] or DataFrameFormat instance, got: '{}'",
                type_name
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


/// Flag for indicating how the raw data for single and multi-choices questions 
/// should be displayed.
/// 
/// Members:
/// `Value`: 
///     display code value (the `code` attribute of a `<value>` tag).
///     i.e., [1, 1, 2, ...]
/// `Label`: 
///     display choice text (content of a `<value>` tag).
///     i.e., ["London", "London", "Scotland", ...]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Default, Debug, Clone, EnumString, EnumIter, PartialEq, Eq)]
pub enum Show {
    /// Display code value (the `code` attribute of a `<value>` tag), i.e. [1, 1, 2, ...].
    #[default]
    #[strum(serialize = "value")]
    Value,

    /// Display choice text (content of a `<value>` tag), i.e. ["London", "London", "Scotland", ...].
    #[strum(serialize = "label")]
    Label,
}

impl Show {
    #[cfg(feature = "pyo3")]
    pub(crate) fn from_py_any<'py>(value: Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(value) = value.extract::<Show>() {
            Ok(value)
        } else if let Ok(value) = value.extract::<String>() {
            Self::from_str(&value)
                .map_err(|_| exceptions::PyValueError::new_err(format!(
                    "Expect input to be one of ['value', 'label'] or `Show` instance, got: '{}'", value
                )))
        } else {
            let type_name = get_type_name(&value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect input to be one of ['value', 'label'] or `Show` instance, got: '{}'",
                type_name
            )))
        }
    }
}

impl Flag for Show {
    fn to_string(&self) -> String {
        match self {
            Self::Value => "value".into(),
            Self::Label => "label".into(),
        }
    }
}
