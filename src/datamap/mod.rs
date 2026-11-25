#![doc = include_str!("../doc/datamap_survey_rs.md")]

pub mod question;
pub(crate) mod read;
pub mod record;
pub mod response;
pub(crate) mod survey_datamap;
pub mod survey_info;
pub mod text;

#[cfg(not(feature="datamap-only"))]
mod parsing;

#[cfg(not(feature="datamap-only"))]
use std::{fs::File, path::Path};

#[cfg(not(feature="datamap-only"))]
use polars::{
    frame::row::{AnyValueBuffer, Row},
    prelude::*,
};

pub(crate) use question::{Location, Question, QuestionType, Spread, Usage};
pub use read::{read_datamap, read_from_reader, read_from_string};

#[cfg(not(feature="datamap-only"))]
pub(crate) use record::DataFormat;

pub(crate) use response::{Code, Response};
pub use survey_datamap::Datamap;

#[cfg(not(feature="datamap-only"))]
use tempfile::NamedTempFile;

pub(crate) use text::{Text, Variant};

#[cfg(not(feature="datamap-only"))]
use crate::data::SurveyData;

#[cfg(not(feature="datamap-only"))]
const BATCH_SIZE: usize = 4096; // no. rows when writing to paquet cache

/// Reads .asc or .csv datafile with datamap provided.
///
/// Supports both UTF-8 and BOM-marked UTF-16LE/BE encodings.
/// Invalid sequences become `�` (U+FFFD), each occupying one fixed-width position.
/// Returns [`SurveyData`].
/// The returned data is not stored in the record.
///
/// This `read_data` function will stream data into a temporary parquet cache file (`.parquet`) to be
/// used later. Keep the returned [`SurveyData`] alive while using its cache.
///
/// ```no_run
/// use triple_s::{read_data, read_datamap};
///
/// // Load a survey with no data
/// let datamap = read_datamap("path/to/datamap.sss")?;
///
/// // then load the data later
/// let survey_data = read_data(&datamap, "new/data/path.asc")?; // loads data
///
/// // Export to poalrs dataframe
/// let df = survey_data.to_polars()?;
/// println!("{:?}", df);
///
/// # Ok::<(), anyhow::Error>(())
/// ```
#[cfg(not(feature="datamap-only"))] 
pub fn read_data(datamap: &Datamap, path_data: impl AsRef<Path>) -> PolarsResult<SurveyData> {
    let path_data = path_data.as_ref();
    let path_string = path_data.to_string_lossy(); // for logging error

    let data_format = DataFormat::from_path(path_data)
        .map_err(|err| PolarsError::IO {
            error: Arc::new(std::io::ErrorKind::InvalidInput.into()),
            msg: Some(format!("{:?}", err).into()),
        })
        .with_context(|| format!("When trying to read data from: '{}'", path_string))?;
    let (parquet_file, cache) = NamedTempFile::with_suffix(".parquet")?.into_parts();

    // get parquet schema
    let record = datamap.record().clone(); // only clones pointer
    let schema = Schema::from_iter(
        record
            .content()
            .iter()
            .map(|item| item.read().polars_field()),
    );

    // prepare to write parquet file
    let mut writer = ParquetWriter::new(parquet_file).batched(&schema)?;
    let mut batch_buffer = Vec::with_capacity(BATCH_SIZE);

    let row_scan = record
        .scan_data_file(path_data, &data_format)
        .map_err(|err| PolarsError::ComputeError(format!("{:?}", err).into()))
        .with_context(|| format!("When trying to read data from: '{}'", path_string))?;

    for row in row_scan {
        let row = row
            .map_err(|err| PolarsError::ComputeError(format!("{:?}", err).into()))
            .with_context(|| format!("When trying to read data from: '{}'", path_string))?;
        batch_buffer.push(row);

        if batch_buffer.len() == BATCH_SIZE {
            write_batch(&mut writer, &schema, &mut batch_buffer)?;
        }
    }
    // write the final batch
    write_batch(&mut writer, &schema, &mut batch_buffer)?;
    writer.finish()?;

    SurveyData::from_cache(cache, Some(Arc::new(schema)))
}

/// Write batches of rows into a parquet database with given schema.
#[inline]
#[cfg(not(feature="datamap-only"))]
fn write_batch(
    writer: &mut polars::io::parquet::write::BatchedWriter<File>,
    schema: &Schema,
    batch_buffer: &mut Vec<Row<'static>>,
) -> PolarsResult<()> {
    if batch_buffer.is_empty() {
        return Ok(());
    }
    let height = batch_buffer.len();
    let mut buffers: Vec<AnyValueBuffer<'static>> = schema
        .iter_values()
        .map(|dtype| (dtype, height).into())
        .collect();
    // Rows are no longer needed after writing. Move their values into the
    // column builders instead of cloning every value (including strings/lists).
    for row in batch_buffer.drain(..) {
        for (value, buffer) in row.0.into_iter().zip(&mut buffers) {
            polars_ensure!(buffer.add(value).is_some(), ComputeError: "Unable to append parsed value to column buffer");
        }
    }
    let columns = buffers
        .into_iter()
        .zip(schema.iter_names())
        .map(|(buffer, name)| {
            let mut column = buffer.into_series()?.into_column();
            column.rename(name.clone());
            Ok(column)
        })
        .collect::<PolarsResult<Vec<_>>>()?;
    let df = DataFrame::new(height, columns)?;
    writer.write_batch(&df)?;
    Ok(())
}
