//! Instructions for running topline tables for a survey struct.


#![allow(unused)]

// #[cfg(feature="pyo3")]
// use pyo3::prelude::*;
use std::sync::Arc;
use crate::options::{AnalysisOptions, LabelOptions};

#[derive(Debug, Clone)]
pub struct CrossTab {
    pub question_name: Arc<str>,
    pub columns: Vec<Arc<str>>,
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



