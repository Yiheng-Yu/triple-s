// mod read;
mod entities;
pub mod error;
pub mod prelude;
mod utils;

pub use entities::{
    CodeFormat,
    DataSource,
    Meta,
    Mode,
    Survey,
    Text,
    Usage,
    Variant,
    Version,
};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[cfg(feature = "pyo3")]
#[pymodule]
mod triple_s {
    use pyo3::prelude::*;

    #[pymodule_export]
    pub use super::{
        CodeFormat,
        DataSource,
        Meta,
        Mode,
        Survey,
        Text,
        Usage,
        Variant,
        Version,
    };

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}
