//! Configuration options for reading/ exporting/ intereacting with objects
#[cfg(feature = "pyo3")]
use std::str::FromStr;

// use std::sync::Arc;
#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*};
use strum::{EnumIter, EnumString};

use crate::prelude::*;
#[cfg(feature = "pyo3")]
use crate::utils::get_type_name;

/// General configurations
#[derive(Debug, Clone, Default)]
pub struct DisplayOptions {
    /// Specifies the DataFrame library used for Python outputs
    ///
    /// Default `Polars`.
    pub py_dataframe_format: DataFrameFormat,

    /// Controls whether choice responses are represented by their values or labels.
    pub response_display: ResponseDisplay,
}

impl DisplayOptions {
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
            DataFrameFormat::from_str(&value).map_err(|_| {
                exceptions::PyValueError::new_err(format!(
                    "Expect input to be one of ['pandas', 'polars'] or DataFrameFormat \
                     instance, got: '{}'",
                    value
                ))
            })
        } else {
            let type_name = get_type_name(&value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect input to be one of ['pandas', 'polars'] or DataFrameFormat instance, \
                 got: '{}'",
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
            Self::from_str(&value).map_err(|_| {
                exceptions::PyValueError::new_err(format!(
                    "Expect input to be one of ['value', 'label'] or `ResponseDisplay` \
                     instance, got: '{}'",
                    value
                ))
            })
        } else {
            let type_name = get_type_name(&value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect input to be one of ['value', 'label'] or `ResponseDisplay` instance, \
                 got: '{}'",
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

/// Options for running crosstabs (e.g., [`QuestionData::value_counts`], [`Survey::corsstab`])
#[allow(unused)]
pub struct CrossTabOptions<'a> {
    /// Toggle on to include a column of code values for each response item
    /// Used only for single/ multi-choice questions
    pub include_code_value: bool,

    /// Toggle on to add a column that shows the question name
    pub show_question_name: bool,

    /// Toggle on to fill the question name column with name of the question, only used when "show_question_name" is `true`. Default `false`.
    pub fill_question_name_column: bool,

    /// Column name for the question name column, default "question", only used when `show_question_name` is `true`
    pub question_name_col: Option<&'a str>,

    /// Column name for the response code column, default "code", only used when `include_code_value` is `true`
    pub code_col: Option<&'a str>,

    /// Column name for the response label column, default "label"
    pub label_col: Option<&'a str>,

    /// Column name for the value counts column, default "count" (same as polars)
    pub count_col: Option<&'a str>,
}

impl<'a> Default for CrossTabOptions<'a> {
    fn default() -> Self {
        Self {
            include_code_value: false,
            show_question_name: false,
            fill_question_name_column: false,
            question_name_col: Some("question"),
            count_col: Some("total"),
            label_col: Some("label"),
            code_col: Some("code"),
        }
    }
}

#[allow(unused)]
impl<'a> CrossTabOptions<'a> {
    /// Create new instance with default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle on to include a column of code values for each response item
    /// Used only for single/ multi-choice questions
    pub fn with_include_code_value(mut self, value: bool) -> Self {
        self.include_code_value = value;
        self
    }

    /// Toggle on to add a column that shows the question name
    pub fn with_show_question_name(mut self, value: bool) -> Self {
        self.show_question_name = value;
        self
    }

    /// Toggle on to fill the question name column with name of the question, only used when "show_question_name" is `true`. Default `false`.
    pub fn with_fill_question_name_column(mut self, value: bool) -> Self {
        self.fill_question_name_column = value;
        self
    }

    /// Column name for the question name column, default "question", only used when `show_question_name` is `true`
    pub fn with_question_name_col(mut self, value: Option<&'a str>) -> Self {
        self.question_name_col = value;
        self
    }

    /// Column name for the response code column, default "code", only used when `include_code_value` is `true`
    pub fn with_code_col(mut self, value: Option<&'a str>) -> Self {
        self.code_col = value;
        self
    }

    /// Column name for the response label column, default "label"
    pub fn with_label_col(mut self, value: Option<&'a str>) -> Self {
        self.label_col = value;
        self
    }

    /// Column name for the value counts column of the question, default "total"
    pub fn with_count_col(mut self, value: Option<&'a str>) -> Self {
        self.count_col = value;
        self
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_resp_display_eq() {
        let mut config = DisplayOptions::default();
        config.response_display = ResponseDisplay::Label;
        assert_eq!(config.show_response_label(), true);

        config.response_display = ResponseDisplay::Value;
        assert_eq!(config.show_response_label(), false)
    }
}
