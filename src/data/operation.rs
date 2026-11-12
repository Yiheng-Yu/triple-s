//! Collects various polars functions to be used here and there

use polars::prelude::*;

/// Count unique values of this Series and a column "counts" with dtype IdxType
///
/// This function basically calls [`polars::Series::value_counts`] under the hood, apart from multi-choice
/// questions ([`QuestionData::question_type()`] is [`QuestionType::Multiple`]),
/// where the choice for each respondant is counted individually.
///
/// # Arguments
/// - `count_nan` (`bool`):
///     - Toogle on/ off for counting Null values, note that multi-choices entries of an empty list (`[]`) will also count as `null`
/// - `name` (`impl Into<PlSmallStr>`, Optional):
///     - Name of the column, default `count`
pub(crate) fn frequency_counts(
    column: &Column,
    count_nan: bool,
    name: Option<&str>,
) -> PolarsResult<DataFrame> {
    let colname_title = column.name();
    let colname_stats = name.unwrap_or("count");

    // Then we do `value_counts`
    let to_count = match column.dtype() {
        DataType::List(_) => column
            .explode(ExplodeOptions {
                empty_as_null: true,
                keep_nulls: true,
            })?
            .take_materialized_series(),
        _ => column._get_backing_series(),
    };

    let to_count = if count_nan {
        to_count
    } else {
        to_count.drop_nulls()
    };

    to_count.value_counts(true, true, colname_stats.into(), false)?.sort(
        [&colname_title],
        SortMultipleOptions::default()
            .with_nulls_last(true)
            .with_maintain_order(false)
            .with_order_descending(false),
    )
}

/// Count total amount of not-null entries in a column, and return result as dataframe of single row
pub(crate) fn count_not_null(column: &Column, name: &str) -> PolarsResult<DataFrame> {
    let base_size = column.len() - column.null_count();
    df! {
        column.name().as_str() => [None::<u64>],
        "label" => ["*Total*"],
        name => [base_size as u64],
    }
}

/// This function returns a lazyframe to be collected later (basically needs to add text labels)
/// 
/// Inputs:
/// - target: sidebreak
/// - group_by: topbreak
pub(crate) fn create_crosstab(
    lf: &LazyFrame,
    target: &str,
    group_by: &str,
) -> PolarsResult<LazyFrame> {
    let lf = lf.clone()
        .with_columns([
            concat_list([col(group_by)])?.alias(group_by),
        ])
    .explode(
        cols([group_by]),
        ExplodeOptions {
            empty_as_null: false,
            keep_nulls: false,
        },
    )
    .with_columns([concat_list([col(target)])?.alias(target),])
    .explode(
        cols([target]),
        ExplodeOptions {
            empty_as_null: false,
            keep_nulls: false,
        },
    );

    let columns = lf
        .clone()
        .select([col(group_by)])
        .unique(None, UniqueKeepStrategy::Any)
        .sort([group_by], Default::default())
        .collect()?;

    let pivot = lf
        .clone()
        .pivot(
            cols([group_by]),
            Arc::new(columns),
            cols([target]),
            cols([target]),
            len(),
            false,
            "_".into(),
            polars::frame::PivotColumnNaming::Auto,
        )
        .sort([target], SortMultipleOptions::default());

    Ok(pivot)
}

pub (crate) fn calc_base_size(
    lf: &LazyFrame,
    target: &str,
    group_by: &str,
) -> PolarsResult<DataFrame> {
    
    let df = lf.clone()
        .group_by([col(group_by)])
        .agg([col(target).count()])
        .collect()?;

    Ok(df)
}