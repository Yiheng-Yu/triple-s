mod classic;
mod error;
mod survey;
use anyhow::Result;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub use classic::{SSSClassicLineType, SSSType, read_sss_classic};
pub use error::IoError;
pub use survey::{Choice, Location, Meta, Question, Record, Response, Spread, Survey};

pub fn read_sss(sss_path: &str) -> Result<Survey> {
    let input: File = File::open(sss_path).map_err(IoError::from)?;
    let mut lines = BufReader::new(input).lines();
    let first_line = lines
        .next()
        .ok_or_else(|| IoError::internal("file seems to be empty."))??;
    let sss_version = SSSType::parse(&first_line)?;

    match sss_version {
        SSSType::Classic => {
            let survey = read_sss_classic(lines)?;
            Ok(survey)
        }
        SSSType::XML => {
            let message = error::InternalError::new("XML parsing is not implemented yet");
            Err(message.into())
        }
    }
}
