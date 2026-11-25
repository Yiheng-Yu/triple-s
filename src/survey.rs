#[cfg(not(feature="datamap-only"))]
use std::collections::HashMap;
#[cfg(feature = "pyo3")]
use std::sync::Arc;
#[cfg(not(feature="datamap-only"))]
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
#[cfg(not(feature="datamap-only"))]
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::{IntoPyObjectExt, exceptions, prelude::*};
#[cfg(feature = "pyo3")]
use pyo3_polars::{PyDataFrame, error::PyPolarsErr};

#[cfg(not(feature = "datamap-only"))]
use rayon::prelude::*;

#[cfg(not(feature="datamap-only"))]
use crate::{
    data::{QuestionData, SurveyData, operation},
    options::{AnalysisOptions, SkipQType, LabelOptions},
    error::QuestionNotFound,
};

#[cfg(feature="datamap-only")]
use anyhow::Result;

use crate::{
    options::SurveyOptions,
    datamap::Datamap,
    utils::{ArcLock, lock_it_up},
};
#[cfg(feature = "pyo3")]
use crate::{
    SurveyInfo,
    data::row_iterator::RowIterator,
    options::{DataFrameFormat, ResponseDisplay},
    prelude::*,
    utils::{extract_to_vec_str, get_type_name, normalize_py_index, polars_to_pandas},
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

Keyword Arguments:
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

    #[cfg(not(feature="datamap-only"))]
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
    #[cfg(not(feature="datamap-only"))]
    pub fn data(&self) -> RwLockReadGuard<'_, SurveyData> {
        self.data.read()
    }

    /// Write access to respondent data. Replacing it updates existing question views.
    #[cfg(not(feature="datamap-only"))]
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
    #[cfg(not(feature="datamap-only"))]
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
    /// Create new `Survey` instance from datamap
    #[cfg(feature="datamap-only")]
    pub fn new(datamap: Datamap) -> Result<Self> {
        Ok(Self { 
            datamap,
            config: lock_it_up(SurveyOptions::default())
        })
    }

    /// Create new `Survey` instance
    #[cfg(not(feature="datamap-only"))]
    pub fn new(datamap: Datamap, data: SurveyData) -> PolarsResult<Self> {
        if !data.has_data() {
            Ok(Self {
                datamap,
                data: lock_it_up(data),
                config: lock_it_up(SurveyOptions::default()),
            })
        } else {
            let columns = datamap
                .schema()
                .into_iter()
                .map(|(colname, dtype)| col(colname).cast(dtype))
                .collect::<Vec<Expr>>();

            let lazyframe = data.lazy().unwrap().clone().with_columns(columns);
            let data_new = SurveyData::from_lazyframe(lazyframe)?;

            Ok(Self {
                datamap,
                data: lock_it_up(data_new),
                config: lock_it_up(SurveyOptions::default()),
            })
        }
    }

    /// Creates a new empty `Survey` instance
    #[cfg(feature="datamap-only")]
    pub fn empty() -> Self {
        Self::new(Datamap::empty()).unwrap()
    }

    /// Creates a new empty `Survey` instance
    #[cfg(not(feature="datamap-only"))]
    pub fn empty() -> Self {
        let datamap = Datamap::empty();
        let data = SurveyData::empty();
        Self::new(datamap, data).unwrap()
    }

    /// List out number of responses loaded
    ///
    /// This function may fail during polars' [`LazyFrame::collect()`] function call
    #[cfg(not(feature="datamap-only"))]
    pub fn num_response(&self) -> PolarsResult<usize> {
        if self.has_data() {
            self.data().num_response()
        } else {
            Ok(0)
        }
    }

    /// Check if data loaded or not
    #[cfg(not(feature="datamap-only"))]
    pub fn has_data(&self) -> bool {
        self.data().has_data()
    }

    /// Gets a shared [`QuestionData`] view, or `None` if the question name doesn't exist.
    #[cfg(not(feature="datamap-only"))]
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
    #[cfg(not(feature="datamap-only"))]
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
    #[cfg(not(feature="datamap-only"))]
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

    /// Gets survey data as a lazy frame, using the question types in the datamap.
    /// Questions with type [`QuestionType::None`](crate::QuestionType) are skipped.
    /// An error is returned when a data column has no matching question.
    #[cfg(not(feature="datamap-only"))]
    pub fn lazy(&self) -> PolarsResult<LazyFrame> {
        let dtype_lookup = self.datamap()
            .record()
            .content()
            .clone()
            .into_iter()
            .map(|question| {
                let question = question.read();
                if question.is_none_type() {
                    (question.name_small_str(), None)
                } else {
                    (question.name_small_str(), Some(question.polars_dtype()))
                }
            })
            .collect::<HashMap<PlSmallStr, Option<DataType>>>();
        
        let mut lazyframe = self.data().lazy()?.clone();
        let schema = lazyframe.collect_schema()?;
        let mut columns: Vec<Expr> = Vec::with_capacity(schema.len());
        schema.iter_names()
            .try_for_each(|colname| {
                let dtype_found = dtype_lookup
                    .get(colname.as_str())
                    .ok_or(PolarsError::ComputeError(format!(
                        "Survey data column '{}' is not defined in the datamap.",
                        colname
                    ).into()))?
                    .as_ref();

                // Ignore `QuestionType::None`
                if let Some(dtype) = dtype_found {
                    columns.push(col(colname.as_str()).cast(dtype.clone()));
                }
                Ok::<(), PolarsError>(())
            })?;

        Ok(lazyframe
            .select(columns)
            .with_simplify_expr(true)
            .with_comm_subexpr_elim(true)
            .with_comm_subplan_elim(true))
    }

    /// Export survey raw data to polars dataframe
    ///
    /// `response_label`:
    ///     Set `true` to display response label text for single and multi-choice questions (`["London", "London", "Scotland", ...]`), set `false` to show values instead (`[1, 1, 2, ...]`).
    #[cfg(not(feature="datamap-only"))]
    pub fn to_polars(&self, response_label: bool) -> PolarsResult<DataFrame> {
        if response_label {
            let mut lf = self.lazy()?;

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
#[cfg(not(feature="datamap-only"))]
impl Survey {
    /// Gets a [topline table](https://en.wikipedia.org/wiki/Contingency_table) for a given question (`row`) with another question (`column`) as the header.
    ///
    /// Uses [`AnalysisOptions`] to select the row question and column groups, and
    /// [`LabelOptions`] to configure optional totals and output labels.
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
    ///         normalise: false,
    ///     },
    ///     LabelOptions {
    ///         show_row_total: true,
    ///         show_column_total: true,
    ///         include_response_codes: false,
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
    #[allow(clippy::mutable_key_type)]
    pub fn crosstab(
        &self,
        args: AnalysisOptions<'_>,
        label_options: LabelOptions<'_>,
    ) -> PolarsResult<DataFrame> {
        // We first locate question names and columns to group by
        let all_question_names = self.question_names();
        let all_question_names = all_question_names
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>();

        let question_name =
            operation::find_column_name(&all_question_names, args.question_name)?;

        let target_columns = args
            .columns
            .into_iter()
            .map(|colname| operation::find_column_name(&all_question_names, *colname))
            .collect::<PolarsResult<Vec<&str>>>()?;

        self.crosstab_internal(
            question_name,
            &target_columns,
            args.normalise,
            label_options,
        )?
        .collect()
    }

    /// Builds lazyframe query plan for getting crosstab to be collected later
    ///
    /// This function expect all input column names to be CASE SENSITIVE
    fn crosstab_internal<'a>(
        &self,
        question_name: &'a str,
        target_columns: &'a [&'a str],
        normalise: bool,
        label_options: LabelOptions<'a>,
    ) -> PolarsResult<LazyFrame> {
        // prep
        let original = self.lazy()?;

        // Skip the rest if no columns was defined
        if target_columns.is_empty() {
            return self
                .get_question_data(question_name)
                .ok_or::<PolarsError>(QuestionNotFound::new(question_name).into())?
                .value_counts_internal(normalise, label_options);
        }

        let mut headers = target_columns.into_iter();
        // ..then creating crosstabs
        let first = match headers.next() {
            None => DataFrame::empty().lazy(),
            Some(first) => {
                let tab = operation::create_crosstab(
                    &original,
                    question_name,
                    first,
                    label_options.show_column_total,
                    normalise,
                )?;
                self.label_output_column(tab, first)?
            }
        };
        let mut output = headers.try_fold(first, |left, colname| {
            let right = operation::create_crosstab(
                &original,
                question_name,
                colname,
                label_options.show_column_total,
                normalise,
            )?;
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
        if label_options.show_row_total {
            output = operation::frequency_counts(
                original.clone(),
                question_name,
                Some(label_options.col_count.unwrap_or(LabelOptions::COUNT)),
                label_options.show_column_total,
                normalise,
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

        // Select question name, response code, response label, then counts.
        let mut columns = Vec::new();
        if label_options.include_question_name {
            columns.push(label_options.question_name_column(question_name));
        }
        if label_options.include_response_codes {
            columns.push(
                col(question_name)
                    .alias(label_options.col_code_value.unwrap_or(LabelOptions::CODE)),
            );
        }

        let response_label_map = self
            .datamap()
            .get(question_name)
            .ok_or(QuestionNotFound::new(question_name))?
            .response_label_map();
        columns.push(label_options.match_response_label(question_name, response_label_map)?);
        columns.push(all().exclude_cols([question_name]).as_expr());

        Ok(output
            .select(columns)
            .with_simplify_expr(true)
            .with_comm_subexpr_elim(true))
    }

    /// Just a quick and dirty function for column text replacement
    #[inline(always)]
    #[allow(clippy::mutable_key_type)]
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

    /// Creates a crosstab table for every selected question in the current instance,
    /// skipping questions listed in `ignore` and question type defined in `skip_type`.
    ///
    /// Returns: [(`question_name`, `corsstab`), ..]
    ///
    /// Arguments:
    ///
    /// - `columns`:
    ///     - List of question names to be used as corsstab columns.
    ///
    /// - `ignore`:
    ///     - list of question names to ignore. This overrides `skip_type`.
    ///     - i.e., you can skip certain single-choice question even if `skip_type.single` was `false`
    ///
    /// - `skip_type`:
    ///     - Ignore questions of certaion [`QuestionType`](crate::datamap::QuestionType).
    ///     - The default `skip_type`s are:
    ///
    ///     > ```text
    ///     > SkipQType {
    ///     >     text: true,
    ///     >     numeric: true,
    ///     >     datetime: true,
    ///     >     single: false,
    ///     >     multiple: false
    ///     > }
    ///     > ```
    ///
    /// - `normalise`:
    ///     - Show percentages instead of counts.
    ///
    /// - `label_options`:
    ///     - Configure how the output should be labelled. For more info please view [`LabelOptions`]'s documentation.
    pub fn quick_summary(
        &self,
        columns: Option<&[&str]>,
        ignore: Option<&[&str]>,
        skip_type: SkipQType,
        normalise: bool,
        label_options: LabelOptions<'_>,
    ) -> PolarsResult<Vec<(String, DataFrame)>> {
        // Get name of columns to be processed, case sensitive
        let question_names = self.question_names();
        let question_names_ref = question_names
            .iter()
            .map(String::as_str)
            .collect::<Vec<&str>>();

        let columns = columns
            .map(|cols| {
                cols.iter()
                    .map(|name| operation::find_column_name(&question_names_ref, *name))
                    .collect::<PolarsResult<Vec<&str>>>()
            })
            .transpose()?
            .unwrap_or_default();

        let ignore = ignore
            .map(|cols| {
                cols.iter()
                    .map(|name| operation::find_column_name(&question_names_ref, *name))
                    .collect::<PolarsResult<Vec<&str>>>()
            })
            .transpose()?
            .unwrap_or_default();

        // Building queries
        let queries = self
            .datamap()
            .record()
            .content()
            // .iter()
            .par_iter()
            .filter_map(|question| {
                let question = question.read();
                let question_name = question.name();
                if !ignore.is_empty() & ignore.contains(&question_name.as_str()) {
                    None
                } else if skip_type.should_skip(&question) {
                    None
                } else {
                    Some(
                        self.crosstab_internal(
                            &question_name,
                            &columns,
                            normalise,
                            label_options.clone(),
                        )
                        .map(|lf| (question_name, lf)),
                    )
                }
            })
            .collect::<PolarsResult<Vec<(String, LazyFrame)>>>()?;

        // Executing query
        let progress_spinner = ProgressBar::new_spinner()
            .with_style(
                ProgressStyle::with_template("{spinner:.cyan} Parsing: {pos} tables").unwrap(),
            )
            .with_finish(indicatif::ProgressFinish::WithMessage("Done".into()));

        queries
            .into_par_iter()
            .map(|(qname, lazyframe)| {
                lazyframe
                    .collect()
                    .with_context(|| {
                        format!(
                            "When trying to get summary table for question {:?}",
                            qname.clone()
                        )
                    })
                    .map(|df| (qname, df))
            })
            .progress_with(progress_spinner)
            .collect()
    }
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
    /// Keyword Arguments:
    ///     data (SurveyData | None): Respondent data. None creates a survey without loaded data. Defaults to None.
    ///
    /// Returns:
    ///     Survey: The constructed Survey instance.
    #[pyo3(signature=(datamap: "Datamap", data: "SurveyData"=None))]
    #[new]
    fn py_new(datamap: Datamap, data: Option<SurveyData>) -> PyResult<Self> {
        let data = data.unwrap_or_default();
        Self::new(datamap, data).map_err(|err| PyPolarsErr::Polars(err).into())
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
        let mut lazy = self.lazy()
            .map_err(PyPolarsErr::Polars)?;

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
    ///     `question_name`  String
    ///     `response`       String
    ///     `label`          String
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
    /// Keyword Arguments:
    ///     output ('polars' | 'pandas' | DataFrameFormat, *optional*): Select what kind of dataframe to output:
    ///         'polars' | DataFrameFormat.Polars: polars.DataFrame
    ///         'pandas' | DataFrameFormat.Pandas: pandas.DataFrame
    ///
    ///         Default:
    ///             consistent with the survey.dataframe_format
    /// Returns:
    ///     polars.DataFrame | pandas.DataFrame: Value counts with response labels.
    /// 
    #[pyo3(name="response_label_map", signature=(
        output: "'polars'|'pandas'|None"=None
    ))]
    fn py_get_response_label_map<'py>(
        &self,
        py: Python<'py>,
        output: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let polars_frame = self
            .response_label_map()
            .map_err(|err| PyPolarsErr::Polars(err))?;

        let format = output
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
    /// Keyword Arguments:
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
    /// Keyword Arguments:
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

    /// Create a cross tab for a question in the survey.
    ///
    /// You can compare one question's answers by one or more other questions.
    /// Response labels are always shown. For multiple-choice questions, each
    /// selected response is counted separately.
    /// Set `normalise=True` to return proportions instead of counts, keeping
    /// the first row as total counts.
    ///
    /// Args:
    ///     row (str): Name of the question to show as rows, case insensitive.
    ///
    /// Keyword Arguments:
    ///     columns (str | list[str], *optional*): 
    ///         Names of the questions to show as columns, case insensitive.
    ///     
    ///     show_row_total (bool, *optional*):
    ///         Include the overall count column for the row question. Default `True`.
    ///     
    ///     show_column_total (bool, *optional*):
    ///         Include the first row of totals. Each column shows its response count,
    ///         and the overall count shows respondents with non-null row answers.
    ///         Default `True`.
    ///     
    ///     colname_stats (str, *optional*):
    ///         Name of the overall count column. Used when `show_row_total=True`.
    ///         Default `'total'`.
    ///     
    ///     colname_label (str, *optional*):
    ///         Name of the row-response label column. Default `'label'`.
    ///     
    ///     colname_code (str, *optional*):
    ///         Name of the row-response code column. Used when
    ///         `include_response_codes=True`. Default `'code'`.
    ///     
    ///     include_response_codes (bool, *optional*):
    ///         Include the original row-response values in a separate column.
    ///         The total row has a null code value. Default `False`.
    ///     
    ///     include_question_name (bool, *optional*):
    ///         Include a `question` column with the row question's name in the
    ///         first displayed row and nulls in the remaining rows. Default `False`.
    ///     
    ///     output ('polars' | 'pandas' | DataFrameFormat, *optional*):
    ///         Choose a Polars or pandas dataframe, using a string or a
    ///         `config.DataFrameFormat` member. Default `None` follows
    ///         `survey.dataframe_format` without changing that setting.
    ///     
    ///     normalise (bool, *optional*):
    ///         Return counts as relative frequencies normalised to 1.0, using
    ///         each column's total. The first row keeps the original total counts.
    ///         For example, `0.75` represents 75%. Default `False`.
    ///
    /// Returns:
    ///     polars.DataFrame | pandas.DataFrame: 
    ///         Row labels, optional question and code columns, and counts grouped by the column questions' response labels.
    ///
    /// Note:
    ///     You can pass `None` for any keyword option to use its default.
    ///     Missing answers are excluded from response counts. Empty multiple-choice
    ///     lists count towards the overall respondent total but add no selections.
    ///     Normalised results use floating-point columns. Totals are used as
    ///     denominators even when `show_column_total=False`. For multiple-choice
    ///     row questions, proportions can add up to more than 1.0.
    ///
    /// Examples:
    ///     Suppose `Q2` asks "Do you cycle to work?" and `Q4` records the region.
    ///     You can compare the answers by region and include the response codes:
    ///
    ///     .. code-block:: python
    ///
    ///         table = survey.crosstab(
    ///             "Q2", ["Q4"], include_response_codes=True,
    ///             include_question_name=True, colname_stats="answers",
    ///         )
    ///         print(table)
    ///
    ///     This gives you `question`, `code`, `label`, and `answers` columns,
    ///     followed by columns such as `Q4 - North` and `Q4 - South`.
    ///     You can hide the first totals row with `show_column_total=False`,
    ///     or request a pandas dataframe with `output="pandas"`.
    #[pyo3(
        name = "crosstab",
        signature=(
            row: "str", 
            columns: "str | list[str]"=None,
            show_row_total: "bool"=None,
            show_column_total: "bool"=None,
            colname_stats: "str" = "total",
            colname_label: "str" = "label",
            colname_code: "str" = "code",
            include_response_codes: "bool" = false,
            include_question_name: "bool" = false,
            output: "'polars'|'pandas'|DataFrameFormat"=None,
            normalise: "bool"=false,
        )
    )]
    fn py_crosstab<'py>(
        &self,
        py: Python<'py>,
        row: &str,
        columns: Option<&Bound<'py, PyAny>>,
        show_row_total: Option<bool>,
        show_column_total: Option<bool>,
        colname_stats: Option<&str>,
        colname_label: Option<&str>,
        colname_code: Option<&str>,
        include_response_codes: Option<bool>,
        include_question_name: Option<bool>,
        output: Option<Bound<'py, PyAny>>,
        normalise: Option<bool>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let column_names = columns
            .map(extract_to_vec_str)
            .transpose()?
            .unwrap_or_default();
        let columns = column_names.iter().map(String::as_str).collect::<Vec<_>>();

        let label_options = LabelOptions::new()
            .with_show_row_total(show_row_total.unwrap_or(true))
            .with_show_column_total(show_column_total.unwrap_or(true))
            .with_include_question_name(include_question_name.unwrap_or(false))
            .with_include_response_codes(include_response_codes.unwrap_or(false))
            // think this would be easier for the user and makes documentation less confusing
            // as the user can just call `ffill()` to fill nan values with question name
            // should they want to.
            .with_fill_question_name_column(false)
            .with_col_count(colname_stats)
            .with_col_label(colname_label)
            .with_col_code(colname_code);

        let arguments = AnalysisOptions {
            question_name: row,
            columns: &columns,
            normalise: normalise.unwrap_or(false),
        };

        // run corsstabs -> use py.detach() to make sure it runs with multithread
        let frame = py
            .detach(|| self.crosstab(arguments, label_options))
            .map_err(PyPolarsErr::Polars)?;

        match output
            .map(|value| DataFrameFormat::from_py_any(value))
            .transpose()?
            .unwrap_or(self.config().py_dataframe_format.clone())
        {
            DataFrameFormat::Polars => PyDataFrame(frame).into_bound_py_any(py),
            DataFrameFormat::Pandas => polars_to_pandas(frame, py),
        }
    }

    /// Creates a crosstab table for every selected question in the current instance,
    /// skipping questions listed in `ignore` and question type defined in `skip_type`.
    ///
    /// Keyword Arguments:
    ///     column (str | list[str], *optional*): Names of the questions to show as columns, case insensitive.
    ///     
    ///     ignore (str | list[str], *optional*): Names of the questions ignore, case insensitive.    
    ///
    ///     normalise (bool, *optional*):
    ///         Return counts as relative frequencies normalised to 1.0, using
    ///         each column's total. The first row keeps the original total counts.
    ///         Default `False`.
    ///     
    ///     skip_text (bool, *optional*):
    ///         Toggle on to skip `character` questions. Default `True`.
    ///     
    ///     skip_numeric (bool, *optional*):
    ///         Toggle on to skip `quantity` questions. Default `True`.
    ///
    ///     skip_datetime (bool, *optional*):
    ///         Toggle on to skip `date` and `time` questions. Default `True`.
    ///
    ///     skip_single (bool, *optional*):
    ///         Toggle on to skip `single`-choice questions. Default `False`.
    ///
    ///     skip_multiple (bool, *optional*):
    ///         Toggle on to skip `multi`-choice questions. Default `False`.
    ///
    ///     colname_stats (str, *optional*):
    ///         Name of the overall count column. Used when `show_row_total=True`.
    ///         Default `'total'`.
    ///     
    ///     colname_label (str, *optional*):
    ///         Name of the row-response label column. Default `'label'`.
    ///     
    ///     colname_code (str, *optional*):
    ///         Name of the row-response code column. Used when
    ///         `include_response_codes=True`. Default `'code'`.
    ///     
    ///     include_response_codes (bool, *optional*):
    ///         Include the original row-response values in a separate column.
    ///         The total row has a null code value. Default `False`.
    ///     
    ///     include_question_name (bool, *optional*):
    ///         Include a `question` column with the row question's name in the
    ///         first displayed row and nulls in the remaining rows. 
    ///         Default `False`.
    ///     
    ///     output ("polars" | "pandas" | DataFrameFormat, *optional*):
    ///         Choose a Polars or pandas dataframe.
    ///         Default `None`
    ///     
    ///     normalise (bool, *optional*):
    ///         Return counts as relative frequencies normalised to 1.0, using
    ///         each column's total. The first row keeps the original total counts.
    ///
    /// Returns:
    ///     [(String, polars.DataFrame | pandas.DataFrame)]:
    ///     List of (question_name, crosstab_data)
    /// 
    #[pyo3(
        name = "quick_summary",
        signature = (
            columns: "str|list[str]" = None,
            ignore: "list[str]" = None,
            normalise: "bool" = false,
            skip_text: "bool" = true,
            skip_numeric: "bool" = true,
            skip_datetime: "bool" = true,
            skip_single: "bool" = false,
            skip_multiple: "bool" = false,
            colname_stats: "str" = "total",
            colname_label: "str" = "label",
            colname_code: "str" = "code",
            include_response_codes: "bool" = false,
            include_question_name: "bool" = false,
            output: "'polars'|'pandas'|DataFrameFormat" = None,
        )
    )]
    fn py_quick_summary<'py>(
        &self,
        py: Python<'py>,
        columns: Option<&Bound<'py, PyAny>>,
        ignore: Option<&Bound<'py, PyAny>>,
        normalise: Option<bool>,
        skip_text: Option<bool>,
        skip_numeric: Option<bool>,
        skip_datetime: Option<bool>,
        skip_single: Option<bool>,
        skip_multiple: Option<bool>,
        colname_stats: Option<&str>,
        colname_label: Option<&str>,
        colname_code: Option<&str>,
        include_response_codes: Option<bool>,
        include_question_name: Option<bool>,
        output: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Vec<(String, Bound<'py, PyAny>)>> {
        // Default arguments
        let normalise = normalise.unwrap_or(false);

        let column_names = columns
            .map(extract_to_vec_str)
            .transpose()?
            .unwrap_or_default();
        let columns = column_names.iter().map(String::as_str).collect::<Vec<_>>();

        let ignored_names = ignore
            .map(extract_to_vec_str)
            .transpose()?
            .unwrap_or_default();
        let ignore = ignored_names.iter().map(String::as_str).collect::<Vec<_>>();

        let skip_types = SkipQType {
            text: skip_text.unwrap_or(true),
            numeric: skip_numeric.unwrap_or(true),
            datetime: skip_datetime.unwrap_or(true),
            single: skip_single.unwrap_or(false),
            multiple: skip_multiple.unwrap_or(false),
        };

        let label_options = LabelOptions::new()
            .with_include_question_name(include_question_name.unwrap_or(false))
            .with_include_response_codes(include_response_codes.unwrap_or(false))
            .with_fill_question_name_column(false)
            .with_col_count(colname_stats)
            .with_col_label(colname_label)
            .with_col_code(colname_code);

        let format = output
            .map(|value| DataFrameFormat::from_py_any(value))
            .transpose()?
            .unwrap_or(self.config().py_dataframe_format.clone());

        py.detach(|| {
            self.quick_summary(
                Some(&columns),
                Some(&ignore),
                skip_types,
                normalise,
                label_options,
            )
        })
        .map_err(PyPolarsErr::Polars)?
        .into_iter()
        .map(|(qname, qdata)| {
            let qdata = match &format {
                &DataFrameFormat::Polars => PyDataFrame(qdata).into_bound_py_any(py),
                &DataFrameFormat::Pandas => polars_to_pandas(qdata, py),
            }?;
            Ok((qname, qdata))
        })
        .collect()
    }
}

// ---------- Helpers -----------
/// Iterates each `QuestionData` in a survey
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

#[cfg(not(feature="datamap-only"))]
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
