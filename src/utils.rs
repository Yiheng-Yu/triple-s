use std::{
    io::{self, BufRead, Read},
    sync::Arc,
};

use anyhow::{Context, Result};
use encoding_rs::{Decoder, DecoderResult, UTF_8};
use parking_lot::RwLock;
#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*, types::PySlice};

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

/// Used to read various raw files with various file encodings, mostly to accomdate Microsoft.
/// oh god please save me
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
            let (status, consumed, written) =
                self.decoder.decode_to_utf8_without_replacement(input, &mut self.output, last);
            self.source.consume(consumed);
            self.position = 0;
            self.length = written;
            match status {
                DecoderResult::Malformed(..) => {
                    self.finished = true;
                    self.length = 0;
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Malformed Unicode data",
                    ));
                }
                DecoderResult::InputEmpty if last => self.finished = true,
                _ => {}
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
        let mut truncated = TextReader::new(&[0xff, 0xfe, 0x41][..]);
        assert!(truncated.read_to_string(&mut String::new()).is_err());
    }
}
