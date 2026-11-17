//! Immutable parsing plans shared by fixed-width and CSV batch workers.
use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveTime, Timelike};
use either::Either;
use parking_lot::RwLockReadGuard;
use polars::{frame::row::Row, prelude::*};
use rayon::prelude::*;

use crate::{Question, datamap::QuestionType};

#[derive(Debug)]
pub(crate) struct FixedWidthParser {
    pub question_name: String,
    pub start: usize,
    pub end: usize,
    question_type: QuestionType,
    is_literal: bool,
    // Retain existing multiple-choice parsing and validation semantics.
    multiple: Option<Question>,
}

impl FixedWidthParser {
    pub fn from_question(question: RwLockReadGuard<'_, Question>) -> Result<Self> {
        let range = question.get_data_range();
        anyhow::ensure!(range.start <= range.end, "Invalid question range");
        let question_type = question.question_type().clone();
        Ok(Self {
            question_name: question.name(),
            start: range.start,
            end: range.end,
            is_literal: question.format().as_ref().is_some_and(|f| f.is_literal()),
            multiple: (question_type == QuestionType::Multiple).then(|| question.clone()),
            question_type,
        })
    }

    #[inline]
    fn parse(&self, field: &str) -> Result<AnyValue<'static>> {
        if let Some(question) = &self.multiple {
            question.parse_multiple_data(field)
            // question.parse_multiple_data(field)
            // question
            //     .parse_question_value(field)
            //     .map(AnyValue::into_static)
        } else {
            parse_cell_value(field, &self.question_type, self.is_literal).map_err(Into::into)
        }
        .with_context(|| format!("When parsing question '{}'", self.question_name))
    }
}

pub(crate) fn parse_fixed(parsers: &[FixedWidthParser], row: &str) -> Result<Row<'static>> {
    let ascii = row.is_ascii();
    let offsets: Vec<_> = if ascii {
        Vec::new()
    } else {
        row.char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(row.len()))
            .collect()
    };
    // A fallible collect cannot use the iterator's exact length to reserve space.
    // Reserve once rather than repeatedly reallocating wide respondent rows.
    let mut values = Vec::with_capacity(parsers.len());
    for parser in parsers {
        let field = if ascii {
            row.get(parser.start..parser.end)
        } else {
            offsets
                .get(parser.start)
                .zip(offsets.get(parser.end))
                .and_then(|(&start, &end)| row.get(start..end))
        }
        .with_context(|| {
            format!("Invalid data range for question '{}'", parser.question_name)
        })?;
        values.push(parser.parse(field)?);
    }
    Ok(Row(values))
}

pub(crate) fn parse_csv(
    parsers: &[FixedWidthParser],
    row: &csv::StringRecord,
) -> Result<Row<'static>> {
    let mut values = Vec::with_capacity(parsers.len());
    for parser in parsers {
        let field = row.get(parser.start).with_context(|| {
            format!(
                "Missing CSV field for question '{}': requires field {} ({} fields present)",
                parser.question_name,
                parser.start + 1,
                row.len(),
            )
        })?;
        values.push(parser.parse(field)?);
    }
    Ok(Row(values))
}

/// Bounded, ordered batches. Small inputs avoid Rayon scheduling overhead.
pub(crate) fn parallel_rows<I, F>(
    mut records: I,
    parse: F,
) -> impl Iterator<Item = Result<Row<'static>>>
where
    I: Iterator,
    I::Item: Send,
    F: Fn(I::Item) -> Result<Row<'static>> + Sync,
{
    if rayon::current_num_threads() == 1 {
        let mut finished = false;
        return Either::Left(std::iter::from_fn(move || {
            if finished {
                return None;
            }
            let row = parse(records.next()?);
            finished = row.is_err();
            Some(row)
        }));
    }

    let mut finished = false;
    Either::Right(
        std::iter::from_fn(move || {
            if finished {
                return None;
            }
            let mut batch = Vec::with_capacity(1024);
            batch.extend(records.by_ref().take(1024));
            if batch.is_empty() {
                return None;
            }

            let mut rows: Vec<_> = batch.into_par_iter().map(&parse).collect();
            if let Some(index) = rows.iter().position(Result::is_err) {
                rows.truncate(index + 1);
                finished = true;
            }
            Some(rows)
        })
        .flatten(),
    )
}

#[inline]
fn parse_cell_value(
    raw: &str,
    question_type: &QuestionType,
    is_literal: bool,
) -> PolarsResult<AnyValue<'static>> {
    let data = raw.trim();
    if data.is_empty()
        && !matches!(question_type, QuestionType::Character | QuestionType::None)
    {
        return Ok(AnyValue::Null);
    }
    match question_type {
        &QuestionType::None => Err(PolarsError::ComputeError(
            "Unable to parse data with question type `None`".into(),
        )),
        &QuestionType::Character => Ok(AnyValue::StringOwned(data.into())),
        &QuestionType::Quantity => {
            let value: f64 = data.parse().map_err(|err| {
                PolarsError::InvalidOperation(
                    format!(
                        "Error raised when trying to convert '{}' to float: {}",
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
                        "Error raised when trying to convert '{}' to boolean: {}",
                        data, err
                    )
                    .into(),
                )
            })?;
            match value {
                0 => Ok(AnyValue::Boolean(false)),
                1 => Ok(AnyValue::Boolean(true)),
                _ => {
                    return Err(PolarsError::InvalidOperation(
                        format!(
                            "Error raised when trying to convert '{}' to boolean. Expect '0' \
                             or '1'",
                            data,
                        )
                        .into(),
                    ));
                }
            }
        }
        &QuestionType::Date => {
            let value = NaiveDate::parse_from_str(data, "%Y%m%d")
                .map_err(|err| {
                    PolarsError::InvalidOperation(
                        format!(
                            "Error raised when trying to convert '{}' to date: {}",
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
                        "Error raised when trying to convert '{}' to time: {}",
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
                Ok(AnyValue::StringOwned(data.into()))
            } else {
                let value: u32 = data.parse().map_err(|err| {
                    PolarsError::InvalidOperation(
                        format!(
                            "Error raised when trying to convert '{}' to unsigned integer: {}",
                            data, err
                        )
                        .into(),
                    )
                })?;
                Ok(AnyValue::UInt32(value))
            }
        }
        &QuestionType::Multiple => {
            unreachable!("Multiple-choice fields use the snapshotted question parser")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{QuestionOptions, utils::lock_it_up};

    fn plan() -> Vec<FixedWidthParser> {
        [("last", 4, 5), ("first", 1, 2), ("overlap", 2, 4)]
            .into_iter()
            .map(|(name, start, end)| {
                let q = lock_it_up(
                    QuestionOptions::new(name)
                        .with_question_type(QuestionType::Character)
                        .with_location(start, Some(end))
                        .finish()
                        .unwrap(),
                );
                FixedWidthParser::from_question(q.read()).unwrap()
            })
            .collect()
    }

    #[test]
    fn indexed_ranges_support_unicode_gaps_overlap_and_order() {
        let p = plan();
        let row = parse_fixed(&p, "A中😀DE").unwrap();
        assert_eq!(
            row.0,
            vec![
                AnyValue::StringOwned("DE".into()),
                AnyValue::StringOwned("A中".into()),
                AnyValue::StringOwned("中😀D".into())
            ]
        );
        assert!(parse_fixed(&p, "abc").is_err());
    }

    #[test]
    fn owned_scalar_values_match_question_parsing() {
        use crate::CodeFormat;

        for (kind, format, field) in [
            (QuestionType::Quantity, None, " 123.456 "),
            (QuestionType::Logical, None, "1"),
            (QuestionType::Logical, None, "0"),
            (QuestionType::Single, None, "42"),
            (QuestionType::Single, Some(CodeFormat::Literal), " A "),
            (QuestionType::Character, None, " 中😀 text "),
            (QuestionType::Date, None, "20261115"),
            (QuestionType::Time, None, "123456"),
        ] {
            let question = lock_it_up(
                QuestionOptions::new("Q")
                    .with_question_type(kind)
                    .with_format(format)
                    .finish()
                    .unwrap(),
            );
            let parser = FixedWidthParser::from_question(question.read()).unwrap();
            let question = question.read();
            for field in [field, "", "   "] {
                assert_eq!(
                    parser.parse(field).unwrap(),
                    question.parse_question_value(field).unwrap().into_static(),
                );
            }
        }
        assert!(parse_cell_value("", &QuestionType::None, false).is_err());
    }

    #[test]
    fn batches_preserve_order_and_stop_at_first_error() {
        for threads in [1, 4] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            let rows = pool.install(|| {
                parallel_rows(0..2200, |i| {
                    anyhow::ensure!(i != 1050 && i != 1090, "bad {i}");
                    Ok(Row(vec![AnyValue::UInt32(i)]))
                })
                .collect::<Vec<_>>()
            });
            assert_eq!(rows.len(), 1051);
            for (i, row) in rows[..1050].iter().enumerate() {
                assert_eq!(row.as_ref().unwrap().0[0], AnyValue::UInt32(i as u32));
            }
            assert!(
                rows.last()
                    .unwrap()
                    .as_ref()
                    .unwrap_err()
                    .to_string()
                    .contains("1050")
            );
        }
    }

    /// Run with: cargo test --release benchmark_parsing -- --ignored --nocapture
    #[test]
    #[ignore]
    fn benchmark_parsing() {
        use std::{hint::black_box, time::Instant};
        let fixed = "123.456 ".repeat(100);
        let csv = csv::StringRecord::from(vec!["123.456"; 100]);
        let count = 20000;
        for format in ["fixed", "csv"] {
            let questions: Vec<_> = (0..100)
                .map(|i| {
                    QuestionOptions::new(&format!("Q{i}"))
                        .with_question_type(QuestionType::Quantity)
                        .with_location(
                            if format == "fixed" { i * 8 + 1 } else { i + 1 },
                            Some(if format == "fixed" { i * 8 + 8 } else { i + 1 }),
                        )
                        .finish()
                        .unwrap()
                })
                .collect();
            let plans: Vec<_> = questions
                .iter()
                .map(|q| {
                    let q = lock_it_up(q.clone());
                    FixedWidthParser::from_question(q.read()).unwrap()
                })
                .collect();
            let start = Instant::now();
            for _ in 0..count {
                let values = questions
                    .iter()
                    .map(|q| {
                        let field = if format == "fixed" {
                            q.locate_data_asc(&fixed).unwrap()
                        } else {
                            q.locate_data_csv(&csv).unwrap()
                        };
                        q.parse_question_value(field).unwrap().into_static()
                    })
                    .collect::<Vec<_>>();
                black_box(values);
            }
            eprintln!("{format} legacy sequential: {:?}", start.elapsed());
            for threads in [1, 4] {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .unwrap();
                let start = Instant::now();
                pool.install(|| {
                    for row in parallel_rows(0..count, |_| {
                        if format == "fixed" {
                            parse_fixed(&plans, &fixed)
                        } else {
                            parse_csv(&plans, &csv)
                        }
                    }) {
                        black_box(row.unwrap());
                    }
                });
                eprintln!("{format} planned {threads} workers: {:?}", start.elapsed());
            }
        }
    }
}
