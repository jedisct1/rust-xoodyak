use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    InvalidBufferLength,
    InvalidParameterLength,
    KeyRequired,
    TagMismatch,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidBufferLength => write!(f, "Invalid buffer length"),
            Error::InvalidParameterLength => write!(f, "Key too long"),
            Error::KeyRequired => write!(f, "A key is required"),
            Error::TagMismatch => write!(f, "Tag mismatch"),
        }
    }
}
