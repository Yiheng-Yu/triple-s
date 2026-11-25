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
//! # Crate features
//!
//! The current crate comes with three feature flags:
//!
//! - `default`:
//!     - Activated on default, this feature allows you to read triple-s survey as well as it's data.
//!
//! - `datamap-only`:
//!     - Reads only triple-s datamap, turning this on will disable data-reading behaviour, which should significantly reduce the compile time (it skips a LOT of polars dependencies).
//!     - Activated by: setting `default-features` to be `false` and adding `datamap-only` to the list of features:
//!     
//!     ```text
//!     triple_s = { .., default-features = false, feature = ["datamap-only"] }
//!     ```
//!
//! - `pyo3`:
//!     - Used for binding with python
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
//! use triple_s::{read_data, read_datamap};
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

#[cfg(not(feature = "datamap-only"))]
mod data;
pub mod datamap;
pub mod error;
pub mod options;
mod survey;

pub mod prelude;
mod utils;

use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Read},
    path::Path,
};

use anyhow::{Context, Result};
#[cfg(not(feature = "datamap-only"))]
pub use data::{QuestionData, SurveyData};
#[cfg(not(feature = "datamap-only"))]
pub use datamap::read_data;
pub use datamap::{
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
    record::{self, DataFormat, Record},
    response::{self, Code, DataRange, Response},
    survey_datamap::Datamap,
    survey_info::{self, SurveyInfo, Version},
    text::{self, Mode, Text, Variant},
};
use error::EventLog;
use log::warn;
#[cfg(not(feature = "datamap-only"))]
pub use options::{AnalysisOptions, LabelOptions};
pub use prelude::DeepCopy;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use quick_xml::reader::Reader as XmlReader;
pub use survey::Survey;

use crate::datamap::read::FileType;

// TODO: SET UP DOCUMENTATION ON CREATING A NEW DATAMAP FROM PYTHON/ RUST SIDE!

// -------------------------------------------------------------------------------------------------

/// Read triple_s file.
///
/// Arguments:
/// `path_sss`:
///     path to the triple-s datamap file.
///
/// Returns:
/// [`Survey`] containing the datamap.
#[cfg(feature = "datamap-only")]
pub fn read_triple_s(path_sss: impl AsRef<Path>) -> Result<Survey> {
    log::info!("Reading datamap..");
    let datamap = read_datamap(path_sss)?;
    Survey::new(datamap)
}

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
#[cfg(not(feature = "datamap-only"))]
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
            datamap
                .record_mut()
                ._set_href(Some(path_data.to_string_lossy().into_owned()));
            *datamap.record_mut().data_format_mut() = Some(DataFormat::from_path(path_data)?);
            data
        }
    };

    Survey::new(datamap, data).map_err(|err| err.context_trace().into())
}

/// Read triple-s datamap.
///
/// This functions auto detects triple-s version (classic/ xml), and currently
/// supports up till triple-s version 2.0 with non-hierarchial data. Triple-s
/// 3.0 stuff _should_be_ mostly okay, but fully tested (yet).
/// Invalid UTF-8 sequences in XML input are replaced with U+FFFD (`�`).
///
/// ## Example usage:
///
/// ```no_run
/// use triple_s::read_datamap;
///
/// let survey = read_datamap("../path/to/survey.sss").unwrap();
/// println!("{:?}", survey.title());
///
/// Ok::<(), Box<dyn std::error::Error>>(());
/// ```
pub fn read_datamap(sss_path: impl AsRef<Path>) -> Result<Datamap> {
    let path = sss_path.as_ref();
    let input = File::open(path)
        .map_err(error::FileReadError::from)
        .with_context(|| format!("When opening '{}'", path.display()))?;

    let source = BufReader::new(input);
    let decoder = crate::utils::create_reader(source);
    let reader = BufReader::new(decoder);
    let (survey, read_log) = read_from_reader(reader)
        .with_context(|| format!("When reading '{}'", path.display()))?;

    let context = format!("Reading datamap: '{:?}'", path.as_os_str());
    if let Some(summary) = read_log.warning_summary(context) {
        warn!("{}", summary);
    }

    Ok(survey)
}

/// create datamap instance from utf-8 string, auto detects file types (sss
/// classic/ xml)
pub fn read_from_string(data: impl AsRef<str>) -> Result<Datamap> {
    let (survey, read_log) = read_from_reader(data.as_ref().as_bytes())?;
    if let Some(summary) = read_log.warning_summary("Reading from string") {
        warn!("{}", summary);
    }

    Ok(survey)
}

/// create datamap instance from a reader/ stream, auto detects file types (sss
/// classic/ xml)
/// Invalid UTF-8 sequences in XML input are replaced with U+FFFD (`�`).
pub fn read_from_reader<R: BufRead>(mut input: R) -> Result<(Datamap, EventLog)> {
    let mut prefix = Vec::new();
    let mut first_line = true;
    loop {
        if input.read_until(b'\n', &mut prefix)? == 0 {
            return Err(error::FileReadError::new("file seems to be empty").into());
        }
        if first_line && prefix.starts_with(b"\xef\xbb\xbf") {
            prefix.drain(..3);
        }
        first_line = false;
        if let Some(start) = prefix.iter().position(|byte| !byte.is_ascii_whitespace()) {
            prefix.drain(..start);
            break;
        }
        prefix.clear();
    }
    let file_type = FileType::parse(std::str::from_utf8(&prefix[..1])?)?;
    let builder = match file_type {
        FileType::Classic => {
            let input = BufReader::new(Cursor::new(prefix).chain(input));
            crate::datamap::read::classic::create_survey_builder(input.lines()).with_context(
                || "trying to create survey builder from the data file".to_string(),
            )?
        }
        FileType::Xml => {
            let read_result = {
                input.read_to_end(&mut prefix)?;
                let text = std::str::from_utf8(&prefix)?;
                let mut reader = XmlReader::from_str(text);

                reader.config_mut().trim_text(true);
                crate::datamap::read::xml::create_survey_builder(reader)
            };

            read_result.with_context(|| {
                "trying to create survey builder from the data file".to_string()
            })?
        }
    };
    builder.build()
}

#[cfg(feature = "pyo3")]
#[pymodule]
mod triple_s {
    use anyhow::Context;
    use pyo3::prelude::*;
    #[pymodule_export]
    pub use pyo3_polars::error::NoDataError;

    use super::{read_from_reader, read_triple_s};
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

    #[pymodule]
    pub mod config {
        #[pymodule_export]
        pub use crate::options::{DataFrameFormat, ResponseDisplay};
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

        let read_result = py.detach(|| {
            read_from_reader(std::io::BufReader::new(input))
                .with_context(|| format!("When reading '{}'", path_datamap))
        });
        let (survey, read_log) = read_result?;
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
    /// Keyword Arguments:
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
    pub fn py_read_triple_s(
        py: Python<'_>,
        path_datamap: &str,
        path_data: Option<&str>,
    ) -> PyResult<Survey> {
        py.detach(|| read_triple_s(path_datamap, path_data).map_err(|err| err.into()))
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
    pub fn py_read_data(
        py: Python<'_>,
        datamap: Datamap,
        path_data: &str,
    ) -> PyResult<SurveyData> {
        py.detach(|| {
            crate::datamap::read_data(&datamap, path_data)
                .map_err(|err| crate::utils::py_value_error(err.into()))
        })
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
