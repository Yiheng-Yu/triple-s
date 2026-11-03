#[cfg(feature = "pyo3")]
use chrono::{NaiveDate, NaiveTime};
use parking_lot::RwLockReadGuard;
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::{
    IntoPyObjectExt,
    prelude::*,
    types::{PyDict, PyTuple},
};
#[cfg(feature = "pyo3")]
use pyo3_polars::error::PyPolarsErr;

use super::SurveyData;
use crate::{Question, QuestionType, utils::ArcLock};
#[cfg(feature = "pyo3")]
use crate::{
    DataRange, Text, error::InternalError, prelude::*, utils::{anyvalue_to_python, convert_date, convert_time, normalize_py_index},
};

/// Survey data for a single question.
///
/// Obtain a view with [`crate::Survey::get_question_data`]. The view shares its
/// question metadata and respondent data with the survey and remains usable after
/// the survey is dropped. Replacing the survey data also updates existing views.
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "QuestionData", skip_from_py_object)
)]
#[derive(Clone)]
pub struct QuestionData {
    data: ArcLock<SurveyData>,
    question: Question,
}

impl QuestionData {
    pub(crate) fn new(data: ArcLock<SurveyData>, question: Question) -> Self {
        Self { data, question }
    }

    /// name of the question
    #[inline]
    pub fn name(&self) -> String {
        self.question.name()
    }

    /// Type of the question
    #[inline]
    pub fn question_type(&self) -> RwLockReadGuard<'_, QuestionType> {
        self.question.question_type()
    }

    /// Get [`LazyFrame`] of the question data
    #[inline]
    pub fn lazy(&self) -> PolarsResult<LazyFrame> {
        Ok(self.data.read().lazy()?.clone().select([col(self.name())]))
    }

    /// Export contents to a polars [`Column`]
    #[inline]
    pub fn to_column(&self) -> PolarsResult<Column> {
        self.lazy()?.collect()?.into_columns().into_iter().next().ok_or(
            PolarsError::ColumnNotFound(
                format!("Unable to find column '{}' in `SurveyData`", self.name()).into(),
            ),
        )
    }

    /// Export contents to a polars [`Series`]
    pub fn to_series(&self) -> PolarsResult<Series> {
        self.to_column().map(Column::take_materialized_series)
    }

    /// Create a DataFrame with the unique values of this Series and a column "counts" with dtype IdxType
    ///
    /// # Arguments
    /// - `include_nan` (`bool`) - Toogle on/ off for counting Null values too.
    pub fn value_counts(&self, include_nan: bool) -> PolarsResult<DataFrame> {
        let series = if include_nan {
            self.to_series()?
        } else {
            self.to_column()?.drop_nulls().take_materialized_series()
        };

        series.value_counts(true, true, self.name().into(), false)
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl QuestionData {
    /// Name of the question
    #[getter(name)]
    fn py_get_name(&self) -> String {
        self.name()
    }

    /// Title of the question
    #[getter(label)]
    fn py_get_label(&self) -> Text {
        self.question.label().clone()
    }

    /// Minimum and maximum values allowed for a quantity response, 
    /// or an implicit range of codes for a numeric categorical response. 
    /// For character responses, the end stores the maximum size plus one. 
    /// String categorical responses do not support a range.
    #[getter(range)]
    fn py_get_range(&self) -> Option<DataRange> {
        self.question.response().range().clone()
    }

    /// Type of the question ('single', 'multiple' etc.,)
    #[getter]
    // `type` is a preserved keyword in rust
    fn get_type(&self) -> Option<String> {
        let qtype = self.question.question_type();
        match qtype.clone() {
            QuestionType::None => None,
            _ => Some(qtype.to_string()),
        }
    }

    /// Get a copy of question data as a Python tuple.
    #[getter(data)]
    fn py_get_data<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let series = self.to_series().map_err(PyPolarsErr::Polars)?;
        let dtype = self.question.polars_dtype();
        match dtype {
            DataType::Null => PyTuple::new(py, (0..series.len()).map(|_| py.None())),
            DataType::String => {
                PyTuple::new(py, series.str().map_err(PyPolarsErr::Polars)?.iter())
            }
            DataType::UInt32 => {
                PyTuple::new(py, series.u32().map_err(PyPolarsErr::Polars)?.iter())
            }
            DataType::Float64 => {
                PyTuple::new(py, series.f64().map_err(PyPolarsErr::Polars)?.iter())
            }
            DataType::Date => {
                let iterator = series
                    .date()
                    .map_err(PyPolarsErr::Polars)?
                    .physical()
                    .iter()
                    .map(|days| days.map(convert_date).transpose())
                    .collect::<PyResult<Vec<Option<NaiveDate>>>>()?;
                PyTuple::new(py, iterator)
            }
            DataType::Time => {
                let iterator = series
                    .time()
                    .map_err(PyPolarsErr::Polars)?
                    .physical()
                    .iter()
                    .map(|days| days.map(convert_time).transpose())
                    .collect::<PyResult<Vec<Option<NaiveTime>>>>()?;
                PyTuple::new(py, iterator)
            }
            DataType::List(inner) if *inner == DataType::UInt32 => {
                let values = series
                    .iter()
                    .map(|s| anyvalue_to_python(py, s))
                    .collect::<PyResult<Vec<_>>>()?;
                PyTuple::new(py, values)
            }
            _ => Err(
                InternalError::new(format!("Undefined question dtype: '{:?}'", dtype)).into(),
            ),
        }
    }

    fn __repr__(&self) -> PyResult<String> {
        let mut content = Vec::new();

        content.push(("name", format!("{:?}", self.name())));
        match self.get_type() {
            Some(value) => content.push(("type", format!("{:?}", value))),
            None => content.push(("type", "None".into())),
        };

        const MAX_ROWS: usize = 5;
        let series = self.to_series().map_err(PyPolarsErr::Polars)?;
        let mut sample = series
            .iter()
            .take(MAX_ROWS)
            .map(|v| match v {
                AnyValue::Null => "None".to_string(),
                AnyValue::String(s) => format!("{s:?}"),
                AnyValue::StringOwned(s) => format!("{s:?}"),
                AnyValue::Boolean(b) => if b { "True" } else { "False" }.to_string(),
                _ => v.to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ");
        if series.len() > MAX_ROWS {
            sample.push_str(", ...");
        }
        let sample = format!("[{}]", sample);

        let value = content
            .into_iter()
            .chain(std::iter::once(("data", sample)))
            .map(|(name, value)| format!("{}: {}", name, value))
            .collect::<Vec<String>>()
            .join("\n  ");
        Ok(format!("QuestionData({{\n  {}\n}})", value))
    }

    fn __getitem__<'py>(
        &'py self,
        py: Python<'py>,
        index: isize,
    ) -> PyResult<Bound<'py, PyAny>> {
        let col = self.to_column().map_err(PyPolarsErr::Polars)?;
        let total = col.len();
        let index = normalize_py_index(index, total).unwrap_or(index as usize);
        let value = col.get(index).map_err(PyPolarsErr::Polars)?;
        anyvalue_to_python(py, value)
    }

    fn __iter__(&self) -> PyResult<ColumnIterator> {
        let column = self.to_column().map_err(PyPolarsErr::Polars)?;
        Ok(ColumnIterator { column, index: 0 })
    }

    /// Return the answers as a polars Series.
    #[pyo3(name = "to_polars")]
    fn py_to_polars(&self) -> PyResult<pyo3_polars::PySeries> {
        self.to_series()
            .map(pyo3_polars::PySeries)
            .map_err(|err| PyPolarsErr::Polars(err).into())
    }

    /// Export the answers as a pandas Series object.
    #[pyo3(name = "to_pandas")]
    pub fn py_to_pandas<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame = self.py_to_polars()?.into_bound_py_any(py)?;
        let kwargs = PyDict::new(py);
        kwargs.set_item("use_pyarrow_extension_array", true)?;
        frame.call_method("to_pandas", (), Some(&kwargs))
    }

    /// Export the answers as a list.
    #[pyo3(name = "to_list")]
    pub fn py_to_list<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let frame = self.py_to_polars()?.into_bound_py_any(py)?;
        frame.call_method("to_list", (), None)
    }
}

// ---------- Helpers -----------
/// Iterates through response data of a question
#[cfg(feature = "pyo3")]
#[pyclass]
pub(crate) struct ColumnIterator {
    column: Column,
    index: usize,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl ColumnIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__<'py>(&mut self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        if self.index >= self.column.len() {
            return Ok(None);
        }

        let value = self.column.get(self.index).map_err(PyPolarsErr::Polars)?;

        self.index += 1;
        anyvalue_to_python(py, value).map(Some) // bound already checked
    }
}
