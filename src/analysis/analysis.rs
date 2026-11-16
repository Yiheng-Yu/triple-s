use std::sync::Arc;
use parking_lot::{RwLockReadGuard};
use crate::utils::{ArcLock, lock_it_up};

#[derive(Debug)]
pub struct CrossTab {
    question: ArcLock<String>,
    group_by: Vec<ArcLock<String>>,
}

impl CrossTab {
    pub fn question<'a>(&'a self) -> RwLockReadGuard<'a, String>{
        self.question.read()
    }
}