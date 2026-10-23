#![doc = include_str!("../doc/rs_question.md")]

use std::{collections::HashSet, range::Range, str::FromStr};

use anyhow::{Context, Result};
use chrono::{NaiveDate, NaiveTime};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use polars::prelude::*;
#[cfg(feature = "pyo3")]
use pyo3::{
    exceptions,
    prelude::*,
    types::{PyDict, PyList, PyTuple},
};
#[cfg(feature = "pyo3")]
use pyo3_polars::{PySeries, error::PyPolarsErr};
use quick_xml::{events::BytesText, writer::Writer};
use strum::{AsRefStr, EnumIter, EnumString};

use crate::{
    Entry,
    EventLog,
    error::{self, DataReadError, InvalidArg, RangeError},
    member::{Code, Response, Text, Value},
    prelude::{DeepCopy, *},
    utils::{ArcLock, lock_it_up},
};
#[cfg(feature = "pyo3")]
use crate::{
    error::missing_key_err,
    utils::{get_type_name, py_value_error, single_quote},
};

/// Creating new question with options given.
///
/// # Example:
///
/// ```rust
/// use triple_s::{Code, QuestionOptions, Text};
///
/// let question = QuestionOptions::default()
///     .with_name("Q1")
///     .with_index(67)
///     .with_label(Text::new("Some label"))
///     .with_values(vec![Code::new_unsigned(1, "Maybe", None, None)])
///     .finish()?;
/// assert_eq!(question.name(), "Q1".to_string());
/// assert_eq!(question.label().value(), "Some label");
/// assert_eq!(question.response().len(), 1);
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug, Default)]
pub struct QuestionOptions {
    index: u32,
    name: String,
    label: Text,
    filter: Option<String>,
    format: Option<CodeFormat>,
    usage: Option<Usage>,
    question_type: QuestionType,
    location: (u32, Option<u32>),
    spread: Option<(u32, Option<u32>)>,
    values: Vec<Code>,
}

impl QuestionOptions {
    /// # Example
    ///
    /// ```rust
    /// use triple_s::question::QuestionOptions;
    ///
    /// let question = QuestionOptions::new("Q1").finish()?;
    /// assert_eq!(question.name(), "Q1".to_string());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Sets index of the question.
    ///
    /// # Example
    ///
    /// ```rust
    /// use triple_s::question::QuestionOptions;
    ///
    /// let question = QuestionOptions::new("Q1").with_index(23).finish()?;
    ///
    /// assert_eq!(question.index().clone(), 23 as u32);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn with_index(mut self, index: u32) -> Self {
        self.index = index;
        self
    }

    /// Sets name of the question.
    ///
    /// # Example
    ///
    /// ```rust
    /// use triple_s::question::QuestionOptions;
    ///
    /// let question = QuestionOptions::new("Q1").with_name("Q2").finish()?;
    ///
    /// assert_eq!(&question.name(), "Q2");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets label of the question.
    pub fn with_label(mut self, label: Text) -> Self {
        self.label = label;
        self
    }

    /// Sets filter of the question.
    pub fn with_filter(mut self, filter: Option<String>) -> Self {
        self.filter = filter;
        self
    }

    /// Sets code format of the question.
    pub fn with_format(mut self, format: Option<CodeFormat>) -> Self {
        self.format = format;
        self
    }

    /// Sets usage of the question.
    pub fn with_usage(mut self, usage: Option<Usage>) -> Self {
        self.usage = usage;
        self
    }

    /// Sets type of the question.
    pub fn with_question_type(mut self, question_type: QuestionType) -> Self {
        self.question_type = question_type;
        self
    }

    /// Sets the data location
    pub fn with_location(mut self, start: u32, finish: Option<u32>) -> Self {
        self.location = (start, finish);
        self
    }

    /// Sets the spread
    pub fn with_spread(mut self, subfields: u32, width: Option<u32>) -> Self {
        self.spread = Some((subfields, width));
        self
    }

    /// sets list of response values
    pub fn with_values(mut self, values: Vec<Code>) -> Self {
        self.values = values;
        self
    }

    /// Creates [`Question`] from current instance.
    ///
    /// ```rust
    /// # use triple_s::question::QuestionOptions;
    ///
    /// let question = QuestionOptions::new("Q1").finish()?;
    /// assert_eq!(question.name(), "Q1".to_string());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn finish(self) -> Result<Question> {
        let response = Response::new(None, self.values);

        Ok(Question {
            index: self.index,
            name: lock_it_up(self.name),
            label: self.label,
            filter: lock_it_up(self.filter),
            format: lock_it_up(self.format),
            usage: lock_it_up(self.usage),
            question_type: lock_it_up(self.question_type),
            location: Location::new(self.location.0, self.location.1)?,
            spread: self.spread.map(|(sub, width)| Spread::new(sub, width)).transpose()?,
            response,
            data: lock_it_up(Vec::new()),
        })
    }
}

/// A survey variable, represented by a `<variable>` element in Triple-S XML.
#[cfg_attr(
    not(feature = "pyo3"),
    doc = include_str!("../doc/rs_question.md")
)]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Question", from_py_object),
    doc = include_str!("../doc/py_question.md")
)]
#[derive(Debug)]
pub struct Question {
    index: u32,
    name: ArcLock<String>,
    label: Text,
    filter: ArcLock<Option<String>>,
    format: ArcLock<Option<CodeFormat>>,
    usage: ArcLock<Option<Usage>>,
    question_type: ArcLock<QuestionType>,
    location: Location,
    spread: Option<Spread>,
    response: Response,
    data: ArcLock<Vec<ArcLock<Entry>>>,
}

impl Clone for Question {
    /// Creates a shallow copy that shares lock-backed metadata and response
    /// contents.
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
            data: self.data.clone(),
        }
    }
}

impl Default for Question {
    fn default() -> Self {
        Self::new("", QuestionType::None, None, None, None, None, None, None).unwrap()
    }
}

impl DeepCopy for Question {
    #[doc = include_str!("../doc/rs_deep_copy_impl.md")]
    fn deepcopy(&self) -> Self {
        let data = self
            .data
            .read()
            .iter()
            .map(|item| lock_it_up(item.read().deepcopy()))
            .collect::<Vec<_>>();

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
            data: lock_it_up(data),
        }
    }
}

/// # Fields
///
/// ---
impl Question {
    /// Returns the variable's `ident` attribute.
    pub fn index(&self) -> &u32 {
        &self.index
    }

    /// Returns a mutable reference to the question identifier.
    ///
    /// # Example:
    /// ```rust
    /// use triple_s::question::QuestionOptions;
    ///
    /// let mut question = QuestionOptions::default().with_index(67).finish()?;
    /// assert_eq!(question.index(), &67);
    ///
    /// *question.index_mut() = 32;
    /// assert_eq!(question.index(), &32);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn index_mut(&mut self) -> &mut u32 {
        &mut self.index
    }

    /// Returns the variable name
    pub fn name(&self) -> String {
        self.name.read().clone()
    }

    /// Returns a write guard for updating the question name.
    ///
    /// # Example:
    /// ```rust
    /// use triple_s::question::QuestionOptions;
    ///
    /// let mut question = QuestionOptions::default().with_name("Q1").finish()?;
    /// assert_eq!(question.name().as_str(), "Q1");
    ///
    /// *question.name_mut() = "32".to_string();
    /// assert_eq!(question.name().as_str(), "32");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn name_mut(&mut self) -> RwLockWriteGuard<'_, String> {
        self.name.write()
    }

    /// Returns the label or original question text.
    ///
    /// The text may include language-specific or mode-specific variants.
    pub fn label(&self) -> &Text {
        &self.label
    }

    /// Returns a mutable reference for updating the question text and its
    /// variants.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::question::QuestionOptions;
    ///
    /// let mut question = QuestionOptions::new("Q1").finish()?;
    ///
    /// *question.label_mut().value_mut() = "Updated question".to_string();
    /// assert_eq!(question.label().value(), "Updated question");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn label_mut(&mut self) -> &mut Text {
        &mut self.label
    }

    /// Returns the data location described by `<position>`.
    ///
    /// IMPORTANT NOTE: Data location is 1-indexed (start counting from 1
    /// instead of 0)
    pub fn location(&self) -> &Location {
        &self.location
    }

    /// Returns a mutable reference for updating the data location.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::question::{Location, QuestionOptions};
    ///
    /// let mut question = QuestionOptions::new("Q1").finish()?;
    /// *question.location_mut() = Location::new(2, Some(4))?;
    /// assert_eq!(*question.location().start(), 2);
    /// assert_eq!(*question.location().finish(), 4);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn location_mut(&mut self) -> &mut Location {
        &mut self.location
    }

    /// Returns the optional `<spread>` layout for a multiple-response variable.
    ///
    /// `spread` field controls how to parse response data of questions with
    /// multi-choice reponses.
    ///
    ///  For more information on how this was done, please check the relevant
    /// documentation of [`Spread`] struct.
    pub fn spread(&self) -> &Option<Spread> {
        &self.spread
    }

    /// Returns a mutable reference for updating or replacing the optional
    /// spread layout.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::question::{QuestionOptions, Spread};
    ///
    /// let mut question = QuestionOptions::new("Q1")
    ///     .with_question_type(triple_s::question::QuestionType::Multiple)
    ///     .with_location(1, Some(2))
    ///     .finish()?;
    ///
    /// *question.spread_mut() = Some(Spread::new(2, Some(1))?);
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn spread_mut(&mut self) -> &mut Option<Spread> {
        &mut self.spread
    }

    /// Question filter, this corresponds to the `<filter>` node of a
    /// `<variable>` element.
    pub fn filter(&self) -> RwLockReadGuard<'_, Option<String>> {
        self.filter.read()
    }

    /// A read-write view for updating the `filter` field
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::question::QuestionOptions;
    ///
    /// let mut question = QuestionOptions::new("Q1").finish()?;
    /// *question.filter_mut() = Some("Eligible".to_string());
    /// assert_eq!(question.filter().as_deref(), Some("Eligible"));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn filter_mut(&mut self) -> RwLockWriteGuard<'_, Option<String>> {
        self.filter.write()
    }

    /// Returns the optional code format: `numeric` or `literal`.
    ///
    /// Please refer to the documentation of [`CodeFormat`] for more details.
    pub fn format(&self) -> RwLockReadGuard<'_, Option<CodeFormat>> {
        self.format.read()
    }

    /// Returns a write guard for updating the optional response code format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::question::{CodeFormat, QuestionOptions};
    ///
    /// let mut question = QuestionOptions::new("Q1").finish()?;
    /// *question.format_mut() = Some(CodeFormat::Literal);
    /// assert_eq!(*question.format(), Some(CodeFormat::Literal));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn format_mut(&mut self) -> RwLockWriteGuard<'_, Option<CodeFormat>> {
        self.format.write()
    }

    /// Returns the optional role declared by the `use` attribute.
    /// `serial` identifies records, while `weight` supplies their analysis
    /// weights.
    pub fn usage(&self) -> RwLockReadGuard<'_, Option<Usage>> {
        self.usage.read()
    }

    /// Returns a write guard for updating the optional variable role.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::question::{QuestionOptions, Usage};
    ///
    /// let mut question = QuestionOptions::new("Q1").finish()?;
    /// *question.usage_mut() = Some(Usage::Serial);
    /// assert_eq!(*question.usage(), Some(Usage::Serial));
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn usage_mut(&mut self) -> RwLockWriteGuard<'_, Option<Usage>> {
        self.usage.write()
    }

    /// Returns the variable type that determines how its data and response
    /// definition are interpreted.
    ///
    /// Mutable access to question_type is disabled because it would mess up
    /// data location. (technically you shouldn't convert any question type
    /// into any other quesiton types anyways).
    ///
    /// Conversion between _some_ question types are allowed, please refer to
    /// the implementation details in [`Question::set_question_type()`] for
    /// more details. To update question type, call
    /// [`Question::set_question_type()`] instead.
    pub fn question_type(&self) -> RwLockReadGuard<'_, QuestionType> {
        self.question_type.read()
    }

    /// Returns a write guard for updating the variable type.
    pub(crate) fn question_type_mut(&mut self) -> RwLockWriteGuard<'_, QuestionType> {
        self.question_type.write()
    }

    /// Returns the response definition, including explicit codes and an
    /// optional range.
    pub fn response(&self) -> &Response {
        &self.response
    }

    /// Returns a mutable reference for updating or replacing the response
    /// definition.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::question::QuestionOptions;
    ///
    /// let mut question = QuestionOptions::new("Q1").finish()?;
    /// *question.response_mut() = triple_s::response::Response::new(
    ///     None,
    ///     vec![triple_s::response::Code::new_unsigned(1, "Yes", None, None)],
    /// );
    /// assert_eq!(question.response().values().len(), 1);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn response_mut(&mut self) -> &mut Response {
        &mut self.response
    }

    /// Response data of the question
    pub fn data(&self) -> RwLockReadGuard<'_, Vec<ArcLock<Entry>>> {
        self.data.read()
    }

    /// Read-write access of the `data` field
    pub fn data_mut(&mut self) -> RwLockWriteGuard<'_, Vec<ArcLock<Entry>>> {
        self.data.write()
    }
}

/// <hr />
///
/// # Content management
impl Question {
    /// Adds a new response item to the current list of responses.
    ///
    /// # Example
    ///
    /// ```rust
    /// use triple_s::{Code, QuestionOptions, QuestionType, Text, Value};
    ///
    /// let mut question = QuestionOptions::new("Q1")
    ///     .with_question_type(QuestionType::Single)
    ///     .with_label(Text::new("Did you enjoy it?"))
    ///     .with_values(vec![Code::new_unsigned(1, "Maybe", None, None)])
    ///     .finish()?;
    /// assert_eq!(question.response().len(), 1);
    ///
    /// question.try_add_value(Value::UnSigned(2), Some("Nah"), Some("Q1=2"), Some(1.0))?;
    /// assert_eq!(question.response().len(), 2);
    /// assert_eq!(
    ///     question.response().values()[1].read().score().clone(),
    ///     Some(1.0)
    /// );
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn try_add_value(
        &mut self,
        code: Value,
        label: Option<&str>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Result<(), anyhow::Error> {
        let label = label.unwrap_or_default();
        let new = Code::new(code, label, filter, score);
        self.response_mut().add_value(new).with_context(|| {
            format!(
                "Error raised when tying to add value for question {}",
                self.name()
            )
        })
    }

    /// Adds a new response item to the current list of responses,
    /// or overwriting existing entriey with the same `code` field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use triple_s::{Code, QuestionOptions, QuestionType, Text, Value};
    ///
    /// let mut question = QuestionOptions::new("Q1")
    ///     .with_question_type(QuestionType::Single)
    ///     .with_label(Text::new("Did you enjoy it?"))
    ///     .with_values(vec![Code::new_unsigned(1, "Maybe", None, None)])
    ///     .finish()?;
    /// assert_eq!(question.response().len(), 1);
    ///
    /// question.set_value(Value::UnSigned(1), Some("Nah"), Some("Q1=2"), Some(1.0))?;
    /// assert_eq!(question.response().len(), 1);
    /// assert_eq!(
    ///     question.response().values()[0].read().score().clone(),
    ///     Some(1.0)
    /// );
    ///
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn set_value(
        &mut self,
        code: Value,
        label: Option<&str>,
        filter: Option<&str>,
        score: Option<f64>,
    ) -> Result<(), anyhow::Error> {
        // if !self.does_code_exist(&code) {
        let label = label.unwrap_or_default();
        let new = Code::new(code, label, filter, score);
        self.response_mut().set_value(new).with_context(|| {
            format!(
                "Error raised when tying to set value for question {}",
                self.name()
            )
        })
    }

    /// Append a new `Code` to existing instance.
    ///
    /// Raises error if already exists items with same `code` field.
    pub fn add_value(&mut self, value: Code) -> Result<()> {
        self.response_mut().add_value(value).with_context(|| {
            format!(
                "Error raised when tying to append value for question {}",
                self.name()
            )
        })
    }

    /// Returns the explicit code format as `"numeric"` or `"literal"`, or
    /// `None` if unset.
    pub fn get_format_name(&self) -> Option<String> {
        self.format.read().as_ref().map(|item| item.to_string())
    }

    /// Sets the code format from `"numeric"` or `"literal"`, or clears it with
    /// `None`. Returns an error for an unrecognised format.
    pub fn set_format_from_str(&self, value: Option<&str>) -> Result<()> {
        match value {
            None => {
                *self.format.write() = None;
                Ok(())
            }
            Some(format) => {
                let format = CodeFormat::from_str(format).with_context(|| {
                    format!(
                        "Expected to be `None` or one of: {:?}",
                        CodeFormat::available_options()
                    )
                })?;
                *self.format.write() = Some(format);
                Ok(())
            }
        }
    }

    /// Sets the variable role from `"serial"` or `"weight"`, or clears it with
    /// `None`.
    ///
    /// Returns an error for an unrecognised role.
    pub fn set_usage_from_str(&mut self, value: Option<&str>) -> Result<()> {
        match value {
            None => {
                *self.usage_mut() = None;
                Ok(())
            }
            Some(usage) => {
                let usage = Usage::from_str(usage).with_context(|| {
                    format!(
                        "Expected to be `None` or one of: {:?}",
                        Usage::available_options()
                    )
                })?;
                *self.usage_mut() = Some(usage);
                Ok(())
            }
        }
    }

    /// Update `question_type` attribute.
    ///
    /// *Note: conversion from Single -> mutiple only allowed
    /// when `Question::format()` is not `Some(CodeFormat::Literal)`*
    pub fn set_question_type(&mut self, new_type: QuestionType) -> Result<()> {
        let current_type = self.question_type().clone();
        let new_type_cloned = new_type.clone();
        match (current_type, new_type_cloned) {
            (QuestionType::Single, QuestionType::Multiple) => {
                if Some(&CodeFormat::Literal) == self.format.read().as_ref() {
                    return Err(InvalidArg::new("Cannot convert 'single' Question with 'literal' format into question type 'multiple'!. 'literal' format is exclusively used for question type ['single', None]."))
                        .with_context(|| format!("Error raised when trying to update question type for question {}", self.name()));
                };
                *self.question_type.write() = QuestionType::Multiple;
                Ok(())
            }
            (_, _) => {
                *self.question_type.write() = new_type;
                Ok(())
            }
        }
    }

    /// Get range of data locations from the `location` field,
    /// as `std::range::Range<u32>` object.
    ///
    /// Note that triple-s is 1 indexed and output is 0-indexed.
    ///
    /// # Example
    ///
    /// ```
    /// use std::range::Range;
    ///
    /// use triple_s::QuestionOptions;
    ///
    /// let question = QuestionOptions::new("Q1").with_location(3, None).finish()?;
    /// let data_range = question.get_data_range();
    ///
    /// assert_eq!(data_range, Range { start: 2, end: 3 });
    ///
    /// # Ok::<_, anyhow::Error>(())
    /// ```
    pub fn get_data_range(&self) -> Range<usize> {
        let start = (*self.location().start() as usize).saturating_sub(1);
        let end = *self.location().finish() as usize;
        Range { start, end }
    }

    /// extract raw data from one row of raw data (i.e., one respondent)
    fn extract_row(&self, row: &str) -> Result<String> {
        let data_location = self.get_data_range();
        let data = row
            .get(data_location)
            .ok_or(error::RangeError(format!(
                "Invalid data range (start: {}, end: {}) for row of length {}.",
                data_location.start,
                data_location.end,
                row.len()
            )))
            .with_context(|| {
                format!("When trying to parse data for question '{}'", self.name())
            })?
            .to_string();

        Ok(data)
    }

    /// Extracts this question's field from a fixed-width respondent row and
    /// parses it into an [`Entry`].
    pub(crate) fn parse_row(&self, row: &str) -> Result<Entry> {
        let data = self.extract_row(row)?;
        self.parse_field(&data)
    }

    /// Parse an extracted fixed-width field or decoded CSV field.
    pub(crate) fn parse_field(&self, data: &str) -> Result<Entry> {
        let question_type = self.question_type().clone();

        // `match` switch returns a Result<> because I want to add some extra
        // context later on
        let entry = match question_type {
            QuestionType::Character => Ok(Entry::String(data.to_string())),
            QuestionType::None => Err(DataReadError(
                "Unable to parse data with question type `None`".to_string(),
            )),
            QuestionType::Quantity => {
                let data = data.trim();
                if data.is_empty() {
                    return Ok(Entry::None());
                };

                let raw = data.parse::<f64>()?;
                Ok(Entry::Quantity(raw))
            }

            QuestionType::Logical => {
                let data = data.trim();
                if data.is_empty() {
                    return Ok(Entry::None());
                };

                let raw = data.parse::<usize>()?;
                match raw {
                    0 => Ok(Entry::Boolean(false)),
                    1 => Ok(Entry::Boolean(true)),
                    _ => Err(DataReadError(format!(
                        "Unable to parse boolean data from input: `{}`",
                        data
                    ))),
                }
            }

            QuestionType::Date => {
                let data = data.trim();
                if data.is_empty() {
                    return Ok(Entry::None());
                };

                let raw = NaiveDate::parse_from_str(data, "%Y%m%d")?;
                Ok(Entry::Date(raw))
            }

            QuestionType::Time => {
                let data = data.trim();
                if data.is_empty() {
                    return Ok(Entry::None());
                };

                let raw = NaiveTime::parse_from_str(data, "%H%M%S")?;
                Ok(Entry::Time(raw))
            }

            QuestionType::Single => {
                let data = data.trim();
                if data.is_empty() {
                    return Ok(Entry::None());
                };

                let raw =
                    if self.format().as_ref().is_some_and(|item| item == &CodeFormat::Literal)
                    {
                        Entry::String(data.into())
                    } else {
                        let raw = data.parse::<u32>()?;
                        Entry::Single(raw)
                    };

                Ok(raw)
            }

            QuestionType::Multiple => {
                let data = data.trim();
                if data.is_empty() {
                    return Ok(Entry::None());
                };

                let raw_data = self.parse_multiple_data(data)?;
                self.validate_multiple_read(&raw_data).with_context(|| {
                    format!("When trying to extract data for question `{}`", self.name())
                })?;
                Ok(Entry::Multiple(raw_data))
            }
        };

        entry.with_context(|| {
            format!("When trying to extract data for question `{}`", self.name())
        })
    }

    /// parsing multi-code data
    ///
    /// this being an indipendent function because it's bit complicated.
    fn parse_multiple_data(&self, data: &str) -> Result<HashSet<u32>> {
        match self.spread() {
            None => Self::parse_binary_multiple(data),
            Some(spread) => Self::parse_subfields_multiple(spread, data),
        }
    }

    fn parse_binary_multiple(data: &str) -> Result<HashSet<u32>> {
        let mut response: HashSet<u32> = HashSet::new();
        for (index, chr) in data.chars().enumerate() {
            match chr {
                '0' => Ok(()),
                '1' => {
                    response.insert(index as u32 + 1);
                    Ok(())
                }
                _ => Err(DataReadError(format!(
                    "Invalid multiple-choice response '{}' at position {}: expected '0' or '1'.",
                    chr,
                    index + 1,
                ))),
            }?;
        }
        Ok(response)
    }

    fn parse_subfields_multiple(spread: &Spread, data: &str) -> Result<HashSet<u32>> {
        let mut response: HashSet<u32> = HashSet::new();
        match *spread.width() {
            Some(width) => {
                for chunk in data.as_bytes().chunks(width as usize) {
                    let value = std::str::from_utf8(chunk)?.trim();
                    let code = value.parse::<u32>()?;
                    if code != 0 {
                        response.insert(code);
                    }
                }
                Ok(response)
            }
            None => {
                let num_response = *spread.subfield() as usize;
                let chunk_size = data.len().div_ceil(num_response);
                for chunk in data.as_bytes().chunks(chunk_size) {
                    let value = std::str::from_utf8(chunk)?.trim();
                    let code = value.parse::<u32>()?;
                    if code != 0 {
                        response.insert(code);
                    }
                }
                Ok(response)
            }
        }
    }

    fn validate_multiple_read(&self, raw_data: &HashSet<u32>) -> Result<()> {
        let available = self
            .response()
            .values()
            .iter()
            .map(|item| {
                let value = item.read().code().clone();
                match value {
                    Value::UnSigned(value) => Ok(value),
                    _ => Err(error::TypeMismatch::new(format!(
                        "Expect `unsignede integer` value type, got: `{:?}`",
                        value
                    )))
                    .with_context(|| {
                        format!("When trying to extract data for question `{}`", self.name())
                    }),
                }
            })
            .collect::<Result<HashSet<u32>>>()?;

        // validate raw data
        let diff = raw_data.difference(&available).collect::<Vec<&u32>>();
        if !diff.is_empty() {
            let message = format!(
                "Found unidentified code {:?} for question {}",
                diff,
                self.name()
            );
            Err(InvalidArg::new(message).into())
        } else {
            Ok(())
        }
    }

    /// Export content of the `data` field to polars series
    pub fn to_polars(&self) -> Result<Series, PolarsError> {
        let colname = self.name();
        let data = self
            .data()
            .iter()
            .map(|item| item.read().clone().into())
            .collect::<Vec<AnyValue>>();

        Series::from_any_values(colname.into(), &data, true)
    }
}

/// <hr />
///
/// # Create new question instance
impl Question {
    /// Create a question with index zero and an empty response definition.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::question::{Question, QuestionType};
    ///
    /// let question = Question::new(
    ///     "Q1",
    ///     QuestionType::Single,
    ///     None,
    ///     None,
    ///     None,
    ///     None,
    ///     None,
    ///     None,
    /// )?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    #[allow(clippy::too_many_arguments)]
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
        let label = label.map(Text::new).unwrap_or_default();
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
            data: lock_it_up(Vec::new()),
        };

        Ok(instance)
    }

    /// Parses a question from a Triple-S classic or XML fragment, detecting the
    /// format automatically. Returns an error if the fragment cannot be
    /// parsed into a question.
    ///
    /// # Example: read from triple-s classic
    ///
    /// ```
    /// use triple_s::question::Question;
    ///
    /// let original = r#"
    ///    VARIABLE 10
    ///    NAME "WEBSITE"
    ///    LABEL "Test label"
    ///    TYPE CHARACTER
    ///    POSITION 1042 TO 1241
    ///    SIZE 200
    ///    END VARIABLE
    ///    "#;
    /// let question = Question::from_string(original.trim())?;
    /// assert_eq!(question.name(), "WEBSITE".to_string());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Example: read from triple-s xml
    /// ```rust
    /// use triple_s::question::Question;
    ///
    /// let original = r#"
    ///     <variable ident="8" type="multiple">
    ///         <name>Q5</name>
    ///         <label>Two favourite attractions visited</label>
    ///         <position start="62" finish="63" />
    ///         <spread subfields="2" />
    ///         <values>
    ///             <value code="1">Sherwood Forest</value>
    ///         </values>
    ///     </variable>
    /// "#;
    ///
    /// let question = Question::from_string(original)?;
    /// assert_eq!(question.name(), "Q5".to_string());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_string(input: &str) -> Result<Self> {
        let is_classic = !input.trim().starts_with('<');
        let stream = input.lines().map(|line| Ok::<_, error::BadLine>(line.to_owned()));

        let builder = if is_classic {
            super::read::classic::collect_question_stream(stream, 0)?
        } else {
            let mut reader = quick_xml::Reader::from_str(input);
            reader.config_mut().trim_text(true);
            super::read::xml::collect_question_stream(reader)?
        };

        let (instance, warnings) = builder.build(is_classic)?;
        let event_log = EventLog { warnings };
        if let Some(warning) = event_log.warning_summary("Deserialising Question") {
            log::warn!("{}", warning)
        };

        Ok(instance)
    }

    /// create new instance from PyAny
    #[cfg(feature = "pyo3")]
    pub(crate) fn from_py_any(value: Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(val) = value.extract::<Bound<'_, PyDict>>() {
            Self::py_from_dict(val)
        } else if let Ok(val) = value.extract::<Question>() {
            Ok(val)
        } else {
            let type_name = get_type_name(&value)?;
            Err(exceptions::PyTypeError::new_err(format!(
                "Expect to be Question, or dict kind, got '{}'.",
                type_name
            )))
        }
    }

    /// Serialises the question as a Triple-S classic `VARIABLE` block.
    /// Returns an error for unsupported types or response definitions,
    /// including literal single-response codes.
    pub fn to_classic(&self) -> Result<String> {
        let name = self.name.read().clone();
        let mut output = format!("VARIABLE {}\nNAME \"{name}\"\n", self.index);
        output.push_str(&format!("LABEL \"{}\"\n", self.label.to_classic()));
        output.push_str(&self.location.to_classic());
        output.push('\n');

        let question_type = self.question_type.read().clone();
        match question_type {
            QuestionType::Character => {
                output.push_str("TYPE CHARACTER\n");
                if let Some(size_range) = self.response.range().as_ref() {
                    match size_range.end().clone() {
                        Value::UnSigned(end) => {
                            let size_string = format!("SIZE {}\n", end.saturating_sub(1));
                            output.push_str(&size_string);
                        }
                        _ => {
                            let error = error::InternalError::new(format!(
                                "Unexpected `Value` variant find in question [{}]: '{:?}'",
                                self.name(),
                                size_range.end().clone()
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
                    output.push_str(&spread.to_classic());
                    output.push('\n');
                }
                output.push_str(&format!(
                    "TYPE {}\n",
                    question_type.to_string().to_uppercase()
                ));
                output.push_str(
                    &self
                        .response
                        .to_classic(&question_type)
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

    /// Serialises the question as a Triple-S XML `<variable>` element.
    /// Returns an error if its response definition cannot be serialised.
    pub fn to_xml(&self) -> Result<String> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
        self.write_xml(&mut writer)?;
        String::from_utf8(writer.into_inner()).map_err(anyhow::Error::from)
    }

    /// Writes a Triple-S XML `<variable>` element to the supplied writer.
    /// Returns an error if writing or serialising the response definition
    /// fails.
    pub fn write_xml<W: std::io::Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        // write variable attributes
        let mut element = {
            let index = self.index.to_string();
            let type_name = self.question_type().to_string();

            writer
                .create_element("variable")
                .with_attribute(("ident", index.as_str()))
                .with_attribute(("type", type_name.as_str()))
        };

        // `use`
        if let Some(value) = self.usage().as_ref() {
            let value = value.to_string();
            element = element.with_attribute(("use", value.as_str()));
        };

        // `format`
        if let Some(value) = self.format().as_ref() {
            let value = value.to_string();
            element = element.with_attribute(("format", value.as_str()));
        };

        // write child elements
        element.write_inner_content(|children| {
            // name
            children
                .create_element("name")
                .write_text_content(BytesText::new(self.name().as_str()))?;

            // question label
            self.label.write_xml(children, "label", Vec::new())?;

            // filter
            if let Some(filter) = self.filter().as_ref() {
                children
                    .create_element("filter")
                    .write_text_content(BytesText::new(filter))?;
            };

            // data location
            let location = self.location();
            let location_start = location.start.read().to_string();
            let location_finish = location.finish.read().to_string();
            let location_attributes = if location_start != location_finish {
                vec![
                    ("start", location_start.as_str()),
                    ("finish", location_finish.as_str()),
                ]
            } else {
                vec![("start", location_start.as_str())]
            };
            children
                .create_element("position")
                .with_attributes(location_attributes)
                .write_empty()?;

            // spread
            if let Some(spread) = self.spread.as_ref() {
                let subfield = spread.subfield.read().to_string();
                let mut attributes = vec![("subfields", subfield.as_str())];
                if let Some(width) = spread.width().as_ref() {
                    let width = width.to_string();
                    attributes.push(("width", width.as_str()));
                    children
                        .create_element("spread")
                        .with_attributes(attributes)
                        .write_empty()?;
                } else {
                    children
                        .create_element("spread")
                        .with_attributes(attributes)
                        .write_empty()?;
                };
            };

            self.response()
                .write_xml(children, self.question_type.read().clone())
                .with_context(|| {
                    format!(
                        "When trying to convert responses to xml for question '{}'",
                        self.name()
                    )
                })
                .map_err(std::io::Error::other)
        })?;
        Ok(())
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
/// A question in a survey
///
/// Test text
impl Question {
    /// Create a question with index zero and an empty response definition.
    /// Accepts a type name or a `QuestionType` flag; optional metadata defaults
    /// to `None`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from triple_s import Question
    ///
    /// question = Question("Q1", "single")
    /// ```
    #[new]
    #[pyo3(signature=(
        name:"str", 
        question_type:"str|QuestionType|None"=None,
        label:"str"=None, 
        filter:"str"=None, 
        format:"'numeric'|'literal'|None"=None,
        usage:"'serial'|'weight'|None"=None,
        location=None,
        spread=None,
    ))]
    fn py_new<'py>(
        name: &str,
        question_type: Option<&Bound<'py, PyAny>>,
        label: Option<&str>,
        filter: Option<&str>,
        format: Option<&str>,
        usage: Option<&str>,
        location: Option<Location>,
        spread: Option<Spread>,
    ) -> PyResult<Self> {
        let question_type = match question_type {
            None => QuestionType::None,
            Some(question_type) => {
                if let Ok(value) = question_type.extract::<QuestionType>() {
                    value
                } else if let Ok(value) = question_type.extract::<&str>() {
                    QuestionType::from_str(value)
                        .with_context(|| format!("Expect 'question_type' to be one of the `QuestionType` flags, or one of: {:?}", QuestionType::available_options()))
                        .map_err(py_value_error)?
                } else {
                    return Err(exceptions::PyTypeError::new_err(get_type_name(
                        question_type,
                    )?));
                }
            }
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

    /// Returns a representation containing the question name, type, and label.
    fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<String> {
        let mut info = Vec::new();
        // name
        info.push(("name", single_quote(self.name())));
        info.push((
            "question_type",
            single_quote(self.question_type.read().as_ref()),
        ));

        // main text
        let label_repr = self.label.__repr__(py)?;
        info.push(("label", label_repr));

        // filter
        if let Some(filter) = self.filter().as_ref() {
            info.push(("filter", format!("'{}'", filter)));
        };

        // data location
        info.push(("location", self.location().__repr__()));

        // format
        if let Some(fmt) = self.format().as_ref() {
            info.push(("format", format!("'{}'", fmt.to_string())));
        };

        // question responses
        let response_repr = self.response().__repr__().replace("\n", "\n  "); // manual indentation;
        info.push(("response", response_repr));

        // putting everything together
        let formatted = info
            .into_iter()
            .map(|(field, value)| format!("{}={}", field, value))
            .collect::<Vec<String>>()
            .join(",\n  ");

        Ok(format!("Question(\n  {}\n)", formatted))
    }

    /// Creates an independent deep copy of this question.
    /// Changes to the copy do not affect the original.
    fn __copy__(&self) -> Self {
        self.deepcopy()
    }

    /// Creates an independent deep copy of this question.
    /// Changes to the copy do not affect the original.
    fn __deepcopy__(&self) -> Self {
        self.deepcopy()
    }

    /// Creates an independent deep copy of this question.
    /// Changes to the copy do not affect the original.
    fn copy(&self) -> Self {
        self.deepcopy()
    }

    #[getter]
    // `type` is a preserved keyword in rust
    fn get_type(&self) -> Option<String> {
        let qtype = self.question_type.read();
        match qtype.clone() {
            QuestionType::None => None,
            _ => Some(qtype.to_string()),
        }
    }

    #[setter]
    fn set_type<'py>(&mut self, question_type: Option<&Bound<'py, PyAny>>) -> PyResult<()> {
        let new_type = match question_type {
            None => QuestionType::None,
            Some(question_type) => {
                if let Ok(qtype) = question_type.extract::<QuestionType>() {
                    qtype
                } else if let Ok(qtype) = question_type.extract::<&str>() {
                    QuestionType::from_str(qtype)
                        .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))
                        .with_context(|| {
                            format!(
                                "Expected to be one of: {:?}, got: '{}'",
                                QuestionType::available_options(),
                                qtype
                            )
                        })?
                } else {
                    let type_name = get_type_name(question_type)?;
                    let message = format!(
                        "Expect to be triple_s.QuestionType variant or one of these strings: {:?}, got: '{}'",
                        QuestionType::available_options(),
                        type_name
                    );
                    return Err(exceptions::PyTypeError::new_err(message));
                }
            }
        };

        self.set_question_type(new_type)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))
    }

    /// Returns the variable name recorded in the `<name>` element.
    #[getter(name)]
    fn py_get_name(&self) -> PyResult<String> {
        Ok(self.name.read().to_string())
    }

    /// Sets the variable name recorded in the `<name>` element.
    #[setter(name)]
    fn py_set_name(&mut self, name: &str) -> PyResult<()> {
        *self.name_mut() = name.to_string();
        Ok(())
    }

    /// Returns the response definition, including explicit codes and an
    /// optional range. Its interpretation depends on the question type;
    /// character variables use it to describe their maximum size.
    #[getter(response)]
    fn py_get_response(&self) -> PyResult<Response> {
        Ok(self.response().clone())
    }

    #[setter(response)]
    fn py_set_response(&mut self, response: Response) {
        *self.response_mut() = response;
    }

    /// Returns the label or original question text.
    /// The text may include language-specific or mode-specific variants.
    #[getter(label)]
    fn py_get_label(&self) -> PyResult<Text> {
        Ok(self.label.clone())
    }

    /// Raw data for the question
    /// Data is retrieved via `load_data()` method of `Survey` or `Record`
    /// object.
    #[getter(data)]
    fn py_get_data<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self.data.read().clone().into_iter().map(|item| item.read().clone());

        PyTuple::new(py, items)
    }

    /// Export data as polars.Series
    #[pyo3(name = "to_polars")]
    fn py_to_polars(&self) -> PyResult<pyo3_polars::PySeries> {
        self.to_polars().map(PySeries).map_err(|err| PyPolarsErr::from(err).into())
    }

    /// Export loaded data as a pandas Series named after this question.
    /// Multiple-choice responses are sorted lists; missing responses are None.
    #[pyo3(name = "to_pandas")]
    pub(crate) fn py_to_pandas<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let values = PyList::empty(py);
        for entry in self.data().iter() {
            match &*entry.read() {
                Entry::None() => values.append(py.None())?,
                Entry::Single(value) => values.append(*value)?,
                Entry::Multiple(items) => {
                    let mut items: Vec<u32> = items.iter().copied().collect();
                    items.sort_unstable();
                    values.append(items)?;
                }
                Entry::Boolean(value) => values.append(*value)?,
                Entry::Quantity(value) => values.append(*value)?,
                Entry::Date(value) => values.append(*value)?,
                Entry::Time(value) => values.append(*value)?,
                Entry::String(value) => values.append(value.as_str())?,
            }
        }
        let kwargs = PyDict::new(py);
        kwargs.set_item("name", self.name())?;
        py.import("pandas")?.getattr("Series")?.call((values,), Some(&kwargs))
    }

    /// Sets the label from a string or `Text`, or resets it to empty with
    /// `None`. A string updates the existing text; a `Text` object replaces
    /// it.
    #[setter(label)]
    fn py_set_label<'py>(&mut self, label: Option<&Bound<'py, PyAny>>) -> PyResult<()> {
        if let Some(bound) = label {
            if let Ok(string) = bound.extract::<String>() {
                *self.label.value_mut() = string;
                Ok(())
            } else if let Ok(object) = bound.extract::<Text>() {
                self.label = object;
                Ok(())
            } else {
                let type_name = get_type_name(bound)?;
                Err(exceptions::PyTypeError::new_err(format!(
                    "Expect one of: Text, str, None, got: {}",
                    type_name
                )))
            }
        } else {
            self.label = Text::default();
            Ok(())
        }
    }

    /// Returns the optional name of a previously defined logical variable used
    /// as a filter. That variable determines whether this question is
    /// available for a record.
    #[getter(filter)]
    fn py_get_filter(&self) -> PyResult<Option<String>> {
        Ok(self.filter.read().clone())
    }

    /// Sets the name of the logical variable used as a filter, or clears it
    /// with `None`. The referenced variable should precede this question in
    /// the record.
    #[setter(filter)]
    fn py_set_filter(&mut self, filter: Option<String>) -> PyResult<()> {
        *self.filter_mut() = filter;
        Ok(())
    }

    /// Returns the data location described by `<position>`.
    /// Triple-S uses one-based character positions for fixed-width records and
    /// one-based field numbers for CSV records.
    #[getter(location)]
    fn py_get_location(&self) -> PyResult<Location> {
        Ok(self.location.clone())
    }

    #[setter(location)]
    fn py_set_location(&mut self, location: Location) -> PyResult<()> {
        location.validate().map_err(|err| exceptions::PyValueError::new_err(err.0))?;
        *self.location_mut() = location;
        Ok(())
    }

    /// Updates data location of the current question.
    ///
    /// Example:
    ///
    /// ```python
    /// from triple_s import Question
    ///
    /// question = Question("Q1", "single")
    /// print("Before update: ", question.location)
    /// question.set_location(start=0, finish=30)
    /// print("After update: ", question.location)
    /// ```
    #[pyo3(name="set_location", signature=(start:"int"=None, finish:"int"=None))]
    fn py_update_location(&mut self, start: Option<u32>, finish: Option<u32>) -> PyResult<()> {
        if let Some(start) = start {
            *self.location_mut().start_mut() = start;
        };

        if let Some(finish) = finish {
            *self.location_mut().finish_mut() = finish;
        };

        self.location()
            .validate()
            .map_err(|err| exceptions::PyValueError::new_err(err.0))
    }

    /// Returns the optional `<spread>` layout for a multiple-response variable.
    ///
    /// `spread` field controls how to parse response data of questions with
    /// multi-choice reponses, more information can be found in the help
    /// documentation of `Spread` class:
    ///
    /// ```python
    /// from triple_s import Spread
    ///
    /// help(Spread)
    /// ```
    #[getter(spread)]
    fn py_get_spread(&self) -> PyResult<Option<Spread>> {
        Ok(self.spread.as_ref().map(|item| item.clone()))
    }

    /// Flag for indicating the data type of `code` field in a <value> node.
    /// Only used for `Single` type of questions.
    ///
    /// 'numeric':
    /// The default. Code value being u32, `format` attribute `numeric`
    ///
    /// 'literal:
    /// Introduced since SSS 2.0, basically something like:
    // ```xml
    /// <variable ident="11" type="single" format="literal">
    ///     <values>
    ///         <value code="A">Within 3 months</value>
    ///         <value code="B">Between 3 months and 1 year</value>
    ///         <value code="C">More than 1 years time</value>
    ///     </values>
    /// </variable>
    /// ```
    #[getter(format)]
    fn py_get_format(&self) -> PyResult<Option<String>> {
        Ok(self.get_format_name())
    }

    /// Sets the response code format, or clears the explicit format with
    /// `None`. Accepts a `CodeFormat` flag or a format string.
    #[setter(format)]
    fn py_set_format<'py>(&self, format: &Bound<'py, PyAny>) -> PyResult<()> {
        if let Ok(value) = format.extract::<Option<CodeFormat>>() {
            *self.format.write() = value;
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

    /// Returns the optional role declared by the `use` attribute.
    /// `serial` identifies records, while `weight` supplies their analysis
    /// weights.
    #[getter(usage)]
    fn py_get_usage(&self) -> PyResult<Option<String>> {
        Ok(self.usage.read().as_ref().map(|item| item.to_string()))
    }

    /// Sets the variable role, or clears the explicit role with `None`.
    /// Accepts a `Usage` flag or a role string.
    #[setter(usage)]
    fn py_set_usage<'py>(&mut self, value: &Bound<'py, PyAny>) -> PyResult<()> {
        if let Ok(value) = value.extract::<Option<Usage>>() {
            *self.usage_mut() = value;
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

    /// Parses a question from a Triple-S classic or XML fragment, detecting the
    /// format automatically. Raises `ValueError` if the fragment cannot be
    /// parsed into a question.
    ///
    /// ```python
    /// from triple_s import Question
    ///
    /// original = r"""
    ///    VARIABLE 10
    ///    NAME "WEBSITE"
    ///    LABEL "Test label"
    ///    TYPE CHARACTER
    ///    POSITION 1042 TO 1241
    ///    SIZE 200
    ///    END VARIABLE
    ///    """
    /// question = Question.from_string(original)
    /// print(question)
    /// ```
    ///
    /// # Example: read from triple-s xml
    /// ```python
    /// from triple_s import Question
    ///
    /// original = r"""
    ///     <variable ident="8" type="multiple">
    ///         <name>Q5</name>
    ///         <label>Two favourite attractions visited</label>
    ///         <position start="62" finish="63" />
    ///         <spread subfields="2" />
    ///         <values>
    ///             <value code="1">Sherwood Forest</value>
    ///         </values>
    ///     </variable>
    /// """
    ///
    /// question = Question.from_string(original)
    /// print(question)
    /// ```
    #[staticmethod]
    #[pyo3(name="from_string", signature=(question_fragment:"str"))]
    fn py_from_string(question_fragment: String) -> PyResult<Self> {
        Self::from_string(&question_fragment)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))
    }

    /// Serialises the question as a Triple-S classic `VARIABLE` block.
    #[pyo3(name = "to_classic")]
    fn py_to_classic(&self) -> PyResult<String> {
        self.to_classic().map_err(py_value_error)
    }

    /// Serialises the question as a Triple-S XML `<variable>` element.
    #[pyo3(name = "to_xml")]
    fn py_to_xml(&self) -> PyResult<String> {
        self.to_xml().map_err(py_value_error)
    }

    /// Create new `Question` instance from dictionary with following keys and
    /// values:
    ///
    /// 'index' (int|None, *optional*): index of a question, default 1,
    ///
    /// 'name' (str):
    ///     name of the question
    ///
    /// 'question_type' (str|QuestionType):
    ///     type of the question.
    ///     See QuestionType.available_options() for list of allowed string
    /// values.
    ///
    /// 'location' (dict|Location|None, *optional*):
    ///     data location.
    ///     See help(Location.from_dict) for allowed keys in the dictionary.
    ///     If unset, would defult to Location(start=1, finish=1)
    ///
    /// 'label' (str|Text|dict):
    ///     question title.
    ///     See help(Text.from_dict) for allowed keys in the dictionary.
    ///     If unset, would defult to Text("")
    ///
    /// 'format' (str|CodeFormat|None, *optional*):
    ///     code format.
    ///     See CodeFormat.available_options() for list of allowed string values.
    ///
    /// 'filter' (str|None, *optional*):
    ///     Question filter.
    ///
    /// 'response' (Response|dict|None, *optional*):
    ///     `Response` instance or dictionary to pass to Response.from_dict(),
    ///     Would default to the default response values according to each question type if left unset
    #[pyo3(name = "from_dict")]
    #[staticmethod]
    fn py_from_dict(content: Bound<'_, PyDict>) -> PyResult<Self> {
        let name =
            content.get_item("name")?.ok_or(missing_key_err("name"))?.extract::<String>()?;

        let qtype = content
            .get_item("question_type")?
            .map(QuestionType::from_py_any)
            .transpose()?
            .ok_or(missing_key_err("question_type"))?;

        let format = content
            .get_item("format")?
            .filter(|value| !value.is_none())
            .map(CodeFormat::from_py_any)
            .transpose()?;

        let usage = content
            .get_item("usage")?
            .filter(|value| !value.is_none())
            .map(Usage::from_py_any)
            .transpose()?;

        let filter = content
            .get_item("filter")?
            .map(|item| item.extract::<Option<String>>())
            .transpose()?
            .flatten();

        let index = content
            .get_item("index")?
            .map(|item| item.extract::<u32>())
            .transpose()?
            .unwrap_or(1);

        let label = content
            .get_item("label")?
            .filter(|value| !value.is_none())
            .map_or(Ok(Text::new("")), Text::from_py_any)?;

        let mut instance = QuestionOptions::new(name)
            .with_label(label)
            .with_question_type(qtype)
            .with_format(format)
            .with_filter(filter)
            .with_usage(usage)
            .with_index(index)
            .finish()
            .map_err(|err| exceptions::PyValueError::new_err(format!("{:?}", err)))?;

        // fields that aren't very convinent to build from QuestionOptions
        // api,,,
        let location = content
            .get_item("location")?
            .filter(|value| !value.is_none())
            .map(|val| {
                if let Ok(location) = val.extract::<Location>() {
                    Ok(location)
                } else if let Ok(value) = val.extract::<Bound<'_, PyDict>>() {
                    Location::py_from_dict(value)
                } else {
                    let type_name = get_type_name(&val)?;
                    Err(exceptions::PyTypeError::new_err(format!(
                        "Expect `dict` or `Location` instance, got: '{}'",
                        type_name
                    )))
                }
            })
            .transpose()?
            .unwrap_or(Location::py_new(1, Some(1))?);
        *instance.location_mut() = location;

        let spread = content
            .get_item("spread")?
            .filter(|value| !value.is_none())
            .map(|val| {
                if let Ok(spread) = val.extract::<Spread>() {
                    Ok(spread)
                } else if let Ok(value) = val.extract::<Bound<'_, PyDict>>() {
                    Spread::py_from_dict(value)
                } else {
                    let type_name = get_type_name(&val)?;
                    Err(exceptions::PyTypeError::new_err(format!(
                        "Expect `dict` or `Spread` instance, got: '{}'",
                        type_name
                    )))
                }
            })
            .transpose()?;
        *instance.spread_mut() = spread;

        match content.get_item("response")? {
            // key doesn't exist
            None => Ok(instance),

            // key exists and value is python's NoneType
            Some(value) if value.is_none() => Ok(instance),

            // key exists and value isn't python's NoneType
            Some(value) => {
                let response = if let Ok(response) = value.extract::<Response>() {
                    response
                } else if let Ok(response) = value.extract::<Bound<'_, PyDict>>() {
                    Response::py_from_dict(response)?
                } else {
                    let type_name = get_type_name(&value)?;
                    let error_message = format!(
                        "Invalida input type when trying to parse 'response' for question '{}': Expect value of 'response' key to be NoneType, dictionary or `Response` instance, got: '{}'",
                        instance.name(),
                        type_name
                    );
                    return Err(exceptions::PyTypeError::new_err(error_message));
                };

                // cast to correct response type depending on current
                // question_type
                let response = match instance.question_type().clone() {
                    QuestionType::None => response,
                    QuestionType::Multiple => response
                        .cast::<u32>()
                        .with_context(|| {
                            format!("When trying to process question '{}'", instance.name())
                        })
                        .map_err(|err| {
                            exceptions::PyValueError::new_err(format!("{:?}", err))
                        })?,

                    QuestionType::Single => match instance.format().as_ref() {
                        Some(CodeFormat::Literal) => response
                            .cast::<String>()
                            .with_context(|| {
                                format!(
                                    "When trying to process question '{}'",
                                    instance.name()
                                )
                            })
                            .map_err(|err| {
                                exceptions::PyValueError::new_err(format!("{:?}", err))
                            })?,
                        _ => response
                            .cast::<u32>()
                            .with_context(|| {
                                format!(
                                    "When trying to process question '{}'",
                                    instance.name()
                                )
                            })
                            .map_err(|err| {
                                exceptions::PyValueError::new_err(format!("{:?}", err))
                            })?,
                    },
                    QuestionType::Logical => response
                        .cast::<u32>()
                        .with_context(|| {
                            format!("When trying to process question '{}'", instance.name())
                        })
                        .map_err(|err| {
                            exceptions::PyValueError::new_err(format!("{:?}", err))
                        })?,
                    QuestionType::Quantity => response
                        .cast::<f64>()
                        .with_context(|| {
                            format!("When trying to process question '{}'", instance.name())
                        })
                        .map_err(|err| {
                            exceptions::PyValueError::new_err(format!("{:?}", err))
                        })?,
                    QuestionType::Date => response
                        .cast::<u32>()
                        .with_context(|| {
                            format!("When trying to process question '{}'", instance.name())
                        })
                        .map_err(|err| {
                            exceptions::PyValueError::new_err(format!("{:?}", err))
                        })?,
                    QuestionType::Time => response
                        .cast::<u32>()
                        .with_context(|| {
                            format!("When trying to process question '{}'", instance.name())
                        })
                        .map_err(|err| {
                            exceptions::PyValueError::new_err(format!("{:?}", err))
                        })?,
                    QuestionType::Character => {
                        return Err(exceptions::PyValueError::new_err(format!(
                            "Cannot set 'response' for question_type 'character' !"
                        )))
                        .with_context(|| {
                            format!("When trying to process question '{}'", instance.name())
                        })
                        .map_err(|err| {
                            exceptions::PyValueError::new_err(format!("{:?}", err))
                        })?;
                    }
                };

                *instance.response_mut() = response;
                Ok(instance)
            }
        }
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
    #[doc = include_str!("../doc/rs_deep_copy_impl.md")]
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

/// Fields
///
/// ---
impl Location {
    /// Start of a data location, this corresponse to the `start` attribute of
    /// the <position> element
    pub fn start(&self) -> RwLockReadGuard<'_, u32> {
        self.start.read()
    }

    /// read-write access to the `start` field
    pub fn start_mut(&mut self) -> RwLockWriteGuard<'_, u32> {
        self.start.write()
    }

    /// End of a data location, this corresponds to the `end` attribute of the
    /// <position> element
    pub fn finish(&self) -> RwLockReadGuard<'_, u32> {
        self.finish.read()
    }

    /// read-write access to the `end` field
    pub fn finish_mut(&mut self) -> RwLockWriteGuard<'_, u32> {
        self.finish.write()
    }
}

impl Location {
    /// Returns an error if `finish` is less than `start`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use triple_s::question::Location;
    ///
    /// let location = Location::new(1, Some(20))?;
    /// let single_column = Location::new(1, None)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(start: u32, finish: Option<u32>) -> Result<Self, RangeError> {
        let location = Self {
            start: lock_it_up(start),
            finish: lock_it_up(finish.unwrap_or(start)),
        };
        location.validate()?;
        Ok(location)
    }

    /// Export to sss classic string
    pub fn to_classic(&self) -> String {
        if *self.finish.read() != *self.start.read() {
            format!("POSITION {} TO {}", self.start.read(), self.finish.read())
        } else {
            format!("POSITION {}", self.start.read())
        }
    }

    /// updates `start` field
    #[inline]
    pub fn set_start(&mut self, start: u32) -> Result<(), error::InvalidArg> {
        *self.start.write() = start;
        self.validate().map_err(|err| InvalidArg::new(err.0))
    }

    /// updates `finish` field
    #[inline]
    pub fn set_finish(&mut self, finish: u32) -> Result<(), error::InvalidArg> {
        if finish >= *self.start.read() {
            *self.finish.write() = finish;
            self.validate().map_err(|err| InvalidArg::new(err.0))
        } else {
            Err(error::InvalidArg::new(format!(
                "The `end` of a Location cannot be larger than its `start` location ({}).",
                self.start.read()
            )))
        }
    }

    /// updates both `start` and `finish` field together
    ///
    /// Example:
    ///
    /// ```
    /// use triple_s::Location;
    ///
    /// let mut location = Location::new(1, Some(20))?;
    /// location.set_range(2, 3)?;
    ///
    /// assert_eq!(location.start().clone(), 2);
    /// assert_eq!(location.finish().clone(), 3);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    #[inline]
    pub fn set_range(&mut self, start: u32, finish: u32) -> Result<(), error::InvalidArg> {
        *self.start_mut() = start;
        *self.finish_mut() = finish;
        self.validate().map_err(|err| InvalidArg::new(err.0))
    }

    /// checks if `finish()` is greater or equal than `start()`
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
    /// Raises `ValueError` if `finish` is less than `start`.
    ///
    /// # Examples
    ///
    /// ```python
    /// from triple_s import Location
    ///
    /// location = Location(1, 20)
    /// single_column = Location(1)
    /// ```
    #[new]
    #[pyo3(signature=(start:"int", finish:"int"=None))]
    fn py_new(start: u32, finish: Option<u32>) -> PyResult<Self> {
        Self::new(start, finish)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.eq(other)
    }

    fn __repr__(&self) -> String {
        format!(
            "Location(start={}, end={})",
            self.start().to_string(),
            self.finish().to_string()
        )
    }

    /// Start of a data location, this corresponse to the `start` attribute of
    /// the <position> element
    #[getter(start)]
    fn py_get_start(&self) -> u32 {
        self.start.read().clone()
    }

    #[setter(start)]
    fn py_set_start(&mut self, start: u32) -> PyResult<()> {
        self.set_start(start)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))
    }

    /// End of a data location, this corresponds to the `end` attribute of the
    /// <position> element
    #[getter(finish)]
    fn py_get_finish(&self) -> u32 {
        self.finish.read().clone()
    }

    #[setter(finish)]
    fn py_set_finish(&mut self, finish: u32) -> PyResult<()> {
        self.set_finish(finish)
            .map_err(|err| exceptions::PyValueError::new_err(format!("{err:#}")))
    }

    /// Creates new instance from dictionary with the following keys:
    ///
    /// 'start' (int): start of a data location
    /// 'finish' (int, *optional*): end of a data location
    #[staticmethod]
    #[pyo3(name = "from_dict")]
    fn py_from_dict(value: Bound<'_, PyDict>) -> PyResult<Self> {
        let start =
            value.get_item("start")?.ok_or(missing_key_err("start"))?.extract::<u32>()?;
        let finish = value
            .get_item("finish")?
            .map(|val| val.extract::<Option<u32>>()) // allows dict(start=1, finish=None)
            .transpose()?
            .flatten();

        Self::py_new(start, finish)
    }
}

#[doc=include_str!("../doc/spread.md")]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "triple_s", name = "Spread", from_py_object)
)]
#[derive(Debug, Default)]
pub struct Spread {
    subfield: ArcLock<u32>,
    width: ArcLock<Option<u32>>,
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
    #[doc = include_str!("../doc/rs_deep_copy_impl.md")]
    fn deepcopy(&self) -> Self {
        Self {
            subfield: lock_it_up(*self.subfield.read()),
            width: lock_it_up(self.width.read().as_ref().map(|item| *item)),
        }
    }
}

/// # Fields
///
/// ---
impl Spread {
    /// The number of subfields within the overall field that is defined by the
    /// `<position>` element.
    pub fn subfield(&self) -> RwLockReadGuard<'_, u32> {
        self.subfield.read()
    }

    /// read-write view of the `subfield` attribute
    pub fn subfield_mut(&mut self) -> RwLockWriteGuard<'_, u32> {
        self.subfield.write()
    }

    /// Denotes the width of each subfield.
    pub fn width(&self) -> RwLockReadGuard<'_, Option<u32>> {
        self.width.read()
    }

    /// read-write view of the `width` attribute
    pub fn width_mut(&mut self) -> RwLockWriteGuard<'_, Option<u32>> {
        self.width.write()
    }
}

/// Creating new instannce
///
/// ---
impl Spread {
    /// Returns an error if the count or an explicitly supplied width is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use triple_s::question::Spread;
    ///
    /// let spread = Spread::new(2, Some(1))?;
    /// let without_width = Spread::new(2, None)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

    pub(crate) fn to_classic(&self) -> String {
        match &*self.width.read() {
            Some(width) => format!("SPREAD {} OF {}", self.subfield.read(), width),
            None => format!("SPREAD {}", self.subfield.read()),
        }
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Spread {
    /// # Examples
    ///
    /// ```python
    /// from triple_s import Spread
    ///
    /// spread = Spread(2, subfield_width=1)
    /// without_width = Spread(2)
    /// ```
    #[new]
    #[pyo3(signature=(num_subfields, subfield_width=None))]
    fn py_new(num_subfields: u32, subfield_width: Option<u32>) -> PyResult<Self> {
        Self::new(num_subfields, subfield_width).map_err(py_value_error)
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.eq(other)
    }

    /// The number of subfields within the overall field that is defined by the
    /// `<position>` element.
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

    /// Denotes the width of each subfield.
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

    /// Creates new instance from dictionary with the following keys:
    ///
    /// 'num_subfields' (int): number of subfields.
    /// 'subfield_width' (int, *optional*): width of each subfield.
    #[staticmethod]
    #[pyo3(name = "from_dict")]
    fn py_from_dict(value: Bound<'_, PyDict>) -> PyResult<Self> {
        let num_subfields = value
            .get_item("num_subfields")?
            .ok_or(missing_key_err("num_subfields"))?
            .extract::<u32>()?;
        let width = value
            .get_item("subfield_width")?
            .map(|val| val.extract::<Option<u32>>())
            .transpose()?
            .flatten();
        Self::py_new(num_subfields, width)
    }
}

/// Flag for indicating the data type of `code` field in a `<value>` node. Only
/// used for `Single` type of questions.
///
/// Unsigned:
/// The default. Code value being u32, `format` attribute `numeric`
///
/// Literal:
/// Introduced since SSS 2.0, basically something like:
///
/// ```xml
/// <variable ident="11" type="single" format="literal">
///     <values>
///         <value code="B">Between 3 months and 1 year</value>
///         <value code="C">More than 1 years time</value>
///     </values>
/// </variable>
/// ```
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Debug, Clone, EnumString, EnumIter, Default, PartialEq)]
pub enum CodeFormat {
    /// Single/ Multiple response type with code value being u32, `format`
    /// attribute `numeric`
    #[default]
    #[strum(serialize = "numeric")]
    UnSigned,
    /// Single response question with `format='literal'`, introduced since SSS
    /// 2.0. Literal response codes are strings and do not support a value
    /// range. An example would be:
    ///
    /// ```xml
    /// <variable ident="11" type="single" format="literal">
    ///     <values>
    ///         <value code="A">Within 3 months</value>
    ///         <value code="B">Between 3 months and 1 year</value>
    ///         <value code="C">More than 1 years time</value>
    ///     </values>
    /// </variable>
    /// ```
    #[strum(serialize = "literal")]
    Literal,
}

impl DeepCopy for CodeFormat {
    fn deepcopy(&self) -> Self {
        self.clone()
    }
}

impl Flag for CodeFormat {
    fn to_string(&self) -> String {
        match self {
            Self::Literal => "literal".into(),
            Self::UnSigned => "numeric".into(),
        }
    }
}

#[cfg(feature = "pyo3")]
impl PyEnum for CodeFormat {}

/// describes the role of this variable in the datamap.
/// Only a subset of variable types may have a use attribute (as detailed below)
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Clone, Debug, Default, EnumString, EnumIter, PartialEq)]
pub enum Usage {
    /// plain data
    #[default]
    Null,

    /// this variable contains the serial number (or other identification field)
    #[strum(serialize = "serial")]
    Serial,
    /// this variable contains the weighting
    #[strum(serialize = "weight")]
    Weight,
}

impl Flag for Usage {
    fn to_string(&self) -> String {
        match self {
            Self::Null => "".into(),
            Self::Serial => "serial".into(),
            Self::Weight => "weight".into(),
        }
    }
}

impl DeepCopy for Usage {
    fn deepcopy(&self) -> Self {
        self.clone()
    }
}

#[cfg(feature = "pyo3")]
impl PyEnum for Usage {}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Usage {
    #[pyo3(name = "available_options")]
    #[staticmethod]
    fn py_available_options() -> Vec<String> {
        Self::available_options()
    }
}

/// type of question
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Debug, Default, PartialEq, Clone, EnumString, Eq, AsRefStr, EnumIter)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum QuestionType {
    /// No response values. Used for non-standard note or instruction pages,
    /// such as a blank page with just a 'Next Page' button.
    #[default]
    None,
    /// Single response question with numeric or literal response codes.
    #[strum(serialize = "single")]
    Single,

    /// Multiple response question with numeric response codes.
    #[strum(serialize = "multiple")]
    Multiple,

    /// Logical response type
    #[strum(serialize = "logical")]
    Logical,

    /// Quantity type of response.
    #[strum(serialize = "quantity")]
    Quantity,

    /// character response type
    #[strum(serialize = "character")]
    Character,

    /// date response type
    #[strum(serialize = "date")]
    Date,

    /// time response type
    #[strum(serialize = "time")]
    Time,
}

impl Flag for QuestionType {
    fn to_string(&self) -> String {
        match self {
            Self::None => "".into(),
            _ => self.as_ref().into(),
        }
    }
}

#[cfg(feature = "pyo3")]
impl PyEnum for QuestionType {}

#[cfg(feature = "pyo3")]
#[pymethods]
impl QuestionType {
    #[pyo3(name = "available_options")]
    #[staticmethod]
    fn py_available_options() -> Vec<String> {
        Self::available_options()
    }
}

#[cfg(test)]
mod test {
    use strum::IntoEnumIterator;

    #[test]
    fn test_qtype_from_str() {
        for original in QuestionType::iter() {
            if original != QuestionType::None {
                let string = original.to_string();
                let parsed = QuestionType::from_str(&string).unwrap();
                assert_eq!(parsed, original)
            }
        }
    }

    use super::*;
    use crate::member::{Variant, test::normalize_xml};

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
        question.location.set_range(23, 55).unwrap();
        question.label = Text::new("some label");
        question
    }

    #[test]
    fn test_set_question_data_location_success() {
        let mut question = new_filled_question();
        question.location.set_range(3, 30).unwrap();
        assert_eq!(question.location.start.read().clone(), 3);
        assert_eq!(question.location.finish.read().clone(), 30);

        question.location.set_start(0).unwrap();
        assert_eq!(question.location().start().clone(), 0);

        question.location.set_finish(3).unwrap();
        assert_eq!(question.location().finish().clone(), 3);
    }

    #[test]
    fn test_set_question_data_location_fail() {
        let mut question = new_filled_question();
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
        let mut label = question.label.clone();
        *label.value_mut() = "THIS".into();
        assert_eq!("THIS", question.label.value());

        label
            .add_variant(Variant::new("variant1", Some("analysis"), None).unwrap())
            .unwrap();
        assert_eq!(
            label.variants.read().get(0).unwrap().clone(),
            question.label.variants.read().get(0).unwrap().clone()
        )
    }

    #[test]
    fn test_question_add_code_success() {
        let mut question = new_filled_question();
        question.try_add_value(Value::UnSigned(3), Some("ok"), None, Some(1.5)).unwrap();

        assert_eq!(
            question.response().values()[0].read().score().clone(),
            Some(1.5)
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

        let string = parsed.to_classic().unwrap();
        assert!(string.contains("SIZE 200"));
    }

    fn prase_as_question_and_re_export(original: &str) -> String {
        let question = Question::from_string(original).unwrap();
        question.to_xml().unwrap()
    }

    #[test]
    fn test_write_xml_question_type_multi() {
        let original = r#"
            <variable ident="8" type="multiple">
                <name>Q5</name>
                <label>Two favourite attractions visited</label>
                <position start="62" finish="63" />
                <spread subfields="2" />
                <!--same answer list as Q3-->
                <values>
                    <value code="1">Sherwood Forest</value>
                    <value code="2">Nottingham Castle</value>
                    <value code="3">&quot;Friar Tuck&quot; Restaurant</value>
                    <value code="4">&quot;Maid Marion&quot; Cafe</value>
                    <value code="5">Mining museum</value>
                    <value code="9">Other</value>
                </values>
            </variable>
        "#
        .trim();

        let exported = prase_as_question_and_re_export(original);
        assert_eq!(normalize_xml(original), normalize_xml(&exported))
    }

    #[test]
    fn test_write_xml_question_type_logical() {
        let original = r#"
        <variable ident="10" type="logical">
            <name>Q7</name>
            <label>Would come again</label>
            <position start="67" />
        </variable>
        "#
        .trim();

        let exported = prase_as_question_and_re_export(original);
        assert_eq!(normalize_xml(original), normalize_xml(&exported))
    }

    #[test]
    fn test_write_xml_question_type_character() {
        let original = r#"
        <variable ident="6" type="character">
            <name>Q3.a</name>
            <label>Other attractions visited</label>
            <position start="31" finish="60" />
            <size>30</size>
        </variable>
        "#
        .trim();

        let exported = prase_as_question_and_re_export(original);
        assert_eq!(normalize_xml(original), normalize_xml(&exported))
    }

    #[test]
    fn test_write_xml_question_type_quantity() {
        let original = r#"
        <variable ident="9" type="quantity">
            <name>Q6</name>
            <label>Miles travelled</label>
            <position start="64" finish="66" />
            <values>
                <range from="1" to="499" />
                <value code="500">500 or more</value>
                <value code="999">Not stated</value>
            </values>
            <!-- labelled values on a quantity -->
        </variable>
        "#
        .trim();

        let exported = prase_as_question_and_re_export(original);
        assert_eq!(normalize_xml(original), normalize_xml(&exported))
    }

    #[test]
    fn test_write_xml_question_type_date_time() {
        let original_date = r#"
        <variable ident="2" type="date">
        <!-- date variable -->
            <name>Q1.a</name>
            <label>Date of visit</label>
            <position start="7" finish="14"/>
            <values>
                <range from="20050101" to="20051231" />
            </values>
        </variable>

        "#
        .trim();

        let exported = prase_as_question_and_re_export(original_date);
        assert_eq!(normalize_xml(original_date), normalize_xml(&exported));

        let original_time = r#"
        

        <variable ident="3" type="time">
        <!-- time variable -->
            <name>Q1.b</name>
            <label>Time of visit</label>
            <position start="15" finish="20"/>
            <!-- implicit range from="000000" to="235959" -->
        </variable>
        "#
        .trim();

        let exported = prase_as_question_and_re_export(original_time);
        assert_eq!(normalize_xml(original_time), normalize_xml(&exported))
    }

    #[test]
    fn test_write_xml_question_type_single() {
        let original = r#"
        <variable ident="7" type="single">
            <name>Q4</name>
            <label>Overall impression</label>
            <position start="61" />
            <values>
                <!-- scores -->
                <value code="1" score="2">Very Good</value>
                <value code="2" score="1">Good</value>
                <value code="3" score="0">OK</value>
                <value code="4" score="-1">Poor</value>
                <value code="5" score="-2">Very poor</value>
                <value code="9">DK/NS</value>
                <!-- missing score -->
            </values>
        </variable>
        "#
        .trim();

        let exported = prase_as_question_and_re_export(original);
        assert_eq!(normalize_xml(original), normalize_xml(&exported))
    }

    #[test]
    fn test_write_xml_question_type_single_literal() {
        let original = r#"
        <variable ident="11" type="single" format="literal">
            <name>Q8</name>
            <label>When is that most likely to be</label>
            <position start="68" />
            <filter>Q7</filter>
            <!-- filter -->
            <values>
                <value code="A">Within 3 months</value>
                <value code="B">Between 3 months and 1 year</value>
                <value code="C">More than 1 years time</value>
            </values>
            <!-- literal code values -->
        </variable>
        "#
        .trim();

        let exported = prase_as_question_and_re_export(original);
        assert_eq!(normalize_xml(original), normalize_xml(&exported))
    }

    #[test]
    fn test_write_xml_question_type_single_numeric() {
        let original = r#"
        <variable ident="7" type="single" format="numeric">
            <name>Q4</name>
            <label>Overall impression</label>
            <position start="61" />
            <values>
                <!-- scores -->
                <value code="1" score="2">Very Good</value>
                <value code="2" score="1">Good</value>
                <value code="3" score="0">OK</value>
                <value code="4" score="-1">Poor</value>
                <value code="5" score="-2">Very poor</value>
                <value code="9">DK/NS</value>
                <!-- missing score -->
            </values>
        </variable>
        "#
        .trim();

        let exported = prase_as_question_and_re_export(original);
        assert_eq!(normalize_xml(original), normalize_xml(&exported))
    }

    #[test]
    fn test_extract_row() {
        let row = "0123456789101112";
        let locations = [
            ((1, 1), "0"),
            ((1, 2), "01"),
            ((1, 3), "012"),
            ((2, 3), "12"),
        ];

        for (loc, correct) in locations.into_iter() {
            let question = QuestionOptions::new("tmp")
                .with_location(loc.0, Some(loc.1))
                .finish()
                .unwrap();
            let sliced = question.extract_row(row).unwrap();
            assert_eq!(&sliced, correct)
        }
    }

    #[test]
    fn test_question_data_mut() {
        let mut question = new_filled_question();
        let entry = Entry::Single(3);
        question.data_mut().push(lock_it_up(entry));
        assert_eq!(question.data().len(), 1);

        let check = question.data().get(0).unwrap().read().clone();
        assert_eq!(check, Entry::Single(3));

        *question.data_mut().get_mut(0).unwrap() = lock_it_up(Entry::None());
        let check = question.data().get(0).unwrap().read().clone();
        assert_eq!(check, Entry::None());
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
            let mut question = new_empty_question();
            assert_eq!(question.py_get_usage().unwrap(), None);

            for value in ["serial", "weight"] {
                let input = pyo3::types::PyString::new(py, value);
                question.py_set_usage(&input).unwrap();
                assert_eq!(question.py_get_usage().unwrap(), Some(value.into()));
            }
        });
    }
}
