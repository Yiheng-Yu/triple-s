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
    use crate::entities::Code;

    pub trait CodeType:Clone + Display + Sized + PartialEq + ToString {}

    impl CodeType for u64 {}
    impl CodeType for f64 {}
    impl CodeType for String {}

    pub trait CrateCategorical<Type> 
    where 
        Type: CodeType,
        Self: Sized,
    {
        fn new_single(values: Vec<Code<Type>>) -> Result<Self>;
        fn new_multiple(values: Vec<Code<Type>>) -> Result<Self>;
    }

    /// wrap current instance in Ok(), just to save some boilderplate code,,,
    pub trait Wrap: Sized {
        fn wrap(self) -> Result<Self> {
            Ok(self)
        }
    }

}