mod classic;
pub mod error;
mod survey;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Context, Result};
pub use classic::{SSSClassicLineType, SSSType, read_sss_classic};
pub use survey::{Choice, Location, Meta, Question, Record, Response, Spread, Survey};

pub fn read_sss(sss_path: &str) -> Result<Survey> {
    let input: File = File::open(sss_path).map_err(error::FileReadError::from)?;
    let mut lines = BufReader::new(input).lines();
    let first_line = lines
        .next()
        .ok_or(error::FileReadError::new("file seems to be empty"))?
        .with_context(|| format!("When trying to read from: '{}'", sss_path))?;

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
