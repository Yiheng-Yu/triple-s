//! Configuration options for reading/ exporting/ intereacting with objects
#[cfg(feature = "pyo3")]
use std::str::FromStr;
use std::sync::Arc;

#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*};
use strum::{EnumIter, EnumString};

use crate::prelude::*;
#[cfg(feature = "pyo3")]
use crate::utils::get_type_name;

/// General configurations
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Specifies the DataFrame library used for Python outputs
    ///
    /// Default `Polars`.
    pub py_dataframe_format: DataFrameFormat,

    /// Controls whether choice responses are represented by their values or labels.
    pub response_display: ResponseDisplay,
}

impl Config {
    /// boolean for checking if `response_display == ResponseDisplay::Label`
    pub fn show_response_label(&self) -> bool {
        self.response_display == ResponseDisplay::Label
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

#[cfg_attr(
    any(not(feature = "pyo3"), doc),
    doc = r###"
Controls whether choice responses are represented by their values or labels.

Members:
`Value`:
    display code value (the `code` attribute of a `<value>` tag).
    i.e., [1, 1, 2, ...]
`Label`:
    display choice text (content of a `<value>` tag).
    i.e., ["London", "London", "Scotland", ...]
"###
)]
#[cfg_attr(
    all(feature = "pyo3", not(doc)),
    doc = r###"
Controls whether choice responses are represented by their values or labels.

Notes:
    Members:

    ``Value``:
        display code value (the `code` attribute of a `<value>` tag).
        i.e., [1, 1, 2, ...]

    ``Label``:
        display choice text (content of a `<value>` tag).
        i.e., ["London", "London", "Scotland", ...]
"###
)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Default, Debug, Clone, EnumString, EnumIter, PartialEq, Eq)]
pub enum ResponseDisplay {
    /// Display code value (the `code` attribute of a `<value>` tag), i.e. [1, 1, 2, ...].
    #[default]
    #[strum(serialize = "value")]
    Value,

    /// Display choice text (content of a `<value>` tag), i.e. ["London", "London", "Scotland", ...].
    #[strum(serialize = "label")]
    Label,
}

impl ResponseDisplay {
    #[cfg(feature = "pyo3")]
    pub(crate) fn from_py_any<'py>(value: Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(value) = value.extract::<ResponseDisplay>() {
            Ok(value)
        } else if let Ok(value) = value.extract::<String>() {
            Self::from_str(&value)
                .map_err(|_| exceptions::PyValueError::new_err(format!(
                    "Expect input to be one of ['value', 'label'] or `ResponseDisplay` instance, got: '{}'", value
                )))
        } else {
            let type_name = get_type_name(&value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect input to be one of ['value', 'label'] or `ResponseDisplay` instance, got: '{}'",
                type_name
            )))
        }
    }
}

impl Flag for ResponseDisplay {
    fn to_string(&self) -> String {
        match self {
            Self::Value => "value".into(),
            Self::Label => "label".into(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ValueCountArg {
    pub question_name: Arc<str>,
    pub count_nan: bool,
}

impl ValueCountArg {
    pub fn new(question_name: impl Into<String>, count_nan: bool) -> Self {
        Self {
            question_name: Arc::from(question_name.into().as_str()),
            count_nan: count_nan
        }
    }
}

#[cfg(test)]
mod test {

    use std::collections::HashMap;
    use super::*;

    #[test]
    fn test_resp_display_eq() {
        let mut config = Config::default();
        config.response_display = ResponseDisplay::Label;
        assert_eq!(config.show_response_label(), true);
        
        config.response_display = ResponseDisplay::Value;
        assert_eq!(config.show_response_label(), false)
    }

    #[test]
    fn test_value_count_arg_eq() {
        assert_eq!(
            ValueCountArg::new("haha", true),
            ValueCountArg::new("haha", true),
        );

        assert_ne!(
            ValueCountArg::new("haha", true),
            ValueCountArg::new("haha", false),
        );

        assert_ne!(
            ValueCountArg::new("haha ", true),
            ValueCountArg::new("haha", true),
        )
    }

    #[test]
    fn test_hashmap_with_value_count_as_key() {
        let k1 = ValueCountArg::new("haha", true);
        let k2 = ValueCountArg::new("haha", false);

        let map = HashMap::from([
            (k1.clone(), "what"),
            (k2.clone(), "this is k2")
        ]);

        assert_eq!(map[&k1], "what");
        assert_eq!(map.get(&k1).unwrap().clone(), "what");

        assert_eq!(map[&k2], "this is k2");
        assert_eq!(map.get(&k2).unwrap().clone(), "this is k2");
    }
}