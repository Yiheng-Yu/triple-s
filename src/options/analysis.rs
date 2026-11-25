//! Options for running various survey data analysis methods

use std::{collections::HashMap, ops::Sub};

use parking_lot::RwLockReadGuard;
use polars::prelude::*;

use crate::{
    datamap::{Question, QuestionType},
    utils::ArcLock,
};

/// Types of questions to skip when running map functions.
///
/// Default:
///
/// ```text
/// SkipQType {
///     text: true,
///     numeric: true,
///     datetime: true,
///     single: false,
///     multiple: false
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SkipQType {
    /// Skip [`QuestionType::Character`], default `true`
    pub text: bool,

    /// Skip [`QuestionType::Quantity`], default `true`
    pub numeric: bool,

    /// Skip [`QuestionType::Date`] & [`QuestionType::Time`], default `true`
    pub datetime: bool,

    /// Skip [`QuestionType::Single`], default `false`
    pub single: bool,

    /// Skip [`QuestionType::Multiple`], default `false`
    pub multiple: bool,
}

impl Default for SkipQType {
    fn default() -> Self {
        Self {
            text: true,
            numeric: true,
            datetime: true,
            single: false,
            multiple: false,
        }
    }
}

impl SkipQType {
    /// Checks if the question should be skipped based on its `question_type` field.
    pub fn should_skip(&self, question: &RwLockReadGuard<'_, Question>) -> bool {
        match *question.question_type() {
            QuestionType::None => true, // always skip
            QuestionType::Multiple => self.multiple,
            QuestionType::Date | QuestionType::Time => self.datetime,
            QuestionType::Logical | QuestionType::Single => self.single,
            QuestionType::Character => self.text,
            QuestionType::Quantity => self.numeric,
        }
    }
}

/// Arguments for running cross tabs for a particular question in a survey.
#[derive(Debug, Clone)]
pub struct AnalysisOptions<'a> {
    /// Name of the question to use as the corsstab rows.
    pub question_name: &'a str,

    /// List of question names to be used as the corsstab columns.
    pub columns: &'a [&'a str],

    /// Return counts as relative frequencies normalised to 1.0.
    ///
    /// Each response count is divided by its column total. The first row,
    /// when shown, keeps the original total counts.
    pub normalise: bool,
}

/// Options for the labelling summary table outputs (e.g., [`QuestionData::value_counts`](crate::data::QuestionData), [`Survey::corsstab`](crate::survey::Survey))
#[derive(Debug, Clone)]
pub struct LabelOptions<'a> {
    /// Include the overall count column in crosstabs. Default `true`.
    pub show_row_total: bool,

    /// Include the first row of totals in crosstabs. Default `true`.
    pub show_column_total: bool,

    /// Toggle on to include a column of code values for each response item
    /// Used only for single/ multi-choice questions
    pub include_response_codes: bool,

    /// Toggle on to add a column that shows the question name
    pub include_question_name: bool,

    /// Toggle on to fill the question name column with name of the question, only used when "include_question_name" is `true`. Default `false`.
    pub fill_question_name_column: bool,

    /// Column name for the question name column, default "question", only used when `include_question_name` is `true`
    pub col_question_name: Option<&'a str>,

    /// Column name for the response code column, default "code", only used when `include_response_codes` is `true`
    pub col_code_value: Option<&'a str>,

    /// Column name for the response label column, default "label"
    pub col_code_label: Option<&'a str>,

    /// Column name for the value counts column, default "count" (same as polars)
    pub col_count: Option<&'a str>,
}

impl<'a> Default for LabelOptions<'a> {
    fn default() -> Self {
        Self {
            show_row_total: true,
            show_column_total: true,
            include_response_codes: false,
            include_question_name: false,
            fill_question_name_column: false,
            col_question_name: Some(Self::QUESTION),
            col_count: Some(Self::COUNT),
            col_code_label: Some(Self::LABEL),
            col_code_value: Some(Self::CODE),
        }
    }
}

impl<'a> LabelOptions<'a> {
    /// default value for field `col_code_value`
    pub const CODE: &'static str = "code";
    /// default value for field `col_count`
    pub const COUNT: &'static str = "total";
    /// default value for field `col_code_label`
    pub const LABEL: &'static str = "label";
    /// default value for field `col_question`
    pub const QUESTION: &'static str = "question";

    /// Toggle whether to include the overall count column.
    pub fn with_show_row_total(mut self, value: bool) -> Self {
        self.show_row_total = value;
        self
    }

    /// Toggle whether to include the first row of totals.
    pub fn with_show_column_total(mut self, value: bool) -> Self {
        self.show_column_total = value;
        self
    }

    /// Create new instance with default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle on to include a column of code values for each response item
    /// Used only for single/ multi-choice questions
    pub fn with_include_response_codes(mut self, value: bool) -> Self {
        self.include_response_codes = value;
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

    /// Column name for the question name column, default Self::QUESTION, only used when `include_question_name` is `true`
    pub fn with_col_question_name(mut self, value: Option<&'a str>) -> Self {
        self.col_question_name = value;
        self
    }

    /// Column name for the response code column, default Self::CODE, only used when `include_response_codes` is `true`
    pub fn with_col_code(mut self, value: Option<&'a str>) -> Self {
        self.col_code_value = value;
        self
    }

    /// Column name for the response label column, default Self::LABEL
    pub fn with_col_label(mut self, value: Option<&'a str>) -> Self {
        self.col_code_label = value;
        self
    }

    /// Column name for the value counts column of the question, default "total"
    pub fn with_col_count(mut self, value: Option<&'a str>) -> Self {
        self.col_count = value;
        self
    }

    /// Creates a new column
    pub fn question_name_column(&self, question_name: &str) -> Expr {
        let colname = self.col_question_name.unwrap_or(Self::QUESTION);
        if self.fill_question_name_column {
            lit(question_name.to_string()).alias(colname)
        } else {
            lit(question_name.to_string())
                .extend_constant(lit(NULL), len().sub(lit(1)))
                .alias(colname)
        }
    }

    /// Convert current configuration to polars expression
    pub fn into_expression(&self, question_name: &'a str) -> Vec<Expr> {
        let mut columns = Vec::new();
        let colname_stats = self.col_count.unwrap_or(Self::COUNT);

        // append question name column? or not
        if self.include_question_name {
            if self.fill_question_name_column {
                let name = self.col_question_name.unwrap_or(Self::QUESTION);
                let qname_column = lit(question_name.to_string());
                columns.push(qname_column.alias(name));
            } else {
                columns.push(self.question_name_column(question_name));
            }
        }

        // append response code column? or not
        if self.include_response_codes {
            let name = self.col_code_value.unwrap_or(Self::CODE);
            columns.push(col(question_name).alias(name));
        }

        // always include
        let colname_resp_label = self.col_code_label.unwrap_or(Self::LABEL);

        columns.push(col(colname_resp_label));
        columns.push(col(colname_stats));
        columns
    }
}

impl<'a> LabelOptions<'a> {
    /// Returns a column expression
    #[allow(clippy::mutable_key_type)]
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

        let colname_resp_label = self.col_code_label.unwrap_or(Self::LABEL);
        Ok(col(question_name)
            .replace_strict(
                lit(row_old.clone()),
                lit(row_new),
                Some(col(question_name).cast(DataType::String)),
                Some(DataType::String),
            )
            .fill_null(lit("Total")) // Self::TODO!
            .alias(colname_resp_label))
    }
}
