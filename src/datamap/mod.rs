pub mod question;
pub mod read;
pub mod record;
pub mod response;
pub mod survey_info;
pub mod text;
pub mod datamap;

use std::path::Path;

pub use datamap::Datamap;
pub use question::{Location, Question, QuestionType, Spread, Usage};
pub use response::{Code, Response};
pub use text::{Text, Variant};

use anyhow::Result;
use crate::data::SurveyData;


pub fn read_data(datamap: &Datamap, path_data: impl AsRef<Path>) -> Result<SurveyData> {
    let path_data = path_data.as_ref();
    todo!()
}