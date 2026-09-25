mod flags;
mod survey_info;
mod question;
mod record;
mod responses;
mod texts;

pub use flags::{CodeFormat, DataSource, Mode, Usage, Version};
pub use survey_info::SurveyInfo;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

pub use question::{Location, Question};
pub use record::Record;
pub use texts::{Text, Variant};
pub use responses::{
    Code,
    Value,
    Response,
    Category,
    Quantity,
};
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
