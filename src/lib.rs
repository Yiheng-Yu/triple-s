// mod xml;
mod entities;
pub mod error;
mod parser;

pub use entities::{
    Character,
    Code,
    Format,
    Language,
    Meta,
    Mode,
    Multiple,
    Position,
    Quantity,
    Question,
    RangeDate,
    RangeTime,
    Record,
    Response,
    Single,
    Spread,
    Survey,
    Text,
    TextVariant,
    Version,
};
pub use parser::{read_from_string, read_sss_file};
