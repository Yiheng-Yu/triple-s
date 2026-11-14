//! Collects various polars functions to be used here and there

use polars::prelude::*;

/// Build a lazy frequency table with a value column and an IdxType count column.
///
/// List entries are exploded so each choice is counted individually. Empty lists
/// and null lists count as null when `count_nan` is true. Despite its name,
/// `count_nan` controls nulls, not floating-point NaNs.
///
/// Only the schema is resolved here; data execution is deferred until collection.
pub(crate) fn frequency_counts(
    frame: LazyFrame,
    column_name: &str,
    name: Option<&str>,
) -> PolarsResult<LazyFrame> {
    let mut to_count = frame.select([col(column_name)]);
    if matches!(
        to_count.collect_schema()?.try_get(column_name)?,
        DataType::List(_)
    ) {
        to_count = to_count.explode(
            cols([column_name]),
            ExplodeOptions {
                empty_as_null: true,
                keep_nulls: true,
            },
        );
    }

    Ok(to_count
        .filter(col(column_name).is_not_null())
        .group_by([col(column_name)])
        .agg([len().alias(name.unwrap_or("count"))])
        .sort(
            [column_name],
            SortMultipleOptions::default()
                .with_nulls_last(true)
                .with_maintain_order(false)
                .with_order_descending(false),
        ))
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
    original: &LazyFrame,
    target: &str,
    group_by: &str,
) -> PolarsResult<LazyFrame> {
    // create base sizes
    let base_size = frequency_counts(original.clone(), group_by, Some("Base"))?
        .with_column(col(group_by).cast(DataType::String))
        .collect()?
        .transpose(
            Some(target),
            Some(either::Either::Left(group_by.to_string())),
        )?;

    // create cross tabs
    let lf = original
        .clone()
        .with_columns([concat_list([col(group_by)])?.alias(group_by)])
        .explode(
            cols([group_by]),
            ExplodeOptions {
                empty_as_null: false,
                keep_nulls: false,
            },
        )
        .with_columns([concat_list([col(target)])?.alias(target)])
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
            Arc::new(columns.clone()),
            cols([target]),
            cols([target]),
            len(),
            false,
            "_".into(),
            polars::frame::PivotColumnNaming::Auto,
        )
        .sort([target], SortMultipleOptions::default());

    concat(
        [base_size.lazy(), pivot],
        UnionArgs {
            diagonal: true,
            from_partitioned_ds: true,
            parallel: true,
            rechunk: true,
            to_supertypes: true,
            strict: true,
            maintain_order: false,
        },
    )
}
