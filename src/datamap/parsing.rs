//! Used internal only, used for parsing datamap data

use anyhow::Result;
use chrono::{NaiveDate, NaiveTime, Timelike};
use parking_lot::RwLockReadGuard;
use polars::prelude::*;

use crate::{
    CodeFormat,
    Question,
    datamap::{QuestionType, Spread},
};

/// For parsing fixed-width data
#[derive(Debug)]
pub(crate) struct FixedWidthParser {
    pub question_name: String,
    pub start: usize,
    pub end: usize,
    pub question_type: QuestionType,
    pub is_literal: bool,
    pub spread: Option<Spread>,
}

impl FixedWidthParser {
    pub fn from_question(question: RwLockReadGuard<'_, Question>) -> Result<Self> {
        let range = question.get_data_range();
        let question_type = question.question_type().clone();
        let spread = question.spread().clone();
        let is_literal = question
            .format()
            .as_ref()
            .is_some_and(|val| val.is_literal());

        Ok(Self {
            question_name: question.name(),
            start: range.start,
            end: range.end,
            question_type: question_type,
            is_literal: is_literal,
            spread: spread,
        })
    }

    /// Get character from `Self::start` to `Self::end`
    fn slice_string<'a>(&self, string: &'a str) -> Result<&'a str> {
        let start = string
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.start)
            .unwrap_or(string.len());

        let end = string
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.end)
            .unwrap_or(string.len());

        string
            .get(start..end)
            .ok_or_else(|| anyhow::anyhow!("Invalid character range"))
    }

    pub fn parse_row<'a>(&'a self, row: &'a str) -> PolarsResult<AnyValue<'a>> {
        let content = self
            .slice_string(row)
            .map_err(|err| PolarsError::ComputeError(format!("{:#?}", err).into()))
            .with_context(|| {
                format!(
                    "When trying to parse fixed width data for question '{}'",
                    self.question_name
                )
            })?;

        parse_cell_value(content, &self.question_type, self.is_literal).with_context(|| {
            format!(
                "When trying to parse fixed width data for question '{}'",
                self.question_name
            )
        })
    }
}

fn parse_cell_value<'a>(
    raw: &'a str,
    question_type: &'a QuestionType,
    is_literal: bool,
) -> PolarsResult<AnyValue<'a>> {
    let data = raw.trim();
    match question_type {
        &QuestionType::None => Err(PolarsError::ComputeError(
            "Unable to parse data with question type `None`".into(),
        )),
        &QuestionType::Character => Ok(AnyValue::String(data)),
        &QuestionType::Quantity => {
            let value: f64 = data.parse().map_err(|err| {
                PolarsError::InvalidOperation(
                    format!(
                        "Error raised when trying to convert '{}' to float: {:#?}",
                        data, err
                    )
                    .into(),
                )
            })?;
            Ok(AnyValue::Float64(value))
        }
        &QuestionType::Logical => {
            let value = data.parse::<usize>().map_err(|err| {
                PolarsError::InvalidOperation(
                    format!(
                        "Error raised when trying to convert '{}' to boolean: {:#?}",
                        data, err
                    )
                    .into(),
                )
            })?;
            match value {
                0 => Ok(AnyValue::Boolean(false)),
                1 => Ok(AnyValue::Boolean(true)),
                _ => return Err(PolarsError::InvalidOperation(format!(
                    "Error raised when trying to convert '{}' to boolean. Expect '0' or '1'", 
                    data,
                ).into()))
            }
        }
        &QuestionType::Date => {
            let value = NaiveDate::parse_from_str(data, "%Y%m%d")
                .map_err(|err| {
                    PolarsError::InvalidOperation(
                        format!(
                            "Error raised when trying to convert '{}' to date: {:#?}",
                            data, err
                        )
                        .into(),
                    )
                })?
                .signed_duration_since(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
                .num_days() as i32;
            Ok(AnyValue::Date(value))
        }
        &QuestionType::Time => {
            let value = NaiveTime::parse_from_str(data, "%H%M%S").map_err(|err| {
                PolarsError::InvalidOperation(
                    format!(
                        "Error raised when trying to convert '{}' to time: {:#?}",
                        data, err
                    )
                    .into(),
                )
            })?;

            let time = value.num_seconds_from_midnight() as i64 * 1_000_000_000
                + value.nanosecond() as i64;
            Ok(AnyValue::Time(time))
        }
        &QuestionType::Single => {
            if is_literal {
                Ok(AnyValue::String(data))
            } else {
                let value: u32 = data.parse().map_err(|err| {
                    PolarsError::InvalidOperation(format!(
                    "Error raised when trying to convert '{}' to unsigned integer: {:#?}", 
                    data,
                    err
                ).into())
                })?;
                Ok(AnyValue::UInt32(value))
            }
        }
        &QuestionType::Multiple => {
            todo!()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slice_string() {
        let slicer = FixedWidthParser {
            question_name: "q1".into(),
            start: 1,
            end: 3,
            question_type: QuestionType::Character,
            spread: None,
        };

        let cases = [
            ("sentence 1", "en"),
            ("s🦔🦔🦔entence", "🦔🦔"),
            (" 中文含有", "中文"),
            ("中é😀文", "é😀"),
            ("，，，", "，，"),
        ];

        for (string, expect) in cases.into_iter() {
            let read = slicer.slice_string(string).unwrap();
            assert_eq!(read, expect);
        }
    }
}
