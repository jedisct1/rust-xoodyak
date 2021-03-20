use zeroize::Zeroize;

use crate::error::Error;

pub const AUTH_TAG_BYTES: usize = 16;

#[derive(Clone, Debug, Default, Eq)]
pub struct Tag([u8; AUTH_TAG_BYTES]);

impl Tag {
    #[inline(always)]
    pub(crate) fn inner_mut(&mut self) -> &mut [u8; AUTH_TAG_BYTES] {
        &mut self.0
    }

    #[inline]
    pub fn verify(&self, bin: [u8; AUTH_TAG_BYTES]) -> Result<(), Error> {
        if &Tag::from(bin) == self {
            Ok(())
        } else {
            Err(Error::TagMismatch)
        }
    }
}

impl Drop for Tag {
    #[inline]
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Tag) -> bool {
        other
            .0
            .iter()
            .zip(self.0.iter())
            .fold(0, |c, (a, b)| c | (a ^ b))
            == 0
    }
}

impl AsRef<[u8]> for Tag {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Tag> for [u8; AUTH_TAG_BYTES] {
    #[inline(always)]
    fn from(tag: Tag) -> Self {
        tag.0
    }
}

impl From<[u8; AUTH_TAG_BYTES]> for Tag {
    #[inline(always)]
    fn from(bin: [u8; AUTH_TAG_BYTES]) -> Self {
        Tag(bin)
    }
}
