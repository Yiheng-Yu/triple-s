mod meta;
mod flags;
mod record;
mod question;
mod response;
mod texts;

pub use flags::{DataSource, Mode, Version, CodeFormat, Usage};
pub use meta::Meta;
pub use record::Record;
pub use question::Question;
pub use texts::{Text, Variant};

use crate::utils::ArcLock;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg_attr(feature="pyo3", pyclass(module = "triple_s", name = "Survey"))]
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