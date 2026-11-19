//! Options for running various survey data analysis methods

use std::{collections::HashMap, ops::Sub};

use polars::prelude::*;

use crate::utils::ArcLock;

/// Arguments for running cross tabs for a particular question in a survey.
#[derive(Debug)]
pub struct AnalysisOptions<'a> {
    /// Name of the question to use as the corsstab rows.
    pub question_name: &'a str,
    /// List of question names to be used as the corsstab columns.
    pub columns: &'a [&'a str],
    /// show row total
    pub show_row_total: bool,
    /// show column total
    pub show_column_total: bool,
}

/// Options for the labelling summary table outputs (e.g., [`QuestionData::value_counts`], [`Survey::corsstab`])
#[allow(unused)]
pub struct LabelOptions<'a> {
    /// Toggle on to include a column of code values for each response item
    /// Used only for single/ multi-choice questions
    pub include_code_value: bool,

    /// Toggle on to add a column that shows the question name
    pub include_question_name: bool,

    /// Toggle on to fill the question name column with name of the question, only used when "include_question_name" is `true`. Default `false`.
    pub fill_question_name_column: bool,

    /// Column name for the question name column, default "question", only used when `include_question_name` is `true`
    pub question_name_col: Option<&'a str>,

    /// Column name for the response code column, default "code", only used when `include_code_value` is `true`
    pub code_col: Option<&'a str>,

    /// Column name for the response label column, default "label"
    pub label_col: Option<&'a str>,

    /// Column name for the value counts column, default "count" (same as polars)
    pub count_col: Option<&'a str>,
}

impl<'a> Default for LabelOptions<'a> {
    fn default() -> Self {
        Self {
            include_code_value: false,
            include_question_name: false,
            fill_question_name_column: false,
            question_name_col: Some("question"),
            count_col: Some("total"),
            label_col: Some("label"),
            code_col: Some("code"),
        }
    }
}

impl<'a> LabelOptions<'a> {
    /// Create new instance with default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle on to include a column of code values for each response item
    /// Used only for single/ multi-choice questions
    pub fn with_include_code_value(mut self, value: bool) -> Self {
        self.include_code_value = value;
        self
    }

    /// Toggle on to add a column that shows the question name
    pub fn with_include_question_name(mut self, value: bool) -> Self {
        self.include_question_name = value;
        self
    }

    /// Toggle on to fill the question name column with name of the question, only used when "include_question_name" is `true`. Default `false`.
    pub fn with_fill_question_name_column(mut self, value: bool) -> Self {
        self.fill_question_name_column = value;
        self
    }

    /// Column name for the question name column, default "question", only used when `include_question_name` is `true`
    pub fn with_question_name_col(mut self, value: Option<&'a str>) -> Self {
        self.question_name_col = value;
        self
    }

    /// Column name for the response code column, default "code", only used when `include_code_value` is `true`
    pub fn with_code_col(mut self, value: Option<&'a str>) -> Self {
        self.code_col = value;
        self
    }

    /// Column name for the response label column, default "label"
    pub fn with_label_col(mut self, value: Option<&'a str>) -> Self {
        self.label_col = value;
        self
    }

    /// Column name for the value counts column of the question, default "total"
    pub fn with_count_col(mut self, value: Option<&'a str>) -> Self {
        self.count_col = value;
        self
    }

    /// Convert current configuration to polars expression
    pub fn into_expression(self, question_name: &'a str) -> Vec<Expr> {
        let mut columns = Vec::new();
        let colname_stats = self.count_col.unwrap_or("total");

        // append question name column? or not
        if self.include_question_name & self.fill_question_name_column {
            let name = self.question_name_col.unwrap_or("question");
            let qname_column = lit(question_name.to_string());
            columns.push(qname_column.alias(name));
        }

        if self.include_question_name & !self.fill_question_name_column {
            let name = self.question_name_col.unwrap_or("question");
            let qname_column = lit(question_name.to_string())
                .extend_constant(lit(NULL), len().sub(lit(1)))
                .alias(name);
            columns.push(qname_column.alias(name));
        }

        // append response code column? or not
        if self.include_code_value {
            let name = self.code_col.unwrap_or("code");
            columns.push(col(question_name).alias(name));
        }

        // always include
        let colname_resp_label = self.label_col.unwrap_or("label");

        columns.push(col(colname_resp_label));
        columns.push(col(colname_stats));
        columns
    }
}

impl<'a> LabelOptions<'a> {
    pub(crate) fn match_response_label(
        &'a self,
        question_name: &str,
        response_label_map: HashMap<AnyValue<'static>, ArcLock<String>>,
    ) -> PolarsResult<Expr> {
        let (row_old, row_new): (Vec<AnyValue<'static>>, Vec<String>) = response_label_map
            .iter()
            .map(|(id, label)| (id.clone(), label.read().to_string()))
            .unzip();
        let row_old = Series::from_any_values("old".into(), &row_old, true)?;
        let row_new = Series::new("new".into(), row_new);

        let colname_resp_label = self.label_col.unwrap_or("label");
        Ok(col(question_name)
            .replace_strict(
                lit(row_old.clone()),
                lit(row_new),
                Some(col(question_name).cast(DataType::String)),
                Some(DataType::String),
            )
            .fill_null(lit("Total")) // TODO!
            .alias(colname_resp_label))
    }
}
