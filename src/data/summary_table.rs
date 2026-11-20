//! Instructions for running topline tables for a survey struct.

// #[cfg(feature="pyo3")]
// use pyo3::prelude::*;

// use crate::options::{AnalysisOptions, LabelOptions};

// #[cfg_attr(
//     feature = "pyo3",
//     pyclass(module = "triple_s", name = "CrossTab", from_py_object)
// )]
// #[derive(Debug, Clone)]
// pub struct CrossTab<'a> {
//     pub content: AnalysisOptions<'a>,
//     pub config: LabelOptions<'a>
// }

// impl CrossTab {
//     pub fn with_code_column(&mut self, column_name: Option<&str>) {
//         self.config.col_code_value = column_name;
//     }
// }