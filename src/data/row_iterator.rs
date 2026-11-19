//! Iterator-local, bounded row conversion shared by Survey and SurveyData.
use polars::prelude::*;
use pyo3::{
    prelude::*,
    types::{PyDict, PyIterator},
};
use pyo3_polars::{PyDataFrame, error::PyPolarsErr};

use super::SurveyData;

// Limit both tall and wide buffers. A single exceptionally wide row is the
// minimum possible buffer. These limits count cells, not payload bytes.
const INITIAL_BUFFER_ROWS: usize = 256;
const INITIAL_BUFFER_CELLS: usize = 65_536;
const MAX_BUFFER_ROWS: usize = 4096;
const MAX_BUFFER_CELLS: usize = 262_144;

#[pyclass]
pub(crate) struct RowIterator {
    lazy: LazyFrame,
    rows: Option<Py<PyIterator>>,
    index: i64,
    buffer_rows: usize,
    max_buffer_rows: usize,
    final_buffer: bool,
    finished: bool,
    // Own the source's temporary-file guard.
    // Drop this after the current buffer and query have been released.
    _source: SurveyData,
}

impl RowIterator {
    pub(crate) fn new(source: SurveyData, lazy: LazyFrame, width: usize) -> Self {
        Self {
            lazy,
            rows: None,
            index: 0,
            buffer_rows: INITIAL_BUFFER_ROWS.min((INITIAL_BUFFER_CELLS / width.max(1)).max(1)),
            max_buffer_rows: MAX_BUFFER_ROWS.min((MAX_BUFFER_CELLS / width.max(1)).max(1)),
            final_buffer: false,
            finished: false,
            _source: source,
        }
    }

    fn collect_buffer(&self) -> PolarsResult<DataFrame> {
        self.lazy
            .clone()
            .slice(self.index, self.buffer_rows as IdxSize)
            .collect()
    }

    fn finish(&mut self) {
        self.finished = true;
        self.rows = None;
        self.lazy = LazyFrame::default();
        self._source = SurveyData::empty();
    }
}

#[pymethods]
impl RowIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub(crate) fn __next__<'py>(
        &mut self,
        py: Python<'py>,
    ) -> PyResult<Option<Bound<'py, PyAny>>> {
        loop {
            if let Some(rows) = &self.rows {
                match rows.bind(py).clone().next() {
                    Some(Ok(row)) => {
                        self.index += 1;
                        return Ok(Some(row));
                    }
                    Some(Err(error)) => {
                        // Retry from the first unconsumed row, never silently skip
                        // the remainder of a buffer after a conversion error.
                        self.rows = None;
                        return Err(error);
                    }
                    None => {
                        self.rows = None;
                        if self.final_buffer {
                            self.finish();
                        } else {
                            self.buffer_rows =
                                (self.buffer_rows * 2).min(self.max_buffer_rows);
                        }
                    }
                }
            }
            if self.finished {
                return Ok(None);
            }

            let frame = py
                .detach(|| self.collect_buffer())
                .map_err(PyPolarsErr::Polars)?;
            if frame.height() == 0 {
                self.finish();
                return Ok(None);
            }
            let final_buffer = frame.height() < self.buffer_rows;
            let options = PyDict::new(py);
            options.set_item("named", true)?;
            options.set_item("buffer_size", self.buffer_rows)?;
            // Polars converts a whole small buffer to native Python values once.
            // Its iterator creates each result dictionary only when requested.
            let rows = PyDataFrame(frame)
                .into_pyobject(py)?
                .call_method("iter_rows", (), Some(&options))?
                .try_iter()?
                .unbind();
            self.rows = Some(rows);
            self.final_buffer = final_buffer;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_buffers_are_bounded_and_slice_the_correct_rows() -> PolarsResult<()> {
        let values = Series::new("Q0".into(), (0..1000i32).collect::<Vec<_>>());
        let columns = (0..3000)
            .map(|i| {
                let mut column = values.clone().into_column();
                column.rename(format!("Q{i}").into());
                column
            })
            .collect();
        let frame = DataFrame::new(1000, columns)?;
        let mut rows = RowIterator::new(SurveyData::empty(), frame.lazy(), 3000);
        for offset in [0, 21, 990, 1000] {
            rows.index = offset;
            let buffer = rows.collect_buffer()?;
            assert_eq!(buffer.height(), (1000 - offset as usize).min(21));
            assert_eq!(buffer.width(), 3000);
            assert!(buffer.height() * buffer.width() <= MAX_BUFFER_CELLS);
            if buffer.height() > 0 {
                assert_eq!(
                    buffer.column("Q2999")?.get(0)?,
                    AnyValue::Int32(offset as i32)
                );
            }
        }
        Ok(())
    }

    #[test]
    fn iterator_keeps_temporary_source_alive() -> anyhow::Result<()> {
        let source = SurveyData::from_polars(df!("Q" => [1i32, 2, 3])?)?;
        let path = std::path::PathBuf::from(source.cache().unwrap());
        let rows = RowIterator::new(source.clone(), source.lazy()?.clone(), 1);
        drop(source);
        assert!(path.exists());
        assert_eq!(rows.collect_buffer()?.height(), 3);
        drop(rows);
        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn exhausted_iterator_releases_its_source_before_drop() -> anyhow::Result<()> {
        let source = SurveyData::from_polars(df!("Q" => [1i32, 2, 3])?)?;
        let path = std::path::PathBuf::from(source.cache().unwrap());
        let lazy = source.lazy()?.clone();
        let mut rows = RowIterator::new(source, lazy, 1);
        assert!(path.exists());
        rows.finish();
        assert!(!path.exists());
        Ok(())
    }
}
