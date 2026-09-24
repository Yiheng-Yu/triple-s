// mod read;
pub mod error;
pub mod prelude;
mod entities;
mod utils;

pub use entities::{
    Survey,
    Meta,
    DataSource,
    Version,
    Mode,
    CodeFormat,
    Usage,
    Text, 
    Variant,
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
        Survey,
        Meta,
        DataSource,
        Version,
        Mode,
        CodeFormat,
        Usage,
        Text, 
        Variant,
    };

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}
