use std::{
    fmt::Debug, 
    path::PathBuf, 
    
    sync::Arc};

use polars::prelude::*;

use crate::{
    utils::ArcLock,
    Question
};


pub struct SurveyData {
    path: PathBuf,
    plan: LazyFrame
}

impl Debug for SurveyData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SurveyData")
            .field_with("path", |f| write!(f, "{:?}", self.path))
            .field_with("plan", |f| write!(f, "LazyFrame"))
            .finish()
    }
}