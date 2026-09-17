use std::{fs::File, io::{BufRead, BufReader}};
use std::str::FromStr;

use anyhow::{Context, Result};
use quick_xml::{
    encoding::DecodingReader, events::{BytesStart, Event, attributes::Attributes}, reader::Reader
};

use crate::{
    survey::{ Survey, Mode, Version },
    error::UnknownAttribute,
};

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
    let mut survey = Survey::empty_xml();
    let mut buf = Vec::new();
    loop {
        let event = reader.read_event_into(&mut buf)?;
        match event {
            Event::Start(ele) => { 
                let name = ele.name();
                let tag = name.local_name().into_inner(); 

                match tag {
                    "sss" => set_sss_attributes(&mut survey, ele.attributes())?,
                    // keep adding new cases to parse here
                    "date" => {
                        let text = extract_text(&mut reader,ele)?;
                        survey.meta.set_date(Some(text))?;
                    }
                    "time" => {
                        let text = extract_text(&mut reader,ele)?;
                        survey.meta.set_time(Some(text))?;
                    }
                    "origin" => {
                        let text = extract_text(&mut reader,ele)?;
                        survey.meta.set_origin(Some(text))?;
                    }
                    "user" => {
                        let text = extract_text(&mut reader,ele)?;
                        survey.meta.set_user(Some(text))?;
                    }
                    _ => {
                        // ()
                        panic!("{:#?}", ele)
                    }
                };

                // println!("{:?} | {:#?}", tag, ele);

                // survey meta

                // // THIS METHOD WILL JUST INCLUDE ALL THE HTML ETC., AND READ THEM AS TEXT
                // // EXTREMELEY USEFULL FOR COLLECTING CONTENT NEEDED
                // if tag_name.as_ref() == "title" {
                //     let end = ele.to_end().into_owned();
                //     let read = reader.read_text_into(end.name(), &mut buf)?;
                //     println!("TITLE parsed: {:?}", read);
                // }
            }
            Event::Text(ele) => {
                let text = ele.to_string();
                // println!("text = {text}");
            }
            
            Event::End(ele) => { 
                // println!("{:?}", ele);
            }
            // Event::CData(ele) => { todo!() }
            // Event::Comment(ele) => { todo!() }
            // Event::Decl(ele) => { todo!() }
            // Event::PI(ele) => { todo!() }
            // Event::DocType(ele) => { todo!() }
            Event::Empty(ele) => { 
                // SPECIALLY IMPORTANT FOR PARSING HTML ESCAPES
                // println!("{:?}", ele);
            }
            // Event::GeneralRef(ele) => { todo!() }
            Event::Eof => { break }
            _ => (),
        }
    }

    Ok(survey)
}

#[inline]
fn set_sss_attributes(survey: &mut Survey, attributes: Attributes<'_> ) -> Result<()> {
    for attr_read in attributes {
        let attr = attr_read?;
        let attr_name = attr.key.into_inner();
        match attr_name {
            "version" => survey.meta.set_version(Some(attr.value.to_string()))?,
            "modes" => {
                let mode_options = parse_mode_options(attr.value.to_string())?;
                survey.meta.set_mode_options(mode_options)?;
            }
            // "options" => survey.meta.options = attr.value,
            _ => return Err(UnknownAttribute::new(attr_name, attr.value))
                .with_context(|| format!("Failed to parse survey meta."))
        }
    }
    Ok(())
}

#[inline]
fn parse_mode_options(value: String) -> Result<Vec<Mode>> {
    value
        .split_whitespace()
        .map(|s| Mode::from_str(s.trim()))
        .collect::<Result<Vec<Mode>, _>>()
        .map_err(Into::into)
}

#[inline]
fn extract_text<R>(reader: &mut Reader<R>, start: BytesStart) -> Result<String>
where
    R: BufRead,
{
    let mut buf = Vec::new();
    let end = start.to_end().into_owned();
    let result = reader.read_text_into(end.name(), &mut buf)?;
    Ok(result.to_string())
}