use std::sync::Arc;
use parking_lot::{
    RawRwLock, 
    RwLock, 
    ArcRwLockReadGuard, 
    ArcRwLockWriteGuard,
};
use anyhow::{Context, Result};
use crate::prelude::DeepCopy;

pub type ArcLock<T> = Arc<RwLock<T>>;

impl<T: Clone> DeepCopy for ArcLock<T> {
    type Output = ArcLock<T>;

    #[inline]
    fn deepcopy(&self) -> Self::Output {
        let inner = self.read().clone();
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

/// A mutable collection of shared, lockable values, used mainly for sharing referenced obejcts
/// with Python's (uncanny) dynamic typing
#[derive(Debug)]
pub struct ArcList<T: Clone>(pub ArcLock<Vec<ArcLock<T>>>);

impl<T: Clone> ArcList<T> {
    pub fn read(&self) -> ArcRwLockReadGuard<RawRwLock, Vec<ArcLock<T>>> {
        self.0.read_arc()
    }

    pub fn write(&self) -> ArcRwLockWriteGuard<RawRwLock, Vec<ArcLock<T>>> {
        self.0.write_arc()
    }

    pub fn get(&self, index: usize) -> Option<ArcLock<T>> {
        self.read().get(index).map(|s| s.clone())
    }

    pub fn len(&self) -> usize {
        self.read().len()
    }

    /// makes a deep copy of the data inside so that mutating the copied data
    /// won't affect the original
    pub fn deepcopy(&self) -> Self {
        let inner = self.0.read().clone()
            .iter()
            .map(|item| lock_it_up(item.read().clone()))
            .collect::<Vec<ArcLock<T>>>();
        Self(lock_it_up(inner))
    }

    /// makes a copy of just the pointer inside, this method returns a sharable, mutable copy of current instance.
    /// mutating copied data WILL affect the original instance.
    pub fn clone_ptr(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Clone> AsRef<ArcLock<Vec<ArcLock<T>>>> for ArcList<T> {
    fn as_ref(&self) -> &ArcLock<Vec<ArcLock<T>>> {
        &self.0
    }
}

// impl<T: Clone> Clone for ArcList<T> {
//     /// makes a deep copy of the data inside so that mutating the copied data
//     /// won't affect the original
//     fn clone(&self) -> Self {
//         let inner = self.0.read().clone()
//             .iter()
//             .map(|item| lock_it_up(item.read().clone()))
//             .collect::<Vec<ArcLock<T>>>();
//         Self(lock_it_up(inner))
//     }
// } 
