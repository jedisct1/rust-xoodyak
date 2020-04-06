#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

mod error;
mod tag;
mod xoodoo;
mod xoodyak;

pub use crate::error::Error as XoodyakError;
pub use crate::tag::Tag as XoodyakTag;
pub use crate::tag::AUTH_TAG_BYTES as XOODYAK_AUTH_TAG_BYTES;
pub use crate::xoodoo::*;
pub use crate::xoodyak::*;

#[cfg(test)]
mod test;
