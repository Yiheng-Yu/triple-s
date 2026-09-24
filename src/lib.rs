mod read;
pub mod error;
pub mod prelude;
mod entities;

mod utils;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
mod triple_s {
    use pyo3::prelude::*;

    #[pymodule_export]
    pub use super::entities::{
        Survey,
        Meta,
        DataSource,
        Version,
        Mode,
    };

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}
