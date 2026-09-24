mod meta;
mod flags;
mod record;
mod question;
mod response;
mod texts;

pub use flags::{DataSource, Mode, Version};
pub use meta::Meta;
pub use record::Record;
pub use question::Question;

use crate::utils::ArcLock;
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