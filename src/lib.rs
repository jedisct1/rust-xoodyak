#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

mod error;
mod tag;
mod xoodoo;
mod xoodyak;

pub mod prelude {
    pub use crate::xoodoo::*;
    pub use crate::xoodyak::*;
}

pub use crate::error::*;
pub use crate::tag::*;
pub use prelude::*;

#[cfg(test)]
mod test;
