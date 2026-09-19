// mod xml;
mod entities;
pub mod error;
mod parser;

pub use entities::{
    Character,
    Code,
    Language,
    Mode,
    Multiple,
    RangeDate,
    RangeTime,
    Response,
    Single,
    Survey,
    Text,
    TextVariant,
    Version,
};
pub use parser::read_sss_file;

// pub use survey::{
//     Value,
//     Format,
//     Language,
//     Location,
//     Meta,
//     Mode,
//     Question,
//     Record,
//     Response,
//     Spread,
//     Survey,
//     Text,
//     TextVariant,
//     Version,
// };
// pub use xml::read_sss_file_xml_file;
