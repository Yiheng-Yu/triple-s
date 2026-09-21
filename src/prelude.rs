use std::fmt::Display;

use anyhow::Result;

use crate::{
    error::NotFiniteError,
};


mod sealed {
    pub trait Sealed {}

    impl Sealed for u64 {}
    impl Sealed for f64 {}
    impl Sealed for String {}
}

pub trait CodeType: sealed::Sealed + Clone + Display + PartialEq {
    /// check if code was valid or not
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl CodeType for u64 {}
impl CodeType for f64 {
    fn validate(&self) -> Result<()> {
        if self.is_finite() {
            Ok(())
        } else {
            Err(NotFiniteError.into())
        }
    }
}
impl CodeType for String {}

/// wrap current instance in Ok()
pub trait Wrap: Sized {
    fn wrap<E>(self) -> Result<Self, E> {
        Ok(self)
    }
}