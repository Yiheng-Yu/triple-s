mod entities;
pub mod error;
pub mod prelude;
mod utils;

pub use entities::{
    Code,
    CodeFormat,
    DataSource,
    Location,
    Mode,
    Question,
    QuestionType,
    Response,
    Spread,
    Survey,
    SurveyInfo,
    Text,
    Usage,
    Value,
    Variant,
    Version,
};
use error::EventLog;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

mod read;
pub use read::{read_datamap, read_from_reader, read_from_string};

/// A Python module implemented in Rust.
#[cfg(feature = "pyo3")]
#[pymodule]
mod triple_s {
    use anyhow::Context;
    use pyo3::exceptions;

    #[pymodule_export]
    use super::flags;
    use super::*;
    #[pymodule_export]
    pub use super::{
        Code,
        Location,
        Question,
        Response,
        Spread,
        Survey,
        SurveyInfo,
        Text,
        Usage,
        Variant,
    };
    use crate::read::read_from_reader;

    #[pyfunction(name="read_datamap", signature=(datamap_path: "str"))]
    pub fn py_read_datamap(py: Python<'_>, datamap_path: &str) -> PyResult<Survey> {
        let input = std::fs::File::open(datamap_path)
            .map_err(|err| exceptions::PyIOError::new_err(err.to_string()))?;
        let (survey, read_log) = read_from_reader(std::io::BufReader::new(input))
            .with_context(|| format!("When reading '{}'", datamap_path))
            .map_err(|err| exceptions::PyIOError::new_err(err.to_string()))?;
        if read_log.warnings.len() > 0 {
            read_log.print_warning_summary(py, &format!("Reading '{datamap_path}'"))?
        };
        Ok(survey)
    }

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let flags = m.getattr("flags")?;
        let modules = PyModule::import(m.py(), "sys")?.getattr("modules")?;
        modules.set_item("triple_s.flags", &flags)?;
        Ok(())
    }
}

#[cfg(feature = "pyo3")]
#[pymodule]
mod flags {

    #[pymodule_export]
    pub use crate::{CodeFormat, DataSource, Mode, QuestionType, Usage, Value, Version};
}
