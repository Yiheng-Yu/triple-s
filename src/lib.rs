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
    pub use super::{
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
        Question,
        Code,
        Value,
        Response,
        Category,
        Quantity,
    };

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}
