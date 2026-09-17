mod classic;
mod xml;

pub mod error;
mod survey;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Context, Result};
use classic::SSSType;
pub use classic::read_sss_file_classic;
pub use survey::{Choice, Location, Meta, Question, Record, Response, Spread, Survey};
pub use xml::read_sss_file_xml_file;

pub fn read_sss_file(sss_path: &str) -> Result<Survey> {
    let input: File = File::open(sss_path).map_err(error::FileReadError::from)?;
    let mut lines = BufReader::new(input).lines();
    let first_line = lines
        .next()
        .ok_or(error::FileReadError::new("file seems to be empty"))?
        .with_context(|| format!("When trying to read from: '{}'", sss_path))?;

    let sss_version = SSSType::parse(&first_line)?;
    match sss_version {
        SSSType::Classic => read_sss_file_classic(lines),
        SSSType::XML => read_sss_file_xml_file(sss_path),
    }
}
