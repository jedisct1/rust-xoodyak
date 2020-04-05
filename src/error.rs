#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    KeyTooLong,
    KeyRequired,
}
