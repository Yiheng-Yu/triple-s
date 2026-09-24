use pyo3::prelude::*;

use std::sync::Arc;
use parking_lot::{RwLock};

use super::{
    ArcLock,
    Question,
    DataSource,
};

#[pyclass(module = "triple_s", name = "Record", skip_from_py_object)]
#[derive(Debug, Default, Clone)]
pub struct Record {
    pub id: char,

    // pub format: Option<CodeFormat>,
    /// (Optional) 'skip' attribute can be used to ignore one or more initial
    /// records in the Data File. This will be most useful for csv Data
    /// Files where the first line is often used as documentation (e.g.
    /// names for the columns/fields) for the succeeding values.
    pub skip: Option<String>,

    /// (Optional) Used to specify an explicit location for the Data
    /// File that is described by this Triple-S XML specification. Note that
    /// using an href attribute ties the specification to the Data File and
    /// may cause problems if the Metadata File and Data File are moved.
    pub href: Option<String>,

    pub data_source: DataSource,
    /* NOTE: THIS KEPT AS ARC RWLOCK IS VERY IMPORTANT FOR PYTHON BINDING TO WORK!!! */
    pub content: ArcLock<Vec<ArcLock<Question>>>
}