use std::collections::HashMap;
#[cfg(feature = "pyo3")]
use std::sync::Arc;

use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::{IntoPyObjectExt, exceptions, prelude::*};
#[cfg(feature = "pyo3")]
use pyo3_polars::{PyDataFrame, error::PyPolarsErr};

#[cfg(feature = "pyo3")]
use crate::{
    SurveyInfo,
    data::row_iterator::RowIterator,
    options::{DataFrameFormat, ResponseDisplay},
    prelude::*,
    utils::{get_type_name, normalize_py_index, polars_to_pandas},
};
use crate::{
    LabelOptions, data::{QuestionData, SurveyData, operation}, datamap::Datamap, error::QuestionNotFound, options::{AnalysisOptions, SurveyOptions}, utils::{ArcLock, lock_it_up},
};

// rust docstring
#[cfg_attr(
    any(not(feature = "pyo3"), doc), 
    doc = include_str!("./doc/survey_rs.md")
)]
// pyo3 docstring
#[cfg_attr(
    all(feature = "pyo3", not(doc)), 
    doc = include_str!("./doc/survey_py.md")
)]
// pyo3 signatures
#[cfg_attr(
    all(feature = "pyo3", not(doc)),
    doc = r###"
Args:
    datamap (Datamap): Survey definition used to interpret the respondent data.

Keyword Args:
    data (SurveyData | None): Respondent data. None creates a survey without loaded data. Defaults to None.

Returns:
    Survey: The constructed Survey instance.
"###
)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Survey", skip_from_py_object)
)]
#[derive(Debug, Clone)]
pub struct Survey {
    datamap: Datamap,
    data: ArcLock<SurveyData>,
    config: ArcLock<SurveyOptions>,
}

/// Fields
///
/// ---
impl Survey {
    /// Title of survey
    pub fn title(&self) -> Option<String> {
        self.datamap().title()
    }

    /// Sets survey title
    #[inline]
    pub fn set_title(&mut self, title: Option<String>) {
        *self.datamap_mut().info_mut().title_mut() = title;
    }

    /// Name of survey
    pub fn name(&self) -> Option<String> {
        self.datamap().name()
    }

    /// Sets name of a survey
    #[inline]
    pub fn set_name(&mut self, name: Option<String>) {
        *self.datamap_mut().info_mut().name_mut() = name;
    }

    /// triple-s datamap
    pub fn datamap(&self) -> &Datamap {
        &self.datamap
    }

    /// A read-write view of triple-s datamap
    pub fn datamap_mut(&mut self) -> &mut Datamap {
        &mut self.datamap
    }

    /// Read access to shared survey data.
    pub fn data(&self) -> RwLockReadGuard<'_, SurveyData> {
        self.data.read()
    }

    /// Write access to respondent data. Replacing it updates existing question views.
    pub fn data_mut(&self) -> RwLockWriteGuard<'_, SurveyData> {
        self.data.write()
    }

    /// List of question names in the current instance
    pub fn question_names(&self) -> Vec<String> {
        self.datamap().question_names()
    }

    /// Configuration options
    pub fn config(&self) -> RwLockReadGuard<'_, SurveyOptions> {
        self.config.read()
    }

    /// Mutable view of survey configuration options
    pub fn config_mut(&mut self) -> RwLockWriteGuard<'_, SurveyOptions> {
        self.config.write()
    }

    /// Creates a dataframe collating response labels for all questions in the survey.
    /// Very handy for look up some values.
    ///
    /// This method will return dataframe with the following columns:
    ///
    /// |column|dtype|
    /// |--|--|
    /// |`question_name`|String|
    /// |`response`|String|
    /// |`label`|String|
    ///
    /// Note:
    ///
    /// Polars only supports dataframe with *single data type*, hence all response values needs to
    /// be converted to string beforehand. You can convert them back to the correct type later.
    ///
    /// An example output may look like:
    ///
    /// ```bash
    /// shape: (26, 3)
    /// ┌───────────────┬──────────┬─────────────────────────────────┐
    /// │ question_name ┆ response ┆ label                           │
    /// │ ---           ┆ ---      ┆ ---                             │
    /// │ str           ┆ str      ┆ str                             │
    /// ╞═══════════════╪══════════╪═════════════════════════════════╡
    /// │ Q2            ┆ 0        ┆ No, this is the first visit     │
    /// │ Q2            ┆ 1        ┆ I have visited before within t… │
    /// │ Q2            ┆ 2        ┆ I visited before that           │
    /// │ Q3            ┆ 1        ┆ Sherwood � Forest               │
    /// │ Q3            ┆ 2        ┆ Nottingham � Castle             │
    /// │ …             ┆ …        ┆ …                               │
    /// │ Q6            ┆ 500.0    ┆ 500 or more                     │
    /// │ Q6            ┆ 999.0    ┆ Not stated                      │
    /// │ Q8            ┆ "A"      ┆ Within 3 months                 │
    /// │ Q8            ┆ "B"      ┆ Between 3 months and 1 year     │
    /// │ Q8            ┆ "C"      ┆ More than 1 years time          │
    /// └───────────────┴──────────┴─────────────────────────────────┘
    /// ```
    pub fn response_label_map(&self) -> PolarsResult<DataFrame> {
        let maps = self.datamap.response_label_map();

        let capacity: usize = maps.values().map(|m| m.len()).sum();
        let mut names = StringChunkedBuilder::new("question_name".into(), capacity);
        let mut responses = StringChunkedBuilder::new("response".into(), capacity);
        let mut labels = StringChunkedBuilder::new("label".into(), capacity);

        for (name, map) in maps {
            for (code, value) in map {
                names.append_value(name.as_str());
                responses.append_value(code.to_string());

                let guard = value.read();
                labels.append_value(guard.as_str());
            }
        }

        DataFrame::new_infer_height(vec![
            names.finish().into_column(),
            responses.finish().into_column(),
            labels.finish().into_column(),
        ])
    }
}

/// Method calls
///
/// ---
impl Survey {
    /// Create new `Survey` instance
    pub fn new(datamap: Datamap, data: SurveyData) -> Self {
        Self {
            datamap,
            data: lock_it_up(data),
            config: lock_it_up(SurveyOptions::default()),
        }
    }

    /// Creates a new empty `Survey` instance
    pub fn empty() -> Self {
        let datamap = Datamap::empty();
        let data = SurveyData::empty();
        Self::new(datamap, data)
    }

    /// List out number of responses loaded
    ///
    /// This function may fail during polars' [`LazyFrame::collect()`] function call
    pub fn num_response(&self) -> PolarsResult<usize> {
        if self.has_data() {
            self.data().num_response()
        } else {
            Ok(0)
        }
    }

    /// Check if data loaded or not
    pub fn has_data(&self) -> bool {
        self.data().has_data()
    }

    /// Gets a shared [`QuestionData`] view, or `None` if the question name doesn't exist.
    pub fn get_question_data(&self, name: &str) -> Option<QuestionData> {
        let question = self.datamap.get(name)?.clone();
        Some(QuestionData::new(
            Arc::clone(&self.data),
            question,
            self.config.read().clone(),
        ))
    }

    /// Get data for specific row
    ///
    /// Args:
    ///
    /// `response_label`:
    ///     - Set `true` to display response label text for single and multi-choice questions (`["London", "London", "Scotland", ...]`)
    ///     - Set `false` to show values instead (`[1, 1, 2, ...]`).
    pub fn get_row(&self, row_index: usize, response_label: bool) -> PolarsResult<DataFrame> {
        let data = self.data().get_row(row_index)?;
        if !response_label {
            Ok(data)
        } else {
            let label_lookup = self.datamap().response_label_map();
            let query = Self::label_response_values(data.schema().clone(), &label_lookup)?;
            data.lazy().select(query).collect()
        }
    }

    /// Creates a polars expr for replacing categorical answers with response label texts
    fn label_response_values(
        schema: SchemaRef,
        label_lookup: &HashMap<PlSmallStr, HashMap<AnyValue<'static>, ArcLock<String>>>,
    ) -> PolarsResult<Vec<Expr>> {
        schema
            .iter()
            .map(|(colname, dtype)| {
                let Some(value_map) = label_lookup.get(colname) else {
                    return Ok(col(colname.clone()));
                };

                let (old, new): (Vec<AnyValue<'static>>, Vec<String>) = value_map
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone().read().to_string()))
                    .unzip();

                let old = Series::from_any_values("key".into(), &old, true)?;
                let new = Series::new("value".into(), new);

                let expr = match dtype {
                    DataType::List(_) => {
                        col(colname.clone()).list().eval(element().replace_strict(
                            lit(old),
                            lit(new),
                            Some(element()),
                            None::<DataType>,
                        ))
                    }
                    _ => col(colname.clone()).replace_strict(
                        lit(old),
                        lit(new),
                        Some(col(colname.clone())),
                        None::<DataType>,
                    ),
                };

                Ok(expr)
            })
            .collect()
    }

    /// Export survey raw data to polars dataframe
    ///
    /// `response_label`:
    ///     Set `true` to display response label text for single and multi-choice questions (`["London", "London", "Scotland", ...]`), set `false` to show values instead (`[1, 1, 2, ...]`).
    pub fn to_polars(&self, response_label: bool) -> PolarsResult<DataFrame> {
        if response_label {
            let mut lf = self.data().lazy()?.clone();

            let label_lookup = self.datamap().response_label_map();
            let query = Self::label_response_values(lf.collect_schema()?, &label_lookup)?;
            lf.select(query).collect()
        } else {
            self.data().to_polars()
        }
    }
}

/// Creating quick analysis
///
/// ---
impl Survey {
    /// Gets a [topline table](https://en.wikipedia.org/wiki/Contingency_table) for a given question (`row`) with another question (`column`) as the header.
    ///
    /// Uses [`AnalysisOptions`] to select the row question, column groups, and optional totals.
    ///
    /// An example output looks like:
    ///
    /// ## Example: looking at respondent age by their AI usage:
    ///
    /// ```ignore
    /// use triple_s::{read_triple_s, AnalysisOptions, LabelOptions};
    ///
    /// let survey = read_triple_s("path/to/survey.sss", "path/to/datamap.asc")?;
    /// let result = survey.corstab(
    ///     AnalysisOptions {
    ///         question_name: "Age",
    ///         columns: &["AIFreq_1", ],
    ///         show_row_total: true,
    ///         show_column_total: true
    ///     },
    ///     LabelOptions {
    ///         include_code_value: false,
    ///         include_question_name: false,
    ///         ..Default::default()
    ///     }
    /// )?;
    ///
    /// println!("{:?}", result);
    /// ```
    ///
    /// This would in term gives you an output that looks like this:
    ///
    /// ```text
    /// ┌────────────────────┬─────────────┬──────────────────┬─────────────────────────────────┬───────────────┬───────────────────┐
    /// │ Age                ┆ Age - Total ┆ AIFreq_1 - Daily ┆ AIFreq_1 - Monthly or Infreque… ┆ AIFreq_1 - NA ┆ AIFreq_1 - Weekly │
    /// │ ---                ┆ ---         ┆ ---              ┆ ---                             ┆ ---           ┆ ---               │
    /// │ str                ┆ u32         ┆ u32              ┆ u32                             ┆ u32           ┆ u32               │
    /// ╞════════════════════╪═════════════╪══════════════════╪═════════════════════════════════╪═══════════════╪═══════════════════╡
    /// │ Null               ┆ 30903       ┆ 8403             ┆ 712                             ┆ 19472         ┆ 2316              │
    /// │ Under 18 years old ┆ 969         ┆ 62               ┆ 33                              ┆ 830           ┆ 44                │
    /// │ 18-24 years old    ┆ 4944        ┆ 809              ┆ 106                             ┆ 3760          ┆ 269               │
    /// │ 25-34 years old    ┆ 9267        ┆ 2489             ┆ 165                             ┆ 5956          ┆ 657               │
    /// │ 35-44 years old    ┆ 8522        ┆ 2809             ┆ 195                             ┆ 4860          ┆ 658               │
    /// │ 45-54 years old    ┆ 4198        ┆ 1437             ┆ 112                             ┆ 2256          ┆ 393               │
    /// │ 55-64 years old    ┆ 1829        ┆ 581              ┆ 67                              ┆ 980           ┆ 201               │
    /// │ 65 years or older  ┆ 822         ┆ 178              ┆ 29                              ┆ 534           ┆ 81                │
    /// │ Prefer not to say  ┆ 352         ┆ 38               ┆ 5                               ┆ 296           ┆ 13                │
    /// └────────────────────┴─────────────┴──────────────────┴─────────────────────────────────┴───────────────┴───────────────────┘
    /// ```
    pub fn crosstab(
        &self, 
        args: AnalysisOptions<'_>, 
        label_options: LabelOptions<'_>
    ) -> PolarsResult<DataFrame> {
        // prep
        let question_name = args.question_name;
        let original = self.data().lazy()?.clone();

        // get crosstab for columns
        let mut headers = args.columns.into_iter();
        let first = match headers.next() {
            None => DataFrame::empty().lazy(),
            Some(&first) => {
                let tab =
                    operation::create_crosstab(&original, question_name, first, args.show_column_total)?;
                self.label_output_column(tab, first)?
            }
        };
        let mut output = headers.try_fold(first, |left, &colname| {
            let right =
                operation::create_crosstab(&original, question_name, colname, args.show_column_total)?;
            let right = self.label_output_column(right, colname)?;
            Ok::<LazyFrame, PolarsError>(left.join(
                right,
                [col(question_name)],
                [col(question_name)],
                JoinArgs {
                    how: JoinType::Full,
                    validation: JoinValidation::OneToOne,
                    nulls_equal: true,
                    coalesce: JoinCoalesce::CoalesceColumns,
                    maintain_order: MaintainOrderJoin::Left,
                    suffix: None,
                    slice: None,
                    build_side: None,
                },
            ))
        })?;

        // row total
        if args.show_row_total {
            output = operation::frequency_counts(
                original.clone(),
                question_name,
                Some(&format!("{question_name} - Total")),
                args.show_column_total,
            )?
            .join(
                output,
                [col(question_name)],
                [col(question_name)],
                JoinArgs {
                    how: JoinType::Full,
                    validation: JoinValidation::OneToOne,
                    nulls_equal: true,
                    coalesce: JoinCoalesce::CoalesceColumns,
                    maintain_order: MaintainOrderJoin::Left,
                    suffix: None,
                    slice: None,
                    build_side: None,
                },
            );
        };

        // take a record of column names (to be used later)

        let summary_columns = output
            .collect_schema()?;

        let summary_columns = summary_columns
            .iter_names()
            .map(|colname| col(colname.as_str()));

        // Update response labels
        let response_label_map = self.datamap()
            .get(question_name)
            .ok_or(QuestionNotFound::new(question_name))?
            .response_label_map();
        let response_label_replace = label_options.match_response_label(
            question_name, 
            response_label_map
        )?;
        let output = output.with_column(response_label_replace);

        // Final formatting (column rename, add question name etc.,)
        let format_exprs = label_options
            .into_expression(&question_name)
            .into_iter()
            .chain(summary_columns)
            .collect::<Vec<Expr>>();

        // sort order & output
        output.select(format_exprs)
            .with_simplify_expr(true)
            .with_comm_subexpr_elim(true)
            .collect()
    }

    #[inline(always)]
    fn label_output_column(
        &self,
        result: LazyFrame,
        question_name: &str,
    ) -> PolarsResult<LazyFrame> {
        let lookup = self
            .datamap()
            .get(question_name)
            .ok_or(QuestionNotFound::new(question_name))?
            .response_label_map();

        let (old, new): (Vec<String>, Vec<String>) = lookup
            .iter()
            .map(|(id, label)| {
                (
                    id.to_string(),
                    format!("{} - {}", question_name, label.read().as_str()),
                )
            })
            .unzip();

        Ok(result.rename(old, new, false))
    }

    // #[inline(always)]
    // fn label_output_row(
    //     &self,
    //     result: LazyFrame,
    //     question_name: &str,
    // ) -> PolarsResult<LazyFrame> {
    //     let (row_old, row_new): (Vec<AnyValue<'static>>, Vec<String>) = self
    //         .datamap()
    //         .get(question_name)
    //         .ok_or(QuestionNotFound::new(question_name))?
    //         .response_label_map()
    //         .iter()
    //         .map(|(id, label)| (id.clone(), label.read().to_string()))
    //         .unzip();
    //     let row_old = Series::from_any_values("old".into(), &row_old, true)?;
    //     let row_new = Series::new("new".into(), row_new);

    //     Ok(result
    //         .with_columns([col(question_name)
    //             .replace_strict(
    //                 lit(row_old.clone()),
    //                 lit(row_new),
    //                 Some(col(question_name).cast(DataType::String)),
    //                 Some(DataType::String),
    //             )
    //             .fill_null(lit("Null"))
    //             .alias("label")])
    //         .select([
    //             col("label").alias(question_name),
    //             all().exclude_cols([question_name, "label"]).as_expr(),
    //         ]))
    // }
}

/// Pyo3 implementations
///
/// ---
#[cfg(feature = "pyo3")]
#[pymethods]
impl Survey {
    /// Creates new `Survey` instance
    ///
    /// Args:
    ///     datamap (Datamap): Survey definition used to interpret the respondent data.
    ///
    /// Keyword Args:
    ///     data (SurveyData | None): Respondent data. None creates a survey without loaded data. Defaults to None.
    ///
    /// Returns:
    ///     Survey: The constructed Survey instance.
    #[pyo3(signature=(datamap: "Datamap", data: "SurveyData"=None))]
    #[new]
    fn py_new(datamap: Datamap, data: Option<SurveyData>) -> Self {
        let data = data.unwrap_or_default();
        Self::new(datamap, data)
    }

    fn __repr__(&self) -> PyResult<String> {
        let mut content = Vec::new();

        // name of the survey
        let name = self.py_get_name().unwrap_or("None".to_string());
        content.push(("name", format!("{:?}", name)));

        // title of the survey
        let title = self.py_get_title().unwrap_or("None".to_string());
        content.push(("title", format!("{:?}", title)));

        // list of survey questions
        let data = self.data();
        let mut question_repr = if data.has_data() {
            data.py_repr_content().map_err(PyPolarsErr::Polars)?
        } else {
            let mut q_repr = self.datamap().py_question_repr();
            q_repr.push(("num_response", "0".to_string()));
            q_repr
        };
        content.append(&mut question_repr);

        // SurveyOptions: dataframe library
        let dataframe_format = self.config().py_dataframe_format.to_string();
        content.push(("dataframe_format", format!("{:?}", dataframe_format)));

        // SurveyOptions: return label or value
        let response_display = self.config().response_display.to_string();
        content.push(("response_display", format!("{:?}", response_display)));

        // formatting
        let value = content
            .into_iter()
            .map(|(fieldname, value)| format!("{}: {}", fieldname, value))
            .collect::<Vec<String>>()
            .join("\n  ");
        Ok(format!("Survey({{\n  {}\n}})", value))
    }

    fn __getitem__<'py>(
        &self,
        py: Python<'py>,
        index: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Read individual row
        if let Ok(index) = index.extract::<isize>() {
            let height = self.num_response().map_err(PyPolarsErr::Polars)?;
            let row_index = normalize_py_index(index, height)
                .ok_or(exceptions::PyIndexError::new_err(index))?;

            let show_resp_label = self.config().show_response_label();

            // let row = py.detach(|| {
            let row = self
                .get_row(row_index, show_resp_label)
                .map_err(PyPolarsErr::Polars)?;
            // })?;

            PyDataFrame(row)
                .into_bound_py_any(py)?
                .call_method("to_dicts", (), None)? // convert to list of dictionaries
                .call_method("__getitem__", (0,).into_pyobject(py)?, None) // take the first row

        // get data for specific question
        } else if let Ok(question_name) = index.extract::<&str>() {
            let view = self
                .get_question_data(question_name)
                .ok_or_else(|| exceptions::PyKeyError::new_err(question_name.to_owned()))?;
            Ok(Py::new(py, view)?.into_bound(py).into_any())

        // Error
        } else {
            let type_name = get_type_name(&index)?;
            Err(exceptions::PyTypeError::new_err(type_name))
        }
    }

    /// Iterate each response row by row, same as `iter_rows()`
    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<RowIterator>> {
        self.iter_rows(py)
    }

    /// Iter each individual respondant data
    ///
    /// Returns:
    ///     Iterator[dict[str, object]]: An iterator over respondent rows.
    ///     Captures the current data and display settings, buffering a small
    ///     number of rows locally. New iterators use subsequent changes.
    fn iter_rows(&self, py: Python<'_>) -> PyResult<Py<RowIterator>> {
        let source = self.data().clone();
        let mut lazy = source.lazy()?.clone();

        let label_query = if self.config().show_response_label() {
            let label_lookup = self.datamap().response_label_map();
            let schema = lazy.collect_schema().map_err(PyPolarsErr::Polars)?;
            Self::label_response_values(schema, &label_lookup).map_err(PyPolarsErr::Polars)?
        } else {
            self.question_names().into_iter().map(col).collect()
        };

        let width = label_query.len();
        let transformed = lazy
            .select(label_query)
            .with_check_order(true)
            .with_simplify_expr(true)
            .with_comm_subexpr_elim(true)
            .with_comm_subplan_elim(true);
        let iterator = RowIterator::new(source, transformed, width);
        Py::new(py, iterator)
    }

    /// Iterate over the survey questions and their response data.
    ///
    /// Returns:
    ///     Iterator[QuestionData]: An iterator over question data in survey order.
    fn iter_questions(&self, py: Python<'_>) -> PyResult<Py<QuestionDataIterator>> {
        let iterator = QuestionDataIterator {
            survey: Arc::new(self.clone()),
            index: 0,
        };

        Py::new(py, iterator)
    }

    /// (optional) Name of the survey
    #[getter(name)]
    fn py_get_name(&self) -> Option<String> {
        self.datamap().info().name().clone()
    }

    #[setter(name)]
    fn py_set_name(&mut self, name: Option<String>) -> PyResult<()> {
        self.set_name(name);
        Ok(())
    }

    /// (optional) Title of the survey
    #[getter(title)]
    fn py_get_title(&self) -> Option<String> {
        self.datamap().info().title().clone()
    }

    #[setter(title)]
    fn py_set_title(&mut self, title: Option<String>) -> PyResult<()> {
        self.set_title(title);
        Ok(())
    }

    /// Datamap
    #[getter(datamap)]
    fn py_get_datamap(&self) -> Datamap {
        self.datamap.clone()
    }

    /// Survey data
    #[getter(data)]
    fn py_get_data(&self) -> SurveyData {
        self.data().clone()
    }

    /// Replace respondent data while keeping existing question views attached.
    #[setter(data)]
    fn py_set_data(&self, data: SurveyData) {
        *self.data_mut() = data;
    }

    /// Survey metadata, shared with `datamap.info`.
    #[getter(info)]
    fn py_get_info(&self) -> SurveyInfo {
        self.datamap.info().clone()
    }

    /// Default DataFrame library used when returning results to Python.
    /// Supported values are `"pandas"` and `"polars"`, default `"polars"`
    ///
    /// Note:
    ///
    ///     All DataFrame operations are performed internally using Rust Polars,
    ///     regardless of the selected output format.
    ///
    ///     When `"pandas"` is selected, the resulting Polars DataFrame is converted
    ///     to pandas using `to_pandas()`. This may take a bit longer but shouldn't be too much, since majorities of the
    ///     computation-heavy stuff are already handled on the Rust side.
    #[getter(dataframe_format)]
    fn py_get_dataframe_format(&self) -> String {
        self.config().py_dataframe_format.to_string()
    }

    #[setter(dataframe_format)]
    fn py_set_dataframe_format(&mut self, value: Bound<'_, PyAny>) -> PyResult<()> {
        self.config_mut().py_dataframe_format = DataFrameFormat::from_py_any(value)?;
        Ok(())
    }

    /// Controls whether choice responses are represented by their values or labels; can be 'value' or 'label'.
    ///
    /// Default 'value'
    #[getter(response_display)]
    fn py_get_response_display(&self) -> String {
        self.config().response_display.to_string()
    }

    #[setter(response_display)]
    fn py_set_response_display(&mut self, value: Bound<'_, PyAny>) -> PyResult<()> {
        self.config_mut().response_display = ResponseDisplay::from_py_any(value)?;
        Ok(())
    }

    /// Total number of responses
    #[getter(num_response)]
    fn py_get_num_response(&self) -> PyResult<usize> {
        self.num_response()
            .map_err(|err| PyPolarsErr::Polars(err).into())
    }

    /// List of question names loaded to the survey
    #[getter(question_names)]
    fn py_quesiton_names(&self) -> Vec<String> {
        self.datamap().question_names()
    }

    /// Creates a dataframe collating response labels for all questions in the survey.
    /// Very handy for look up some values.
    ///
    /// This method will return dataframe with the following columns:
    ///
    /// .. table::
    ///     :align: left
    ///
    ///     =================  ======
    ///     column             dtype
    ///     =================  ======
    ///     ``question_name``  String
    ///     ``response``       String
    ///     ``label``          String
    ///     =================  ======
    ///
    /// Note:
    ///
    ///     Polars only supports dataframe with *single data type*, hence all response values needs to
    ///     be converted to string beforehand. You can convert them back to the correct type later.
    ///
    ///
    /// ```text
    /// shape: (26, 3)
    /// ┌───────────────┬──────────┬─────────────────────────────────┐
    /// │ question_name ┆ response ┆ label                           │
    /// │ ---           ┆ ---      ┆ ---                             │
    /// │ str           ┆ str      ┆ str                             │
    /// ╞═══════════════╪══════════╪═════════════════════════════════╡
    /// │ Q2            ┆ 0        ┆ No, this is the first visit     │
    /// │ Q2            ┆ 1        ┆ I have visited before within t… │
    /// │ Q2            ┆ 2        ┆ I visited before that           │
    /// │ Q3            ┆ 1        ┆ Sherwood � Forest               │
    /// │ Q3            ┆ 2        ┆ Nottingham � Castle             │
    /// │ …             ┆ …        ┆ …                               │
    /// │ Q6            ┆ 500.0    ┆ 500 or more                     │
    /// │ Q6            ┆ 999.0    ┆ Not stated                      │
    /// │ Q8            ┆ "A"      ┆ Within 3 months                 │
    /// │ Q8            ┆ "B"      ┆ Between 3 months and 1 year     │
    /// │ Q8            ┆ "C"      ┆ More than 1 years time          │
    /// └───────────────┴──────────┴─────────────────────────────────┘
    /// ```
    ///
    /// Keyword Args:
    ///     format ('polars' | 'pandas' | DataFrameFormat, *optional*): Select what kind of dataframe to output:
    ///         'polars' | DataFrameFormat.Polars: polars.DataFrame
    ///         'pandas' | DataFrameFormat.Pandas: pandas.DataFrame
    ///
    ///         Default:
    ///             consistent with the survey.dataframe_format
    /// Returns:
    ///     polars.DataFrame | pandas.DataFrame: Value counts with response labels.
    #[pyo3(name="response_label_map", signature=(format: "'polars'|'pandas'|None"=None))]
    fn py_get_response_label_map<'py>(
        &self,
        py: Python<'py>,
        format: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let polars_frame = self
            .response_label_map()
            .map_err(|err| PyPolarsErr::Polars(err))?;

        let format = format
            .map(|value| DataFrameFormat::from_py_any(value))
            .transpose()?
            .unwrap_or(self.config.read().py_dataframe_format.clone());

        match format {
            DataFrameFormat::Polars => PyDataFrame(polars_frame).into_bound_py_any(py),
            DataFrameFormat::Pandas => polars_to_pandas(polars_frame, py),
        }
    }

    /// Check if data loaded or not
    ///
    /// Returns:
    ///     bool: Whether respondent data is loaded.
    #[pyo3(name = "has_data")]
    fn py_has_data(&self) -> bool {
        self.has_data()
    }

    /// Export survey raw data to polars dataframe
    ///
    /// Kwargs:
    ///     response_label (bool, *optional*):
    ///         Set `True` to display response label text for single and multi-choice questions (`["London", "London", "Scotland", ...]`),
    ///         Set `False` to show values instead (`[1, 1, 2, ...]`).
    ///         Default to be consistent with `Survey.response_label` flag
    #[pyo3(name = "to_polars", signature=(response_label: "bool"=None))]
    fn py_to_polars(&self, response_label: Option<bool>) -> PyResult<PyDataFrame> {
        let response_label = response_label.unwrap_or(self.config().show_response_label());

        self.to_polars(response_label)
            .map(PyDataFrame)
            .map_err(|err| PyPolarsErr::Polars(err).into())
    }

    /// Export survey raw data to pandas dataframe
    ///
    /// Kwargs:
    ///     response_label (bool, *optional*):
    ///         Set `True` to display response label text for single and multi-choice questions (`["London", "London", "Scotland", ...]`),
    ///         Set `False` to show values instead (`[1, 1, 2, ...]`).
    ///         Default to be consistent with `Survey.response_label` flag
    #[pyo3(name = "to_pandas", signature=(response_label: "bool"=None))]
    fn py_to_pandas<'py>(
        &self,
        py: Python<'py>,
        response_label: Option<bool>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let response_label = response_label.unwrap_or(self.config().show_response_label());
        let df = self
            .to_polars(response_label)
            .map_err(PyPolarsErr::Polars)?;
        polars_to_pandas(df, py)
    }

    /// Create a `topline table <https://en.wikipedia.org/wiki/Contingency_table>`_ for a question in the survey.
    ///
    /// Args:
    ///     row (str): name of the question to analyse
    ///     column (str): name of the question for topline
    ///
    /// Kwrags:
    ///     show_row_total (bool *optional*):
    ///         Toogle on to show total sizes for the `row`
    ///         Default True
    ///     
    ///     show_column_total (bool *optional*):
    ///         Toggle on to show total size for each column.
    ///         Defaul tRUE
    ///     
    ///     format ('pandas' | 'polars' | None, *optional*):
    ///         Which dataframe library this function should return,
    ///         default to be consistent with the `dataframe_format` attribute of the current instance (`Survey.dataframe_format`).
    ///
    /// Examples:
    ///
    ///     Compare AI usage by respondent age
    ///     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    ///
    ///     Using stack overflow 2026 survey data as an example, suppose you want to look at the IDE people admire by each country..
    ///
    ///     We starts with reading the data:
    ///
    ///     .. code-block:: python
    ///
    ///         >>> from triple_s import read_triple_s
    ///         >>> survey = read_triple_s("stackoverflow_2026.sss", "stackoverflow_2026.asc")
    ///
    ///
    ///     Then use `Survey.crosstab()` method to get the data:
    ///
    ///     .. code-block:: python
    ///
    ///         >>> survey.crosstab("Age", ["AIFreq_1"], show_row_total=True, show_column_total=True)
    ///
    ///     This would give you a table that looks like:
    ///     
    ///     ```text
    ///     ┌────────────────────┬─────────────┬──────────────────┬─────────────────────────────────┬───────────────┬───────────────────┐
    ///     │ Age                ┆ Age - Total ┆ AIFreq_1 - Daily ┆ AIFreq_1 - Monthly or Infreque… ┆ AIFreq_1 - NA ┆ AIFreq_1 - Weekly │
    ///     │ ---                ┆ ---         ┆ ---              ┆ ---                             ┆ ---           ┆ ---               │
    ///     │ str                ┆ u32         ┆ u32              ┆ u32                             ┆ u32           ┆ u32               │
    ///     ╞════════════════════╪═════════════╪══════════════════╪═════════════════════════════════╪═══════════════╪═══════════════════╡
    ///     │ Null               ┆ 30903       ┆ 8403             ┆ 712                             ┆ 19472         ┆ 2316              │
    ///     │ Under 18 years old ┆ 969         ┆ 62               ┆ 33                              ┆ 830           ┆ 44                │
    ///     │ 18-24 years old    ┆ 4944        ┆ 809              ┆ 106                             ┆ 3760          ┆ 269               │
    ///     │ 25-34 years old    ┆ 9267        ┆ 2489             ┆ 165                             ┆ 5956          ┆ 657               │
    ///     │ 35-44 years old    ┆ 8522        ┆ 2809             ┆ 195                             ┆ 4860          ┆ 658               │
    ///     │ 45-54 years old    ┆ 4198        ┆ 1437             ┆ 112                             ┆ 2256          ┆ 393               │
    ///     │ 55-64 years old    ┆ 1829        ┆ 581              ┆ 67                              ┆ 980           ┆ 201               │
    ///     │ 65 years or older  ┆ 822         ┆ 178              ┆ 29                              ┆ 534           ┆ 81                │
    ///     │ Prefer not to say  ┆ 352         ┆ 38               ┆ 5                               ┆ 296           ┆ 13                │
    ///     └────────────────────┴─────────────┴──────────────────┴─────────────────────────────────┴───────────────┴───────────────────┘
    ///     ```
    #[pyo3(
        name = "crosstab",
        signature=(
            row: "str", 
            column: "list[str]"=None,
            show_row_total: "bool"=None,
            show_column_total: "bool"=None,
            format: "'polars'|'pandas'|None"=None
        )
    )]
    fn py_crosstab<'py>(
        &self,
        py: Python<'py>,
        row: &str,
        column: Option<Vec<String>>,
        show_row_total: Option<bool>,
        show_column_total: Option<bool>,
        format: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let columns: Vec<&str> = column
            .as_deref()
            .unwrap_or_default()
            .iter()
            .map(String::as_str)
            .collect();

        let arguments = AnalysisOptions {
            question_name: row,
            columns: &columns,
            show_row_total: show_row_total.unwrap_or(true),
            show_column_total: show_column_total.unwrap_or(true),
        };
        let frame = self.crosstab(arguments, LabelOptions::default())
            .map_err(PyPolarsErr::Polars)?;

        match format
            .map(|value| DataFrameFormat::from_py_any(value))
            .transpose()?
            .unwrap_or(self.config().py_dataframe_format.clone())
        {
            DataFrameFormat::Polars => PyDataFrame(frame).into_bound_py_any(py),
            DataFrameFormat::Pandas => polars_to_pandas(frame, py),
        }
    }
}

// ---------- Helpers -----------
/// Iterates each ``QuestionData``` in a survey
#[cfg(feature = "pyo3")]
#[pyclass]
pub(crate) struct QuestionDataIterator {
    survey: Arc<Survey>,
    index: usize,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl QuestionDataIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<QuestionData>> {
        let question = {
            let questions = self.survey.datamap.record().content();
            let Some(question) = questions.get(self.index) else {
                return Ok(None);
            };
            question.read().clone()
        };
        let instance = QuestionData::new(
            Arc::clone(&self.survey.data),
            question,
            self.survey.config().clone(),
        );

        self.index += 1;
        Ok(Some(instance))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::read_triple_s;

    const PATH_DATAMAP_V2_0: &str = "tests/sss-examples/sss-2.0/example1.sss";
    const PATH_DATAFILE_V2_0: &str = "tests/sss-examples/sss-2.0/example1.asc";

    fn get_survey_struct() -> Survey {
        read_triple_s(PATH_DATAMAP_V2_0, Some(PATH_DATAFILE_V2_0)).unwrap()
    }

    #[test]
    fn test_response_label_dataframe() {
        let survey = get_survey_struct();
        let _ = survey.response_label_map().unwrap();
    }
}
