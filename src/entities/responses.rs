use std::range::Range;

#[cfg(feature="pyo3")]
use pyo3::prelude::*;

use crate::utils::{
    ArcLock,
};

pub enum Response {
    OpenEnd,
    Categorical,
}

pub struct Fields {
    // move CodeFormat here???
}
