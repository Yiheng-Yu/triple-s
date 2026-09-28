use std::str::FromStr;

use anyhow::{Context, Result};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::*};

#[cfg(feature = "pyo3")]
use crate::utils::{get_type_name, py_value_error, single_quote};
use crate::{
    EventLog,
    entities::{CodeFormat, QuestionType, Response, Text, Usage, Value},
    error::{self, RangeError},
    prelude::{DeepCopy, *},
    utils::{ArcLock, lock_it_up},
};

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Question", from_py_object)
)]
#[derive(Debug)]
pub struct Question {
    index: u32,
    name: ArcLock<String>,
    label: Option<Text>,
    filter: ArcLock<Option<String>>,
    format: ArcLock<Option<CodeFormat>>,
    usage: ArcLock<Option<Usage>>,
    question_type: ArcLock<QuestionType>,
    location: Location,
    spread: Option<Spread>,
    response: Response,
}

impl Question {
    pub fn index(&self) -> &u32 {
        &self.index
    }

    pub fn index_mut(&mut self) -> &mut u32 {
        &mut self.index
    }

    pub fn label(&self) -> &Option<Text> {
        &self.label
    }

    pub fn label_mut(&mut self) -> &mut Option<Text> {
        &mut self.label
    }

    pub fn location(&self) -> &Location {
        &self.location
    }

    pub fn location_mut(&mut self) -> &mut Location {
        &mut self.location
    }

    pub fn spread(&self) -> &Option<Spread> {
        &self.spread
    }

    pub fn spread_mut(&mut self) -> &mut Option<Spread> {
        &mut self.spread
    }

    pub fn response(&self) -> &Response {
        &self.response
    }

    pub fn response_mut(&mut self) -> &mut Response {
        &mut self.response
    }

    pub fn set_response(&mut self, response: Response) {
        self.response = response;
    }

    pub fn set_name(&self, value: impl Into<String>) {
        *self.name.write() = value.into();
    }

    pub fn set_filter(&self, value: Option<String>) {
        *self.filter.write() = value;
    }

    pub fn set_format(&self, value: Option<CodeFormat>) {
        *self.format.write() = value;
    }

    pub fn set_usage(&self, value: Option<Usage>) {
        *self.usage.write() = value;
    }

    pub fn set_question_type(&self, value: QuestionType) {
        *self.question_type.write() = value;
    }

    pub fn name(&self) -> RwLockReadGuard<'_, String> {
        self.name.read()
    }

    pub fn name_mut(&mut self) -> RwLockWriteGuard<'_, String> {
        self.name.write()
    }

    pub fn filter(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.filter.read()
    }

    pub fn filter_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.filter.write()
    }

    pub fn format(&self) -> RwLockReadGuard<'_, Option<CodeFormat>> {
        self.format.read()
    }

    pub fn format_mut(&mut self) -> RwLockWriteGuard<'_, Option<CodeFormat>> {
        self.format.write()
    }

    pub fn usage(&self) -> RwLockReadGuard<'_, Option<Usage>> {
        self.usage.read()
    }

    pub fn usage_mut(&mut self) -> RwLockWriteGuard<'_, Option<Usage>> {
        self.usage.write()
    }

    pub fn question_type(&self) -> RwLockReadGuard<'_, QuestionType> {
        self.question_type.read()
    }

    pub fn question_type_mut(&mut self) -> RwLockWriteGuard<'_, QuestionType> {
        self.question_type.write()
    }
}

impl Clone for Question {
    /// creates a shallow copy of the current instance (only clones the pointer,
    /// not the data it points to.)
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            name: self.name.clone(),
            label: self.label.clone(),
            filter: self.filter.clone(),
            format: self.format.clone(),
            usage: self.usage.clone(),
            question_type: self.question_type.clone(),
            location: self.location.clone(),
            spread: self.spread.clone(),
            response: self.response.clone(),
        }
    }
}

impl Default for Question {
    fn default() -> Self {
        Self::new("", QuestionType::None, None, None, None, None, None, None).unwrap()
    }
}

impl DeepCopy for Question {
    fn deepcopy(&self) -> Self {
        Self {
            index: self.index,
            name: self.name.deepcopy(),
            label: self.label.deepcopy(),
            filter: self.filter.deepcopy(),
            format: self.format.deepcopy(),
            usage: self.usage.deepcopy(),
            question_type: lock_it_up(self.question_type.read().clone()),
            location: self.location.deepcopy(),
            spread: self.spread.as_ref().map(|item| item.deepcopy()),
            response: self.response.deepcopy(),
        }
    }
}

impl Question {
    pub fn from_string(input: &str) -> Result<Self> {
        let is_classic = !input.trim().starts_with('<');
        let stream = input.lines().map(|line| Ok::<_, error::BadLine>(line.to_owned()));
        let builder = crate::read::classic::collect_question_stream(stream, 0)?;

        let (instance, warnings) = builder.build(is_classic)?;
        let event_log = EventLog { warnings };
        if let Some(warning) = event_log.warning_summary("Deserialising Question") {
            log::warn!("{}", warning)
        };

        Ok(instance)
    }

    pub fn to_sss_classic(&self) -> Result<String> {
        let name = self.name.read().clone();
        let mut output = format!("VARIABLE {}\nNAME \"{name}\"\n", self.index);
        if let Some(label) = &self.label {
            output.push_str(&format!("LABEL \"{}\"\n", label.to_sss_classic()));
        }
        output.push_str(&self.location.to_sss_classic());
        output.push('\n');

        let question_type = self.question_type.read().clone();
        match question_type {
            QuestionType::Character => {
                output.push_str("TYPE CHARACTER\n");
                if let Some(size_range) = self.response.range().as_ref() {
                    match size_range.end {
                        Value::UnSigned(end) => {
                            let size_string = format!("SIZE {}\n", end.saturating_sub(1));
                            output.push_str(&size_string);
                        }
                        _ => {
                            let error = error::InternalError::new(format!(
                                "Unexpected `Value` variant find in question [{}]: '{:?}'",
                                self.name(),
                                size_range.end.clone()
                            ));
                            return Err(error.into());
                        }
                    }
                }
            }
            QuestionType::Logical => output.push_str("TYPE LOGICAL\n"),
            QuestionType::Single | QuestionType::Multiple | QuestionType::Quantity => {
                if question_type == QuestionType::Single
                    && self.format.read().as_ref() == Some(&CodeFormat::Literal)
                {
                    return Err(error::UnsupportedFeature::new(
                        "SSS Classic",
                        "None numeric code values.",
                    )
                    .into());
                }
                if question_type == QuestionType::Multiple
                    && let Some(spread) = &self.spread
                {
                    output.push_str(&spread.to_sss_classic());
                    output.push('\n');
                }
                output.push_str(&format!(
                    "TYPE {}\n",
                    question_type.to_string().to_uppercase()
                ));
                output.push_str(
                    &self
                        .response
                        .to_sss_classic(&question_type)
                        .with_context(|| format!("Question '{name}'"))?,
                );
            }
            QuestionType::None | QuestionType::Date | QuestionType::Time => {
                return Err(error::UnsupportedFeature::new(
                    "classic",
                    format!("{question_type:?}"),
                )
                .into());
            }
        }
        output.push_str(&format!("END VARIABLE {}\n", self.index));
        Ok(output)
    }

    pub fn new(
        name: &str,
        question_type: QuestionType,
        label: Option<&str>,
        filter: Option<&str>,
        format: Option<&str>,
        usage: Option<&str>,
        location: Option<Location>,
        spread: Option<Spread>,
    ) -> Result<Self> {
        let name = name.to_string();
        let label = label.map(Text::new);
        let filter = lock_it_up(filter.map(|s| s.to_string()));
        let format = format.map(CodeFormat::from_str).transpose().with_context(|| {
            format!(
                "Expect value to be one of: {:?} or None",
                CodeFormat::available_options()
            )
        })?;
        let format = lock_it_up(format);

        let usage = usage.map(Usage::from_str).transpose().with_context(|| {
            format!(
                "Expect value to be one of: {:?} or None",
                Usage::available_options()
            )
        })?;
        let usage = lock_it_up(usage);

        // let location = lock_it_up(location.unwrap_or_default());
        let location = location.unwrap_or_default();

        let instance = Self {
            index: 0,
            name: lock_it_up(name),
            label,
            filter,
            format,
            usage,
            question_type: lock_it_up(question_type),
            location,
            spread,
            response: Response::default(),
        };

        Ok(instance)
    }

    pub fn get_format_name(&self) -> Option<String> {
        self.format.read().as_ref().map(|item| item.to_string())
    }

    pub fn set_format_from_str(&self, value: Option<&str>) -> Result<()> {
        match value {
            None => {
                self.set_format(None);
                Ok(())
            }
            Some(format) => {
                let format = CodeFormat::from_str(format).with_context(|| {
                    format!(
                        "Expected to be `None` or one of: {:?}",
                        CodeFormat::available_options()
                    )
                })?;
                self.set_format(Some(format));
                Ok(())
            }
        }
    }

    pub fn set_usage_from_str(&self, value: Option<&str>) -> Result<()> {
        match value {
            None => {
                self.set_usage(None);
                Ok(())
            }
            Some(usage) => {
                let usage = Usage::from_str(usage).with_context(|| {
                    format!(
                        "Expected to be `None` or one of: {:?}",
                        Usage::available_options()
                    )
                })?;
                self.set_usage(Some(usage));
                Ok(())
            }
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Question {
    #[new]
    #[pyo3(signature=(
        name:"str", 
        question_type:"str|QuestionType",
        label:"str"=None, 
        filter:"str"=None, 
        format:"'numeric'|'literal'|None"=None,
        usage:"'serial'|'weight'|None"=None,
        location=None,
        spread=None,
    ))]
    fn py_new<'py>(
        name: &str,
        question_type: &Bound<'py, PyAny>,
        label: Option<&str>,
        filter: Option<&str>,
        format: Option<&str>,
        usage: Option<&str>,
        location: Option<Location>,
        spread: Option<Spread>,
    ) -> PyResult<Self> {
        let question_type = if let Ok(value) = question_type.extract::<QuestionType>() {
            value
        } else if let Ok(value) = question_type.extract::<&str>() {
            QuestionType::from_str(value)
                .with_context(|| format!("Expect 'question_type' to be one of the `QuestionType` flags, or one of: {:?}", QuestionType::available_options()))
                .with_context(||format!("When initializing question: '{}'", name))
                .map_err(py_value_error)?
        } else {
            return Err(exceptions::PyTypeError::new_err(get_type_name(
                question_type,
            )?));
        };

        Self::new(
            name,
            question_type,
            label,
            filter,
            format,
            usage,
            location,
            spread,
        )
        .map_err(py_value_error)
    }

    fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<String> {
        let mut info = Vec::new();
        info.push(("name", single_quote(self.name())));
        info.push((
            "question_type",
            single_quote(self.question_type.read().as_ref()),
        ));

        let label_repl =
            self.label.as_ref().map_or(Ok("None".into()), |label| label.__repr__(py))?;

        info.push(("label", label_repl));
        let formatted = info
            .into_iter()
            .map(|(field, value)| format!("{}={}", field, value))
            .collect::<Vec<String>>()
            .join(",\n  ");

        Ok(format!("Question(\n  {}\n)", formatted))
    }

    fn __copy__(&self) -> Self {
        self.deepcopy()
    }

    fn __deepcopy__(&self) -> Self {
        self.deepcopy()
    }

    /// makes a deep copy of current instance.
    fn copy(&self) -> Self {
        self.deepcopy()
    }

    #[getter]
    fn get_name(&self) -> PyResult<String> {
        Ok(self.name.read().to_string())
    }

    #[setter(name)]
    fn py_set_name(&mut self, name: &str) -> PyResult<()> {
        self.set_name(name);
        Ok(())
    }

    #[getter]
    fn get_label(&self) -> PyResult<Option<Text>> {
        let output = self.label.as_ref().map(|label| label.clone());
        Ok(output)
    }

    #[setter]
    fn set_label<'py>(&mut self, label: Option<&Bound<'py, PyAny>>) -> PyResult<()> {
        if let Some(bound) = label {
            if let Ok(string) = bound.extract::<&str>() {
                if let Some(current) = &mut self.label {
                    current.set_value(string);
                } else {
                    self.label = Some(Text::new(string))
                };
                Ok(())
            } else if let Ok(object) = bound.extract::<Text>() {
                self.label = Some(object);
                Ok(())
            } else {
                let type_name = get_type_name(bound)?;
                Err(exceptions::PyTypeError::new_err(format!(
                    "Expect one of: Text, str, None, got: {}",
                    type_name
                )))
            }
        } else {
            self.label = None;
            Ok(())
        }
    }

    #[getter]
    fn get_filter(&self) -> PyResult<Option<String>> {
        Ok(self.filter.read().clone())
    }

    #[setter(filter)]
    fn py_set_filter(&self, filter: Option<String>) -> PyResult<()> {
        self.set_filter(filter);
        Ok(())
    }

    #[getter]
    fn get_location(&self) -> PyResult<Location> {
        Ok(self.location.clone())
    }

    #[getter]
    fn get_spread(&self) -> PyResult<Option<Spread>> {
        Ok(self.spread.as_ref().map(|item| item.clone()))
    }

    #[getter(format)]
    fn py_get_format(&self) -> PyResult<Option<String>> {
        Ok(self.get_format_name())
    }

    #[setter(format)]
    fn py_set_format<'py>(&self, format: &Bound<'py, PyAny>) -> PyResult<()> {
        if let Ok(value) = format.extract::<Option<CodeFormat>>() {
            self.set_format(value);
            Ok(())
        } else if let Ok(value) = format.extract::<Option<&str>>() {
            self.set_format_from_str(value).map_err(py_value_error)
        } else {
            let type_name = get_type_name(format)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect 'format' to be one of: 'numeric'|'literal'|None, got: {}",
                type_name
            )))
        }
    }

    #[getter(usage)]
    fn py_get_usage(&self) -> PyResult<Option<String>> {
        Ok(self.usage.read().as_ref().map(|item| item.to_string()))
    }

    #[setter(usage)]
    fn py_set_usage<'py>(&self, value: &Bound<'py, PyAny>) -> PyResult<()> {
        if let Ok(value) = value.extract::<Option<Usage>>() {
            self.set_usage(value);
            Ok(())
        } else if let Ok(value) = value.extract::<Option<&str>>() {
            self.set_usage_from_str(value).map_err(py_value_error)
        } else {
            let type_name = get_type_name(value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect 'usage' to be one of: 'serial'|'weight'|None, got: {}",
                type_name
            )))
        }
    }

    #[staticmethod]
    #[pyo3(name="from_string", signature=(question_fragment:"str"))]
    fn py_from_string(question_fragment: String) -> PyResult<Self> {
        Self::from_string(&question_fragment)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
    }

    #[pyo3(name = "to_classic")]
    fn py_to_classic(&self) -> PyResult<String> {
        self.to_sss_classic()
            .map_err(|err| exceptions::PyRuntimeError::new_err(err.to_string()))
    }
}

/// Defines the part of the data record that is allocated to holding the value
/// of the variable.
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Location", from_py_object)
)]
#[derive(Debug, Default)]
pub struct Location {
    start: ArcLock<u32>,
    finish: ArcLock<u32>,
}

impl DeepCopy for Location {
    fn deepcopy(&self) -> Self {
        Self {
            start: lock_it_up(*self.start.read()),
            finish: lock_it_up(*self.finish.read()),
        }
    }
}

impl Clone for Location {
    fn clone(&self) -> Self {
        Self {
            start: self.start.clone(),
            finish: self.finish.clone(),
        }
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        (*self.start.read() == *other.start.read())
            & (*self.finish.read() == *other.finish.read())
    }
}

impl Location {
    pub fn start(&self) -> RwLockReadGuard<'_, u32> {
        self.start.read()
    }

    pub fn start_mut(&mut self) -> RwLockWriteGuard<'_, u32> {
        self.start.write()
    }

    pub fn finish(&self) -> RwLockReadGuard<'_, u32> {
        self.finish.read()
    }

    pub fn finish_mut(&mut self) -> RwLockWriteGuard<'_, u32> {
        self.finish.write()
    }

    /// Construct a location, defaulting the finish to the start.
    pub fn new(start: u32, finish: Option<u32>) -> Result<Self, RangeError> {
        let location = Self {
            start: lock_it_up(start),
            finish: lock_it_up(finish.unwrap_or(start)),
        };
        location.validate()?;
        Ok(location)
    }

    pub fn to_sss_classic(&self) -> String {
        if *self.finish.read() != *self.start.read() {
            format!("POSITION {} TO {}", self.start.read(), self.finish.read())
        } else {
            format!("POSITION {}", self.start.read())
        }
    }

    #[inline]
    pub fn set_start(&mut self, start: u32) -> Result<(), error::InvalidArg> {
        *self.start.write() = start;
        Ok(())
    }

    #[inline]
    pub fn set_finish(&mut self, finish: u32) -> Result<(), error::InvalidArg> {
        if finish >= *self.start.read() {
            *self.finish.write() = finish;
            Ok(())
        } else {
            Err(error::InvalidArg::new(format!(
                "The `end` of a Location cannot be larger than its `start` location ({}).",
                self.start.read()
            )))
        }
    }

    pub fn validate(&self) -> Result<(), RangeError> {
        if *self.finish.read() < *self.start.read() {
            Err(RangeError(format!(
                "Start location ({}) should not exceed finish location ({})!",
                self.start.read(),
                self.finish.read()
            )))
        } else {
            Ok(())
        }
    }

    /// (finish - start) + 1
    pub fn width(&self) -> Result<u32> {
        self.validate()?;
        let difference = self.finish.read().saturating_sub(*self.start.read());
        let width = difference.saturating_add(1);
        Ok(width)
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Location {
    #[new]
    #[pyo3(signature=(start:"int", finish:"int"=None))]
    fn py_new(start: u32, finish: Option<u32>) -> PyResult<Self> {
        Self::new(start, finish)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
    }

    // `start` is a preserved word in Python hence the alias
    #[getter(start)]
    fn py_get_start(&self) -> u32 {
        self.start.read().clone()
    }

    #[setter(start)]
    fn py_set_start(&mut self, start: u32) -> PyResult<()> {
        self.set_start(start)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
    }

    #[getter(finish)]
    fn py_get_finish(&self) -> u32 {
        self.finish.read().clone()
    }

    #[setter(finish)]
    fn py_set_finish(&mut self, finish: u32) -> PyResult<()> {
        self.set_finish(finish)
            .map_err(|err| exceptions::PyValueError::new_err(err.to_string()))
    }
}

#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Spread", from_py_object)
)]
#[derive(Debug, Default)]
pub struct Spread {
    /// The number of subfields within the overall field that is defined by the
    /// <position> element.
    pub subfield: ArcLock<u32>,
    /// Denotes the width of each subfield.
    pub width: ArcLock<Option<u32>>,
}

impl PartialEq for Spread {
    fn eq(&self, other: &Self) -> bool {
        (*self.subfield.read() == *other.subfield.read())
            & (*self.width.read() == *other.width.read())
    }
}

impl Eq for Spread {}

impl Clone for Spread {
    fn clone(&self) -> Self {
        Self {
            subfield: self.subfield.clone(),
            width: self.width.clone(),
        }
    }
}

impl DeepCopy for Spread {
    fn deepcopy(&self) -> Self {
        Self {
            subfield: lock_it_up(*self.subfield.read()),
            width: lock_it_up(self.width.read().as_ref().map(|item| *item)),
        }
    }
}

impl Spread {
    pub fn subfield(&self) -> RwLockReadGuard<'_, u32> {
        self.subfield.read()
    }

    pub fn subfield_mut(&mut self) -> RwLockWriteGuard<'_, u32> {
        self.subfield.write()
    }

    pub fn width(&self) -> RwLockReadGuard<'_, Option<u32>> {
        self.width.read()
    }

    pub fn width_mut(&mut self) -> RwLockWriteGuard<'_, Option<u32>> {
        self.width.write()
    }

    pub fn new(num_subfields: u32, subfield_width: Option<u32>) -> Result<Self> {
        if num_subfields == 0 {
            return Err(
                error::InvalidArg::new("`num_subfields` must be a positive integer.").into(),
            );
        }
        if subfield_width == Some(0) {
            return Err(error::InvalidArg::new(
                "`subfield_width` must be a positive integer or None.",
            )
            .into());
        }

        Ok(Self {
            subfield: lock_it_up(num_subfields),
            width: lock_it_up(subfield_width),
        })
    }

    pub fn to_sss_classic(&self) -> String {
        match &*self.width.read() {
            Some(width) => format!("SPREAD {} OF {}", self.subfield.read(), width),
            None => format!("SPREAD {}", self.subfield.read()),
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Spread {
    /// Width may be omitted; resolving it requires the containing position.
    #[new]
    #[pyo3(signature=(num_subfields, subfield_width=None))]
    fn py_new(num_subfields: u32, subfield_width: Option<u32>) -> PyResult<Self> {
        Self::new(num_subfields, subfield_width).map_err(py_value_error)
    }

    #[getter(num_subfields)]
    fn py_get_num_subfields(&self) -> u32 {
        *self.subfield.read()
    }

    #[setter(num_subfields)]
    fn py_set_num_subfields(&mut self, num_subfields: u32) -> PyResult<()> {
        if num_subfields == 0 {
            return Err(exceptions::PyValueError::new_err(
                "`num_subfields` must be a positive integer.",
            ));
        }
        *self.subfield.write() = num_subfields;
        Ok(())
    }

    #[getter(subfield_width)]
    fn py_get_subfield_width(&self) -> Option<u32> {
        *self.width.read()
    }

    #[setter(subfield_width)]
    fn py_set_subfield_width(&mut self, subfield_width: Option<u32>) -> PyResult<()> {
        if subfield_width == Some(0) {
            return Err(exceptions::PyValueError::new_err(
                "`subfield_width` must be a positive integer or None.",
            ));
        }
        *self.width.write() = subfield_width;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Variant;

    fn new_empty_question() -> Question {
        Question::new(
            "Q1",
            QuestionType::Single,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn new_filled_question() -> Question {
        let mut question = new_empty_question();
        question.location.set_start(23).unwrap();
        question.location.set_finish(55).unwrap();
        question.label = Some(Text::new("some label"));
        question
    }

    #[test]
    fn test_set_question_data_location_success() {
        let mut question = new_empty_question();
        question.location.set_start(23).unwrap();
        question.location.set_finish(55).unwrap();

        assert_eq!(question.location.start.read().clone(), 23);
        assert_eq!(question.location.finish.read().clone(), 55);
    }

    #[test]
    fn test_set_question_data_location_fail() {
        let mut question = new_empty_question();
        assert!(question.location.set_start(23).is_ok());
        assert!(question.location.set_finish(23).is_ok());
        assert!(question.location.set_finish(3).is_err());
    }

    #[test]
    fn test_sharable_mut_question_location() {
        let question = new_filled_question();
        let mut location = question.location.clone();

        location.set_start(2).unwrap();
        assert_eq!(question.location.start.read().clone(), 2);
        location.set_finish(55).unwrap();
        assert_eq!(question.location.finish.read().clone(), 55);
    }

    #[test]
    fn test_sharable_mut_question_label() {
        let question = new_filled_question();
        let mut label = question.label.as_ref().unwrap().clone();
        label.set_value("THIS");
        assert_eq!(
            "THIS",
            question.label.as_ref().map(|item| item.get_value().to_string()).unwrap()
        );

        label
            .add_variant(Variant::new("variant1", Some("analysis"), None).unwrap())
            .unwrap();
        assert_eq!(
            label.variants.read().get(0).unwrap().clone(),
            question
                .label
                .as_ref()
                .map(|item| item.variants.read().get(0).unwrap().clone())
                .unwrap()
        )
    }

    #[test]
    fn test_data_width_because_i_am_dumb() {
        let example = Location::new(1, Some(20)).unwrap();
        let width_calculated = example.width().unwrap();
        let actual = (1..21) // rust is 0-indexed, and sss is 1-indexed. there are 20 numbers from 1->20 in sss
            .map(|i| i)
            .collect::<Vec<i32>>()
            .len() as u32;
        assert_eq!(width_calculated, actual)
    }

    #[test]
    fn test_character_question_size_width_because_i_am_dumb() {
        let fragment = r#"
        VARIABLE 10
        NAME "WEBSITE"
        LABEL "test"
        TYPE CHARACTER
        POSITION 1042 TO 1241
        SIZE 200
        END VARIABLE
        "#;

        let parsed = Question::from_string(fragment.trim()).unwrap();
        let width_calculated = parsed.location().width().unwrap();
        assert_eq!(width_calculated, 200);

        let string = parsed.to_sss_classic().unwrap();
        assert!(string.contains("SIZE 200"));
    }

    #[cfg(feature = "pyo3")]
    #[test]
    fn test_question_format_getter_setter() {
        Python::initialize();
        // somehoe seems that i need `Python` to set these up,,,
        Python::attach(|py| {
            let question = new_empty_question();
            assert_eq!(question.py_get_format().unwrap(), None);

            for value in ["numeric", "literal"] {
                let input = pyo3::types::PyString::new(py, value);
                question.py_set_format(&input).unwrap();
                assert_eq!(question.py_get_format().unwrap(), Some(value.into()));
            }
        });
    }

    #[cfg(feature = "pyo3")]
    #[test]
    fn test_question_usage_getter_setter() {
        Python::initialize();
        Python::attach(|py| {
            let question = new_empty_question();
            assert_eq!(question.py_get_usage().unwrap(), None);

            for value in ["serial", "weight"] {
                let input = pyo3::types::PyString::new(py, value);
                question.py_set_usage(&input).unwrap();
                assert_eq!(question.py_get_usage().unwrap(), Some(value.into()));
            }
        });
    }
}
