extern crate triple_s as tris;

use pyo3::prelude::*;

#[pyclass]
pub struct Survey {
    inner: tris::Survey
}

#[pyclass(dict, module="triple_s", name="Choice")]
pub struct PyChoice {
    inner: tris::Choice
}