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
    Categorical,
    Position,
    Quantity,
    Question,
    RangeDate,
    RangeTime,
    Record,
    Response,
    Spread,
    Survey,
    Text,
    TextVariant,
    Version,
};
pub use parser::{read_from_string, read_sss_file};


pub mod prelude {
    use std::fmt::Display;
    use anyhow::Result;

    pub trait CodeType:Clone + Display + Sized + PartialEq {}

    impl CodeType for u64 {}
    impl CodeType for f64 {}
    impl CodeType for String {}

    /// wrap current instance in Ok(), just to save some boilderplate code,,,
    pub trait Wrap: Sized {
        fn wrap(self) -> Result<Self> {
            Ok(self)
        }
    }

}