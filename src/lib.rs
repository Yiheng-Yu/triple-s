mod classic;
mod xml;

pub mod error;
mod survey;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Context, Result};
pub use classic::read_sss_file_classic;
pub use survey::{
    Choice,
    Format,
    Language,
    Location,
    Meta,
    Mode,
    Question,
    Record,
    Response,
    Spread,
    Survey,
    Text,
    TextVariant,
    Version,
};
pub use xml::read_sss_file_xml_file;

#[derive(Debug, PartialEq)]
pub enum FileType {
    Classic,
    Xml,
}

impl FileType {
    pub fn parse(first_line: &str) -> Result<Self, error::BadLine> {
        match first_line.chars().next().unwrap_or_default() {
            'S' => Ok(Self::Classic),
            '<' => Ok(FileType::Xml),
            _ => Err(error::BadLine::new(format!(
                "Unable to identify sss version from the first line: '{first_line}'"
            ))),
        }
    }
}

pub fn read_sss_file(sss_path: &str) -> Result<Survey> {
    let input: File = File::open(sss_path).map_err(error::FileReadError::from)?;
    let mut lines = BufReader::new(input).lines();
    let first_line = lines
        .next()
        .ok_or(error::FileReadError::new("file seems to be empty"))?
        .with_context(|| format!("When trying to read from: '{}'", sss_path))?;

    let sss_version = FileType::parse(&first_line)?;
    match sss_version {
        FileType::Classic => read_sss_file_classic(lines),
        FileType::Xml => read_sss_file_xml_file(sss_path),
    }
}
