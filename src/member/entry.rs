#![allow(missing_docs)]

use std::collections::HashSet;

use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::{
    prelude::*,
};

use crate::DeepCopy;

/// Raw data extract from survey.
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Question", from_py_object),
    doc = include_str!("../doc/py_question.md")
)]
#[derive(Debug, Clone)]
pub enum Entry {
    None(), // Represent an empty entry
    Single(u32),
    Multiple(HashSet<u32>),
    Boolean(bool),
    Quantity(f64),
    Date(NaiveDate),
    Time(NaiveTime),
    String(String),
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Entry::None(), Entry::None()) => true,
            (Entry::Single(self_), Entry::Single(other_)) => self_ == other_,
            (Entry::Multiple(self_), Entry::Multiple(other_)) => self_ == other_,
            (Entry::Boolean(self_), Entry::Boolean(other_)) => self_ == other_,
            (Entry::Date(self_), Entry::Date(other_)) => self_ == other_,
            (Entry::Time(self_), Entry::Time(other_)) => self_ == other_,
            (Entry::String(self_), Entry::String(other_)) => self_ == other_,
            (Entry::Quantity(self_), Entry::Quantity(other_)) => {
                self_ == other_ || (self_.is_nan() && other_.is_nan())
            }
            _ => false,
        }
    }
}

impl Eq for Entry {}

impl DeepCopy for Entry {
    fn deepcopy(&self) -> Self {
        self.clone()
    }
}

impl<'a> From<Entry> for AnyValue<'a> {
    fn from(value: Entry) -> Self {
        match value {
            Entry::String(item) => AnyValue::StringOwned(item.into()),
            Entry::Boolean(item) => AnyValue::Boolean(item),
            Entry::None() => AnyValue::Null,
            Entry::Date(item) => {
                let days = item.num_days_from_ce()
                    - NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().num_days_from_ce();

                AnyValue::Date(days)
            }
            Entry::Time(item) => {
                let nanos = item.num_seconds_from_midnight() as i64 * 1_000_000_000
                    + item.nanosecond() as i64;
                AnyValue::Time(nanos)
            }
            Entry::Quantity(item) => AnyValue::Float64(item),

            Entry::Single(item) => AnyValue::UInt32(item),
            Entry::Multiple(items) => {
                let mut values: Vec<u32> = items.into_iter().collect();
                values.sort_unstable();
                AnyValue::List(Series::new("".into(), values))
            }
        }
    }
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_tmp() {
        let a: HashSet<u32> = HashSet::from([1, 2, 3]);
        let b: HashSet<u32> = HashSet::from([3, 2, 1]);
        assert_eq!(a, b)
    }
}
