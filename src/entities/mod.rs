mod flags;
mod question;
mod record;
mod responses;
mod survey_info;
mod texts;

pub use flags::{CodeFormat, DataSource, Mode, QType, Usage, Version};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
pub use question::{Location, Question};
pub use record::Record;
pub use responses::{Category, Code, Quantity, Response, Value};
pub use survey_info::SurveyInfo;
pub use texts::{Text, Variant};

use crate::utils::ArcLock;

#[cfg_attr(feature = "pyo3", pyclass(module = "triple_s", name = "Survey"))]
pub struct Survey {
    pub meta: SurveyInfo, // holy hell this works
    pub record: Record,
}

impl Survey {
    pub fn empty() -> Self {
        Self {
            meta: SurveyInfo::with_version(Version::V1_2),
            record: Record::default(),
        }
    }
}
