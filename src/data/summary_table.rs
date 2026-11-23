//! Instructions for running topline tables for a survey struct.
//! currently work in progress

#![allow(unused)]

// #[cfg(feature="pyo3")]
// use pyo3::prelude::*;
use std::sync::Arc;

use crate::options::{AnalysisOptions, LabelOptions};

/// Summary table for a specific question
#[derive(Debug, Clone)]
pub struct Table {
    pub target: Analysis,
    pub operation: Calculate,
}

/// Operations to do
#[derive(Debug, Clone)]
pub enum Calculate {
    Crosstab,
}

/// Question to analyse
#[derive(Debug, Clone)]
pub struct Analysis {
    pub question: Arc<str>,
    pub top: Arc<str>,
}

/// Display options for summary tables
#[derive(Debug, Clone)]
pub struct TableOptions {
    /// include/ exclude summary stats columns/ header rows/ etc.,
    pub include: Include,

    /// column names for each of the item
    pub col_names: ColName,
}

/// Controlling rows/ columns to be included/ excluded in the table output
#[derive(Debug, Clone)]
pub struct Include {
    pub row_total: bool,
    pub col_total: bool,
    pub question_name: bool,
    pub code_value: bool,
}

/// Column names for each of the label columns
#[derive(Debug, Clone)]
pub struct ColName {
    pub row_total: Arc<str>,
    pub col_total: Arc<str>,
    pub question_name: Arc<str>,
    pub code_value: Arc<str>,
}
