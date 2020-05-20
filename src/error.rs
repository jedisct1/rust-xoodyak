use std::fmt::{self, Display};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    KeyTooLong,
    TagMismatch,
    InvalidLength,
    KeyRequired,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::KeyTooLong => write!(f, "Key too long"),
            Error::TagMismatch => write!(f, "Tag mismatch"),
            Error::InvalidLength => write!(f, "Invalid length"),
            Error::KeyRequired => write!(f, "A key is required"),
        }
    }
}
