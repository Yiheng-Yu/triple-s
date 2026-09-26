mod entities;
pub mod error;
pub mod prelude;
mod utils;

pub use entities::{
    CodeFormat,
    DataSource,
    SurveyInfo,
    Mode,
    Survey,
    Text,
    Usage,
    Variant,
    Version,
    Location,
    QType,
    Question,
    Code,
    Value,
    Response,
    Category,
    Quantity,
};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

mod read;
pub use read::read_datamap;

/// A Python module implemented in Rust.
#[cfg(feature = "pyo3")]
#[pymodule]
mod triple_s {
    use pyo3::prelude::*;

    #[pymodule_export]
    use super::flags;

    #[pymodule_export]
    pub use super::{
        SurveyInfo,
        Survey,
        Text,
        Usage,
        Variant,
        Location,
        Question,
        Code,
        Response,
        Category,
        Quantity,
    };

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
    pub use crate::{
        Mode,
        Version,
        DataSource,
        CodeFormat,
        Value,
        Usage,
        QType,
    };
}