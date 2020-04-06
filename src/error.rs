#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    KeyTooLong,
    TagMismatch,
    InvalidLength,
}
