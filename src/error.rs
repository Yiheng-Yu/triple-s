use std::{error::Error, fmt};

#[derive(Debug)]
pub enum IoError {
    Read(ErrorReadFile),
    BadLine(BadLine),
    UnexpectedLineKind(UnexpectedLineKind),
    Internal(String),
}

impl Error for IoError {}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(item) => item.fmt(f),
            Self::BadLine(item) => item.fmt(f),
            Self::UnexpectedLineKind(item) => item.fmt(f),
            Self::Internal(item) => write!(f, "InternalError: {}", item),
        }
    }
}

impl IoError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

impl From<std::io::Error> for IoError {
    fn from(value: std::io::Error) -> Self {
        Self::Read(ErrorReadFile::from(value))
    }
}

impl From<strum::ParseError> for IoError {
    fn from(value: strum::ParseError) -> Self {
        BadLine::new(value.to_string()).into()
    }
}

#[derive(Debug)]
pub struct ErrorReadFile {
    message: String,
}

impl ErrorReadFile {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Error for ErrorReadFile {}

impl From<std::io::Error> for ErrorReadFile {
    fn from(value: std::io::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl From<ErrorReadFile> for IoError {
    fn from(value: ErrorReadFile) -> Self {
        IoError::Read(value)
    }
}

impl fmt::Display for ErrorReadFile {
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

impl From<BadLine> for IoError {
    fn from(value: BadLine) -> Self {
        IoError::BadLine(value)
    }
}

impl fmt::Display for BadLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BadLine: {}", self.message)
    }
}

#[derive(Debug)]
pub struct UnexpectedLineKind {
    kind: crate::survey::LineKind,
    line: String,
}

impl UnexpectedLineKind {
    pub fn new(line_type: crate::survey::LineKind, value: impl Into<String>) -> Self {
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

impl From<UnexpectedLineKind> for IoError {
    fn from(value: UnexpectedLineKind) -> Self {
        Self::UnexpectedLineKind(value)
    }
}
