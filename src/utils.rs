use std::sync::Arc;
use parking_lot::{RwLock};

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