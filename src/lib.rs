#![warn(missing_docs)]

//! A parser for reading parsing [Triple-S](https://www.triple-s.org/) datamap files (.sss).
//!
//! [Triple-S](https://triple-s.org/documentation/faq/) is a file format used for handling survey data.
//! This format has been relatively popular and been used accross many different
//! places, and most of the major survey platforms uses triple-s to transfer
//! survey data from one place to another.
//!
//! It typically consists of two independent files:
//!
//! A. a data file, this can either be a _text_ .asc file that contains the raw
//! data, or a _csv_ that with question name as column headers. Most of the data
//! files are of the former kind.
//!
//! B. a datamap that record how the data file
//! should be read. i.e., where's the data for each question for each respondent
//! etc.
//!
//! The current library provides a (very!) fast and convenient way of reading/
//! loading/ modfying triple-s files. Supports all triple-s file format
//! (triple-s classic & xml) up to triple-s 2.0.  
//!
//! # Quick start
//!
//! ## Read datamap
//! You can read a datamap from an existing file:
//!
//! ```no_run
//! use triple_s::read_datamap;
//!
//! let path = "path/to/the/datamap.sss";
//! let datamap = read_datamap(path)?;
//!
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ..or from in-memory string:
//!
//! ```no_run
//! use triple_s::read_from_string;
//!
//! let datamap_string: &str = r#"{..triple-s xml/ classic}"#;
//! let mut datamap = read_from_string(datamap_string)?;
//!
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! You can also read survey data with path provided
//!
//! ```no_run
//! use triple_s::{read_datamap, read_data};
//!
//! let path = "path/to/the/datamap.sss";
//! let mut datamap = read_datamap(path)?;
//!
//! let path_data = "path/to/datafile.asc";
//! let data = read_data(&datamap, path_data)?;
//!
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! Alternatively, you can load datamap and data together via `read_triple_s`
//! function:
//!
//! ```no_run
//! use triple_s::read_triple_s;
//!
//! let path_datamap: &str = "path/to/the/datamap.sss";
//! let path_data: &str = "path/to/the/data.asc";
//!
//! // Read both the datamap and respondent data.
//! let survey = read_triple_s(path_datamap, Some(path_data))?;
//!
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Datamap object
//!
//! [`Datamap`] represents a survey definition, it's the root of a triple-s
//! datamap file. It mainly consists of two parts:
//! - [`SurveyInfo`](survey_info): Metadata about a survey. i.e., name, title, datetime of the
//!   creation, file format etc.,
//! - [`Record`](record): collection of [`Question`](question)s in a survey.
//!
//! Creating a new [`Datamap`]:
//!
//! ```rust
//! use triple_s::{Datamap, SurveyInfo, Version};
//!
//! let info = SurveyInfo::new(Version::V1_2).with_name(Some("Survey1"));
//! let survey = Datamap::new(info);
//! ```
//!
//! Load a datamap with [`read_datamap`] or [`read_from_string`], then access
//! its metadata and record directly or find a question by name with
//! [`Datamap::get`]. You can also start with [`Datamap::empty`] and
//! build a survey definition programmatically.
//!
//! # IMPORTANT: Trait `DeepCopy`
//!
//! In order to accomdate python's (painful) dynamic typing & mutation, most of
//! the fields for [ `Datamap` ] and its members (`Question`, `Response` etc,) are
//! wrapped inside `Arc<parking_lot::RwLock<T>>`. You should be able to access
//! them via their respective method calls, please refer to the documentation
//! for each item for the list of fields accessible.
//!
//! Because of this, regular `clone()` method will only clone the pointer to the
//! data (the `Arc`), and hence mutating the cloned copy WILL mutate the
//! original item as well:
//!
//! ## Example: Mutation through `clone()`
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
//! Types implementing [`DeepCopy`] provide independent copies of their fields. This trait provides
//! `deepcopy()` method call that actually clones the data that the `Arc`
//! pointers points to. This would crete a stand-alone clone of the object, and
//! mutating the item returned by `deepcopy()` will not update the original
//! data:
//!
//! ## Example: Use `deepcopy` to clone the value
//!
//! Import the `DeepCopy` trait:
//!
//! ```rust
//! use triple_s::{DeepCopy, Text};
//!
//! let original = Text::new("A");
//! let mut copy = original.deepcopy();
//!
//! *copy.value_mut() = "B".to_string(); // will not update the original
//!
//! assert_ne!(original.value(), copy.value());
//! assert_eq!(original.value(), "A".to_string());
//! ```

mod data;
mod datamap;
mod survey;

pub mod error;
pub use data::config;

mod prelude;
mod utils;

use std::path::Path;

use anyhow::Result;
pub use data::{QuestionData, SurveyData};
pub use datamap::{
    read_data,
    Datamap,
    question::{
        self,
        CodeFormat,
        Location,
        Question,
        QuestionOptions,
        QuestionType,
        Spread,
        Usage,
    },
    read::{read_datamap, read_from_reader, read_from_string},
    record::{self, DataFormat, Record},
    response::{self, Code, DataRange, Response},
    survey_info::{self, SurveyInfo, Version},
    text::{self, Mode, Text, Variant},
};
use error::EventLog;
pub use prelude::DeepCopy;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
pub use survey::Survey;

// -------------------------------------------------------------------------------------------------

/// Read triple_s file.
///
/// Arguments:
/// `path_sss`:
///     path to the triple-s datamap file.
/// `path_data`:
///     path to the `.asc` or `.csv` data file that pairs with.
///     if provided, loads respondent data and updates the record's source metadata.
///     Otherwise, reads only the datamap.
///
/// Returns:
/// [`Survey`] containing the datamap and respondent data. Without `path_data`,
/// its [`SurveyData`] is empty.
pub fn read_triple_s(
    path_sss: impl AsRef<Path>,
    path_data: Option<impl AsRef<Path>>,
) -> Result<Survey> {
    log::info!("Reading datamap..");
    let mut datamap = read_datamap(path_sss)?;
    let data = match path_data {
        None => SurveyData::default(), // an empty SurveyData instance
        Some(path_data) => {
            log::info!("Loading data..");
            let path_data = path_data.as_ref();
            let data = read_data(&datamap, path_data)?;
            *datamap.record_mut().href_mut() = Some(path_data.to_string_lossy().into_owned());
            *datamap.record_mut().data_format_mut() = Some(DataFormat::from_path(path_data)?);
            data
        }
    };
    let survey = Survey::new(datamap, data);
    Ok(survey)
}

#[cfg(feature = "pyo3")]
#[pymodule]
mod triple_s {
    use anyhow::Context;
    use pyo3::prelude::*;
    #[pymodule_export]
    pub use pyo3_polars::error::NoDataError;

    #[pymodule_export]
    pub use crate::data::{QuestionData, SurveyData};
    #[pymodule_export]
    pub use crate::datamap::{
        Datamap,
        question::{CodeFormat, Location, Question, QuestionType, Spread, Usage},
        record::{DataFormat, Record},
        response::{Code, DataRange, Response},
        survey_info::{SurveyInfo, Version},
        text::{Mode, Text, Variant},
    };
    #[pymodule_export]
    pub use crate::error::MissingKeyError;
    #[pymodule_export]
    pub use crate::survey::Survey;
    use crate::{datamap::read::read_from_reader, read_triple_s};
    #[pymodule]
    pub mod config {
        #[pymodule_export]
        pub use crate::config::{DataFrameFormat, ResponseDisplay};
    }

    /// Read triple-s datamap
    ///
    /// Args:
    ///     path_datamap (str): path to the triple-s datamap file, supports both classic and xml format.
    ///
    /// Returns:
    ///     Datamap: The parsed survey definition, without respondent data.
    #[pyfunction(name="read_datamap", signature=(path_datamap: "str"))]
    pub fn py_read_datamap(py: Python<'_>, path_datamap: &str) -> PyResult<Datamap> {
        let input = std::fs::File::open(path_datamap)?;

        let (survey, read_log) = read_from_reader(std::io::BufReader::new(input))
            .with_context(|| format!("When reading '{}'", path_datamap))?;
        if read_log.warnings.len() > 0 {
            read_log.print_warning_summary(py, &format!("Reading '{path_datamap}'"))?
        };
        Ok(survey)
    }


    /// Read triple-s datamap, and load data afterwards.
    ///
    /// Args:
    ///     path_datamap (str): path to the triple-s datamap file, supports both classic and xml format.
    /// 
    /// Keyword Args:
    ///     path_data (str, *optional*): path to the `.asc` or `.csv` data file.
    /// 
    /// Returns:
    ///     Survey: [`Survey`] containing `datamap` and `data`. If `path_data` is omitted,
    ///         `has_data()` returns `False`.
    #[pyfunction(
        name="read_triple_s", 
        signature=(
            path_datamap: "str", 
            path_data:"str"=None
        )
    )]
    pub fn py_read_triple_s(path_datamap: &str, path_data: Option<&str>) -> PyResult<Survey> {
        read_triple_s(path_datamap, path_data).map_err(|err| err.into())
    }

    /// Reads .asc or .csv datafile with datamap provided.
    /// 
    /// Args:
    ///     datamap (Datamap): datamap used to read the data.
    ///     path_data (str): path to the .csv or .asc data file.
    ///
    /// Returns:
    ///     SurveyData: The loaded respondent data.
    #[pyfunction(
        name="read_data",
        signature=(
            datamap: "Datamap",
            path_data: "str"
        )
    )]
    pub fn py_read_data(datamap: Datamap, path_data: &str) -> PyResult<SurveyData> {
        crate::datamap::read_data(&datamap, path_data)
            .map_err(|err| crate::utils::py_value_error(err.into()))
    }

    #[pymodule_init]
    fn init(bound: &Bound<'_, PyModule>) -> PyResult<()> {
        // init python logger
        pyo3_log::init();

        // Set submodules
        let modules = bound.py().import("sys")?.getattr("modules")?;
        modules.set_item("triple_s.config", bound.getattr("config")?)?; // config

        Ok(())
    }
}
