use super::internal::{Mode, Phase};
use super::*;

#[derive(Clone, Debug)]
pub enum XoodyakAny {
    Hash(XoodyakHash),
    Keyed(XoodyakKeyed),
}

impl internal::XoodyakCommon for XoodyakAny {
    fn state(&mut self) -> &mut Xoodoo {
        match self {
            XoodyakAny::Hash(x) => x.state(),
            XoodyakAny::Keyed(x) => x.state(),
        }
    }

    fn mode(&self) -> Mode {
        match self {
            XoodyakAny::Hash(x) => x.mode(),
            XoodyakAny::Keyed(x) => x.mode(),
        }
    }

    fn phase(&self) -> Phase {
        match self {
            XoodyakAny::Hash(x) => x.phase(),
            XoodyakAny::Keyed(x) => x.phase(),
        }
    }

    fn set_phase(&mut self, phase: Phase) {
        match self {
            XoodyakAny::Hash(x) => x.set_phase(phase),
            XoodyakAny::Keyed(x) => x.set_phase(phase),
        }
    }

    fn absorb_rate(&self) -> usize {
        match self {
            XoodyakAny::Hash(x) => x.absorb_rate(),
            XoodyakAny::Keyed(x) => x.absorb_rate(),
        }
    }

    fn squeeze_rate(&self) -> usize {
        match self {
            XoodyakAny::Hash(x) => x.squeeze_rate(),
            XoodyakAny::Keyed(x) => x.squeeze_rate(),
        }
    }
}

impl XoodyakAny {
    #[inline]
    fn keyed(&mut self) -> Result<&mut XoodyakKeyed, Error> {
        match self {
            XoodyakAny::Hash(_) => Err(Error::KeyRequired),
            XoodyakAny::Keyed(ref mut x) => Ok(x),
        }
    }

    #[inline]
    pub fn absorb_key_and_nonce(
        &mut self,
        key: &[u8],
        key_id: Option<&[u8]>,
        nonce: Option<&[u8]>,
        counter: Option<&[u8]>,
    ) -> Result<(), Error> {
        self.keyed()?
            .absorb_key_and_nonce(key, key_id, nonce, counter)
    }

    #[inline]
    pub fn ratchet(&mut self) -> Result<(), Error> {
        Ok(self.keyed()?.ratchet())
    }

    #[inline]
    pub fn encrypt(&mut self, out: &mut [u8], bin: &[u8]) -> Result<(), Error> {
        self.keyed()?.encrypt(out, bin)
    }

    #[inline]
    pub fn decrypt(&mut self, out: &mut [u8], bin: &[u8]) -> Result<(), Error> {
        self.keyed()?.decrypt(out, bin)
    }

    #[inline]
    pub fn encrypt_in_place(&mut self, in_out: &mut [u8]) -> Result<(), Error> {
        Ok(self.keyed()?.encrypt_in_place(in_out))
    }

    #[inline]
    pub fn decrypt_in_place(&mut self, in_out: &mut [u8]) -> Result<(), Error> {
        Ok(self.keyed()?.decrypt_in_place(in_out))
    }

    #[inline]
    pub fn aead_encrypt_detached(
        &mut self,
        out: &mut [u8],
        bin: Option<&[u8]>,
    ) -> Result<Tag, Error> {
        self.keyed()?.aead_encrypt_detached(out, bin)
    }

    #[inline]
    pub fn aead_encrypt(&mut self, out: &mut [u8], bin: Option<&[u8]>) -> Result<(), Error> {
        self.keyed()?.aead_encrypt(out, bin)
    }

    #[inline]
    pub fn aead_decrypt_detached(
        &mut self,
        out: &mut [u8],
        auth_tag: &Tag,
        bin: Option<&[u8]>,
    ) -> Result<(), Error> {
        self.keyed()?.aead_decrypt_detached(out, auth_tag, bin)
    }

    #[inline]
    pub fn aead_decrypt(&mut self, out: &mut [u8], bin: &[u8]) -> Result<(), Error> {
        self.keyed()?.aead_decrypt(out, bin)
    }

    #[inline]
    pub fn aead_encrypt_in_place_detached(&mut self, in_out: &mut [u8]) -> Result<Tag, Error> {
        Ok(self.keyed()?.aead_encrypt_in_place_detached(in_out))
    }

    #[inline]
    pub fn aead_encrypt_in_place(&mut self, in_out: &mut [u8]) -> Result<(), Error> {
        self.keyed()?.aead_encrypt_in_place(in_out)
    }

    #[inline]
    pub fn aead_decrypt_in_place_detached(
        &mut self,
        in_out: &mut [u8],
        auth_tag: &Tag,
    ) -> Result<(), Error> {
        self.keyed()?
            .aead_decrypt_in_place_detached(in_out, auth_tag)
    }

    #[inline]
    pub fn aead_decrypt_in_place<'t>(
        &mut self,
        in_out: &'t mut [u8],
    ) -> Result<&'t mut [u8], Error> {
        self.keyed()?.aead_decrypt_in_place(in_out)
    }

    #[cfg(feature = "std")]
    #[inline]
    pub fn encrypt_to_vec(&mut self, bin: &[u8]) -> Result<Vec<u8>, Error> {
        self.keyed()?.encrypt_to_vec(bin)
    }

    #[cfg(feature = "std")]
    #[inline]
    pub fn decrypt_to_vec(&mut self, bin: &[u8]) -> Result<Vec<u8>, Error> {
        self.keyed()?.decrypt_to_vec(bin)
    }

    #[cfg(feature = "std")]
    #[inline]
    pub fn aead_encrypt_to_vec_detached(
        &mut self,

        bin: Option<&[u8]>,
    ) -> Result<(Vec<u8>, Tag), Error> {
        self.keyed()?.aead_encrypt_to_vec_detached(bin)
    }

    #[cfg(feature = "std")]
    #[inline]
    pub fn aead_encrypt_to_vec(&mut self, bin: Option<&[u8]>) -> Result<Vec<u8>, Error> {
        self.keyed()?.aead_encrypt_to_vec(bin)
    }

    #[cfg(feature = "std")]
    #[inline]
    pub fn aead_encrypt_in_place_to_vec(&mut self, in_out: Vec<u8>) -> Result<Vec<u8>, Error> {
        Ok(self.keyed()?.aead_encrypt_in_place_to_vec(in_out))
    }

    #[cfg(feature = "std")]
    #[inline]
    pub fn aead_decrypt_to_vec_detached(
        &mut self,
        auth_tag: Tag,

        bin: Option<&[u8]>,
    ) -> Result<Vec<u8>, Error> {
        self.keyed()?.aead_decrypt_to_vec_detached(auth_tag, bin)
    }

    #[cfg(feature = "std")]
    #[inline]
    pub fn aead_decrypt_to_vec(&mut self, bin: &[u8]) -> Result<Vec<u8>, Error> {
        self.keyed()?.aead_decrypt_to_vec(bin)
    }

    #[cfg(feature = "std")]
    #[inline]
    pub fn aead_decrypt_in_place_to_vec(&mut self, in_out: Vec<u8>) -> Result<Vec<u8>, Error> {
        self.keyed()?.aead_decrypt_in_place_to_vec(in_out)
    }
}

impl XoodyakCommon for XoodyakAny {}
