mod meta;
mod flags;
mod record;
mod question;
mod response;

pub use flags::{DataSource, Mode, Version};
pub use meta::Meta;
pub use record::Record;
pub use question::Question;

use crate::utils::{self, ArcLock, lock_it_up, normalize_py_index};

use std::sync::Arc;
use parking_lot::{RwLock};
use pyo3::prelude::*;

#[pyclass(module = "triple_s", name = "Survey")]
pub struct Survey {
    pub meta: Meta, // holy hell this works
    pub record: Record,
}

impl Survey {
    pub fn empty() -> Self {
        Self {
            meta: Meta::with_version(Version::V1_2),
            record: Record::default()
        }
    }
}