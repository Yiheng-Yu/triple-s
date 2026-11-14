#![doc = include_str!("../doc/rs_response.md")]

use std::{cmp::Ordering, collections::HashMap};

use anyhow::{Context, Result};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::{
    IntoPyObjectExt,
    exceptions,
    prelude::*,
    types::{PyDict, PyFloat, PyInt, PySlice, PyString, PyTuple},
};
use quick_xml::{events::BytesText, writer::Writer};

use crate::{
    datamap::{QuestionType, Text},
    error::{self, InvalidArg, MissingAttributeError, TypeMismatch, UnsupportedFeature},
    prelude::DeepCopy,
    utils::{ArcLock, lock_it_up},
};
#[cfg(feature = "pyo3")]
use crate::{
    error::missing_key_err,
    utils::{anyvalue_to_python, extract_indicies, get_type_name, normalize_py_index},
};

#[doc = include_str!("../doc/rs_response.md")]
#[cfg_attr(
    all(feature = "pyo3", not(doc)),
    doc = r###"
Keyword Args:
    range (DataRange | None): Allowed response range, or None for no range. Defaults to None.
    values (list[Code] | None): Response codes. None creates an empty list. Defaults to None.

Returns:
    Response: The constructed Response instance.
"###
)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Response", module = "triple_s", from_py_object)
)]
#[derive(Debug, Default)]
pub struct Response {
    range: ArcLock<Option<DataRange>>,

    values: ArcLock<Vec<ArcLock<Code>>>,
}

impl Clone for Response {
    fn clone(&self) -> Self {
        Self {
            range: self.range.clone(),
            values: self.values.clone(),
        }
    }
}

impl DeepCopy for Response {
    #[doc = include_str!("../doc/rs_deep_copy_impl.md")]
    fn deepcopy(&self) -> Self {
        let values = self
            .values
            .read()
            .iter()
            .map(|item| lock_it_up(item.read().deepcopy()))
            .collect();

        let range = self.range.read().as_ref().map(|item| item.deepcopy());
        Self {
            range: lock_it_up(range),
            values: lock_it_up(values),
        }
    }
}
/// # Fields
///
/// ---
impl Response {
    /// Minimum and maximum values allowed for a quantity response, or an
    /// implicit range of codes for a numeric categorical response.
    /// For character responses, the end stores the maximum size plus one.
    /// String categorical responses do not support a range.
    pub fn range(&self) -> RwLockReadGuard<'_, Option<DataRange>> {
        self.range.read()
    }

    /// mutable access to the `range` field
    pub fn range_mut(&mut self) -> RwLockWriteGuard<'_, Option<DataRange>> {
        self.range.write()
    }

    /// Response items of a question, mostly used for Single/ Multiple question
    /// types. This corresponds to each individual `<value>` tag under
    /// `<values>` node.
    ///
    /// This is also used for indicating some special values and their meanings
    /// for Quantity question type. (i.e., value of '9999.9999' in a
    /// specific quantity question might mean 'NaN').
    pub fn values(&self) -> RwLockReadGuard<'_, Vec<ArcLock<Code>>> {
        self.values.read()
    }

    /// mutable access to the `values` field
    pub fn values_mut(&mut self) -> RwLockWriteGuard<'_, Vec<ArcLock<Code>>> {
        self.values.write()
    }

    /// Get the lookup table for items in the [`Response::values()`] field, as: { [`Code::value`]: [`Code::label()`] }
    ///
    /// This method returns a Arc<RwLock<String>> for the code label text, this means mutating
    /// code label elsewhere will the hashmap returned from this function.
    ///
    /// # Example: Shared mutation of response label mapping
    ///
    /// ```rust
    /// use polars::prelude::AnyValue;
    /// use triple_s::{Code, Response};
    ///
    /// let original = Response::new(None, vec![Code::new_unsigned(3, "Before", None, None)]);
    ///
    /// // Get first copy
    /// let label_map_1 = original.response_label_map();
    /// let label_copy_1 = label_map_1.get(&AnyValue::UInt32(3)).expect("Key not found");
    ///
    /// // Get second copy
    /// let mut label_map_2 = original.response_label_map();
    /// let mut label_copy_2 = label_map_2.get_mut(&AnyValue::UInt32(3)).expect("Key not found");
    ///
    /// // Value of the first and second copy are the same
    /// assert_eq!(label_copy_1.read().as_str(), "Before");
    /// assert_eq!(label_copy_2.read().as_str(), "Before");
    ///
    /// // Updating value of the second copy...
    /// *label_copy_2.write() = "After".into();
    ///
    /// // And the value for the first copy also changed.
    /// assert_eq!(label_copy_1.read().as_str(), "After");
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    #[allow(clippy::module_inception)]
    pub fn response_label_map(&self) -> HashMap<AnyValue<'static>, ArcLock<String>> {
        self.values()
            .iter()
            .map(|code| {
                let code = code.read();
                let code_value = code.value();
                let code_label = code.label().value.clone();
                (code_value, code_label)
            })
            .collect()
    }
}

/// <hr />
///
/// # Content management
impl Response {
    /// Create a response from an optional range and explicitly listed codes.
    ///
    /// # Example:
    ///
    /// ```
    /// use polars::prelude::AnyValue;
    /// use triple_s::response::{Code, DataRange, Response};
    ///
    /// let data_range = DataRange::new(AnyValue::UInt32(1), AnyValue::UInt32(2))?;
    ///
    /// let response = Response::new(
    ///     Some(data_range),
    ///     vec![Code::new_unsigned(3, "Other", None, None)],
    /// );
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(range: Option<DataRange>, values: Vec<Code>) -> Self {
        Self {
            range: lock_it_up(range),
            values: lock_it_up(values.into_iter().map(lock_it_up).collect()),
        }
    }

    /// Counts total amount of values
    #[inline]
    pub fn len(&self) -> usize {
        self.values().len()
    }

    /// Returns whether the response contains no [`Code`] instances.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values().is_empty()
    }

    /// Update `range` field
    pub fn set_range(&mut self, value: Option<DataRange>) {
        *self.range.write() = value;
    }

    /// Update `values` field
    pub fn set_values(&self, value: Vec<ArcLock<Code>>) {
        *self.values.write() = value;
    }

    /// Append a code.
    /// Raises error if the `code` field of the incoming [ `Code` ] instance already
    /// exists in current instance's `values` (in this case use `set_value`
    /// instead).
    ///
    /// ```
    /// use triple_s::{Code, Response};
    ///
    /// let mut response = Response::new(None, Vec::new());
    /// // adding a value
    /// response.add_value(Code::new_unsigned(1, "Label A", None, Some(0.5)))?;
    ///
    /// // cannot add new code with different variant:
    /// assert!(response.add_value(Code::new_signed(2, "Label A", None, Some(0.5))).is_err());
    ///
    /// // cannot add new code with same `code` field,
    /// // use `set_value` instaed
    /// assert!(response.add_value(Code::new_unsigned(1, "Label B", None, Some(0.5))).is_err());
    ///
    /// Ok::<(), triple_s::error::InvalidArg>(())
    /// ```
    pub fn add_value(&mut self, value: Code) -> Result<(), InvalidArg> {
        let key = value.value();

        if let Some(sample) = self.values.read().iter().next() {
            Self::validate_code_variant(&sample.read().value(), &key)?;
        };

        if self.values.read().iter().any(|item| item.read().value() == key) {
            return Err(InvalidArg::new(format!(
                "Cannot append item '{}' because an item with same 'code' attribute already exists. Use 'set_value' to overwrite existing entry.",
                key.to_text()
            )));
        }
        self.values.write().push(lock_it_up(value));
        Ok(())
    }

    /// Try inserting a new [ `Code` ] instance into `Response::values` vec.
    /// Will overwrite existing entries if new `Code::code()` already exists in
    /// `Response::values`
    ///
    /// # Example
    /// ```
    /// use triple_s::{Code, Response};
    ///
    /// let mut response = Response::new(None, Vec::new());
    /// // adding a value
    /// response.set_value(Code::new_unsigned(1, "Label A", None, Some(0.5)))?;
    /// println!("Total amount of values: {}", response.values().len());
    ///
    /// // cannot add new code with different variant:
    /// assert!(response.set_value(Code::new_signed(2, "Label A", None, Some(0.5))).is_err());
    ///
    /// // overwrites existing item in `Response::values`,
    /// response.set_value(Code::new_unsigned(1, "Label B", None, Some(0.5)))?;
    /// println!("Total amount of values: {}", response.values().len());
    ///
    /// Ok::<(), triple_s::error::InvalidArg>(())
    /// ```
    pub fn set_value(&mut self, value: Code) -> Result<(), InvalidArg> {
        let key = value.value();
        // needs to clone these fields outside of the for loop,
        // or it would deadlock the loop (label() etc., acquires a read lock
        // whilst xx_mut() would acquire a write loop of the same parent field.)
        let label = value.label().clone();
        let filter = value.filter();
        let score = value.score();

        if let Some(sample) = self.values.read().iter().next() {
            Self::validate_code_variant(&sample.read().value(), &key)?;
        };

        for current in self.values.write().iter_mut() {
            let mut current = current.write();
            if current.value() == key {
                *current.label_mut() = label;
                *current.filter_mut() = filter;
                *current.score_mut() = score;
                return Ok(());
            }
        }

        self.values.write().push(lock_it_up(value));
        Ok(())
    }

    /// Cast the `AnyValue` variants in this response's `range` and `values` to
    /// `dtype`.
    ///
    /// Returns a copy of this response with the range bounds and each code's
    /// value converted to `dtype`. Other code fields, such as labels and scores,
    /// are preserved.
    ///
    /// Returns an error if any value cannot be converted to `dtype`.
    pub fn cast(&self, dtype: &DataType) -> Result<Self> {
        let range =
            self.range().as_ref().map(|range| range.cast(dtype)).transpose().with_context(
                || "When tryin to conver data type for `range` field.".to_string(),
            )?;

        let values = self
            .values()
            .iter()
            .map(|item| item.read().cast(dtype))
            .collect::<Result<Vec<Code>>>()?;

        Ok(Self::new(range, values))
    }

    fn get_value_(&self, index: usize) -> Option<ArcLock<Code>> {
        self.values.read().get(index).cloned()
    }

    /// Reads a response value at the given index, just to make accessing
    /// particular item in the `values` a bit easier
    ///
    /// Calls `f` with a shared reference to the [`Code`] stored at `index` and
    /// returns the value produced by the closure.
    ///
    /// Returns [`None`] if no value exists at `index`.
    ///
    /// ```
    /// # use triple_s::{response::Code, response::Response};
    /// let values = vec![
    ///     Code::new_unsigned(1, "label 1", None, Some(1.3)),
    ///     Code::new_unsigned(2, "label 2", None, Some(2.0)),
    /// ];
    /// let instance = Response::new(None, values);
    ///
    /// let label = instance.with_value_read(0, |code| code.label().value());
    /// let filter = instance.with_value_read(0, |code| code.filter());
    /// let score = instance.with_value_read(0, |code| code.score());
    ///
    /// assert_eq!(label, Some("label 1".to_string()));
    /// // **hint: you can use [Option::flatten] to get rid of Option<Option<T>>
    /// assert_eq!(filter, Some(None));
    /// assert_eq!(score, Some(Some(1.3)));
    /// ```
    pub fn with_value_read<R>(&self, index: usize, f: impl FnOnce(&Code) -> R) -> Option<R> {
        self.get_value_(index).map(|item| f(&item.read()))
    }

    /// Modify to a response value at the given index.
    ///
    /// Calls `f` with a mutable reference to the [`Code`] stored at `index` and
    /// returns the value produced by the closure.
    ///
    /// Returns [`None`] if no value exists at `index`.
    ///
    /// The write lock is held only for the duration of the closure, so the
    /// reference passed to `f` cannot escape the closure.
    ///
    /// ```
    /// # use triple_s::{response::Code, response::Response};
    /// let values = vec![
    ///     Code::new_unsigned(1, "label 1", None, Some(1.3)),
    ///     Code::new_unsigned(2, "label 2", None, Some(2.0)),
    /// ];
    /// let mut instance = Response::new(None, values);
    ///
    /// instance.with_value_write(0, |code| {
    ///     .. // do your modifications here
    /// });
    ///
    /// // for example:
    /// let _ = instance.with_value_write(0, |code| *code.score_mut() = Some(2.3));
    /// let score = instance.with_value_read(0, |code| code.score());
    /// assert_eq!(score.flatten(), Some(2.3));
    /// ```
    pub fn with_value_write<R>(
        &mut self,
        index: usize,
        f: impl FnOnce(&mut Code) -> R,
    ) -> Option<R> {
        self.get_value_(index).map(|item| f(&mut item.write()))
    }
}

/// <hr />
///
/// # Exporting to triple-s
impl Response {
    /// Export to triple-s classic text string
    /// Fields that does not exist in triple-s classic will be ignored.
    pub fn to_classic(&self, question_type: &QuestionType) -> Result<String> {
        let mut output = String::from("VALUES\n");
        if let Some(range) = self.range.read().as_ref() {
            let start = range.start().to_classic()?;
            let end = range.end().to_classic()?;

            if *question_type == QuestionType::Quantity && start == end {
                output.push_str(&start);
                output.push('\n');
            } else {
                output.push_str(&format!("{start} TO {end}\n"));
            }
        }
        for code in self.values.read().iter() {
            output.push_str(&code.read().to_classic()?);
            if *question_type == QuestionType::Quantity {
                output.push_str(" SPECIAL");
            }
            output.push('\n');
        }
        output.push_str("END VALUES\n");
        Ok(output)
    }

    fn write_range_xml<W: std::io::Write>(
        range: &DataRange,
        writer: &mut Writer<W>,
    ) -> Result<(), std::io::Error> {
        writer
            .create_element("range")
            .with_attribute(("from", range.start().to_text().as_str()))
            .with_attribute(("to", range.end().to_text().as_str()))
            .write_empty()?;
        Ok(())
    }

    fn write_size_xml<W: std::io::Write>(
        range: &DataRange,
        writer: &mut Writer<W>,
    ) -> Result<()> {
        let size_value = match range.end().clone() {
            AnyValue::UInt32(val) => val.saturating_sub(1),
            _ => {
                return Err(error::InternalError::new(format!(
                    "Expecting `AnyValue::UInt32, got: {:?}",
                    range.end
                ))
                .into());
            }
        };
        writer
            .create_element("size")
            .write_text_content(BytesText::new(&size_value.to_string()))?;
        Ok(())
    }

    fn write_range_and_values_xml<W: std::io::Write>(
        writer: &mut Writer<W>,
        range: Option<DataRange>,
        values: RwLockReadGuard<'_, Vec<ArcLock<Code>>>,
    ) -> Result<()> {
        if let Some(range) = range.as_ref() {
            writer.create_element("values").write_inner_content(|child_writer| {
                Self::write_range_xml(range, child_writer)?;
                for value in values.iter() {
                    value.read().write_xml(child_writer)?;
                }
                Ok(())
            })?;
            Ok(())
        } else {
            writer.create_element("values").write_inner_content(|child_writer| {
                for value in values.iter() {
                    value.read().write_xml(child_writer)?;
                }
                Ok(())
            })?;
            Ok(())
        }
    }

    /// Write to xml event
    ///
    /// Mostly used internally `to_xml`` method calls, exposed as part of public
    /// just in case.
    pub fn write_xml<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
        question_type: QuestionType,
    ) -> Result<()> {
        match question_type {
            QuestionType::None | QuestionType::Logical => Ok(()),
            QuestionType::Character => {
                let range = self.range.read();
                let range = range
                    .as_ref()
                    .ok_or_else(|| MissingAttributeError::new("values", "range"))?;

                Self::write_size_xml(range, writer).map_err(std::io::Error::other)?;
                Ok(())
            }

            QuestionType::Quantity => {
                let values = self.values.read();
                let range = self.range.read();
                let range = range
                    .as_ref()
                    .ok_or_else(|| MissingAttributeError::new("values", "range"))?
                    .clone();
                Self::write_range_and_values_xml(writer, Some(range), values)
            }

            QuestionType::Single | QuestionType::Multiple => {
                let values = self.values.read();
                let range = self.range.read().as_ref().cloned();
                Self::write_range_and_values_xml(writer, range, values)
            }

            QuestionType::Date | QuestionType::Time => {
                if let Some(range) = self.range.read().clone() {
                    let values = self.values.read();
                    Self::write_range_and_values_xml(writer, Some(range), values)
                } else {
                    Ok(())
                }
            }
        }
    }

    #[inline]
    pub(crate) fn validate_code_variant(
        sample: &AnyValue<'static>,
        incoming: &AnyValue<'static>,
    ) -> Result<(), InvalidArg> {
        if sample.dtype() != incoming.dtype() {
            Err(InvalidArg::new(format!(
                "Code value must have the same Polars data type as existing codes; got {incoming:?}, existing {sample:?}"
            )))
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Response {
    /// Create response codes with an optional range.
    ///
    /// Keyword Args:
    ///     range (DataRange | None): Allowed response range, or None for no range. Defaults to None.
    ///     values (list[Code] | None): Response codes. None creates an empty list. Defaults to None.
    ///
    /// Returns:
    ///     Response: The constructed Response instance.
    #[new]
    #[pyo3(signature=(range:"DataRange"=None, values:"list[Code]"=None))]
    fn py_new(range: Option<DataRange>, values: Option<Vec<Code>>) -> Self {
        let values = values.unwrap_or_default();
        Self::new(range, values)
    }

    fn __getitem__<'py>(&self, index: &Bound<'py, PyAny>) -> PyResult<Py<PyAny>> {
        let py = index.py();
        if let Ok(index) = index.extract::<isize>() {
            let index = normalize_py_index(index, self.values().len())
                .ok_or(exceptions::PyIndexError::new_err(index))?;

            let result = self
                .get_value_(index)
                .map(|item| item.read().clone())
                .ok_or(exceptions::PyIndexError::new_err(index))?
                .into_py_any(py)?;
            Ok(result)
        } else if let Ok(indicies) = index.cast::<PySlice>() {
            let total = self.len() as isize;
            let values_read = self.values.read();
            let indicies = extract_indicies(indicies, total)?;
            let result = indicies
                .into_iter()
                .map(|idx| {
                    let result = values_read
                        .get(idx as usize)
                        .ok_or(exceptions::PyIndexError::new_err(idx))?
                        .read()
                        .clone();
                    Ok(result)
                })
                .collect::<PyResult<Vec<Code>>>()?
                .into_py_any(py)?;
            Ok(result)
        } else {
            let type_name = index.get_type().name()?.extract::<String>()?;
            Err(exceptions::PyTypeError::new_err(format!(
                "`index` must be `int` or `slice` (e.g [0..1]), not `{}`",
                type_name
            )))
        }
    }

    pub(crate) fn __repr__(&self) -> String {
        let range =
            self.range.read().as_ref().map(|item| item.__repr__()).unwrap_or("None".into());

        const MAX_CODES: usize = 5;

        let length = self.values.read().len();
        let code_formatted = if length == 0 {
            "[]".to_string()
        } else if length == 1 {
            let code = self.values.read().iter().next().unwrap().read().__repr__();
            format!("[{}]", code)
        } else if length <= MAX_CODES {
            let code = self
                .values
                .read()
                .iter()
                .map(|item| item.read().__repr__())
                .collect::<Vec<String>>()
                .join(",\n  ");

            format!(
                "[\n    {}\n  ]",
                code.replace("\n", "\n  ") // add a bit more indentation
            )
        } else {
            let code = self
                .values
                .read()
                .iter()
                .take(MAX_CODES)
                .map(|item| item.read().__repr__())
                .collect::<Vec<String>>()
                .join(",\n  ");

            format!("[\n    {}\n    ...\n  ]", code.replace("\n", "\n  "))
        };

        format!(
            "Response(\n  range={},\n  values={}\n)",
            range, code_formatted,
        )
    }

    /// Minimum and maximum values allowed for a quantity response, or an
    /// implicit range of codes for a numeric categorical response.
    ///
    /// For character responses, the end stores the maximum size plus one.
    /// String categorical responses do not support a range.
    #[getter(range)]
    fn py_get_range(&self) -> Option<DataRange> {
        self.range.read().as_ref().map(|item| item.clone())
    }

    /// Code values of a question, mostly used for Single/ Multiple question
    /// types.
    ///
    /// This is also used for indicating some special values and their meanings
    /// for Quantity question type. (i.e., value of '9999.9999' in a
    /// specific quantity question might mean 'NaN').
    #[getter(values)]
    fn py_get_values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(
            py,
            self.values.read().iter().map(|item| item.read().clone()),
        )
    }

    /// Dictionary of { code_value: code_label }
    #[getter(response_label)]
    pub(crate) fn py_get_response_label<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let output = PyDict::new(py);

        // iter through instead of calling self.response_label() because HashMap is not ordered
        for code in self.values().iter() {
            let code = code.read();
            let value = anyvalue_to_python(py, code.value())?;
            let text = code.label().deepcopy();
            output.set_item(value, text)?;
        }

        Ok(output)
    }

    /// get [ `Code` ] inside `self.values` with `code.value` given.
    ///
    /// Returns `None` if the input `code` was not found.
    ///
    /// Args:
    ///     value (str | int | float): Code value to look up.
    ///
    /// Returns:
    ///     Code | None: The matching code, or None if it does not exist.
    #[pyo3(name = "get_value", signature = (value: "str|int|float"))]
    fn py_get_value<'py>(&self, value: &Bound<'py, PyAny>) -> PyResult<Option<Code>> {
        if !value.is_instance_of::<PyString>()
            && !value.is_instance_of::<PyInt>()
            && !value.is_instance_of::<PyFloat>()
        {
            return Err(exceptions::PyTypeError::new_err(
                "Input must be str, int, or float",
            ));
        }

        for item in self.values.read().iter() {
            let item = item.read();
            let stored = anyvalue_to_python(value.py(), item.value())?;

            if stored.eq(value)? {
                return Ok(Some(item.clone()));
            }
        }

        Ok(None)
    }

    /// Append a code, rejecting duplicate values and different Polars data types.
    ///
    /// Args:
    ///     value (Code): Response code to insert or update.
    ///
    /// Returns:
    ///     None: This method updates or validates the existing object in place.
    #[pyo3(name = "add_value")]
    fn py_add_value(&mut self, value: Code) -> PyResult<()> {
        self.add_value(value)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:?}")))
    }

    /// Update or append a code, rejecting different Polars data types.
    /// Will overwrite existing code if they share the same 'code' attribute.
    ///
    /// Args:
    ///     value (Code): Response code to insert or update.
    ///
    /// Returns:
    ///     None: This method updates or validates the existing object in place.
    #[pyo3(name = "set_value")]
    fn py_set_value(&mut self, value: Code) -> PyResult<()> {
        self.set_value(value)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:?}")))
    }

    /// create new instance from dictionary with the following keys:
    ///
    /// 'range' (dict|DataRange|None, *optional*):
    ///     data range, default None
    ///
    /// 'values' (list[Code|dict]):
    ///     list of code values or dictionary to to `Code.from_dict`
    ///
    /// Args:
    ///     content (dict): Fields used to construct the Response instance, as described above.
    ///
    /// Returns:
    ///     Response: The constructed Response instance.
    #[staticmethod]
    #[pyo3(name = "from_dict")]
    pub(crate) fn py_from_dict(content: Bound<'_, PyDict>) -> PyResult<Self> {
        let range = content.get_item("range")?.map(DataRange::from_py_any).transpose()?;

        let values = content
            .get_item("values")?
            .map(|item| item.extract::<Vec<Bound<'_, PyAny>>>())
            .transpose()?
            .unwrap_or_default()
            .into_iter()
            .map(|item| Code::from_py_any(item))
            .collect::<PyResult<Vec<Code>>>()?;

        Ok(Self::new(range, values))
    }
}

/// Corresponds to the `<value>` tag of a triple-s xml.
#[cfg_attr(
    all(feature = "pyo3", not(doc)),
    doc = r###"
Args:
    value (str | int | float): Response code value.
    label (str | None): Text label. None uses an empty label.
    filter (str | None): Response filter expression, or None for no filter.
    score (float | None): Score for statistical calculations, or None for no score.

Raises:
    NotImplementedError: Direct construction is not supported. Use Code.unsigned, Code.signed, Code.float, or Code.string.
"###
)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Code", module = "triple_s", from_py_object)
)]
#[derive(Debug)]
pub struct Code {
    /// code value, this can be String, Float or u32
    pub(crate) value: ArcLock<AnyValue<'static>>,
    /// text label
    pub(crate) label: ArcLock<Text>,
    /// Response filter.
    pub(crate) filter: ArcLock<Option<String>>,
    /// Values to be used when computing statistics such as Mean, Standard
    /// Deviation etc.
    pub(crate) score: ArcLock<Option<f64>>,
}

impl Clone for Code {
    /// makes a shallow copy of the data
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            label: self.label.clone(),
            filter: self.filter.clone(),
            score: self.score.clone(),
        }
    }
}

impl DeepCopy for Code {
    #[doc = include_str!("../doc/rs_deep_copy_impl.md")]
    fn deepcopy(&self) -> Self {
        Self {
            value: lock_it_up(self.value.read().clone()),
            label: lock_it_up(self.label.read().clone()),
            filter: lock_it_up(self.filter.read().clone()),
            score: lock_it_up(*self.score.read()),
        }
    }
}

impl PartialEq for Code {
    fn eq(&self, other: &Self) -> bool {
        (self.value() == other.value())
            & (self.label().clone() == other.label().clone())
            & (self.filter() == other.filter())
            & (self.score() == other.score())
    }
}

impl Eq for Code {}

/// The response options of inside a question, usually single/ multiple choice
/// questions.
impl Code {
    /// Returns code value. This corresponds to the `code` attribute of triple-s
    /// `<value>` tag. This can be float, string or int.
    /// Code value `string` is introduced in triple-s xml 2.0 (via
    /// format='literal' flag).
    ///
    /// # Example
    /// ```rust
    /// use polars::prelude::AnyValue;
    /// use triple_s::response::Code;
    ///
    /// let code = Code::new_literal("A", "Tea & coffee", None, None);
    /// assert_eq!(code.value(), AnyValue::StringOwned("A".into()))
    /// ```
    ///
    /// To set new value, call `set_value()` instead.
    pub fn value(&self) -> AnyValue<'static> {
        self.value.read().clone()
    }

    /// Sets the `value` field of current insteance
    ///
    /// # Example: Update code value
    ///
    /// ```
    /// use polars::prelude::*;
    /// use triple_s::Code;
    ///
    /// let mut instance = Code::new_literal("A", "Tea & coffee", None, None);
    /// assert_eq!(instance.value(), AnyValue::StringOwned("A".into()));
    ///
    /// // Update value
    /// instance.set_value(AnyValue::Int32(-3))?;
    /// assert_eq!(instance.value(), AnyValue::Int32(-3));
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// This method would only accept one of [`Int32`, `UInt32`, `Float64`, `StringOwned`, `String`] variant of [`AnyValue`] enum.
    ///
    /// ```should_panic
    /// use polars::prelude::*;
    /// use triple_s::Code;
    ///
    /// let mut instance = Code::new_literal("A", "Tea & coffee", None, None);
    /// instance.set_value(AnyValue::Boolean(false)).unwrap();
    /// ```
    pub fn set_value(&mut self, value: AnyValue<'static>) -> Result<(), TypeMismatch> {
        match &value {
            AnyValue::Int32(..)
            | AnyValue::UInt32(..)
            | AnyValue::Float64(..)
            | AnyValue::StringOwned(..) => {
                *self.value.write() = value;
                Ok(())
            }

            AnyValue::String(val) => {
                let string_value: PlSmallStr = val.to_string().into();
                *self.value.write() = AnyValue::StringOwned(string_value);
                Ok(())
            }

            _ => {
                let err_message = format!(
                    "Expect input to be one of [`Int32`, `UInt32`, `Float64`, `StringOwned`, `String`] variant of `AnyValue` enum, got: `{:?}`",
                    value
                );
                Err(TypeMismatch::new(err_message))
            }
        }
    }

    /// text label of a response option for a question.
    pub fn label(&self) -> RwLockReadGuard<'_, Text> {
        self.label.read()
    }

    /// returns a read-write view of the `label` field.
    pub fn label_mut(&mut self) -> RwLockWriteGuard<'_, Text> {
        self.label.write()
    }

    /// Response filters of a particular value. Not used by triple-s itself but
    /// comes very handy for building complex questions/ survey routings. So
    /// I reckon might as well make it available here.
    pub fn filter(&self) -> Option<String> {
        self.filter.read().clone()
    }

    /// returns a read-write view of the `filter` field.
    pub fn filter_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.filter.write()
    }

    /// Values to be used when computing statistics such as Mean, Standard
    /// Deviation etc.
    pub fn score(&self) -> Option<f64> {
        *self.score.read()
    }

    /// returns a read-write view of the `score` field.
    pub fn score_mut(&mut self) -> RwLockWriteGuard<'_, Option<f64>> {
        self.score.write()
    }

    /// Create a response entry with a value of any supported code type and a
    /// text label with no text variants. Call `add_variant` to append text
    /// variants later.
    ///
    /// # Examples
    ///
    /// ```
    /// use polars::prelude::AnyValue;
    /// use triple_s::response::Code;
    ///
    /// let code = Code::new(AnyValue::Int32(-3), "special value", None, Some(0.3));
    /// assert_eq!(code.value(), AnyValue::Int32(-3));
    /// ```
    ///
    /// adding text variants:
    ///
    /// ```
    /// use polars::prelude::AnyValue;
    /// use triple_s::{Code, Variant};
    ///
    /// let mut code = Code::new(AnyValue::Int32(-3), "special value", None, Some(0.3));
    /// code.label_mut()
    ///     .add_variant(Variant::new("variant", Some("interview"), None)?)?;
    ///
    /// Ok::<_, anyhow::Error>(())
    /// ```
    pub fn new(
        value: AnyValue<'static>,
        label: impl Into<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(value),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score),
        }
    }

    /// Create a response entry with an unsigned integer code and a text label.
    /// `filter` and `score` are optional; pass `None` to leave them unset.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::response::Code;
    ///
    /// let code = Code::new_unsigned(1, "label", None, None);
    /// ```
    pub fn new_unsigned(
        value: u32,
        label: impl Into<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(AnyValue::UInt32(value)),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score),
        }
    }

    /// Create a response entry with a signed integer code and a text label with
    /// no variants. call `add_variant` later to add variants to the text.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::response::Code;
    ///
    /// let code = Code::new_signed(-3, "special value", None, Some(0.3));
    /// ```
    pub fn new_signed(
        value: i32,
        label: impl Into<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(AnyValue::Int32(value)),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score),
        }
    }

    /// Create a response entry with a floating-point code and a text label.
    /// `filter` and `score` are optional; pass `None` to leave them unset.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::response::Code;
    ///
    /// let code = Code::new_float(1.5, "decimal value", None, Some(2.5));
    /// ```
    pub fn new_float(
        value: f64,
        label: impl Into<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(AnyValue::Float64(value)),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score),
        }
    }

    /// Create a response entry with a literal string code and a text label.
    /// `filter` and `score` are optional; pass `None` to leave them unset.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::response::Code;
    ///
    /// let code = Code::new_literal("A", "Tea & coffee", None, None);
    /// ```
    pub fn new_literal(
        value: impl Into<String>,
        label: impl Into<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Self {
        Self {
            value: lock_it_up(AnyValue::StringOwned(value.into().into())),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score),
        }
    }

    /// create new instance from PyAny
    #[cfg(feature = "pyo3")]
    pub(crate) fn from_py_any(value: Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(value) = value.extract::<Self>() {
            Ok(value)
        } else if let Ok(value) = value.extract::<Bound<'_, PyDict>>() {
            Self::py_from_dict(value)
        } else {
            let type_name = get_type_name(&value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect to be Code, dict or NoneType kind, got '{}'.",
                type_name
            )))
        }
    }

    /// Convert the current `code` field from one variant to another,
    /// Please refer to the documentation of [`AnyValue::try_strict_cast`] for details about
    /// type conversion behaviour.
    ///
    /// Example: cast signed -> unsigned
    ///
    /// ```
    /// use polars::prelude::{AnyValue, DataType};
    /// use triple_s::Code;
    ///
    /// // create a new [ `Code` ] with signed `code` field:
    /// let code = Code::new_signed(3, "test", None, None);
    /// assert_eq!(code.value().clone(), AnyValue::Int32(3));
    ///
    /// let unsigned = code.cast(&DataType::UInt32)?;
    /// assert_eq!(unsigned.value().clone(), AnyValue::UInt32(3));
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn cast(&self, dtype: &DataType) -> Result<Self> {
        let mut copy = self.deepcopy();

        let new_value = anyhow::Context::with_context(
            self.value().try_strict_cast(dtype),
            || format!(
                "When trying to convert data type for [ `Code` ] instance with 'code' value '{}'",
                self.value().to_text()
            ),
        )?
        .into_static();

        copy.set_value(new_value)?;
        Ok(copy)
    }

    /// Format as triple-s classic string
    /// Fields incompatible to triple-s classic will be ignored
    pub fn to_classic(&self) -> Result<String> {
        Ok(format!(
            "{} \"{}\"",
            self.value.read().to_classic()?,
            self.label.read().to_classic(),
        ))
    }

    pub(crate) fn write_xml<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), std::io::Error> {
        let code = self.value().to_text();
        let score = self.score().map(|score| score.to_string());

        let mut attributes = vec![("code", code.as_str())];
        if let Some(score) = &score {
            attributes.push(("score", score.as_str()));
        };
        self.label.read().write_xml(writer, "value", attributes)
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Code {
    /// Create a Code instance.
    ///
    /// Args:
    ///     value (str | int | float): Response code value.
    ///     label (str | None): Text label. None uses an empty label.
    ///     filter (str | None): Response filter expression, or None for no filter.
    ///     score (float | None): Score for statistical calculations, or None for no score.
    ///
    /// Raises:
    ///     NotImplementedError: Direct construction is not supported. Use Code.unsigned, Code.signed, Code.float, or Code.string.
    #[new]
    #[allow(unused_variables)]
    fn py_new<'py>(
        value: &Bound<'py, PyAny>,
        label: Option<&Bound<'py, PyAny>>,
        filter: Option<&Bound<'py, PyAny>>,
        score: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Self> {
        let message = [
            "Cannot create new Code instance without defining data type first.",
            "  To create new Code instance, use one of the following:",
            "  For Single/ Multiple/ Date/ Time, use 'Code.unsigned(..)', ",
            "  For Singe response question of `literal` code format, use 'Code.string(..)",
            "  For quantity type of questions, use one of 'Code.float(..)' or 'Code.integer(..)'.",
        ];

        Err(exceptions::PyNotImplementedError::new_err(
            message.join("\n"),
        ))
    }

    /// Used for questions of type 'Single/ Multiple/ Date/ Time'
    ///
    /// Args:
    ///     value (int): Unsigned 32-bit code value.
    ///
    /// Keyword Args:
    ///     label (str | None): Text label. None uses an empty label. Defaults to None.
    ///     filter (str | None): Response filter expression, or None for no filter. Defaults to None.
    ///     score (float | None): Score for statistical calculations, or None for no score. Defaults to None.
    ///
    /// Returns:
    ///     Code: The constructed Code instance.
    #[pyo3(signature=(
        value: "int",
        label: "Option[str]"=None,
        filter: "Option[str]"=None,
        score: "Option[str]"=None,
    ))]
    #[staticmethod]
    fn unsigned(
        value: u32,
        label: Option<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> PyResult<Self> {
        Ok(Self::new(
            AnyValue::UInt32(value),
            label.unwrap_or_default(),
            filter,
            score,
        ))
    }

    /// Used for questions of type 'Single/ Multiple/ Date/ Time'
    ///
    /// Args:
    ///     value (str): Literal code value.
    ///
    /// Keyword Args:
    ///     label (str | None): Text label. None uses an empty label. Defaults to None.
    ///     filter (str | None): Response filter expression, or None for no filter. Defaults to None.
    ///     score (float | None): Score for statistical calculations, or None for no score. Defaults to None.
    ///
    /// Returns:
    ///     Code: The constructed Code instance.
    #[pyo3(signature=(
        value: "str",
        label: "Option[str]"=None,
        filter: "Option[str]"=None,
        score: "Option[str]"=None,
    ))]
    #[staticmethod]
    fn string(
        value: String,
        label: Option<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> PyResult<Self> {
        Ok(Self::new(
            AnyValue::StringOwned(value.into()),
            label.unwrap_or_default(),
            filter,
            score,
        ))
    }

    /// Create a code with a floating-point value.
    ///
    /// Args:
    ///     value (float): Floating-point code value.
    ///
    /// Keyword Args:
    ///     label (str | None): Text label. None uses an empty label. Defaults to None.
    ///     filter (str | None): Response filter expression, or None for no filter. Defaults to None.
    ///     score (float | None): Score for statistical calculations, or None for no score. Defaults to None.
    ///
    /// Returns:
    ///     Code: The constructed Code instance.
    #[pyo3(signature=(
        value: "float",
        label: "Option[str]"=None,
        filter: "Option[str]"=None,
        score: "Option[str]"=None,
    ))]
    #[staticmethod]
    fn float(
        value: f64,
        label: Option<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> PyResult<Self> {
        Ok(Self::new(
            AnyValue::Float64(value),
            label.unwrap_or_default(),
            filter,
            score,
        ))
    }

    /// Create a code with a signed integer value.
    ///
    /// Args:
    ///     value (int): Signed 32-bit code value.
    ///
    /// Keyword Args:
    ///     label (str | None): Text label. None uses an empty label. Defaults to None.
    ///     filter (str | None): Response filter expression, or None for no filter. Defaults to None.
    ///     score (float | None): Score for statistical calculations, or None for no score. Defaults to None.
    ///
    /// Returns:
    ///     Code: The constructed Code instance.
    #[pyo3(signature=(
        value: "float",
        label: "Option[str]"=None,
        filter: "Option[str]"=None,
        score: "Option[str]"=None,
    ))]
    #[staticmethod]
    fn signed(
        value: i32,
        label: Option<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> PyResult<Self> {
        Ok(Self::new(
            AnyValue::Int32(value),
            label.unwrap_or_default(),
            filter,
            score,
        ))
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.eq(other)
    }

    fn __repr__(&self) -> String {
        let mut content = vec![
            format!("value={}", self.value().to_text()),
            format!(
                "label=Text(value='{}', variant=[total: {}])",
                self.label().value(),
                self.label().variants().len(),
            ),
        ];
        if let Some(filter) = self.filter().as_ref() {
            content.push(format!("filter='{}'", filter));
        } else {
            content.push("filter=None".into());
        };
        if let Some(score) = self.score().as_ref() {
            content.push(format!("score={}", score));
        } else {
            content.push("score=None".into());
        };
        let content = content.join(", ");
        format!("Code({})", content)
    }

    /// text label of a response option for a question.
    #[getter(label)]
    fn py_get_label(&self) -> Text {
        self.label.read().clone()
    }

    #[setter(label)]
    fn py_set_label<'py>(&mut self, label: &Bound<'py, PyAny>) -> PyResult<()> {
        if let Ok(text) = label.extract::<String>() {
            *self.label.write().value.write() = text;
            Ok(())
        } else if let Ok(text) = label.extract::<Text>() {
            *self.label.write() = text;
            Ok(())
        } else {
            let type_name = get_type_name(label)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect `str` or `triple_s.Text` instance, found: '{}'",
                type_name
            )))
        }
    }

    /// Code value. This corresponds to the `code` attribute of triple-s
    /// `<value>` tag. This can be float, string or int.
    /// Code value `string` is introduced in triple-s xml 2.0 (via
    /// format='literal' toggle).
    #[getter(value)]
    fn py_get_value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        anyvalue_to_python(py, self.value.read().clone())
    }

    /// Values to be used when computing statistics such as Mean, Standard
    /// Deviation etc.
    #[getter(score)]
    fn py_get_score(&self) -> Option<f64> {
        self.score.read().clone()
    }

    #[setter(score)]
    fn py_set_score(&mut self, score: Option<f64>) {
        *self.score_mut() = score;
    }

    /// response filter
    #[getter(filter)]
    fn py_get_filter(&self) -> Option<String> {
        self.filter.read().clone()
    }

    #[setter(filter)]
    fn py_set_filter(&mut self, filter: Option<String>) {
        *self.filter_mut() = filter;
    }

    /// creates new instance from dictionary with the following keys:
    ///
    /// 'value' (float|int|str):
    ///     code value of the new instance
    ///
    /// 'label' (str|Text|dict|None, *optional*):
    ///     code label of the new instance.
    ///     default Text("")
    ///
    /// 'filter' (str|None, *optional*):
    ///     response filter, default 'None'
    ///
    /// 'score' (float|int|None. *optional*):
    ///     score
    ///
    /// Args:
    ///     content (dict): Fields used to construct the Code instance, as described above.
    ///
    /// Returns:
    ///     Code: The constructed Code instance.
    #[staticmethod]
    #[pyo3(name = "from_dict")]
    fn py_from_dict(content: Bound<'_, PyDict>) -> PyResult<Self> {
        let code = content.get_item("value")?.ok_or(missing_key_err("value"))?;
        let code = value_from_python(code)?;

        let label = content
            .get_item("label")?
            .filter(|value| !value.is_none())
            .map_or(Ok(Text::new("")), Text::from_py_any)?;

        let filter = content
            .get_item("filter")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();

        let score = content
            .get_item("score")?
            .map(|item| item.extract::<Option<f64>>())
            .transpose()?
            .flatten();

        Ok(Self {
            value: lock_it_up(code),
            label: lock_it_up(label),
            filter: lock_it_up(filter),
            score: lock_it_up(score),
        })
    }
}

pub(crate) trait AnyValueExt: Sized {
    fn to_text(&self) -> String;
    fn parse_like(&self, text: &str) -> Result<Self, InvalidArg>;
    fn to_classic(&self) -> Result<String>;
    fn validate(&self) -> Result<(), InvalidArg>;
    fn try_cmp(&self, other: &Self) -> Result<Ordering, InvalidArg>;
}

impl AnyValueExt for AnyValue<'static> {
    fn to_text(&self) -> String {
        match self {
            Self::Float64(value) => value.to_string(),
            _ => self.str_value().into_owned(),
        }
    }

    fn parse_like(&self, text: &str) -> Result<Self, InvalidArg> {
        match self {
            Self::Float64(_) => {
                let value = text.parse::<f64>().map_err(|err| {
                    InvalidArg::new(format!(
                        "Failed to parse value '{}' as float decimal: '{}'",
                        text, err
                    ))
                })?;
                let instance = Self::Float64(value);
                instance.validate()?; // value should not be float NaN.
                Ok(instance)
            }

            Self::Int32(_) => {
                let value = text.parse::<i32>().map_err(|err| {
                    InvalidArg::new(format!(
                        "Failed to parse value '{}' as integer: '{}'",
                        text, err
                    ))
                })?;

                Ok(Self::Int32(value))
            }

            Self::UInt32(_) => {
                let value = text.parse::<u32>().map_err(|err| {
                    InvalidArg::new(format!(
                        "Failed to parse value '{}' as integer: '{}'",
                        text, err
                    ))
                })?;
                Ok(Self::UInt32(value))
            }

            Self::String(_) | Self::StringOwned(_) => Ok(Self::StringOwned(text.into())),
            _ => Err(InvalidArg::new("Unsupported code data type")),
        }
    }

    fn to_classic(&self) -> Result<String> {
        match self {
            Self::UInt32(value) => Ok(value.to_string()),
            Self::Int32(value) => Ok(value.to_string()),
            Self::Float64(value) => Ok(value.to_string()),
            _ => {
                Err(UnsupportedFeature::new("SSS Classic", "None numeric code values.").into())
            }
        }
    }

    fn validate(&self) -> Result<(), InvalidArg> {
        match self {
            Self::Float64(value) => {
                if value.is_nan() {
                    Err(InvalidArg::new("'AnyValue' cannot be float `Nan`!"))
                } else if value.is_infinite() {
                    Err(InvalidArg::new("'AnyValue' cannot be infinite!"))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    fn try_cmp(&self, other: &Self) -> Result<Ordering, InvalidArg> {
        if self.dtype() == other.dtype() {
            match (self, other) {
                (Self::UInt32(a), Self::UInt32(b)) => Ok(a.cmp(b)),
                (Self::Int32(a), Self::Int32(b)) => Ok(a.cmp(b)),

                // - checks if any of them were float nan, if so then raises error
                // - then compares the value behind and returns ordering.
                (Self::Float64(a), Self::Float64(b)) => {
                    self.validate()?;
                    other.validate()?;
                    a.partial_cmp(b).ok_or(InvalidArg::new("Cannot compare float NaN values."))
                }

                (Self::StringOwned(_), Self::StringOwned(_)) => Err(InvalidArg::new(
                    "Cannot compare values between `string` values.",
                )),
                _ => Err(InvalidArg::new(format!(
                    "Can only compare pairs of `AnyValue` instances of the same kind. Got: self: [{:?}] | other: [{:?}]",
                    self, other
                ))),
            }
        } else {
            Err(InvalidArg::new(format!(
                "Can only compare pairs of `AnyValue` instances of the same kind. Got: self: [{:?}] | other: [{:?}]",
                self, other
            )))
        }
    }
}

#[cfg(feature = "pyo3")]
fn value_from_python(value: Bound<'_, PyAny>) -> PyResult<AnyValue<'static>> {
    if let Ok(val) = value.extract::<i32>() {
        Ok(AnyValue::Int32(val))
    } else if let Ok(val) = value.extract::<f64>() {
        Ok(AnyValue::Float64(val))
    } else if let Ok(val) = value.extract::<String>() {
        Ok(AnyValue::StringOwned(val.into()))
    } else {
        let type_name = get_type_name(&value)?;
        Err(exceptions::PyTypeError::new_err(format!(
            "Expect to be one of 'str', 'float', or 'int' type, got: '{}'",
            type_name
        )))
    }
}

/// Minimum and maximum values allowed for a quantity response, or an
/// implicit range of codes for a numeric categorical response.
///
/// For character responses, the `end` stores the maximum size plus one
/// (triple-s is 1-indexed). using `start-end` instead of `from-to` because
/// `from` is a preserved keyword in python.
///
/// Fields:
///
/// - start: lower bound of the range.
/// - end: upper bound of the range.
#[cfg_attr(
    all(feature = "pyo3", not(doc)),
    doc = r###"
Args:
    start (int): Lower bound of the range.
    end (int): Upper bound of the range.

Raises:
    NotImplementedError: Direct construction is not supported. Use DataRange.unsigned, DataRange.signed, or DataRange.float.
"###
)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "DataRange", module = "triple_s", from_py_object)
)]
#[derive(Debug)]
pub struct DataRange {
    start: ArcLock<AnyValue<'static>>,
    end: ArcLock<AnyValue<'static>>,
}

impl DataRange {
    /// Creates new `DataRange` instance with start and end values.
    ///
    /// # Example:
    ///
    /// ```
    /// use polars::prelude::{AnyValue, DataType};
    /// use triple_s::DataRange;
    ///
    /// let range = DataRange::new(AnyValue::Int32(-1), AnyValue::Int32(3))?;
    /// assert_eq!(range.start().clone(), AnyValue::Int32(-1));
    ///
    /// Ok::<(), triple_s::error::InvalidArg>(())
    /// ```
    ///
    ///  This method will raise error if `start`, `end` are not the same data
    /// type, or if `start` was greater than `end`:
    ///
    /// ```
    /// use polars::prelude::{AnyValue, DataType};
    /// use triple_s::DataRange;
    ///
    /// // `start` needs to be lower or equal to `end`:
    /// assert!(DataRange::new(AnyValue::Int32(100), AnyValue::Int32(0)).is_err());
    ///
    /// // cannot create new instance of two different value types:
    /// assert!(DataRange::new(AnyValue::Float64(10.0), AnyValue::Int32(0)).is_err())
    ///
    /// // cannot create new instance with `string` value:
    /// ```
    pub fn new(start: AnyValue<'static>, end: AnyValue<'static>) -> Result<Self, InvalidArg> {
        // try_cmp() does three things at once for comparing float values.
        // - checks if any of them were string, and would raise error.
        // - checks if any of them were float nan, if so then raises error
        // - then compares the value behind and returns ordering.
        let ordering = start.try_cmp(&end)?;

        match ordering {
            Ordering::Greater => Err(InvalidArg::new(format!(
                "Start value ({:?}) cannot be greater than the end value ({:?})!",
                start, end
            ))),
            _ => Ok(Self {
                start: lock_it_up(start),
                end: lock_it_up(end),
            }),
        }
    }

    /// A read-only copy of the `start` value (the `from` attribute of the
    /// \<range\> tag.)
    ///
    /// To update its value, use `set_start()` method.
    pub fn start(&self) -> RwLockReadGuard<'_, AnyValue<'static>> {
        self.start.read()
    }

    /// A read-only copy of the `end` value (the `to` attribute of the \<range\>
    /// tag.)
    pub fn end(&self) -> RwLockReadGuard<'_, AnyValue<'static>> {
        self.end.read()
    }

    /// Update values of `start`.
    ///
    /// # Example
    ///
    /// ```
    /// use polars::prelude::{AnyValue, DataType};
    /// use triple_s::DataRange;
    ///
    /// let mut data_range = DataRange::new(AnyValue::Float64(-1.5), AnyValue::Float64(1.5))?;
    ///
    /// data_range.set_start(AnyValue::Float64(1.2))?;
    /// assert_eq!(data_range.start().clone(), AnyValue::Float64(1.2));
    ///
    /// // 'start' cannot be larger than 'end'
    /// assert!(data_range.set_start(AnyValue::Float64(3_000.0)).is_err());
    ///
    /// Ok::<(), triple_s::error::InvalidArg>(())
    /// ```
    pub fn set_start(&mut self, new: AnyValue<'static>) -> Result<(), InvalidArg> {
        let ordering = new.try_cmp(&self.end.read())
            .map_err(|_| InvalidArg::new(format!(
                "Can only update `start` from variants of the same kind. Got: self: [{:?}] | other: [{:?}]",
                self.start.read(), new
            )))?;

        match ordering {
            Ordering::Greater => Err(InvalidArg::new(format!(
                "New `start` value ('{}') cannot be greater than current `end` value ('{}')",
                new.to_text(),
                self.end.read().to_text()
            ))),
            _ => {
                *self.start.write() = new;
                Ok(())
            }
        }
    }

    /// Update values of `end`
    ///
    /// # Example:
    ///
    /// ```
    /// use polars::prelude::{AnyValue, DataType};
    /// use triple_s::DataRange;
    ///
    /// let mut data_range = DataRange::new(AnyValue::Float64(-1.5), AnyValue::Float64(1.5))?;
    ///
    /// data_range.set_end(AnyValue::Float64(2.2))?;
    /// assert_eq!(data_range.end().clone(), AnyValue::Float64(2.2));
    ///
    /// // 'end' cannot be smaller than 'start'
    /// assert!(data_range.set_end(AnyValue::Float64(-300000.0)).is_err());
    ///
    /// Ok::<(), triple_s::error::InvalidArg>(())
    /// ```
    pub fn set_end(&mut self, new: AnyValue<'static>) -> Result<(), InvalidArg> {
        let ordering = new.try_cmp(&self.start.read())
            .map_err(|_| InvalidArg::new(format!(
                "Can only update `end` from variants of the same kind. Got: self: [{:?}] | other: [{:?}]",
                self.end.read(), new
            )))?;

        match ordering {
            Ordering::Less => Err(InvalidArg::new(format!(
                "New `end` value ('{}') cannot be less than current `start` value ('{}')",
                new.to_text(),
                self.start.read().to_text()
            ))),
            _ => {
                *self.end.write() = new;
                Ok(())
            }
        }
    }

    /// Cast field `start`, `end` from one variant to another.
    ///
    /// See [`AnyValue::try_strict_cast`] documentation for more details about type
    /// conversions.
    ///
    /// Example:
    ///
    /// ```
    /// use polars::prelude::{AnyValue, DataType};
    /// use triple_s::DataRange;
    ///
    /// // create a `DataRange` with unsigned `start` and `end` field:
    /// let unsigned = DataRange::new(AnyValue::UInt32(0), AnyValue::UInt32(1))?;
    ///
    /// // cast to signed integer:
    /// let signed = unsigned.cast(&DataType::Int32)?;
    /// assert_eq!(signed.end().clone(), AnyValue::Int32(1));
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn cast(&self, dtype: &DataType) -> Result<Self> {
        let start = self.start().try_strict_cast(dtype)?.into_static();
        let end = self.end().try_strict_cast(dtype)?.into_static();

        Ok(Self::new(start, end)?)
    }

    /// create new instance from PyAny
    #[cfg(feature = "pyo3")]
    pub(crate) fn from_py_any(value: Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(value) = value.extract::<Self>() {
            Ok(value)
        } else if let Ok(value) = value.extract::<Bound<'_, PyDict>>() {
            Self::py_from_dict(value)
        } else {
            let type_name = get_type_name(&value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect to be DataRange, dict or NoneType kind, got '{}'.",
                type_name
            )))
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl DataRange {
    /// Create a DataRange instance.
    ///
    /// Args:
    ///     start (int): Lower bound of the range.
    ///     end (int): Upper bound of the range.
    ///
    /// Raises:
    ///     NotImplementedError: Direct construction is not supported. Use DataRange.unsigned, DataRange.signed, or DataRange.float.
    #[new]
    #[allow(unused_variables)]
    fn py_new<'py>(start: &Bound<'py, PyAny>, end: &Bound<'py, PyAny>) -> PyResult<Self> {
        let message = [
            "Cannot create new DataRange instance without defining data type first.",
            "  To create new DataRange instance, use one of the following:",
            "  'DataRange.signed(start, end)', 'DataRange.unsigned(start, end)', 'DataRange.float(start, end)'.",
        ];

        Err(exceptions::PyNotImplementedError::new_err(
            message.join("\n"),
        ))
    }

    fn __repr__(&self) -> String {
        format!(
            "DataRange(start={}, end={})",
            self.start().to_text(),
            self.end().to_text(),
        )
    }

    fn __eq__(&self, other: &Self) -> PyResult<bool> {
        Ok(self == other)
    }

    /// creates a new instance with values being exclusively positive integers.
    /// This is used for questions of 'single', 'multiple', and 'date/ time'
    /// types.
    ///
    /// Args:
    ///     start (int): Lower bound of the range.
    ///     end (int): Upper bound of the range.
    ///
    /// Returns:
    ///     DataRange: The constructed DataRange instance.
    #[staticmethod]
    fn unsigned(start: u32, end: u32) -> PyResult<Self> {
        Self::new(AnyValue::UInt32(start), AnyValue::UInt32(end)).map_err(|err| err.into())
    }

    /// creates a new instance with values being integers that can either be
    /// positive/ negative. This is used for questions of 'quantity' type
    /// only.
    ///
    /// Args:
    ///     start (int): Lower bound of the range.
    ///     end (int): Upper bound of the range.
    ///
    /// Returns:
    ///     DataRange: The constructed DataRange instance.
    #[staticmethod]
    fn signed(start: i32, end: i32) -> PyResult<Self> {
        Self::new(AnyValue::Int32(start), AnyValue::Int32(end)).map_err(|err| err.into())
    }

    /// creates a new instance with values being decimals.
    /// This is used for questions of 'quantity' type only.
    ///
    /// Args:
    ///     start (float): Lower bound of the range.
    ///     end (float): Upper bound of the range.
    ///
    /// Returns:
    ///     DataRange: The constructed DataRange instance.
    #[staticmethod]
    fn float(start: f64, end: f64) -> PyResult<Self> {
        Self::new(AnyValue::Float64(start), AnyValue::Float64(end)).map_err(|err| err.into())
    }

    /// Start of the data range.
    #[getter(start)]
    fn py_get_start<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        anyvalue_to_python(py, self.start.read().clone())
    }

    #[setter(start)]
    fn py_set_start<'py>(&mut self, value: &Bound<'py, PyAny>) -> PyResult<()> {
        if let Ok(integer) = value.extract::<i32>() {
            let text = integer.to_string();
            let new = self.start.read().parse_like(&text)?;
            new.validate()?;

            self.set_start(new)
                .map_err(|err| exceptions::PyValueError::new_err(format!("{err:?}")))
        } else if let Ok(float) = value.extract::<f64>() {
            let text = float.to_string();
            let new = self.start.read().parse_like(&text)?;
            new.validate()?;

            self.set_start(new)
                .map_err(|err| exceptions::PyValueError::new_err(format!("{err:?}")))
        } else if let Ok(text) = value.extract::<&str>() {
            let new = self.start.read().parse_like(text)?;
            new.validate()?;

            self.set_start(new)
                .map_err(|err| exceptions::PyValueError::new_err(format!("{err:?}")))
        } else {
            let type_name = get_type_name(value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect to be one of ('str', 'int' or 'float'), got: '{}'",
                type_name
            )))
        }
    }

    /// End of the `<range>`
    #[getter(end)]
    fn py_get_end<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        anyvalue_to_python(py, self.end.read().clone())
    }

    #[setter(end)]
    fn py_set_end<'py>(&mut self, value: &Bound<'py, PyAny>) -> PyResult<()> {
        if let Ok(integer) = value.extract::<i32>() {
            let text = integer.to_string();
            let new = self.end.read().parse_like(&text)?;
            new.validate()?;

            self.set_end(new)
                .map_err(|err| exceptions::PyValueError::new_err(format!("{err:?}")))
        } else if let Ok(float) = value.extract::<f64>() {
            let text = float.to_string();
            let new = self.end.read().parse_like(&text)?;
            new.validate()?;

            self.set_end(new)
                .map_err(|err| exceptions::PyValueError::new_err(format!("{err:?}")))
        } else if let Ok(text) = value.extract::<&str>() {
            let new = self.end.read().parse_like(text)?;
            new.validate()?;

            self.set_end(new)
                .map_err(|err| exceptions::PyValueError::new_err(format!("{err:?}")))
        } else {
            let type_name = get_type_name(value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect to be one of ('str', 'int' or 'float'), got: '{}'",
                type_name
            )))
        }
    }

    /// creates new instance from dictionary with the following keys:
    ///
    /// - start (float|int|str): start of a data range
    /// - end (float|int|str): end of a data range
    ///
    /// Args:
    ///     content (dict): Fields used to construct the DataRange instance, as described above.
    ///
    /// Returns:
    ///     DataRange: The constructed DataRange instance.
    #[staticmethod]
    #[pyo3(name = "from_dict")]
    fn py_from_dict(content: Bound<'_, PyDict>) -> PyResult<Self> {
        let start = content.get_item("start")?.ok_or(missing_key_err("start"))?;
        let start = value_from_python(start)?;

        let end = content.get_item("end")?.ok_or(missing_key_err("end"))?;
        let end = value_from_python(end)?;

        Self::new(start, end).map_err(|err| err.into())
    }
}

impl Clone for DataRange {
    /// Makes a `shallow` copyof the current instance,
    /// mutating field in the shallow copy would also update the instance it
    /// copied from.
    fn clone(&self) -> Self {
        Self {
            start: self.start.clone(),
            end: self.end.clone(),
        }
    }
}

impl DeepCopy for DataRange {
    #[doc = include_str!("../doc/rs_deep_copy_impl.md")]
    fn deepcopy(&self) -> Self {
        Self {
            start: lock_it_up(self.start.read().clone()),
            end: lock_it_up(self.end.read().clone()),
        }
    }
}

impl PartialEq for DataRange {
    fn eq(&self, other: &Self) -> bool {
        (self.start().eq(&other.start())) & (self.end().eq(&other.end()))
    }
}

impl Eq for DataRange {}

#[cfg(test)]
mod test {
    use super::*;
    use crate::datamap::survey_datamap::test::normalize_xml;

    #[test]
    fn test_value_float_parse_like() {
        let base = AnyValue::Float64(3.5);

        assert_eq!(base.parse_like("3.2").unwrap(), AnyValue::Float64(3.2));
        assert_eq!(base.parse_like("-3.2").unwrap(), AnyValue::Float64(-3.2));
        assert_eq!(base.parse_like("3").unwrap(), AnyValue::Float64(3.0));
        assert_eq!(base.parse_like("-3").unwrap(), AnyValue::Float64(-3.0));

        assert!(base.parse_like("Nan").is_err());
        assert!(base.parse_like("inf").is_err());
        assert!(base.parse_like("-inf").is_err());
    }

    #[test]
    fn test_read_response_values() {
        let values = [
            Code::new_unsigned(1, "label", None, Some(1.3)),
            Code::new_unsigned(2, "label", None, Some(1.2)),
            Code::new_unsigned(3, "label", None, Some(2.2)),
            Code::new_unsigned(4, "label", None, Some(2.2)),
            Code::new_unsigned(5, "label", None, Some(2.2)),
        ]
        .into_iter()
        .collect::<Vec<Code>>();
        let instance = Response::new(None, values.clone());

        for (index, correct) in values.into_iter().enumerate() {
            let label =
                instance.with_value_read(index, |code| code.label().value().to_string());
            assert_eq!(label, Some(correct.label().value().to_string()));
            let item_got = instance.with_value_read(index, |val| val.deepcopy()).unwrap();
            assert_eq!(correct.value().clone(), item_got.value().clone());
            assert_eq!(correct.label().deepcopy(), item_got.label().deepcopy());
            assert_eq!(correct.filter().clone(), item_got.filter().clone());
            assert_eq!(correct.score().clone(), item_got.score().clone());
        }
    }

    #[test]
    fn test_write_code_to_xml() {
        let code_with_variant = Code::new_unsigned(2, "Yes", None, None);
        code_with_variant
            .label
            .write()
            .add_variant(
                crate::datamap::Variant::new("Oui", Some("interview"), Some("fr")).unwrap(),
            )
            .unwrap();

        let code_with_analysis_variant = Code::new_unsigned(3, "Yes", None, None);
        code_with_analysis_variant
            .label
            .write()
            .add_variant(
                crate::datamap::Variant::new("Agreed", Some("analysis"), None).unwrap(),
            )
            .unwrap();

        let codes = [
            (
                Code::new_unsigned(1, "label", None, None),
                r#"<value code="1">label</value>"#,
            ),
            (
                Code::new(AnyValue::Int32(-3), "special value", None, Some(0.3)),
                r#"<value code="-3" score="0.3">special value</value>"#,
            ),
            (
                Code::new_float(1.5, "decimal value", None, Some(2.5)),
                r#"<value code="1.5" score="2.5">decimal value</value>"#,
            ),
            (
                Code::new_literal("A", "Tea & coffee", None, None),
                r#"<value code="A">Tea &amp; coffee</value>"#,
            ),
            (
                code_with_variant,
                r#"<value code="2">Yes<text mode="interview" xml:lang="fr">Oui</text></value>"#,
            ),
            (
                code_with_analysis_variant,
                r#"<value code="3">Yes<text mode="analysis">Agreed</text></value>"#,
            ),
        ];

        for (code, correct) in codes {
            let mut writer = Writer::new(Vec::new());
            code.write_xml(&mut writer).unwrap();
            let content = String::from_utf8(writer.into_inner()).unwrap();
            assert_eq!(normalize_xml(&content), normalize_xml(correct));
        }
    }

    #[test]
    fn test_write_character_response_success() {
        let response = Response::new(
            Some(DataRange::new(AnyValue::UInt32(0), AnyValue::UInt32(21)).unwrap()),
            Vec::new(),
        );
        let mut writer = Writer::new(Vec::new());
        response.write_xml(&mut writer, QuestionType::Character).unwrap();
        let content = String::from_utf8(writer.into_inner()).unwrap();
        assert_eq!(normalize_xml(&content), normalize_xml("<size>20</size>"));
    }

    #[test]
    fn test_write_character_response_error() {
        let response = Response::new(None, Vec::new());
        let mut writer = Writer::new(Vec::new());
        assert!(response.write_xml(&mut writer, QuestionType::Character).is_err());
    }

    #[test]
    fn test_write_single_and_multiple_response_to_xml() {
        for question_type in [QuestionType::Single, QuestionType::Multiple] {
            let ranges = [
                (None, r#"<values><value code="3">Other</value></values>"#),
                (
                    Some(DataRange::new(AnyValue::UInt32(1), AnyValue::UInt32(2)).unwrap()),
                    r#"<values><range from="1" to="2"/><value code="3">Other</value></values>"#,
                ),
            ];

            for (range, correct) in ranges {
                let response =
                    Response::new(range, vec![Code::new_unsigned(3, "Other", None, None)]);
                let mut writer = Writer::new(Vec::new());
                response.write_xml(&mut writer, question_type.clone()).unwrap();
                let content = String::from_utf8(writer.into_inner()).unwrap();
                assert_eq!(normalize_xml(&content), normalize_xml(correct));
            }
        }
    }

    #[test]
    fn test_write_date_and_time_response_to_xml() {
        let ranges = [
            (
                QuestionType::Date,
                DataRange::new(AnyValue::UInt32(20050101), AnyValue::UInt32(20051231))
                    .unwrap(),
                r#"<values><range from="20050101" to="20051231"/></values>"#,
            ),
            (
                QuestionType::Time,
                DataRange::new(AnyValue::UInt32(90000), AnyValue::UInt32(173000)).unwrap(),
                r#"<values><range from="90000" to="173000"/></values>"#,
            ),
        ];

        for (question_type, range, correct) in ranges {
            let response = Response::new(Some(range), Vec::new());
            let mut writer = Writer::new(Vec::new());
            response.write_xml(&mut writer, question_type).unwrap();
            let content = String::from_utf8(writer.into_inner()).unwrap();
            assert_eq!(normalize_xml(&content), normalize_xml(correct));
        }
    }

    #[test]
    fn test_write_response_without_xml_values() {
        for question_type in [
            QuestionType::None,
            QuestionType::Logical,
            QuestionType::Date,
            QuestionType::Time,
        ] {
            let response = Response::new(None, Vec::new());
            let mut writer = Writer::new(Vec::new());
            response.write_xml(&mut writer, question_type).unwrap();
            assert!(writer.into_inner().is_empty());
        }
    }

    #[test]
    fn test_write_quantity_response_without_special_values_to_xml() {
        const TYPE: QuestionType = QuestionType::Quantity;

        let ranges = [
            (
                DataRange::new(AnyValue::Int32(20050101), AnyValue::Int32(20051231)).unwrap(),
                r#"<values><range from="20050101" to="20051231" /></values>"#,
            ),
            (
                DataRange::new(AnyValue::UInt32(20050101), AnyValue::UInt32(20051231))
                    .unwrap(),
                r#"<values><range from="20050101" to="20051231" /></values>"#,
            ),
            (
                DataRange::new(AnyValue::Float64(32.21), AnyValue::Float64(79.00)).unwrap(),
                r#"<values><range from="32.21" to="79" /></values>"#,
            ),
        ];

        for (range, correct) in ranges.into_iter() {
            let response = Response::new(Some(range), Vec::new());
            let mut writer = Writer::new(Vec::new());
            response.write_xml(&mut writer, TYPE).unwrap();
            let content = String::from_utf8(writer.into_inner()).unwrap();
            assert_eq!(normalize_xml(&content), normalize_xml(correct));
        }
    }

    #[test]
    fn test_write_quantity_response_with_special_values_to_xml() {
        const TYPE: QuestionType = QuestionType::Quantity;
        let ranges =
            [DataRange::new(AnyValue::Int32(20050101), AnyValue::Int32(20051231)).unwrap()];
        let special_values = vec![Code::new(
            AnyValue::Int32(-3),
            "special value",
            None,
            Some(0.3),
        )];
        let special_values_correct = r#"<value code="-3" score="0.3">special value</value>"#;
        let reponses = ranges
            .into_iter()
            .map(|range| {
                (
                    Response::new(Some(range), special_values.clone()),
                    format!(
                        r#"<values><range from="20050101" to="20051231"/>{}</values>"#,
                        special_values_correct
                    ),
                )
            })
            .collect::<Vec<(Response, String)>>();

        for (response, correct) in reponses.into_iter() {
            let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
            response.write_xml(&mut writer, TYPE).unwrap();
            let content = String::from_utf8(writer.into_inner()).unwrap();
            assert_eq!(normalize_xml(&content), normalize_xml(&correct));
        }
    }

    #[test]
    fn test_value_cast_u32() {
        assert_eq!(
            AnyValue::Int32(3).try_strict_cast(&DataType::UInt32).unwrap(),
            AnyValue::UInt32(3)
        );
        assert_eq!(
            AnyValue::UInt32(3).try_strict_cast(&DataType::UInt32).unwrap(),
            AnyValue::UInt32(3)
        );
        assert_eq!(
            AnyValue::StringOwned("3".into()).try_strict_cast(&DataType::UInt32).unwrap(),
            AnyValue::UInt32(3)
        );
        assert_eq!(
            AnyValue::Float64(3.000).try_strict_cast(&DataType::UInt32).unwrap(),
            AnyValue::UInt32(3)
        );

        assert!(AnyValue::Int32(-1).try_strict_cast(&DataType::UInt32).is_err());
        assert_eq!(
            AnyValue::Float64(3.0001).try_strict_cast(&DataType::UInt32).unwrap(),
            AnyValue::UInt32(3)
        );
        assert!(AnyValue::StringOwned("".into()).try_strict_cast(&DataType::UInt32).is_err());
        assert_eq!(
            AnyValue::StringOwned("3.00".into()).try_strict_cast(&DataType::UInt32).unwrap(),
            AnyValue::UInt32(3)
        );
        assert!(
            AnyValue::StringOwned("what".into()).try_strict_cast(&DataType::UInt32).is_err()
        );
        assert!(
            AnyValue::StringOwned("hi".into()).try_strict_cast(&DataType::UInt32).is_err()
        );
    }

    #[test]
    fn test_value_cast_i32() {
        assert_eq!(
            AnyValue::Int32(-3).try_strict_cast(&DataType::Int32).unwrap(),
            AnyValue::Int32(-3)
        );
        assert_eq!(
            AnyValue::Int32(3).try_strict_cast(&DataType::Int32).unwrap(),
            AnyValue::Int32(3)
        );
        assert_eq!(
            AnyValue::UInt32(3).try_strict_cast(&DataType::Int32).unwrap(),
            AnyValue::Int32(3)
        );
        assert_eq!(
            AnyValue::Float64(3.00).try_strict_cast(&DataType::Int32).unwrap(),
            AnyValue::Int32(3)
        );
        assert_eq!(
            AnyValue::StringOwned("3".into()).try_strict_cast(&DataType::Int32).unwrap(),
            AnyValue::Int32(3)
        );
        assert_eq!(
            AnyValue::StringOwned("-3".into()).try_strict_cast(&DataType::Int32).unwrap(),
            AnyValue::Int32(-3)
        );

        assert_eq!(
            AnyValue::Float64(-3.01).try_strict_cast(&DataType::Int32).unwrap(),
            AnyValue::Int32(-3)
        );
        assert_eq!(
            AnyValue::Float64(3.01).try_strict_cast(&DataType::Int32).unwrap(),
            AnyValue::Int32(3)
        );
        assert_eq!(
            AnyValue::Float64(3.01).try_strict_cast(&DataType::Int32).unwrap(),
            AnyValue::Int32(3)
        );
        assert!(AnyValue::StringOwned("".into()).try_strict_cast(&DataType::Int32).is_err());
        assert_eq!(
            AnyValue::StringOwned("3.00".into()).try_strict_cast(&DataType::Int32).unwrap(),
            AnyValue::Int32(3)
        );
        assert!(
            AnyValue::StringOwned("what".into()).try_strict_cast(&DataType::Int32).is_err()
        );
        assert!(AnyValue::StringOwned("hi".into()).try_strict_cast(&DataType::Int32).is_err());
    }

    #[test]
    fn test_value_cast_float() {
        assert_eq!(
            AnyValue::Int32(-3).try_strict_cast(&DataType::Float64).unwrap(),
            AnyValue::Float64(-3.00)
        );
        assert_eq!(
            AnyValue::Int32(3).try_strict_cast(&DataType::Float64).unwrap(),
            AnyValue::Float64(3.00)
        );
        assert_eq!(
            AnyValue::UInt32(3).try_strict_cast(&DataType::Float64).unwrap(),
            AnyValue::Float64(3.00)
        );
        assert_eq!(
            AnyValue::StringOwned("3.0".into()).try_strict_cast(&DataType::Float64).unwrap(),
            AnyValue::Float64(3.00)
        );
        assert_eq!(
            AnyValue::StringOwned("3".into()).try_strict_cast(&DataType::Float64).unwrap(),
            AnyValue::Float64(3.00)
        );
        assert_eq!(
            AnyValue::StringOwned("-3.0".into())
                .try_strict_cast(&DataType::Float64)
                .unwrap(),
            AnyValue::Float64(-3.00)
        );
        assert_eq!(
            AnyValue::Float64(3.1415926).try_strict_cast(&DataType::Float64).unwrap(),
            AnyValue::Float64(3.1415926)
        );
        assert_eq!(
            AnyValue::Float64(-3.1415926).try_strict_cast(&DataType::Float64).unwrap(),
            AnyValue::Float64(-3.1415926)
        );

        assert!(AnyValue::StringOwned("".into()).try_strict_cast(&DataType::Float64).is_err());
        assert!(
            AnyValue::StringOwned("what".into())
                .try_strict_cast(&DataType::Float64)
                .is_err()
        );
        assert!(
            AnyValue::StringOwned("hi".into()).try_strict_cast(&DataType::Float64).is_err()
        );
    }

    #[test]
    fn test_value_cast_string() {
        assert_eq!(
            AnyValue::Int32(-3).try_strict_cast(&DataType::String).unwrap(),
            AnyValue::StringOwned("-3".into())
        );
        assert_eq!(
            AnyValue::UInt32(3).try_strict_cast(&DataType::String).unwrap(),
            AnyValue::StringOwned("3".into())
        );
        assert_eq!(
            AnyValue::Float64(3.5).try_strict_cast(&DataType::String).unwrap(),
            AnyValue::StringOwned("3.5".into())
        );
        assert_eq!(
            AnyValue::StringOwned("hello".into())
                .try_strict_cast(&DataType::String)
                .unwrap(),
            AnyValue::StringOwned("hello".into())
        );
    }

    #[test]
    fn test_cast_type_mutation() {
        let original = DataRange::new(AnyValue::Int32(3), AnyValue::Int32(5)).unwrap();
        let mut cast = original.cast(&DataType::UInt32).unwrap();
        cast.set_start(AnyValue::UInt32(2)).unwrap();

        assert_eq!(original.start().clone(), AnyValue::Int32(3));
    }

    #[test]
    #[allow(unused_mut)]
    fn test_mutate_response_label_text() {
        let original = Response::new(None, vec![Code::new_unsigned(3, "Before", None, None)]);

        let label_map_1 = original.response_label_map();
        let label_copy_1 = label_map_1.get(&AnyValue::UInt32(3)).unwrap();

        let mut label_map_2 = original.response_label_map();
        let mut label_copy_2 = label_map_2.get_mut(&AnyValue::UInt32(3)).unwrap();

        assert_eq!(label_copy_1.read().as_str(), "Before");
        assert_eq!(label_copy_2.read().as_str(), "Before");

        *label_copy_2.write() = "After".into();
        assert_eq!(label_copy_1.read().as_str(), "After");
    }
}
