mod classic;
mod error;
mod survey;

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub use classic::{SSSClassicLineType, SSSType, read_sss_classic};
pub use error::IoError;
pub use survey::{Choice, Location, Meta, Question, Record, Response, Spread, Survey};

pub fn read_sss(sss_path: &str) -> Result<Survey, IoError> {
    let input: File = File::open(sss_path).map_err(IoError::from)?;
    let mut lines = BufReader::new(input).lines();
    let first_line = lines
        .next()
        .ok_or_else(|| IoError::internal("file seems to be empty."))??;
    match SSSType::parse(&first_line).map_err(IoError::from)? {
        SSSType::Classic => read_sss_classic(lines),
        SSSType::XML => Err(IoError::internal("XML parsing is not implemented yet")),
    }
}
