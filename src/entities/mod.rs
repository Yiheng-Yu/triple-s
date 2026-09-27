mod flags;
mod question;
mod record;
mod responses;
mod survey_info;
mod texts;

pub use flags::{CodeFormat, DataSource, Mode, QuestionType, Usage, Version};

#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*};

pub use question::{Location, Question, Spread};
pub use record::Record;
pub use responses::{Code, Response, Value};
pub use survey_info::SurveyInfo;
pub use texts::{Text, Variant};
use parking_lot::{ArcRwLockReadGuard, RawRwLock};

use crate::utils::ArcLock;
use crate::prelude::Flag;
use anyhow::Result;

#[cfg_attr(feature = "pyo3", pyclass(module = "triple_s", name = "Survey"))]
pub struct Survey {
    pub meta: SurveyInfo, // holy hell this works
    pub record: Record,
}

impl Survey {

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


#[cfg(feature="pyo3")]
#[pymethods]
impl Survey {
    #[pyo3(name="get_question", signature=(question_name:"str"))]
    fn py_get_question(&self, question_name: &str) -> Option<Question> {
        self.record.py_get_question(question_name)
    }

    #[getter(record)]
    fn py_get_record(&self) -> Record {
        self.record.clone()
    }

    #[getter(survey_info)]
    fn py_get_survey_info(&self) -> SurveyInfo {
        self.meta.clone()
    }

    #[pyo3(name="to_classic")]
    fn py_to_classic(&self) -> PyResult<String> {
        self.to_sss_classic()
            .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))
    }
}