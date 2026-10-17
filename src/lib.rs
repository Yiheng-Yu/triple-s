#![warn(missing_docs)]

//! A parser for reading parsing [Triple-S](https://www.triple-s.org/) datamap files (.sss).
//!
//! [Triple-S](https://triple-s.org/documentation/faq/) is a file format used for handling survey data.
//! This format has been relatively popular and been used accross many different
//! places, and most of the major survey platforms uses triple-s to transfer
//! survey data from one place to another.
//!
//! It typically consists of two independent files:
//! A. a data file, this can either be a _text_ .asc file that contains the raw
//! data, or a _csv_ that with question name as column headers. Most of the data
//! files are of the former kind. B. a datamap that record how the data file
//! should be read. i.e., where's the data for each question for each respondent
//! etc.,
//!
//! The current library provides a (very!) fast and convenient way of reading/
//! loading/ modfying triple-s files. Supports all triple-s file format
//! (triple-s classic & xml) up to triple-s 2.0.  
//!
//! # IMPORTANT: Trait `DeepCopy`
//!
//! In order to accomdate python's (painful) dynamic typing & mutation, most of
//! the fields for `Survey` and its members (`Question`, `Response` etc,) are
//! wrapped inside `Arc<parking_lot::RwLock<T>>`. You should be able to access
//! them via their respective method calls, please refer to the documentation
//! for each item for the list of fields accessible.
//!
//! Because of this, regular `clone()` method will only clone the pointer to the
//! data (the `Arc`), and hence mutating the cloned copy WILL mutate the
//! original item as well:
//!
//! ## Example
//!
//! ```rust
//! use triple_s::Text;
//!
//! let original = Text::new("A");
//! let mut copy = original.clone();
//!
//! // this mutates the original data!
//! *copy.value_mut() = "B".to_string();
//! assert_eq!(original.value(), copy.value());
//! assert_eq!(original.value(), "B".to_string());
//! ```
//!
//! All structs in this library comes with `DeepCopy` trait. This trait provides
//! `deepcopy()` method call that actually clones the data that the `Arc`
//! pointers points to. This would crete a stand-alone clone of the object, and
//! mutating the item returned by `deepcopy()` will not update the original
//! data:
//!
//! ## Example
//!
//! Import `DeepCopy` trait, or call `use triple_s::prelude::*` that does the
//! same job:
//!
//! ```rust
//! use triple_s::{DeepCopy, Text};
//! // Or:
//! // use triple_s::prelude::*
//!
//! let original = Text::new("A");
//! let mut copy = original.deepcopy();
//!
//! *copy.value_mut() = "B".to_string(); // will not update the original
//!
//! assert_ne!(original.value(), copy.value());
//! assert_eq!(original.value(), "A".to_string());
//! ```
//!
//! # Quick start
//!
//! You can read a datamap from an existing file:
//!
//! ```no_run
//! use triple_s::read_datamap;
//!
//! let path = "path/to/the/datamap.sss";
//! let datamap = read_datamap(path).unwrap();
//! ```
//! ..or from in-memory string:
//!
//! ```no_run
//! use triple_s::read_from_string;
//!
//! let datamap_string: &str = r#"{..triple-s xml/ classic}"#;
//! let datamap = read_from_string(datamap_string).unwrap();
//! ```
//!
//! ## Survey
//!
//! [`Survey`] represents a survey definition, it's the root of a triple-s
//! datamap file. It mainly consists of two parts:
//! - [`SurveyInfo`](survey_info): Metadata about a survey. i.e., name, title,
//!   datetime of the creation, file format etc.,
//! - [`Record`](record): collection of [`Question`](question)s in a survey.
//!
//! Creating a new [`Survey`]:
//!
//! ```rust
//! use triple_s::{Survey, SurveyInfo, Version};
//!
//! let info = SurveyInfo::new(Version::V1_2).with_name(Some("Survey1"));
//! let survey = Survey::new(info);
//! ```
//!
//! Load a datamap with [`read_datamap`] or [`read_from_string`], then access
//! its metadata and record directly or find a question by name with
//! [`Survey::get_question`]. You can also start with [`Survey::empty`] and
//! build a survey definition programmatically.

mod member;
pub mod error;
pub use member::{question, record, response, survey_info, text};
mod prelude;
mod utils;

use std::path::Path;
use anyhow::Result;

pub use member::{
    Survey,
    question::{CodeFormat, Location, Question, QuestionOptions, QuestionType, Spread, Usage},
    read::{read_datamap, read_from_reader, read_from_string},
    record::{DataSource, Record},
    response::{Code, DataRange, Response, Value},
    survey_info::{SurveyInfo, Version},
    text::{Mode, Text, Variant},
    entry::{Entry},
};
use error::EventLog;
pub use prelude::DeepCopy;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

// ----------------------------------------------------------------------------

/// Read triple_s file.
/// 
/// Arguments:
/// `path_sss`: path to the triple-s datamap file.
/// `path_data`: path to the `.asc` or `.csv` data file that pairs with the datamap.
pub fn read_triple_s(
    path_sss: impl AsRef<Path>, 
    path_data: Option<impl AsRef<Path>>
) -> Result<Survey> {
    let mut survey = read_datamap(path_sss)?;
    if let Some(path_data) = path_data {
        survey.load_data(path_data)?;
    };

    Ok(survey)
} 

/// A Python module implemented in Rust.
#[cfg(feature = "pyo3")]
#[pymodule]
mod triple_s {
    use anyhow::Context;
    use pyo3::prelude::*;

    use crate::{member::read::read_from_reader, read_triple_s};
    #[pymodule_export]
    pub use crate::member::{
        Survey,
        question::{CodeFormat, Location, Question, QuestionType, Spread, Usage},
        record::{DataSource, Record},
        response::{Code, DataRange, Response, Value},
        survey_info::{SurveyInfo, Version},
        text::{Mode, Text, Variant},
    };

    /// Read triple-s datamap
    /// 
    /// Args:
    ///     path_datamap (str): path to the triple-s datamap file, supports both classic and xml format.
    #[pyfunction(name="read_datamap", signature=(path_datamap: "str"))]
    pub fn py_read_datamap(py: Python<'_>, path_datamap: &str) -> PyResult<Survey> {
        let input = std::fs::File::open(path_datamap)?;

        let (survey, read_log) = read_from_reader(std::io::BufReader::new(input))
            .with_context(|| format!("When reading '{}'", path_datamap))?;
        if read_log.warnings.len() > 0 {
            read_log.print_warning_summary(py, &format!("Reading '{path_datamap}'"))?
        };
        Ok(survey)
    }
 
    /// Read triple-s datamap.
    /// 
    /// Args:
    ///     path_datamap (str): path to the triple-s datamap file, supports both classic and xml format.
    ///     path_data (str, *optional*): path to the `.asc` or `.csv` data file.
    #[pyfunction(name="read_triple_s", signature=(path_datamap: "str", path_data:"str"=None))]
    pub fn py_read_triple_s(
        path_datamap: &str,
        path_data: Option<&str>
    ) -> PyResult<Survey> {
        read_triple_s(path_datamap, path_data)
            .map_err(|err| err.into())
    }
}