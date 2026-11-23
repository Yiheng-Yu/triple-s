//! Functions for running various tabulation operations
//! Mostly used for creating summary tables for the survey data.

use polars::prelude::*;

/// Count unique values of this Series and a column "counts" with dtype IdxType
///
/// This function basically calls [`polars::Series::value_counts`] under the hood, apart from multi-choice
/// questions ([`QuestionData::question_type()`] is [`QuestionType::Multiple`]),
/// where the choice for each respondant is counted individually.
///
/// # Arguments
/// - `column_name`: name of the column to do value_counts()
/// - `name` (`impl Into<PlSmallStr>`, Optional):
///     - Name of the column, default `count`
/// - `show_total`: bool, toggle on to add a base size column
/// - `normalise`: Return relative frequencies instead of counts, keeping the base row as counts.
pub(crate) fn frequency_counts(
    frame: LazyFrame,
    column_name: &str,
    colname_agg: Option<&str>,
    show_base_size: bool,
    normalise: bool,
) -> PolarsResult<LazyFrame> {
    let colname_agg = colname_agg.unwrap_or("count");

    let mut to_count = frame.clone().select([col(column_name)]);
    let mut datatype = to_count.collect_schema()?.try_get(column_name)?.clone();

    if let DataType::List(inner) = datatype.clone() {
        to_count = to_count.explode(
            cols([column_name]),
            ExplodeOptions {
                empty_as_null: true,
                keep_nulls: true,
            },
        );
        datatype = *inner;
    };
    let to_count = to_count
        .filter(col(column_name).is_not_null())
        .group_by([col(column_name)])
        .agg([len().alias(colname_agg)])
        .with_column(col(column_name).cast(datatype.clone()))
        .sort(
            [column_name],
            SortMultipleOptions::default()
                .with_nulls_last(false)
                .with_maintain_order(false)
                .with_order_descending(false),
        );

    if show_base_size || normalise {
        let column_total = frame
            .clone()
            .select([col(column_name).count().alias(colname_agg)])
            .with_column(lit(NULL).cast(datatype).alias(column_name))
            .select([col(column_name), col(colname_agg)]);
        let result = concat(
            [column_total, to_count],
            UnionArgs {
                parallel: true,
                rechunk: true,
                to_supertypes: true,
                diagonal: true,
                strict: false,
                from_partitioned_ds: true,
                maintain_order: true,
            },
        )?;
        if normalise {
            let result = normalise_counts(result, column_name);
            Ok(if show_base_size {
                result
            } else {
                result.slice(1, IdxSize::MAX)
            })
        } else {
            Ok(result)
        }
    } else {
        Ok(to_count)
    }
}

/// This function returns a lazyframe to be collected later (basically needs to add text labels)
///
/// This function DOES NOT output a overall frequency_count of the `target` column
/// Column totals reuse the pivot categories and stay lazy until the final collection.
///
/// Inputs:
/// - target: sidebreak
/// - group_by: topbreak
/// - normalise: Return relative frequencies instead of counts, keeping the base row as counts.
pub(crate) fn create_crosstab(
    original: &LazyFrame,
    target: &str,
    group_by: &str,
    show_column_base: bool,
    normalise: bool,
) -> PolarsResult<LazyFrame> {
    // rename the target column
    let group_by_alais = if target == group_by {
        format!("__{}__group_by", group_by)
    } else {
        group_by.to_owned()
    };

    // create cross tabs
    let grouped = original
        .clone()
        .select([col(target), col(group_by).alias(&group_by_alais)])
        .with_columns([concat_list([col(&group_by_alais)])?.alias(&group_by_alais)])
        .explode(
            cols([&group_by_alais]),
            ExplodeOptions {
                empty_as_null: false,
                keep_nulls: false,
            },
        );
    let mut lf = grouped
        .clone()
        .with_columns([concat_list([col(target)])?.alias(target)])
        .explode(
            cols([target]),
            ExplodeOptions {
                empty_as_null: false,
                keep_nulls: false,
            },
        )
        .filter(col(target).is_not_null());
    let original_dtype = lf.collect_schema()?.try_get(target)?.clone();

    let columns = lf
        .clone()
        .select([col(&group_by_alais)])
        .unique(None, UniqueKeepStrategy::Any)
        .sort([&group_by_alais], Default::default())
        .collect()?;

    let pivot = lf
        .pivot(
            cols([&group_by_alais]),
            Arc::new(columns),
            cols([target]),
            cols([target]),
            len(),
            false,
            "_".into(),
            polars::frame::PivotColumnNaming::Auto,
        )
        .with_column(col(target).cast(original_dtype))
        .sort(
            [target],
            SortMultipleOptions::default()
                .with_order_descending(false)
                .with_nulls_last(false),
        );

    // create base row
    if show_column_base || normalise {
        let base_size_row = frequency_counts(
            original.clone(),
            group_by, // frequency_counts also calls case_insensitive() so uhh,,
            None,
            false,
            false,
        )?
        .with_column(
            col(group_by) // make colname matching case-insensitive
                .cast(DataType::String)
                .alias(&group_by_alais),
        )
        .select([col(&group_by_alais), col("count")])
        .collect()?
        .transpose(None, Some(either::Either::Left(group_by_alais)))?;

        let result = concat(
            [base_size_row.lazy(), pivot],
            UnionArgs {
                diagonal: true,
                from_partitioned_ds: true,
                parallel: true,
                rechunk: true,
                to_supertypes: true,
                strict: true,
                maintain_order: true,
            },
        )?;
        if normalise {
            let result = normalise_counts(result, target);
            Ok(if show_column_base {
                result
            } else {
                result.slice(1, IdxSize::MAX)
            })
        } else {
            Ok(result)
        }
    } else {
        Ok(pivot)
    }
}

#[inline(always)]
fn normalise_counts(frame: LazyFrame, question: &str) -> LazyFrame {
    let counts = all().exclude_cols([question]).as_expr();
    frame.with_columns([when(col(question).is_null())
        .then(counts.clone())
        .otherwise(counts.clone().true_div(counts.first()))])
}

/// find column name that matches the target, case insensitive
/// raises error if target was not found
pub(crate) fn find_column_name<'a>(
    all_question_names: &'a [&'a str],
    target: &'a str,
) -> PolarsResult<&'a str> {
    all_question_names
        .iter()
        .find_map(|name| {
            if name.eq_ignore_ascii_case(target) {
                Some(*name)
            } else {
                None
            }
        })
        .ok_or_else(|| PolarsError::ColumnNotFound(target.to_string().into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_counts_normalise() -> PolarsResult<()> {
        let frame = df!["answer" => [Some(1u32), Some(1), Some(2), None]]?.lazy();
        for show_total in [false, true] {
            let expected = df![
                "answer" => [None, Some(1u32), Some(2)],
                "total" => [3.0, 2.0 / 3.0, 1.0 / 3.0],
            ]?;
            let expected = if show_total {
                expected
            } else {
                expected.slice(1, 2)
            };
            let result =
                frequency_counts(frame.clone(), "answer", Some("total"), show_total, true)?
                    .collect()?;
            assert!(
                result.equals_missing(&expected),
                "{result}\nexpected:\n{expected}"
            );
        }
        Ok(())
    }

    #[test]
    fn test_frequency_counts_normalise_multiple_choices() -> PolarsResult<()> {
        let answers = Series::new(
            "answer".into(),
            [
                Some(Series::new("".into(), [1u32, 2])),
                Some(Series::new("".into(), [2u32])),
                Some(Series::new("".into(), Vec::<u32>::new())),
                None,
            ],
        );
        let result =
            frequency_counts(df!["answer" => answers]?.lazy(), "answer", None, true, true)?
                .collect()?;
        let expected = df![
            "answer" => [None, Some(1u32), Some(2)],
            "count" => [3.0, 1.0 / 3.0, 2.0 / 3.0],
        ]?;
        assert!(
            result.equals_missing(&expected),
            "{result}\nexpected:\n{expected}"
        );
        Ok(())
    }

    #[test]
    fn test_create_crosstab_normalise() -> PolarsResult<()> {
        let frame = df![
            "answer" => [Some(1u32), Some(1), Some(2), None],
            "group" => [1u32, 2, 1, 2],
        ]?
        .lazy();
        for normalise in [false, true] {
            for show_total in [false, true] {
                let expected = if normalise {
                    df![
                        "answer" => [None, Some(1u32), Some(2)],
                        "1" => [2.0, 0.5, 0.5],
                        "2" => [2.0, 0.5, 0.0],
                    ]?
                } else {
                    df![
                        "answer" => [None, Some(1u32), Some(2)],
                        "1" => [2u32, 1, 1],
                        "2" => [2u32, 1, 0],
                    ]?
                };
                let expected = if show_total {
                    expected
                } else {
                    expected.slice(1, 2)
                };
                let result =
                    create_crosstab(&frame, "answer", "group", show_total, normalise)?
                        .collect()?
                        .select(["answer", "1", "2"])?;
                assert!(
                    result.equals_missing(&expected),
                    "{result}\nexpected:\n{expected}"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn test_create_crosstab_same_question() -> PolarsResult<()> {
        let frame = df!["answer" => [Some(1u32), Some(1), Some(2), None]]?.lazy();
        for show_total in [false, true] {
            let result = create_crosstab(&frame, "answer", "answer", show_total, false)?
                .collect()?
                .select(["answer", "1", "2"])?;
            let expected = if show_total {
                df![
                    "answer" => [None, Some(1u32), Some(2)],
                    "1" => [2u32, 2, 0],
                    "2" => [1u32, 0, 1],
                ]?
            } else {
                df![
                    "answer" => [Some(1u32), Some(2)],
                    "1" => [2u32, 0],
                    "2" => [0u32, 1],
                ]?
            };
            assert!(
                result.equals_missing(&expected),
                "show_total={show_total}:\n{result}\nexpected:\n{expected}"
            );
        }
        Ok(())
    }
}
