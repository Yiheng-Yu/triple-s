extern crate triple_s as tris;

use pyo3::prelude::*;

// need to learn how to properly do this

#[pyclass(dict, module="triple_s", name="Survey")]
pub struct PySurvey {
    inner: tris::Survey
}