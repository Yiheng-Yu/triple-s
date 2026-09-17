use std::{error::Error, fmt};

#[derive(Debug)]
pub struct OperationError {
    message: String,
}

impl OperationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Error for OperationError {}

impl fmt::Display for OperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OperationError: {}", self.message)
    }
}


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

impl fmt::Display for BadLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BadLine: {}", self.message)
    }
}

#[derive(Debug)]
pub struct UnexpectedLineKind {
    kind: crate::classic::LineKind,
    line: String,
}

impl UnexpectedLineKind {
    pub fn new(line_type: crate::classic::LineKind, value: impl Into<String>) -> Self {
        Self {
            kind: line_type,
            line: value.into(),
        }
    }
}

impl Error for UnexpectedLineKind {}

impl fmt::Display for UnexpectedLineKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unexpected line kind. Line: '{}', kind: {:?}",
            self.line, self.kind
        )
    }
}

#[derive(Debug)]
pub struct UnsupportedFeature {
    current_version: String,
    feature: String,
}

impl Error for UnsupportedFeature {}

impl fmt::Display for UnsupportedFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Feature '{}' is not supported in version '{}'", self.feature, self.current_version)
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
            parent: None
        }
    }

    pub fn with_parent(mut self, parent: impl Into<String> ) -> Self {
        self.parent = Some(parent.into());
        self
    }
}

impl fmt::Display for UnknownAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.parent.as_ref() {
            Some(parent) => write!(f, r#"Unexpected XML attribute!: name: "{}", value: "{}", parent: "{}"."#, self.name, self.value, parent),
            None => write!(f, r#"Unexpected XML attribute!: name: "{}", value: "{}"."#, self.name, self.value),
        }
    }
}
