use std::sync::Arc;
use parking_lot::{RwLock};
use pyo3::{PyErr, exceptions::PyValueError};
pub type ArcLock<T> = Arc<RwLock<T>>;


#[inline]
pub fn lock_it_up<T>(item: T) -> Arc<RwLock<T>> {
    Arc::new(RwLock::new(item))
}

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

pub fn validate_language_tag(language: &str) -> Result<&str, PyErr> {
    let _ = oxilangtag::LanguageTag::parse(language).map_err(|err| {
        PyValueError::new_err(format!(
            "Error parsing language tag: {}. Detail: {}",
            language,
            err.to_string()
        ))
    })?;
    Ok(language)
}