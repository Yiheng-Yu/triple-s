use std::sync::Arc;

use anyhow::{Context, Result};
use parking_lot::RwLock;

use crate::prelude::DeepCopy;

#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*};

pub type ArcLock<T> = Arc<RwLock<T>>;

impl<T:DeepCopy> DeepCopy for ArcLock<T> {
    #[inline]
    fn deepcopy(&self) -> Self {
        let inner = self.read().deepcopy();
        lock_it_up(inner)
    }
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
    object.get_type().name()?.extract::<String>()
        .map_err(|err| exceptions::PyRuntimeError::new_err(format!(
            "Backend error raised from rust: {}", err.to_string()
        )))
}


#[cfg(feature = "pyo3")]
pub fn py_value_error(error: anyhow::Error) -> PyErr {
    exceptions::PyValueError::new_err(error.to_string())
}

pub fn validate_language_tag(language: &str) -> Result<&str> {
    let _ = oxilangtag::LanguageTag::parse(language)
        .with_context(|| format!("Unknown language tag: '{}'.", language))?;
    Ok(language)
}
