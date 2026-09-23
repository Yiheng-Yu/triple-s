///! Survey configuration
use triple_s as sss;
use pyo3::prelude::*;

#[pyclass(name="Format", module="config")]
pub struct PyFormat(pub sss::CodeFormat);

impl From<sss::CodeFormat> for PyFormat {
    fn from(value: sss::CodeFormat) -> Self {
        Self(value)
    }
}

impl From<PyFormat> for sss::CodeFormat {
    fn from(value: PyFormat) -> Self {
        value.0
    }
}

// TODO! Add documentation
#[pyclass(dict, module = "triple_s", name = "Mode")]
pub struct PyMode(pub sss::Mode);

impl From<sss::Mode> for PyMode {
    fn from(value: sss::Mode) -> Self {
        Self(value)
    }
}

impl From<PyMode> for sss::Mode {
    fn from(value: PyMode) -> Self {
        value.0
    }
}

#[pyclass(dict, module = "triple_s", name = "Language")]
pub struct PyLanguage(pub sss::Language);

impl From<sss::Language> for PyLanguage {
    fn from(value: sss::Language) -> Self {
        Self(value)
    }
}

impl From<PyLanguage> for sss::Language {
    fn from(value: PyLanguage) -> Self {
        value.0
    }
}