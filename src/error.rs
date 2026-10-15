#![allow(missing_docs)]

//! List of error messages gets raised.

use std::{error::Error, fmt, sync::Arc};

#[cfg(feature = "pyo3")]
use pyo3::{exceptions, types::PyAnyMethods};
use strum::IntoStaticStr;

/// Error when some part of the code themselves were wrong, usually means there's something wrong
/// with the code itself.
#[derive(Debug)]
pub struct InternalError {
    message: String,
}

impl InternalError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Error for InternalError {}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InternalError: {}", self.message)
    }
}

/// Wrong inputs, this gets converts into `pyo3::exceptions::PyValueError` for python-facing API.
/// with the `pyo3` feature on.
#[derive(Debug)]
pub struct InvalidArg {
    /// the error message
    pub message: String,
}

impl InvalidArg {
    /// creates new instance.
    /// 
    /// ```rust
    /// use triple_s::error::InvalidArg;
    /// 
    /// let error = InvalidArg::new("some error");
    /// assert_eq!(&error.message, "some error")
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Error for InvalidArg {}

impl fmt::Display for InvalidArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InvalidArg: {}", self.message)
    }
}

#[cfg(feature = "pyo3")]
impl From<InvalidArg> for pyo3::PyErr {
    fn from(value: InvalidArg) -> Self {
        exceptions::PyValueError::new_err(format!("{value:?}"))
    }
}

#[derive(Debug)]
pub struct TypeMismatch {
    message: String,
}

impl TypeMismatch {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Error for TypeMismatch {}

impl fmt::Display for TypeMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TypeMismatch: {}", self.message)
    }
}

#[derive(Debug)]
pub struct DataReadError(pub String);

impl Error for DataReadError {}

impl fmt::Display for DataReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DataReadError: {}", self.0)
    }
}

#[cfg(feature="pyo3")]
impl From<DataReadError> for pyo3::PyErr {
    fn from(value: DataReadError) -> Self {
        exceptions::PyValueError::new_err(value.0)
    }
}

/// Error raised when user tring to convert things from type A to type B
/// i.e., trying to export Questions that only exists in sss xml into sss
/// classic Or trying to convert response types that are incompatible with each
/// other.
#[derive(Debug)]
pub struct ConversionError {
    field: String,
    from: String,
    to: String,
}

impl ConversionError {
    pub fn new(
        field: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            from: from.into(),
            to: to.into(),
        }
    }
}

impl Error for ConversionError {}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ConversionError: unable to convert [{}] from '{}' to '{}'",
            self.field, self.from, self.to
        )
    }
}

#[derive(Debug)]
pub struct FileReadError {
    message: String,
}

impl FileReadError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Error for FileReadError {}

impl From<std::io::Error> for FileReadError {
    fn from(value: std::io::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl fmt::Display for FileReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error when reading sss file: {}", self.message)
    }
}

#[derive(Debug)]
pub struct BadLine {
    message: String,
}

impl BadLine {
    pub fn new(message: impl std::fmt::Display) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl Error for BadLine {}

impl From<std::io::Error> for BadLine {
    fn from(value: std::io::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl From<anyhow::Error> for BadLine {
    fn from(value: anyhow::Error) -> Self {
        let error = format!("{value:#}\n\nBacktrace:\n{}", value.backtrace());
        Self::new(error)
    }
}

impl fmt::Display for BadLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BadLine: {}", self.message)
    }
}

#[derive(Debug)]
pub struct UnsupportedFeature {
    pub current_version: String,
    pub feature: String,
}

impl UnsupportedFeature {
    pub fn new(current_version: impl Into<String>, feature: impl Into<String>) -> Self {
        Self {
            current_version: current_version.into(),
            feature: feature.into(),
        }
    }
}

impl Error for UnsupportedFeature {}

impl fmt::Display for UnsupportedFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Feature '{}' is not supported in version '{}'",
            self.feature, self.current_version
        )
    }
}

#[derive(Debug)]
pub struct UnknownAttribute {
    name: String,
    value: String,
    parent: Option<String>,
}

impl Error for UnknownAttribute {}

impl UnknownAttribute {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            parent: None,
        }
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
        self
    }
}

impl fmt::Display for UnknownAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.parent.as_ref() {
            Some(parent) => write!(
                f,
                r#"UnknownAttribute: name: "{}", value: "{}", parent: "{}"."#,
                self.name, self.value, parent
            ),
            None => {
                write!(
                    f,
                    r#"UnknownAttribute: name: "{}", value: "{}"."#,
                    self.name, self.value
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct MissingAttributeError {
    tag: String,
    missing: String,
}

impl Error for MissingAttributeError {}

impl MissingAttributeError {
    pub fn new(tag: impl Into<String>, missing: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            missing: missing.into(),
        }
    }
}

impl fmt::Display for MissingAttributeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MissingAttributeError: Element '{}' is missing a required attribute: '{}'",
            self.tag, self.missing
        )
    }
}

#[derive(Debug)]
pub struct MissingValueError {
    instance: String,
    field: String,
}

impl Error for MissingValueError {}

impl MissingValueError {
    pub fn new(instance: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            instance: instance.into(),
            field: field.into(),
        }
    }
}

impl fmt::Display for MissingValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MissingValueError: Field '{}' of instance '{}' is empty",
            self.field, self.instance
        )
    }
}

#[derive(Debug)]
pub struct DuplicatedKey {
    key: String,
    prev_value: String,
}

impl Error for DuplicatedKey {}

impl DuplicatedKey {
    pub fn new(key: impl std::fmt::Debug, prev_value: impl std::fmt::Debug) -> Self {
        Self {
            key: format!("{:?}", key),
            prev_value: format!("{:?}", prev_value),
        }
    }
}

impl fmt::Display for DuplicatedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Duplicated key: {{key: '{}', value: '{}'}}",
            self.key, self.prev_value
        )
    }
}

#[derive(Debug)]
pub struct NotFiniteError;

impl fmt::Display for NotFiniteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Value must be finite.")
    }
}

impl Error for NotFiniteError {}

#[derive(Debug)]
pub struct LanguageTagError(pub String);

impl Error for LanguageTagError {}

impl fmt::Display for LanguageTagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct RangeError(pub String);

impl Error for RangeError {}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RangeError: {}", self.0)
    }
}

// ======================= warnings =======================
#[derive(Debug)]
pub struct EventLog {
    pub warnings: Vec<Warning>,
}

impl EventLog {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
        }
    }

    pub fn push_warnings(&mut self, warnings: Vec<Warning>) {
        self.warnings.extend(warnings)
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Warning {
    pub location: Option<String>,
    pub subfield: Option<String>,
    pub kind: WarningKind,
    pub message: String,
}

#[derive(Debug, IntoStaticStr)]
pub enum WarningKind {
    UnexpectedAttribute,
    DataLocation,
}

impl WarningKind {
    pub fn name(&self) -> Arc<str> {
        let output: &str = self.into();
        Arc::from(output)
    }
}
impl EventLog {
    /// Format one contextual warning table
    pub fn warning_summary(&self, context: impl Into<String>) -> Option<String> {
        if self.warnings.is_empty() {
            return None;
        }

        let headers = ["#", "Kind", "Location", "Field", "Message"];
        let rows: Vec<[String; 5]> = self
            .warnings
            .iter()
            .enumerate()
            .map(|(index, warning)| {
                [
                    (index + 1).to_string(),
                    warning.kind.name().to_string(),
                    warning.location.as_deref().unwrap_or("-").into(),
                    warning.subfield.as_deref().unwrap_or("-").into(),
                    warning.message.clone(),
                ]
            })
            .collect();

        // do some calculations,, to make things look nicer,,,
        let mut widths = headers.map(str::len);
        for row in &rows {
            for (width, value) in widths.iter_mut().zip(row) {
                *width = (*width).max(value.chars().count());
            }
        }
        let format_row = |row: [&str; 5]| {
            row.iter()
                .zip(widths)
                .map(|(value, width)| format!("{value:<width$}"))
                .collect::<Vec<_>>()
                .join(" | ")
                .trim_end()
                .to_owned()
        };
        let mut summary = format!(
            "{} raised these warnings ({}):\n\n",
            context.into(),
            rows.len()
        );
        summary.push_str(&format_row(headers));
        summary.push('\n');
        summary.push_str(&widths.map(|width| "-".repeat(width)).join("-+-"));
        for row in &rows {
            summary.push('\n');
            summary.push_str(&format_row(std::array::from_fn(|index| {
                row[index].as_str()
            })));
        }
        Some(summary)
    }

    /// Emit one UserWarning summary, respecting Python's warning filters.
    /// Returns an error when the active filter treats warnings as errors.
    #[cfg(feature = "pyo3")]
    pub fn print_warning_summary(
        &self,
        py: pyo3::Python<'_>,
        context: &str,
    ) -> pyo3::PyResult<()> {
        if let Some(summary) = self.warning_summary(context) {
            py.import("warnings")?.call_method1(
                "warn",
                (summary, py.get_type::<pyo3::exceptions::PyUserWarning>(), 1),
            )?;
        }
        Ok(())
    }
}
