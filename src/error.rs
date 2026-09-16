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
