//! Collects various polars functions to be used here and there

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
pub(crate) fn frequency_counts(
    frame: LazyFrame,
    column_name: &str,
    colname_agg: Option<&str>,
    show_base_size: bool,
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

    if show_base_size {
        let column_total = frame
            .clone()
            .select([col(column_name).count().alias(colname_agg)])
            .with_column(lit(NULL).cast(datatype).alias(column_name))
            .select([col(column_name), col(colname_agg)]);
        concat(
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
        )
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
pub(crate) fn create_crosstab(
    original: &LazyFrame,
    target: &str,
    group_by: &str,
    show_column_base: bool,
) -> PolarsResult<LazyFrame> {
    // create cross tabs
    let grouped = original
        .clone()
        .select([col(target), col(group_by)])
        .with_columns([concat_list([col(group_by)])?.alias(group_by)])
        .explode(
            cols([group_by]),
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
        );
    let original_dtype = lf.collect_schema()?.try_get(target)?.clone();

    let columns = lf
        .clone()
        .select([col(group_by)])
        .unique(None, UniqueKeepStrategy::Any)
        .sort([group_by], Default::default())
        .collect()?;

    let pivot = lf
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
        .with_column(col(target).cast(original_dtype))
        .sort(
            [target],
            SortMultipleOptions::default()
                .with_order_descending(false)
                .with_nulls_last(false),
        );

    // create base row
    // let base_size_expr =
    if show_column_base {
        let base_size_row = frequency_counts(original.clone(), group_by, None, false)?
            .with_column(col(group_by).cast(DataType::String))
            .collect()?
            .transpose(None, Some(either::Either::Left(group_by.to_string())))?;

        concat(
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
        )
    } else {
        Ok(pivot)
    }
}
