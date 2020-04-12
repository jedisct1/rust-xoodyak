#![cfg_attr(not(feature = "std"), no_std)]

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
