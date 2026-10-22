#![doc = include_str!("../doc/rs_response.md")]

use std::cmp::Ordering;

use anyhow::Result;
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
#[cfg(feature = "pyo3")]
use pyo3::{
    IntoPyObjectExt,
    exceptions,
    prelude::*,
    types::{PyFloat, PyInt, PySlice, PyString, PyTuple},
};
use quick_xml::{events::BytesText, writer::Writer};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[cfg(feature = "pyo3")]
use crate::utils::{extract_indicies, get_type_name, normalize_py_index};
use crate::{
    member::{QuestionType, Text},
    error::{self, InvalidArg, MissingAttributeError, UnsupportedFeature},
    prelude::DeepCopy,
    utils::{ArcLock, lock_it_up},
};


#[doc = include_str!("../doc/rs_response.md")]
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
    /// use triple_s::response::{Code, DataRange, Response, Value};
    ///
    /// let data_range = DataRange::new(Value::UnSigned(1), Value::UnSigned(2))?;
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
            values: lock_it_up(values.into_par_iter().map(lock_it_up).collect()),
        }
    }

    /// Counts total amount of values
    #[inline]
    pub fn len(&self) -> usize {
        self.values().len()
    }

    /// checks if instance contains any <Code> instance.
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
    /// Raises error if the `code` field of the incoming `Code` instance already exists 
    /// in current instance's `values` (in this case use `set_value` instead).
    /// 
    /// ```
    /// use triple_s::{Response, Code};
    /// 
    /// let mut response = Response::new(
    ///     None,
    ///     Vec::new()
    /// );
    /// // adding a value
    /// response.add_value(Code::new_unsigned(1, "Label A", None, Some(0.5)))?;
    /// 
    /// // cannot add new code with different variant:
    /// assert!(
    ///     response.add_value(Code::new_signed(2, "Label A", None, Some(0.5)))
    ///     .is_err()
    /// );
    /// 
    /// // cannot add new code with same `code` field,
    /// // use `set_value` instaed
    /// assert!(
    ///     response.add_value(Code::new_unsigned(1, "Label B", None, Some(0.5)))
    ///     .is_err()
    /// );
    /// 
    /// Ok::<(), triple_s::error::InvalidArg>(())
    /// ```
    pub fn add_value(&mut self, value: Code) -> Result<(), InvalidArg> {
        let key = value.code();

        if let Some(sample) = self.values.read().iter().next() {
            Self::validate_code_variant(&sample.read().code(), &key)?;
        };

        if self.values.read().iter().any(|item| item.read().code() == key) {
            return Err(InvalidArg::new(format!(
                "Cannot append item '{}' because an item with same 'code' attribute already exists. Use 'set_value' to overwrite existing entry.",
                key.to_string()
            )));
        }
        self.values.write().push(lock_it_up(value));
        Ok(())
    }

    /// Try inserting a new `Code` instance into `Response::values` vec.
    /// Will overwrite existing entries if new `Code::code()` already exists in `Response::values`
    /// 
    /// # Example
    /// ```
    /// use triple_s::{Response, Code};
    /// 
    /// let mut response = Response::new(
    ///     None,
    ///     Vec::new()
    /// );
    /// // adding a value
    /// response.set_value(Code::new_unsigned(1, "Label A", None, Some(0.5)))?;
    /// println!("{:?}", response.values());
    /// 
    /// // cannot add new code with different variant:
    /// assert!(
    ///     response.set_value(Code::new_signed(2, "Label A", None, Some(0.5)))
    ///     .is_err()
    /// );
    /// 
    /// // overwrites existing item in `Response::values`,
    /// response.set_value(Code::new_unsigned(1, "Label B", None, Some(0.5)))?;
    /// println!("{:?}", response.values());
    /// 
    /// Ok::<(), triple_s::error::InvalidArg>(())
    /// ```
    pub fn set_value(&mut self, value: Code) -> Result<(), InvalidArg> {
        let key = value.code();
        // needs to clone these fields outside of the for loop,
        // or it would deadlock the loop (label() etc., acquires a read lock
        // whilst xx_mut() would acquire a write loop of the same parent field.)
        let label = value.label().clone();
        let filter = value.filter();
        let score = value.score();

        if let Some(sample) = self.values.read().iter().next() {
            Self::validate_code_variant(&sample.read().code(), &key)?;
        };

        for current in self.values.write().iter_mut() {
            let mut current = current.write();
            if current.code() == key {
                *current.label_mut() = label;
                *current.filter_mut() = filter;
                *current.score_mut() = score;
                return Ok(());
            }
        }

        self.values.write().push(lock_it_up(value));
        Ok(())
    }

    fn get_value_(&self, index: usize) -> Option<ArcLock<Code>> {
        self.values.read().get(index).cloned()
    }

    /// Reads a response value at the given index, just to make accessing particular item in the `values` a bit easier
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
            .with_attribute(("from", range.start().to_string().as_str()))
            .with_attribute(("to", range.end().to_string().as_str()))
            .write_empty()?;
        Ok(())
    }

    fn write_size_xml<W: std::io::Write>(
        range: &DataRange,
        writer: &mut Writer<W>,
    ) -> Result<()> {
        let size_value = match range.end().clone() {
            Value::UnSigned(val) => val.saturating_sub(1),
            _ => {
                return Err(error::InternalError::new(format!(
                    "Expecting `Value::UnSigned, got: {:?}",
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
    /// Mostly used internally `to_xml`` method calls, exposed as part of public just in case.
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
        sample: &Value,
        incoming: &Value,
    ) -> Result<(), InvalidArg> {
        if !sample.is_same_variant(incoming) {
            Err(InvalidArg::new(format!(
                "Code value must have the same Value variant as existing codes; got {incoming:?}, existing {sample:?}"
            )))
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Response {
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

    /// get `Code` inside `self.values` with `code.code` given.
    /// 
    /// Returns `None` if the input `code` was not found.
    #[pyo3(name = "get_code", signature = (code: "str|int|float"))]
    fn py_get_code<'py>(&self, code: &Bound<'py, PyAny>) -> PyResult<Option<Code>> {
        if !code.is_instance_of::<PyString>()
            && !code.is_instance_of::<PyInt>()
            && !code.is_instance_of::<PyFloat>()
        {
            return Err(exceptions::PyTypeError::new_err(
                "code must be str, int, or float",
            ));
        }

        for item in self.values.read().iter() {
            let item = item.read();
            let stored = item.code().to_py_any(code.py());

            if stored.eq(code)? {
                return Ok(Some(item.clone()));
            }
        }

        Ok(None)
    }

    /// Append a code, rejecting duplicate values and different Value variants.
    #[pyo3(name = "add_value")]
    fn py_add_value(&mut self, value: Code) -> PyResult<()> {
        self.add_value(value)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:?}")))
    }

    /// Update or append a code, rejecting different Value variants.
    /// Will overwrite existing code if they share the same 'code' attribute.
    #[pyo3(name = "set_value")]
    fn py_set_value(&mut self, value: Code) -> PyResult<()> {
        self.set_value(value)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:?}")))
    }
}

/// Corresponds to the `<value>` tag of a triple-s xml.
#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Code", module = "triple_s", from_py_object)
)]
#[derive(Debug)]
pub struct Code {
    /// code value, this can be String, Float or u32
    pub(crate) code: ArcLock<Value>,
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
            code: self.code.clone(),
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
            code: lock_it_up(self.code.read().clone()),
            label: lock_it_up(self.label.read().clone()),
            filter: lock_it_up(self.filter.read().clone()),
            score: lock_it_up(*self.score.read()),
        }
    }
}

impl PartialEq for Code {
    fn eq(&self, other: &Self) -> bool {
        (self.code() == other.code())
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
    /// use triple_s::response::{Code, Value};
    ///
    /// let code = Code::new_literal("A", "Tea & coffee", None, None);
    /// assert_eq!(code.code(), Value::String("A".into()))
    /// ```
    ///
    /// Note that mutating the item returned from `code()` method does would not
    /// update the original code value. To set new value, call `code_mut()`
    /// instead.
    ///
    /// # Example
    /// ```rust
    /// use triple_s::response::{Code, Value};
    ///
    /// let mut example = Code::new_literal("A", "Tea & coffee", None, None);
    /// let mut example_code = example.code();
    ///
    /// match &mut example_code {
    ///     Value::String(item) => *item = "B".to_string(),
    ///     _ => (),
    /// };
    /// assert_eq!(example_code, Value::String("B".into()));
    /// assert_ne!(example_code, example.code());
    /// ```
    pub fn code(&self) -> Value {
        self.code.read().clone()
    }

    /// Returns a mutable `code` field for updating it.
    ///
    /// # Example
    /// ```rust
    /// use triple_s::response::{Code, Value};
    ///
    /// let mut example = Code::new_literal("A", "Tea & coffee", None, None);
    ///
    /// let new_code = Value::UnSigned(32);
    /// *example.code_mut() = new_code.clone();
    /// assert_eq!(example.code(), new_code);
    /// ```
    pub fn code_mut(&mut self) -> RwLockWriteGuard<'_, Value> {
        self.code.write()
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
    /// use triple_s::response::{Code, Value};
    ///
    /// let code = Code::new(Value::Signed(-3), "special value", None, Some(0.3));
    /// assert_eq!(code.code(), Value::Signed(-3));
    /// ```
    ///
    /// adding text variants:
    ///
    /// ```
    /// use triple_s::{Code, Value, Variant};
    ///
    /// let mut code = Code::new(Value::Signed(-3), "special value", None, Some(0.3));
    /// code.label_mut()
    ///     .add_variant(Variant::new("variant", Some("interview"), None)?)?;
    ///
    /// Ok::<_, anyhow::Error>(())
    /// ```
    pub fn new(
        value: Value,
        label: impl Into<String>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Self {
        Self {
            code: lock_it_up(value),
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
            code: lock_it_up(Value::UnSigned(value)),
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
            code: lock_it_up(Value::Signed(value)),
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
            code: lock_it_up(Value::Float(value)),
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
            code: lock_it_up(Value::String(value.into())),
            label: lock_it_up(Text::new(label)),
            filter: lock_it_up(filter.map(|item| item.into())),
            score: lock_it_up(score),
        }
    }

    /// Format as triple-s classic string
    /// Fields incompatible to triple-s classic will be ignored
    pub fn to_classic(&self) -> Result<String> {
        Ok(format!(
            "{} \"{}\"",
            self.code.read().to_classic()?,
            self.label.read().to_classic(),
        ))
    }

    pub(crate) fn write_xml<W: std::io::Write>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), std::io::Error> {
        let code = self.code().to_string();
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
            Value::UnSigned(value),
            label.unwrap_or_default(),
            filter,
            score,
        ))
    }

    /// Used for questions of type 'Single/ Multiple/ Date/ Time'
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
            Value::String(value),
            label.unwrap_or_default(),
            filter,
            score,
        ))
    }

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
            Value::Float(value),
            label.unwrap_or_default(),
            filter,
            score,
        ))
    }

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
            Value::Signed(value),
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
            format!("code={}", self.code().to_string()),
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
    #[getter(code)]
    fn py_get_code<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        self.code.read().to_py_any(py)
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
}

/// Used for recording `code` attribute of triple-s ``<value>``` tag, as well as
/// `from` `to` of `\<range\>`` tag.
///
/// - Unsigned: used in Single/ Multiple/ Quantity question types.
/// - Signed: used in Quantity/ Date/ Time question types.
/// - Float: used in Quantity question type.
/// - String: used in Single question type, introduced from triple-s 2.0 onwards
///
/// # Example
/// ```rust
/// use triple_s::response::Value;
///
/// let value = Value::Float(32.0);
/// ```
#[cfg_attr(
    feature = "pyo3",
    pyclass(name = "Value", module = "triple_s", from_py_object)
)]
#[derive(Debug, Clone)]
pub enum Value {
    /// unsigned int (used for single, mutiple, date and time question types)
    UnSigned(u32),
    /// signed int (used only for quantity qurstion type)
    Signed(i32),
    /// float (used only for quantity qurstion type)
    Float(f64),
    /// string (used only for triple-s 2.0 onwards for question type `Single`)
    String(String),
}

impl Value {
    /// update content of current instance based on incoming `other`,
    /// raises error if `Self` and `Other` are of different variants.
    ///
    /// # Example
    /// ```rust
    /// use triple_s::response::Value;
    ///
    /// let mut value = Value::Float(32.1);
    ///
    /// assert!(value.set_from(Value::Signed(13)).is_err());
    ///
    /// value.set_from(Value::Float(2.5))?;
    /// assert_eq!(value, Value::Float(2.5));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn set_from(&mut self, other: Self) -> Result<(), InvalidArg> {
        match (&mut *self, &other) {
            (Value::Float(slf), Value::Float(oth)) => {
                other.validate()?;
                *slf = *oth;
                Ok(())
            }
            (Value::Signed(slf), Value::Signed(oth)) => {
                *slf = *oth;
                Ok(())
            }
            (Value::UnSigned(slf), Value::UnSigned(oth)) => {
                *slf = *oth;
                Ok(())
            }
            (Value::String(slf), Value::String(oth)) => {
                *slf = oth.clone();
                Ok(())
            }
            _ => Err(InvalidArg::new(format!(
                "Cannot update value from instances of a different variant! Self: [{:?}], other: [{:?}]",
                self, other
            ))),
        }
    }

    /// Trying to parse string into the same variant of `self`.
    /// This is mainly used for parsing pairs of <SOME_TAG from=".." to="..">
    ///
    /// # Example
    /// ```rust
    /// use triple_s::response::Value;
    ///
    /// let instance = Value::UnSigned(32);
    ///
    /// let float_text = "12.5";
    /// assert!(instance.parse_like(float_text).is_err());
    ///
    /// let string_text = "some text";
    /// assert!(instance.parse_like(string_text).is_err());
    ///
    /// let signed_text = "-3";
    /// assert!(instance.parse_like(signed_text).is_err());
    ///
    /// let unsigned_text = "3";
    /// assert_eq!(
    ///     instance.parse_like(unsigned_text).unwrap(),
    ///     Value::UnSigned(3)
    /// )
    /// ```
    pub fn parse_like(&self, text: &str) -> Result<Self, InvalidArg> {
        match self {
            Self::Float(_) => {
                let value = text.parse::<f64>().map_err(|err| {
                    InvalidArg::new(format!(
                        "Failed to parse value '{}' as float decimal: '{}'",
                        text, err
                    ))
                })?;
                let instance = Self::Float(value);
                instance.validate()?; // value should not be float NaN.
                Ok(instance)
            }

            Self::Signed(_) => {
                let value = text.parse::<i32>().map_err(|err| {
                    InvalidArg::new(format!(
                        "Failed to parse value '{}' as integer: '{}'",
                        text, err
                    ))
                })?;

                Ok(Self::Signed(value))
            }

            Self::UnSigned(_) => {
                let value = text.parse::<u32>().map_err(|err| {
                    InvalidArg::new(format!(
                        "Failed to parse value '{}' as integer: '{}'",
                        text, err
                    ))
                })?;
                Ok(Self::UnSigned(value))
            }

            Self::String(_) => Ok(Self::String(text.to_owned())),
        }
    }

    /// Basically calling `to_string()` for the data inside but does not allow
    /// `String` type, since triple-s classic does not support `code`
    /// attribute as strings.
    pub fn to_classic(&self) -> Result<String> {
        match self {
            Self::UnSigned(value) => Ok(value.to_string()),
            Self::Signed(value) => Ok(value.to_string()),
            Self::Float(value) => Ok(value.to_string()),
            Self::String(_) => {
                Err(UnsupportedFeature::new("SSS Classic", "None numeric code values.").into())
            }
        }
    }

    /// convert the data inside to its string representation.
    #[allow(clippy::inherent_to_string)]
    pub(crate) fn to_string(&self) -> String {
        match self {
            Value::Float(item) => item.to_string(),
            Value::Signed(item) => item.to_string(),
            Value::UnSigned(item) => item.to_string(),
            Value::String(item) => item.clone(),
        }
    }

    /// check if (self, other) are the same variant
    pub(crate) fn is_same_variant(&self, other: &Value) -> bool {
        matches!(
            (self, other),
            (Value::Float(_), Value::Float(_))
                | (Value::Signed(_), Value::Signed(_))
                | (Value::UnSigned(_), Value::UnSigned(_))
                | (Value::String(_), Value::String(_))
        )
    }

    /// Rasies error if value ws float nan or infinity
    pub fn validate(&self) -> Result<(), InvalidArg> {
        match self {
            Self::Float(value) => {
                if value.is_nan() {
                    Err(InvalidArg::new("'Value' cannot be float `Nan`!"))
                } else if value.is_infinite() {
                    Err(InvalidArg::new("'Value' cannot be infinite!"))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    /// Try to see if `other` was bigger/ smaller/ equal to `self`.
    /// This method would only compare between pairs of float, signed int or
    /// unsigned int instances, and would raise error when:
    /// - comparing between two strings.
    /// - comparing between float Nan values.
    /// - comparing between to variants of two different types.
    ///
    /// ## Working Example
    ///
    /// ```
    /// use std::cmp::Ordering;
    ///
    /// use triple_s::Value;
    ///
    /// let a = Value::Float(2.5);
    /// let b = Value::Float(3.5);
    ///
    /// let result = a.try_cmp(&b)?;
    /// assert_eq!(result, Ordering::Less);
    ///
    /// Ok::<(), triple_s::error::InvalidArg>(())
    /// ```
    ///
    /// ## Errors
    ///
    /// 1. This method only compares values of the same kind, and will raise
    ///    error if they were not.
    ///
    /// ```
    /// use triple_s::Value;
    ///
    /// let a = Value::UnSigned(2);
    /// let b = Value::Signed(3);
    ///
    /// let result = a.try_cmp(&b);
    /// assert!(result.is_err());
    /// ```
    ///
    /// 2. This method does not compare between two string variants.
    ///
    /// ```
    /// use triple_s::Value;
    ///
    /// let a = Value::String("1".to_string());
    /// let b = Value::String("2".to_string());
    ///
    /// let result = a.try_cmp(&b);
    /// assert!(result.is_err());
    /// ```
    ///
    /// 3. float Nan would raise error
    ///
    /// ```
    /// use triple_s::Value;
    ///
    /// let a = Value::Float(f64::NAN);
    /// let b = Value::Float(3.2);
    ///
    /// let result = a.try_cmp(&b);
    /// assert!(result.is_err());
    /// ```
    pub fn try_cmp(&self, other: &Self) -> Result<Ordering, InvalidArg> {
        if self.is_same_variant(other) {
            match (self, other) {
                (Self::UnSigned(a), Self::UnSigned(b)) => Ok(a.cmp(b)),
                (Self::Signed(a), Self::Signed(b)) => Ok(a.cmp(b)),

                // - checks if any of them were float nan, if so then raises error
                // - then compares the value behind and returns ordering.
                (Self::Float(a), Self::Float(b)) => {
                    self.validate()?;
                    other.validate()?;
                    a.partial_cmp(b).ok_or(InvalidArg::new("Cannot compare float NaN values."))
                }

                (Self::String(_), Self::String(_)) => Err(InvalidArg::new(
                    "Cannot compare values between `string` values.",
                )),
                _ => Err(InvalidArg::new(format!(
                    "Can only compare pairs of `Value` instances of the same kind. Got: self: [{:?}] | other: [{:?}]",
                    self, other
                ))),
            }
        } else {
            Err(InvalidArg::new(format!(
                "Can only compare pairs of `Value` instances of the same kind. Got: self: [{:?}] | other: [{:?}]",
                self, other
            )))
        }
    }

    /// maps current instance into corresponding python objects
    #[cfg(feature = "pyo3")]
    pub(crate) fn to_py_any<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        match self {
            Value::String(item) => PyString::new(py, item).into_any(),
            Value::Float(item) => PyFloat::new(py, item.clone()).into_any(),
            Value::Signed(item) => PyInt::new(py, item).into_any(),
            Value::UnSigned(item) => PyInt::new(py, item).into_any(),
        }
    }
}

#[cfg(feature = "pyo3")]
impl Value {
    fn __is_same_type__(&self, other: &Value) -> bool {
        self.is_same_variant(other)
    }

    fn __data_type__(&self) -> String {
        match self {
            Self::Float(_) => "float".into(),
            Self::Signed(_) => "signed".into(),
            Self::UnSigned(_) => "unsigned".into(),
            Self::String(_) => "string".into(),
        }
    }

    fn __eq__(&self, other: &Value) -> PyResult<bool> {
        if self.is_same_variant(other) {
            let result = self.try_cmp(other)?;
            Ok(matches!(result, Ordering::Equal))
        } else {
            Err(exceptions::PyTypeError::new_err(format!(
                "Cannot compare values of different internal types. Tried to compare between: ['{}', '{}']",
                self.__data_type__(),
                other.__data_type__()
            )))
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::UnSigned(self_), Value::UnSigned(other_)) => self_ == other_,
            (Value::Signed(self_), Value::Signed(other_)) => self_ == other_,
            (Value::String(self_), Value::String(other_)) => self_ == other_,
            (Value::Float(self_), Value::Float(other_)) => {
                self_ == other_ || (self_.is_nan() && other_.is_nan())
            }
            _ => false,
        }
    }
}

impl Eq for Value {}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::UnSigned(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Signed(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
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
    feature = "pyo3",
    pyclass(name = "DataRange", module = "triple_s", from_py_object)
)]
#[derive(Debug)]
pub struct DataRange {
    start: ArcLock<Value>,
    end: ArcLock<Value>,
}

impl DataRange {
    /// Creates new `DataRange` instance with start and end values.
    ///
    /// # Example:
    ///
    /// ```
    /// use triple_s::{DataRange, Value};
    ///
    /// let range = DataRange::new(Value::Signed(-1), Value::Signed(3))?;
    /// assert_eq!(range.start().clone(), Value::Signed(-1));
    ///
    /// Ok::<(), triple_s::error::InvalidArg>(())
    /// ```
    ///
    ///  This method will raise error if `start`, `end` are not the same data
    /// type, or if `start` was greater than `end`:
    ///
    /// ```
    /// use triple_s::{DataRange, Value};
    ///
    /// // `start` needs to be lower or equal to `end`:
    /// assert!(DataRange::new(Value::Signed(100), Value::Signed(0)).is_err());
    ///
    /// // cannot create new instance of two different value types:
    /// assert!(DataRange::new(Value::Float(10.0), Value::Signed(0)).is_err())
    ///
    /// // cannot create new instance with `string` value:
    /// ```
    pub fn new(start: Value, end: Value) -> Result<Self, InvalidArg> {
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
    pub fn start(&self) -> RwLockReadGuard<'_, Value> {
        self.start.read()
    }

    /// A read-only copy of the `end` value (the `to` attribute of the \<range\>
    /// tag.)
    pub fn end(&self) -> RwLockReadGuard<'_, Value> {
        self.end.read()
    }

    /// Update values of `start`.
    ///
    /// # Example
    ///
    /// ```
    /// use triple_s::{DataRange, Value};
    ///
    /// let mut data_range = DataRange::new(Value::Float(-1.5), Value::Float(1.5))?;
    ///
    /// data_range.set_start(Value::Float(1.2))?;
    /// assert_eq!(data_range.start().clone(), Value::Float(1.2));
    ///
    /// // 'start' cannot be larger than 'end'
    /// assert!(data_range.set_start(Value::Float(3_000.0)).is_err());
    ///
    /// Ok::<(), triple_s::error::InvalidArg>(())
    /// ```
    pub fn set_start(&mut self, new: Value) -> Result<(), InvalidArg> {
        let ordering = new.try_cmp(&self.end.read())
            .map_err(|_| InvalidArg::new(format!(
                "Can only update `start` from variants of the same kind. Got: self: [{:?}] | other: [{:?}]",
                self.start.read(), new
            )))?;

        match ordering {
            Ordering::Greater => Err(InvalidArg::new(format!(
                "New `start` value ('{}') cannot be greater than current `end` value ('{}')",
                new.to_string(),
                self.end.read().to_string()
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
    /// use triple_s::{DataRange, Value};
    ///
    /// let mut data_range = DataRange::new(Value::Float(-1.5), Value::Float(1.5))?;
    ///
    /// data_range.set_end(Value::Float(2.2))?;
    /// assert_eq!(data_range.end().clone(), Value::Float(2.2));
    ///
    /// // 'end' cannot be smaller than 'start'
    /// assert!(data_range.set_end(Value::Float(-300000.0)).is_err());
    ///
    /// Ok::<(), triple_s::error::InvalidArg>(())
    /// ```
    pub fn set_end(&mut self, new: Value) -> Result<(), InvalidArg> {
        let ordering = new.try_cmp(&self.start.read())
            .map_err(|_| InvalidArg::new(format!(
                "Can only update `end` from variants of the same kind. Got: self: [{:?}] | other: [{:?}]",
                self.end.read(), new
            )))?;

        match ordering {
            Ordering::Less => Err(InvalidArg::new(format!(
                "New `end` value ('{}') cannot be less than current `start` value ('{}')",
                new.to_string(),
                self.start.read().to_string()
            ))),
            _ => {
                *self.end.write() = new;
                Ok(())
            }
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl DataRange {
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
            self.start().to_string(),
            self.end().to_string(),
        )
    }

    fn __eq__(&self, other: &Self) -> PyResult<bool> {
        Ok((self.start.read().__eq__(&other.start.read())?)
            & (self.end.read().__eq__(&other.end.read())?))
    }

    /// creates a new instance with values being exclusively positive integers.
    /// This is used for questions of 'single', 'multiple', and 'date/ time'
    /// types.
    #[staticmethod]
    fn unsigned(start: u32, end: u32) -> PyResult<Self> {
        Self::new(Value::UnSigned(start), Value::UnSigned(end)).map_err(|err| err.into())
    }

    /// creates a new instance with values being integers that can either be
    /// positive/ negative. This is used for questions of 'quantity' type
    /// only.
    #[staticmethod]
    fn signed(start: i32, end: i32) -> PyResult<Self> {
        Self::new(Value::Signed(start), Value::Signed(end)).map_err(|err| err.into())
    }

    /// creates a new instance with values being decimals.
    /// This is used for questions of 'quantity' type only.
    #[staticmethod]
    fn float(start: f64, end: f64) -> PyResult<Self> {
        Self::new(Value::Float(start), Value::Float(end)).map_err(|err| err.into())
    }

    /// Start of the data range.
    #[getter(start)]
    fn py_get_start<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        self.start.read().to_py_any(py)
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
    fn py_get_end<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        self.end.read().to_py_any(py)
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
    use crate::member::test::normalize_xml;

    #[test]
    fn test_value_float_parse_like() {
        let base = Value::Float(3.5);

        assert_eq!(base.parse_like("3.2").unwrap(), Value::Float(3.2));
        assert_eq!(base.parse_like("-3.2").unwrap(), Value::Float(-3.2));
        assert_eq!(base.parse_like("3").unwrap(), Value::Float(3.0));
        assert_eq!(base.parse_like("-3").unwrap(), Value::Float(-3.0));

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
            assert_eq!(correct.code().clone(), item_got.code().clone());
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
                crate::member::Variant::new("Oui", Some("interview"), Some("fr")).unwrap(),
            )
            .unwrap();

        let code_with_analysis_variant = Code::new_unsigned(3, "Yes", None, None);
        code_with_analysis_variant
            .label
            .write()
            .add_variant(
                crate::member::Variant::new("Agreed", Some("analysis"), None).unwrap(),
            )
            .unwrap();

        let codes = [
            (
                Code::new_unsigned(1, "label", None, None),
                r#"<value code="1">label</value>"#,
            ),
            (
                Code::new(Value::Signed(-3), "special value", None, Some(0.3)),
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
            Some(DataRange::new(Value::UnSigned(0), Value::UnSigned(21)).unwrap()),
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
                    Some(DataRange::new(Value::UnSigned(1), Value::UnSigned(2)).unwrap()),
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
                DataRange::new(Value::UnSigned(20050101), Value::UnSigned(20051231)).unwrap(),
                r#"<values><range from="20050101" to="20051231"/></values>"#,
            ),
            (
                QuestionType::Time,
                DataRange::new(Value::UnSigned(90000), Value::UnSigned(173000)).unwrap(),
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
                DataRange::new(Value::Signed(20050101), Value::Signed(20051231)).unwrap(),
                r#"<values><range from="20050101" to="20051231" /></values>"#,
            ),
            (
                DataRange::new(Value::UnSigned(20050101), Value::UnSigned(20051231)).unwrap(),
                r#"<values><range from="20050101" to="20051231" /></values>"#,
            ),
            (
                DataRange::new(Value::Float(32.21), Value::Float(79.00)).unwrap(),
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
            [DataRange::new(Value::Signed(20050101), Value::Signed(20051231)).unwrap()];
        let special_values = vec![Code::new(
            Value::Signed(-3),
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
}
