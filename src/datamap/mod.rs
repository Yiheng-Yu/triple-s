pub mod question;
pub mod read;
pub mod record;
pub mod response;
pub mod survey_datamap;
pub mod survey_info;
pub mod text;

use std::{fs::File, path::Path};

use polars::{frame::row::Row, prelude::*};
pub use question::{Location, Question, QuestionType, Spread, Usage};
pub use record::DataFormat;
pub use response::{Code, Response};
pub use survey_datamap::Datamap;
use tempfile::NamedTempFile;
pub use text::{Text, Variant};

use crate::data::SurveyData;

const BATCH_SIZE: usize = 500; // no. rows when writing to paquet cache

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
    let schema =
        Schema::from_iter(record.content().iter().map(|item| item.read().polars_field()));

    // prepare to write parquet file
    let mut writer = ParquetWriter::new(parquet_file).batched(&schema)?;
    let mut batch_buffer = Vec::new();

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
fn write_batch(
    writer: &mut polars::io::parquet::write::BatchedWriter<File>,
    schema: &Schema,
    batch_buffer: &mut Vec<Row<'static>>,
) -> PolarsResult<()> {
    let df = DataFrame::from_rows_and_schema(batch_buffer, schema)?;
    writer.write_batch(&df)?;
    batch_buffer.clear();
    Ok(())
}
