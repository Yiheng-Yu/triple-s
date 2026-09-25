
#[cfg(feature="pyo3")]
use pyo3::prelude::*;


pub enum Response {
    OpenEnd,
    Categorical,
}

pub struct Fields {
    // move CodeFormat here???
}
