
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use strum::{EnumIter, EnumString};

use crate::prelude::{DeepCopy, Flag};

#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString, EnumIter)]
pub enum Version {
    #[strum(serialize = "1.0")]
    V1_0,
    #[strum(serialize = "1.1")]
    V1_1,
    #[strum(serialize = "1.2")]
    V1_2,
    #[strum(serialize = "2.0")]
    V2_0,
    #[strum(serialize = "3.0")]
    V3_0,
}

impl Flag for Version {
    fn as_str(&self) -> &'static str {
        match self {
            Self::V1_0 => "1.0",
            Self::V1_1 => "1.1",
            Self::V1_2 => "1.2",
            Self::V2_0 => "2.0",
            Self::V3_0 => "3.0",
        }
    }
}

/// Introduced in SSS Xml 2.0.
///
/// Official documentation:
///
/// ```text
/// (Optional) The use of specialised texts for interviewing and/or analysis within the
/// specification must be signalled by a modes="mode_identifier_list"
/// attribute on the initial <sss> element.
/// Two explicit modes are available: “interview” and “analysis”. In the absence of
/// a mode specification, the appropriate text is assumed to be used in both
/// modes.
/// ```
///
/// Variants:
/// - None: default value when no 'mode' value was defined.
/// - Analysis: mode 'analysis'
/// - Interview: mode 'interview'
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Debug, Default, PartialEq, Clone, EnumString, Eq, EnumIter)]
#[strum(ascii_case_insensitive)]
pub enum Mode {
    #[default]
    None,
    #[strum(serialize = "analysis")]
    Analysis,
    #[strum(serialize = "interview")]
    Interview,
}

impl Flag for Mode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Analysis => "analysis",
            Self::Interview => "interview",
        }
    }
}

impl DeepCopy for Mode {
    fn deepcopy(&self) -> Self {
        self.clone()
    }
}

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
    fn as_str(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Fixed => "fixed",
        }
    }
}

impl DeepCopy for DataSource {
    fn deepcopy(&self) -> Self {
        self.clone()
    }
}

#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Debug, Clone, EnumString, EnumIter, Default, PartialEq)]
pub enum CodeFormat {
    #[default]
    #[strum(serialize = "numeric")]
    Numeric,
    #[strum(serialize = "literal")]
    Literal,
}

impl DeepCopy for CodeFormat {
    fn deepcopy(&self) -> Self {
        self.clone()
    }
}

impl Flag for CodeFormat {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Literal => "literal",
            Self::Numeric => "numeric",
        }
    }
}

/// describes the role of this variable in the survey.
/// Only a subset of variable types may have a use attribute (as detailed below)
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Clone, Debug, Default, EnumString, EnumIter, PartialEq)]
pub enum Usage {
    /// plain data
    #[default]
    Null,

    /// this variable contains the serial number (or other identification field)
    #[strum(serialize = "serial")]
    Serial,
    /// this variable contains the weighting
    #[strum(serialize = "weight")]
    Weight,
}

impl Flag for Usage {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Null => "",
            Self::Serial => "serial",
            Self::Weight => "weight",
        }
    }
}

impl DeepCopy for Usage {
    fn deepcopy(&self) -> Self {
        self.clone()
    }
}
