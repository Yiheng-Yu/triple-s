/// Arguments for running cross tabs for a particular question in a survey.
#[derive(Debug)]
pub struct CrossTabArgs<'a> {
    /// Name of the question to use as the corsstab rows.
    pub question_name: &'a str,
    /// List of question names to be used as the corsstab columns.
    pub columns: &'a [&'a str],
    /// show row total
    pub show_row_total: bool,
    /// show column total
    pub show_column_total: bool,
}
