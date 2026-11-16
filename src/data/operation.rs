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
    frame: LazyFrame,
    column_name: &str,
    name: Option<&str>,
) -> PolarsResult<LazyFrame> {
    let mut to_count = frame.select([col(column_name)]);
    
    let mut datatype = to_count
        .collect_schema()?
        .try_get(column_name)?
        .clone();

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

    Ok(to_count
        .filter(col(column_name).is_not_null())
        .group_by([col(column_name)])
        .agg([len().alias(name.unwrap_or("count"))])
        .with_column(col(column_name).cast(datatype))
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
    DataFrame::new_infer_height(vec![
        Column::full_null(column.name().clone(), 1, &DataType::Null),
        Series::new("label".into(), ["*Total*"]).into(),
        Series::new(name.into(), [base_size as u64]).into(),
    ])
}

/// This function returns a lazyframe to be collected later (basically needs to add text labels)
/// 
/// This function DOES NOT output a overall frequency_count of the `target` column
///
/// Inputs:
/// - target: sidebreak
/// - group_by: topbreak
pub(crate) fn create_crosstab(
    original: &LazyFrame,
    target: &str,
    group_by: &str,
) -> PolarsResult<LazyFrame> {
    // create cross tabs
    let mut lf = original
        .clone()
        .select([target, group_by])
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
    let original_dtype = lf.collect_schema()?
        .try_get(target)?
        .clone();

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
        .with_column(col(target).cast(original_dtype))
        .sort(
            [target], 
            SortMultipleOptions::default()
                .with_order_descending(false)
        );
    Ok(pivot)
    // // create base row
    // let base_size_row = frequency_counts(
    //     original.clone(),
    //     group_by,
    //     None,
    //     // Some("Base")
    // )?
    //     .with_column(col(group_by).cast(DataType::String))
    //     .collect()?
    //     .transpose(
    //         Some(target),
    //         Some(either::Either::Left(group_by.to_string())),
    //     )?;

    // // if !count_total {
    // Ok(concat(
    //     [base_size_row.lazy(), pivot],
    //     UnionArgs {
    //         diagonal: true,
    //         from_partitioned_ds: true,
    //         parallel: true,
    //         rechunk: true,
    //         to_supertypes: true,
    //         strict: true,
    //         maintain_order: true,
    //     },
    // )?
    // .with_simplify_expr(true))

    // } else {

    //     // Add in row base sizes
    //     let overall = frequency_counts(
    //         original.clone().select([col(target)]),
    //         target,
    //         Some("__total"),
    //     )?;

    //     let summary = overall.join(
    //         pivot,
    //         [col(target)],
    //         [col(target)],
    //         JoinArgs::new(JoinType::Left),
    //     );

    //     Ok(concat(
    //         [base_size_row.lazy(), summary],
    //         UnionArgs {
    //             diagonal: true,
    //             from_partitioned_ds: true,
    //             parallel: true,
    //             rechunk: true,
    //             to_supertypes: true,
    //             strict: true,
    //             maintain_order: false,
    //         },
    //     )?
    //     .select([
    //         col("__total").alias("Total"),
    //         all().exclude_cols(["__total"]).as_expr(),
    //     ])
    //     .with_simplify_expr(true))
    // }
}
