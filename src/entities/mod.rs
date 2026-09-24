mod flags;
mod meta;
mod question;
mod record;
mod response;
mod texts;

pub use flags::{CodeFormat, DataSource, Mode, Usage, Version};
pub use meta::Meta;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
pub use question::Question;
pub use record::Record;
pub use texts::{Text, Variant};

use crate::utils::ArcLock;

#[cfg_attr(feature = "pyo3", pyclass(module = "triple_s", name = "Survey"))]
pub struct Survey {
    pub meta: Meta, // holy hell this works
    pub record: Record,
}

impl Survey {
    pub fn empty() -> Self {
        Self {
            meta: Meta::with_version(Version::V1_2),
            record: Record::default(),
        }
    }
}
