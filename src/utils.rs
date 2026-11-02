use std::{
    io::{self, BufRead, Read},
    sync::Arc,
};

use anyhow::{Context, Result};
use encoding_rs::{CoderResult, Decoder, UTF_8};
use parking_lot::RwLock;
#[cfg(feature = "pyo3")]
use polars::datatypes::AnyValue;
#[cfg(feature = "pyo3")]
use pyo3::{IntoPyObjectExt, exceptions, prelude::*, types::PySlice};

use crate::prelude::DeepCopy;

pub type ArcLock<T> = Arc<RwLock<T>>;

impl<T: DeepCopy> DeepCopy for ArcLock<T> {
    #[inline]
    fn deepcopy(&self) -> Self {
        let inner = self.read().deepcopy();
        lock_it_up(inner)
    }
}

pub(crate) fn validate_date(value: &str) -> Result<(), chrono::ParseError> {
    chrono::NaiveDate::parse_from_str(value, "%Y%m%d")?;
    Ok(())
}

pub(crate) fn validate_time(value: &str) -> Result<(), chrono::ParseError> {
    chrono::NaiveTime::parse_from_str(value, "%H%M%S")?;
    Ok(())
}

#[inline]
pub fn lock_it_up<T>(item: T) -> Arc<RwLock<T>> {
    Arc::new(RwLock::new(item))
}

#[cfg(feature = "pyo3")]
pub fn normalize_py_index(index: isize, total: usize) -> Option<usize> {
    let len = total as isize;

    let index = if index < 0 {
        len.checked_add(index)?
    } else {
        index
    };

    if index < 0 || index >= len {
        None
    } else {
        Some(index as usize)
    }
}

#[cfg(feature = "pyo3")]
#[inline]
pub fn get_type_name<'py>(object: &pyo3::Bound<'py, pyo3::PyAny>) -> pyo3::PyResult<String> {
    object.get_type().name()?.extract::<String>().map_err(|err| {
        exceptions::PyRuntimeError::new_err(format!(
            "Backend error raised from rust: {:#}",
            err
        ))
    })
}

#[cfg(feature = "pyo3")]
pub(crate) fn anyvalue_to_python<'py>(
    py: Python<'py>,
    value: AnyValue<'_>,
) -> PyResult<Bound<'py, PyAny>> {
    match value {
        AnyValue::Null => py.None().into_bound_py_any(py),
        AnyValue::Boolean(v) => v.into_bound_py_any(py),
        AnyValue::Int64(v) => v.into_bound_py_any(py),
        AnyValue::Int32(v) => v.into_bound_py_any(py),
        AnyValue::UInt64(v) => v.into_bound_py_any(py),
        AnyValue::UInt32(v) => v.into_bound_py_any(py),
        AnyValue::Float64(v) => v.into_bound_py_any(py),
        AnyValue::Float32(v) => v.into_bound_py_any(py),
        AnyValue::String(v) => v.into_bound_py_any(py),
        AnyValue::StringOwned(v) => v.as_str().into_bound_py_any(py),
        AnyValue::Date(days) => convert_date(days)?.into_bound_py_any(py),
        AnyValue::Time(nanos) => {
            let seconds = u32::try_from(nanos / 1_000_000_000).ok();
            let fraction = u32::try_from(nanos % 1_000_000_000).ok();
            let time = seconds
                .zip(fraction)
                .and_then(|(s, ns)| {
                    chrono::NaiveTime::from_num_seconds_from_midnight_opt(s, ns)
                })
                .ok_or_else(|| exceptions::PyValueError::new_err("Time out of range"))?;
            time.into_bound_py_any(py)
        }
        AnyValue::List(values) => {
            let values = values
                .iter()
                .map(|value| anyvalue_to_python(py, value))
                .collect::<PyResult<Vec<_>>>()?;
            values.into_bound_py_any(py)
        }
        value => Err(exceptions::PyTypeError::new_err(format!(
            "Unsupported Polars dtype: {:?}",
            value.dtype()
        ))),
    }
}

#[cfg(feature = "pyo3")]
#[inline]
pub(crate) fn convert_date(value: i32) -> PyResult<chrono::NaiveDate> {
    chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
        .unwrap()
        .checked_add_signed(chrono::Duration::days(i64::from(value)))
        .ok_or_else(|| exceptions::PyValueError::new_err("Date out of range"))
}

#[cfg(feature = "pyo3")]
#[inline]
pub(crate) fn convert_time(value: i64) -> PyResult<chrono::NaiveTime> {
    let seconds = u32::try_from(value / 1_000_000_000).ok();
    let fraction = u32::try_from(value % 1_000_000_000).ok();
    seconds
        .zip(fraction)
        .and_then(|(s, ns)| chrono::NaiveTime::from_num_seconds_from_midnight_opt(s, ns))
        .ok_or_else(|| exceptions::PyValueError::new_err("Time out of range"))
}

#[cfg(feature = "pyo3")]
pub fn py_value_error(err: anyhow::Error) -> PyErr {
    exceptions::PyValueError::new_err(format!("{err:#}"))
}

pub fn validate_language_tag(language: &str) -> Result<&str> {
    language_tags::LanguageTag::parse(language)
        .with_context(|| format!("Unknown language tag: '{}'.", language))?
        .validate()
        .with_context(|| format!("Unknown language tag: '{}'.", language))?;

    Ok(language)
}

/// wrap strings inside pair of single quotation marks. used for python repl.
#[cfg(feature = "pyo3")]
#[inline]
pub fn single_quote(value: impl std::fmt::Display) -> String {
    format!(r#"'{}'"#, value)
}

#[cfg(feature = "pyo3")]
pub(crate) fn extract_indicies<'py>(
    indicies: &Bound<'py, PySlice>,
    total: isize,
) -> Result<Vec<isize>> {
    let indicies: pyo3::types::PySliceIndices = indicies.indices(total)?;

    // manually adding them one by one instead of calling step_by because
    // index & steps can be negative
    Ok(std::iter::successors(Some(indicies.start), |index| {
        index.checked_add(indicies.step)
    })
    .take(indicies.slicelength)
    .into_iter()
    .collect())
}

/// Stream UTF-8 or BOM-marked UTF-16 data, replacing malformed sequences with U+FFFD.
pub(crate) struct TextReader<R> {
    source: R,
    decoder: Decoder,
    output: [u8; 8192],
    position: usize,
    length: usize,
    finished: bool,
}

impl<R: BufRead> TextReader<R> {
    pub(crate) fn new(source: R) -> Self {
        Self {
            source,
            decoder: UTF_8.new_decoder(),
            output: [0; 8192],
            position: 0,
            length: 0,
            finished: false,
        }
    }
}

impl<R: BufRead> Read for TextReader<R> {
    fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
        if destination.is_empty() {
            return Ok(0);
        }
        loop {
            if self.position < self.length {
                let count = destination.len().min(self.length - self.position);
                destination[..count]
                    .copy_from_slice(&self.output[self.position..self.position + count]);
                self.position += count;
                return Ok(count);
            }
            if self.finished {
                return Ok(0);
            }

            let input = self.source.fill_buf()?;
            let last = input.is_empty();
            // The decoder needs space for one complete UTF-8 character.
            let direct = destination.len() >= 4;
            let output = if direct {
                &mut *destination
            } else {
                &mut self.output
            };
            let (status, consumed, written, _) =
                self.decoder.decode_to_utf8(input, output, last);
            self.source.consume(consumed);
            self.finished = last && status == CoderResult::InputEmpty;
            if direct {
                if written != 0 {
                    return Ok(written);
                }
            } else {
                self.position = 0;
                self.length = written;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Read};

    use super::{TextReader, validate_language_tag};

    #[test]
    fn test_validate_language_tag() {
        assert!(validate_language_tag("what").is_err());
        assert!(validate_language_tag("ok").is_err());
        assert!(validate_language_tag("").is_err());

        assert_eq!(validate_language_tag("en-GB").unwrap(), "en-GB");
        assert_eq!(validate_language_tag("fr").unwrap(), "fr");
        assert_eq!(validate_language_tag("en-US").unwrap(), "en-US");
    }

    #[test]
    fn utf16_streaming_handles_split_code_units() {
        let text = "中文😀\r\n".repeat(3000);
        for little_endian in [true, false] {
            let bytes: Vec<u8> = std::iter::once(0xfeff)
                .chain(text.encode_utf16())
                .flat_map(|unit| {
                    if little_endian {
                        unit.to_le_bytes()
                    } else {
                        unit.to_be_bytes()
                    }
                })
                .collect();
            let mut reader = TextReader::new(BufReader::with_capacity(1, bytes.as_slice()));
            let mut decoded = String::new();
            reader.read_to_string(&mut decoded).unwrap();
            assert_eq!(decoded, text);
        }
    }

    #[test]
    fn malformed_sequences_are_replaced_across_buffer_boundaries() {
        let bytes = b"A\x96B\xc3\xa9\xf0\x9f\x98\x80\xe2\x82!\xff\xc3".repeat(3000);
        let expected = "A�Bé😀�!��".repeat(3000);
        for capacity in [1, 8192] {
            let mut reader =
                TextReader::new(BufReader::with_capacity(capacity, bytes.as_slice()));
            let mut decoded = String::new();
            reader.read_to_string(&mut decoded).unwrap();
            assert_eq!(decoded, expected);
        }

        let mut truncated = TextReader::new(&[0xff, 0xfe, 0x41][..]);
        let mut decoded = String::new();
        truncated.read_to_string(&mut decoded).unwrap();
        assert_eq!(decoded, "�");
    }

    #[test]
    fn text_reader_handles_empty_reads_and_eof() {
        for (input, expected) in [
            (b"".as_slice(), ""),
            (b"\xef\xbb\xbf", ""),
            (b"\xff\xc3", "��"),
        ] {
            let mut reader = TextReader::new(input);
            assert_eq!(reader.read(&mut []).unwrap(), 0);
            let mut decoded = Vec::new();
            let mut byte = [0];
            while reader.read(&mut byte).unwrap() != 0 {
                decoded.push(byte[0]);
            }
            assert_eq!(decoded, expected.as_bytes());
            assert_eq!(reader.read(&mut byte).unwrap(), 0);
        }
    }

    #[test]
    fn text_reader_handles_mixed_read_sizes() {
        let input = b"A\xc3\xa9\xf0\x9f\x98\x80\x96Z".repeat(3000);
        let mut reader = TextReader::new(BufReader::with_capacity(3, input.as_slice()));
        let mut decoded = Vec::new();
        let mut buffer = [0; 8192];
        for size in [0, 1, 3, 4, 2, 8192].into_iter().cycle() {
            let count = reader.read(&mut buffer[..size]).unwrap();
            if size == 0 {
                assert_eq!(count, 0);
                continue;
            }
            if count == 0 {
                break;
            }
            decoded.extend_from_slice(&buffer[..count]);
        }
        assert_eq!(decoded, "Aé😀�Z".repeat(3000).as_bytes());
        assert_eq!(reader.read(&mut buffer).unwrap(), 0);
    }
}
