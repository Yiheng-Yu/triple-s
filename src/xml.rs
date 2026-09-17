use std::{fs::File, io::{BufRead, BufReader}};

use anyhow::Result;
use quick_xml::{
    encoding::DecodingReader, 
    events::Event, 
    reader::Reader
};

use crate::survey::Survey;

pub fn read_sss_file_xml_file(sss_path: &str) -> Result<Survey> {
    let file = File::open(sss_path)?;
    let file = BufReader::new(file);
    let decoder = DecodingReader::new(file);
    let mut reader = Reader::from_reader(decoder);
    reader.config_mut().trim_text(true);

    parse_sss_xml_lines(reader)
}

#[allow(unused_variables, unused_mut)]
fn parse_sss_xml_lines<R>(mut reader: Reader<R>) -> Result<Survey> 
where
    R: BufRead,
{   
    let mut survey = Survey::empty();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ele) => { todo!() }
            Event::End(ele) => { todo!() }
            Event::Text(ele) => { todo!() }
            Event::CData(ele) => { todo!() }
            Event::Comment(ele) => { todo!() }
            Event::Decl(ele) => { todo!() }
            Event::PI(ele) => { todo!() }
            Event::DocType(ele) => { todo!() }
            Event::Empty(ele) => { todo!() }
            Event::GeneralRef(ele) => { todo!() }
            Event::Eof => { todo!() }
        }
    }
}
