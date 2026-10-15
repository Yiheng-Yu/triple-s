use std::sync::Arc;

use anyhow::{Context, Result};
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
    let _ = oxilangtag::LanguageTag::parse(language)
        .with_context(|| format!("Unknown language tag: '{}'.", language))?;
    Ok(language)
}

// #[inline]
// fn double_quote(value: impl Display) -> String {
//     format!(r#""{}""#, value)
// }

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
