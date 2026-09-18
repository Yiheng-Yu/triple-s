#![cfg(feature="classic")]

use anyhow::Result;
use std::sync::Arc;
use std::str::FromStr;

use crate::entities::{
    Survey,
    Text,
};

pub struct Line(Box<str>);

impl Line {
    pub fn split_prefix(&self) -> (&str, &str) {
        todo!()
    }
}


pub fn read_sss_file_classic<I, E>(lines: I) -> Result<Survey>
where
    I: IntoIterator<Item = Result<String, E>>,
    E: std::error::Error,
{
    todo!()
}

impl TryFrom<Line> for Text {
    type Error = anyhow::Error;

    fn try_from(value: Line) -> Result<Self, Self::Error> {
        todo!()
    }
}