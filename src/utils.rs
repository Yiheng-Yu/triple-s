use std::sync::Arc;
use parking_lot::{RwLock};

use anyhow::{Context, Result};
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

pub fn validate_language_tag(language: &str) -> Result<&str> {
    let _ = oxilangtag::LanguageTag::parse(language)
        .with_context(|| format!("Unknown language tag: '{}'.", language))?;
    Ok(language)
}