use pyo3::{
    types::PyString,
    exceptions::PyValueError, 
    prelude::*,
    Borrowed,
};
use std::convert::Infallible;
use strum::{EnumIter, EnumString, IntoEnumIterator};

#[pyclass(from_py_object)]
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

impl Version {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1_0 => "1.0",
            Self::V1_1 => "1.1",
            Self::V1_2 => "1.2",
            Self::V2_0 => "2.0",
            Self::V3_0 => "3.0",
        }
    }

    pub fn available_options() -> Vec<String> {
        Self::iter()
            .map(|version| version.as_str().to_string())
            .collect()
    }
}

// impl<'a, 'py> FromPyObject<'a, 'py> for Version {
//     type Error = PyErr;

//     fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
//         let value = obj
//             .extract::<&str>()
//             .map_err(|_| PyValueError::new_err(
//                 "version must be a string",
//             ))?;

//         value.parse::<Version>().map_err(|_| {
//             PyValueError::new_err(format!(
//                 "invalid version {:?}; expected one of: {:?}",
//                 value,
//                 Self::available_options()
//             ))
//         })
//     }
// }

// impl<'py> IntoPyObject<'py> for Version {
//     type Target = PyString;
//     type Output = Bound<'py, PyString>;
//     type Error = Infallible;

//     fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
//         Ok(PyString::new(py, self.as_str()))
//     }
// }

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
#[pyclass(from_py_object)]
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

// impl Mode {
//     pub fn try_as_str(&self) -> Option<&'static str> {
//         match self {
//             Self::None => None,
//             Self::Analysis => Some("analysis"),
//             Self::Interview => Some("interview"),
//         }
//     }

//     pub fn available_options() -> Vec<&'static str> {
//         Self::iter().filter_map(|mode| mode.try_as_str()).collect()
//     }
// }

// impl<'a, 'py> FromPyObject<'a, 'py> for Mode {
//     type Error = PyErr;

//     fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
//         // Python None -> Mode::None
//         if obj.is_none() {
//             return Ok(Self::None);
//         }

//         // Python str -> Mode variant
//         let value = obj.extract::<&str>()?;

//         value.parse::<Self>().map_err(|_| {
//             PyValueError::new_err(format!(
//                 "invalid mode: {value:?}; expected one of: {}",
//                 Self::available_options().join(", ")
//             ))
//         })
//     }
// }

// impl<'py> IntoPyObject<'py> for Mode {
//     type Target = PyAny;
//     type Output = Bound<'py, PyAny>;
//     type Error = Infallible;

//     fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
//         match self.try_as_str() {
//             Some(mode) => Ok(PyString::new(py, mode).into_any()),
//             None => Ok(py.None().into_bound(py)),
//         }
//     }
// }

#[pyclass(from_py_object)]
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

impl DataSource {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Fixed => "fixed",
        }
    }

    pub fn available_options() -> Vec<String> {
        Self::iter()
            .map(|item| item.as_str().to_string())
            .collect()
    }
}

// impl<'a, 'py> FromPyObject<'a, 'py> for DataSource {
//     type Error = PyErr;

//     fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
//         let value = obj.extract::<&str>()?;

//         value.parse::<Self>().map_err(|_| {
//             PyValueError::new_err(format!(
//                 "invalid data source: {:?}; expected one of: {:?}",
//                 value,
//                 Self::available_options()
//             ))
//         })
//     }
// }

// impl<'py> IntoPyObject<'py> for DataSource {
//     type Target = PyString;
//     type Output = Bound<'py, PyString>;
//     type Error = Infallible;

//     fn into_pyobject(
//         self,
//         py: Python<'py>,
//     ) -> Result<Self::Output, Self::Error> {
//         Ok(PyString::new(py, self.as_str()))
//     }
// }