use std::sync::Arc;

use anyhow::{Context, Result};
use parking_lot::RwLock;

use crate::prelude::DeepCopy;

pub type ArcLock<T> = Arc<RwLock<T>>;

impl<T: DeepCopy<Output = T>> DeepCopy for ArcLock<T> {
    type Output = ArcLock<T>;

    #[inline]
    fn deepcopy(&self) -> Self::Output {
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

pub fn validate_language_tag(language: &str) -> Result<&str> {
    let _ = oxilangtag::LanguageTag::parse(language)
        .with_context(|| format!("Unknown language tag: '{}'.", language))?;
    Ok(language)
}
