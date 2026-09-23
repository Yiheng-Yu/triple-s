use std::{error::Error, fmt};

#[derive(Debug)]
pub struct InternalError {
    message: String,
}

impl InternalError {
    pub fn new(message: impl Into<String>) -> Self {
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

#[derive(Debug)]
pub struct InvalidArg {
    message: String,
}

impl InvalidArg {
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
                r#"Unexpected XML attribute!: name: "{}", value: "{}", parent: "{}"."#,
                self.name, self.value, parent
            ),
            None => {
                write!(
                    f,
                    r#"Unexpected XML attribute!: name: "{}", value: "{}"."#,
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
            "Element '{}' is missing a required attribute: {}",
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
            "Field '{}' of instance '{}' is empty",
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
