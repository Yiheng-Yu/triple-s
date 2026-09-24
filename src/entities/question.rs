use pyo3::prelude::*;

#[pyclass(module="triple_s", name="question", skip_from_py_object)]
#[derive(Debug)]
pub struct Question {}