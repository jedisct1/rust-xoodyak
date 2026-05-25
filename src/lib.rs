#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

mod error;
mod xoodoo;
mod xoodyak;

pub use crate::error::Error as XoodyakError;
pub use crate::xoodoo::Xoodoo;
pub use crate::xoodyak::{
    Tag as XoodyakTag, XoodyakAny, XoodyakCommon, XoodyakHash, XoodyakKeyed,
    AUTH_TAG_BYTES as XOODYAK_AUTH_TAG_BYTES,
};

#[cfg(test)]
mod test;
