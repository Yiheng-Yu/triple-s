mod flags;
mod question;
mod record;
mod responses;
mod survey_info;
mod texts;

pub use flags::{CodeFormat, DataSource, Mode, QuestionType, Usage, Version};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
pub use question::{Location, Question, Spread};
pub use record::Record;
pub use responses::{Code, Response, Value};
pub use survey_info::SurveyInfo;
pub use texts::{Text, Variant};
use parking_lot::{ArcRwLockReadGuard, RawRwLock, RwLockReadGuard};

use crate::utils::ArcLock;
use crate::prelude::Flag;
use anyhow::Result;

#[cfg_attr(feature = "pyo3", pyclass(module = "triple_s", name = "Survey"))]
pub struct Survey {
    pub meta: SurveyInfo, // holy hell this works
    pub record: Record,
}

impl Survey {
    pub fn meta(&self) -> &SurveyInfo {
        &self.meta
    }

    pub fn meta_mut(&mut self) -> &mut SurveyInfo {
        &mut self.meta
    }

    pub fn record(&self) -> &Record {
        &self.record
    }

    pub fn record_mut(&mut self) -> &mut Record {
        &mut self.record
    }

    pub fn get_question(&self, question_name: &str) -> Option<ArcRwLockReadGuard<RawRwLock, Question>> {
        self.record.get_question(question_name)
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        let mut output = format!("SSS\nVERSION {}\n", self.meta.version.to_string());
        for (name, value) in [
            ("DATE", &self.meta.date),
            ("TIME", &self.meta.time),
            ("ORIGIN", &self.meta.origin),
            ("USER", &self.meta.user),
            ("STANDARD", &self.meta.standard),
        ] {
            if let Some(value) = value {
                output.push_str(&format!("{name} \"{value}\"\n"));
            }
        }
        output.push_str("SURVEY\n");
        for (name, value) in [("NAME", &self.meta.name), ("TITLE", &self.meta.title)] {
            if let Some(value) = value {
                output.push_str(&format!("{name} \"{value}\"\n"));
            }
        }
        output.push_str(&self.record.to_sss_classic()?);
        output.push_str("END SURVEY\nEND SSS\n");
        Ok(output)
    }

    pub fn empty() -> Self {
        Self {
            meta: SurveyInfo::with_version(Version::V1_2),
            record: Record::default(),
        }
    }
}
