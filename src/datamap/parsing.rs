//! Immutable parsing plans shared by fixed-width and CSV batch workers.
use std::{collections::HashSet, range::Range};

use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveTime, Timelike};
use either::Either;
use parking_lot::RwLockReadGuard;
use polars::{frame::row::Row, prelude::*};
use rayon::prelude::*;

use crate::{Question, Spread, datamap::QuestionType, error::DataReadError};

#[derive(Debug)]
struct SubfieldArgs {
    pub width: Option<u32>,
    pub subfields: usize,
}

/// Just a VERY thin wrapper around a Question struct
/// that caches some extremely useful information for
/// parsing fixed with dataset.
///
/// Everything of this struct comes from instances of questions,
/// didn't use `Question` instance itself is because I want to reduce
/// the overhead from lots and lots of Arc<> locks.
#[derive(Debug)]
pub(crate) struct ResponseParser {
    pub question_name: String,
    pub start: usize,
    pub end: usize,
    question_type: QuestionType,
    is_literal: bool,
    subfield_args: Option<SubfieldArgs>, // used for read multi-choice data
}

impl ResponseParser {
    fn new(
        name: String,
        range: Range<usize>,
        is_literal: bool,
        question_type: QuestionType,
        spread: Option<&Spread>
    ) -> Result<Self> {
        anyhow::ensure!(range.start <= range.end, "Invalid question range");
        let subfield_args = match &question_type {
            QuestionType::Multiple => spread.map(|spread| SubfieldArgs {
                width: spread.width().clone(),
                subfields: spread.subfield().clone() as usize,
            }),
            _ => None,
        };

        Ok(Self {
            question_name: name,
            start: range.start,
            end: range.end,
            is_literal: is_literal,
            question_type: question_type,
            subfield_args: subfield_args
        })
    }

    pub fn from_question(question: &Question) -> Result<Self> {
        let name = question.name();
        let range = question.get_data_range();
        let question_type = question.question_type().clone();
        let is_literal = question.format().as_ref().is_some_and(|f| f.is_literal());
        let spread = question.spread().as_ref();
        Self::new(name, range, is_literal, question_type, spread)
    }

    pub fn from_question_read_guard(question: RwLockReadGuard<'_, Question>) -> Result<Self> {
        let name = question.name();
        let range = question.get_data_range();
        let question_type = question.question_type().clone();
        let is_literal = question.format().as_ref().is_some_and(|f| f.is_literal());
        let spread = question.spread().as_ref();
        Self::new(name, range, is_literal, question_type, spread)
    }

    #[inline]
    pub(crate) fn parse(&self, field: &str) -> PolarsResult<AnyValue<'static>> {
        polars::prelude::PolarsContext::with_context(
            parse_cell_value(
                field,
                &self.question_type,
                self.is_literal,
                self.subfield_args.as_ref(),
            )
            .map_err(Into::into),
            || format!("When parsing question '{}'", self.question_name),
        )
    }
}

pub(crate) fn parse_fixed(parsers: &[ResponseParser], row: &str) -> Result<Row<'static>> {
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
    parsers: &[ResponseParser],
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

pub(crate) fn parse_cell_value(
    raw: &str,
    question_type: &QuestionType,
    is_literal: bool,
    subfield_args: Option<&SubfieldArgs>,
) -> PolarsResult<AnyValue<'static>> {
    let data = raw.trim();
    if data.is_empty() {
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
            let data_read = match subfield_args.as_ref() {
                None => parse_binary_multiple(data),
                Some(args) => parse_subfields_multiple(data, args),
            };
            let data_read = data_read
                .map_err(|err| PolarsError::ComputeError(format!("{:#?}", err).into()))?;

            let series = UInt32Chunked::from_iter_values("".into(), data_read.into_iter())
                .sort(false)
                .into_series();
            Ok(AnyValue::List(series))
        }
    }
}

fn parse_binary_multiple(data: &str) -> Result<HashSet<u32>> {
    let mut response: HashSet<u32> = HashSet::new();
    for (index, chr) in data.chars().enumerate() {
        match chr {
            '0' => Ok(()),
            '1' => {
                response.insert(index as u32 + 1);
                Ok(())
            }
            _ => Err(DataReadError(format!(
                "Invalid multiple-choice response '{}' at position {}: expected '0' or \
                    '1'.",
                chr,
                index + 1,
            ))),
        }?;
    }
    Ok(response)
}

fn parse_subfields_multiple(data: &str, args: &SubfieldArgs) -> Result<HashSet<u32>> {
    let mut response: HashSet<u32> = HashSet::new();

    match args.width {
        Some(width) => {
            for chunk in data.as_bytes().chunks(width as usize) {
                let value = std::str::from_utf8(chunk)?.trim();
                let code = value.parse::<u32>()?;
                if code != 0 {
                    response.insert(code);
                }
            }
        }
        None => {
            let chunk_size = data.len().div_ceil(args.subfields);
            for chunk in data.as_bytes().chunks(chunk_size) {
                let value = std::str::from_utf8(chunk)?.trim();
                let code = value.parse::<u32>()?;
                if code != 0 {
                    response.insert(code);
                }
            }
        }
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{QuestionOptions, utils::lock_it_up};

    fn plan() -> Vec<ResponseParser> {
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
                ResponseParser::from_question_read_guard(q.read()).unwrap()
            })
            .collect()
    }

    #[test]
    fn test_indexed_ranges_support_unicode_gaps_overlap_and_order() {
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
    fn test_owned_scalar_values_match_question_parsing() {
        use crate::CodeFormat;

        for (kind, format, field, correct) in [
            (QuestionType::Quantity, None, " 123.456 ", AnyValue::Float64(123.456)),
            (QuestionType::Logical, None, "1", AnyValue::Boolean(true)),
            (QuestionType::Logical, None, "0", AnyValue::Boolean(false)),
            (QuestionType::Single, None, "", AnyValue::Null),
            (QuestionType::Single, Some(CodeFormat::Literal), " A ", AnyValue::String("A")),
            (QuestionType::Character, None, " 中😀 text ", AnyValue::String("中😀 text")),
            (QuestionType::Date, None, "20261115", AnyValue::Date(20772)),
            (QuestionType::Time, None, "123456", AnyValue::Time(45_296_000_000_000)),
        ] {
            let question = lock_it_up(
                QuestionOptions::new("Q")
                    .with_question_type(kind)
                    .with_format(format)
                    .finish()
                    .unwrap(),
            );
            let parser = ResponseParser::from_question_read_guard(question.read()).unwrap();
            let parsed = parser.parse(field).unwrap();
            assert_eq!(parsed, correct);
        }
        assert!(parse_cell_value("", &QuestionType::None, false, None).is_err());
    }

    #[test]
    fn test_batches_preserve_order_and_stop_at_first_error() {
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
                    ResponseParser::from_question_read_guard(q.read()).unwrap()
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

                        let parser = ResponseParser::from_question(q).unwrap();
                        parser.parse(field).unwrap().into_static()
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
