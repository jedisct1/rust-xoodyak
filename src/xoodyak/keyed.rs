use super::internal::Xoodyak as _;
use super::internal::{Mode, Phase};
use super::*;

#[derive(Clone, Debug)]
pub struct XoodyakKeyed {
    state: Xoodoo,
    mode: Mode,
    phase: Phase,
    absorb_rate: usize,
    squeeze_rate: usize,
}

impl XoodyakKeyed {
    pub fn new(key: &[u8], key_id: Option<&[u8]>, counter: Option<&[u8]>) -> Result<Self, Error> {
        let mut xoodyak = XoodyakKeyed {
            state: Xoodoo::default(),
            phase: Phase::Up,
            mode: Mode::Hash,
            absorb_rate: HASH_ABSORB_RATE,
            squeeze_rate: HASH_SQUEEZE_RATE,
        };
        xoodyak.absorb_key(key, key_id, counter)?;
        Ok(xoodyak)
    }
}

impl internal::Xoodyak for XoodyakKeyed {
    #[inline(always)]
    fn state(&mut self) -> &mut Xoodoo {
        &mut self.state
    }

    #[inline(always)]
    fn mode(&self) -> Mode {
        self.mode
    }

    #[inline(always)]
    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode
    }

    #[inline(always)]
    fn phase(&self) -> Phase {
        self.phase
    }

    #[inline(always)]
    fn set_phase(&mut self, phase: Phase) {
        self.phase = phase
    }

    #[inline(always)]
    fn absorb_rate(&self) -> usize {
        self.absorb_rate
    }

    #[inline(always)]
    fn set_absorb_rate(&mut self, rate: usize) {
        self.absorb_rate = rate;
    }

    #[inline(always)]
    fn squeeze_rate(&self) -> usize {
        self.squeeze_rate
    }

    #[inline(always)]
    fn set_squeeze_rate(&mut self, rate: usize) {
        self.squeeze_rate = rate;
    }
}

impl Xoodyak for XoodyakKeyed {}

impl XoodyakKeyed {
    pub fn ratchet(&mut self) -> Result<(), Error> {
        if self.mode() != Mode::Keyed {
            return Err(Error::KeyRequired);
        }
        let mut rolled_key = [0u8; RATCHET_RATE];
        self.squeeze_any(&mut rolled_key, 0x10);
        self.absorb_any(&rolled_key, RATCHET_RATE, 0x00);
        Ok(())
    }

    pub fn encrypt(&mut self, out: &mut [u8], bin: &[u8]) -> Result<(), Error> {
        if self.mode() != Mode::Keyed {
            return Err(Error::KeyRequired);
        }
        if out.len() < bin.len() {
            return Err(Error::InvalidLength);
        }
        debug_assert_eq!(self.squeeze_rate, KEYED_SQUEEZE_RATE);
        let mut cu = 0x80;
        for (out_chunk, chunk) in out
            .chunks_mut(KEYED_SQUEEZE_RATE)
            .zip(bin.chunks(KEYED_SQUEEZE_RATE))
        {
            self.up(Some(out_chunk), cu);
            cu = 0x00;
            self.down(Some(chunk), 0x00);
            for (out_chunk_byte, chunk_byte) in out_chunk.iter_mut().zip(chunk) {
                *out_chunk_byte ^= *chunk_byte;
            }
        }
        Ok(())
    }

    pub fn decrypt(&mut self, out: &mut [u8], bin: &[u8]) -> Result<(), Error> {
        if self.mode() != Mode::Keyed {
            return Err(Error::KeyRequired);
        }
        if out.len() < bin.len() {
            return Err(Error::InvalidLength);
        }
        debug_assert_eq!(self.squeeze_rate, KEYED_SQUEEZE_RATE);
        let mut cu = 0x80;
        for (out_chunk, chunk) in out
            .chunks_mut(KEYED_SQUEEZE_RATE)
            .zip(bin.chunks(KEYED_SQUEEZE_RATE))
        {
            self.up(Some(out_chunk), cu);
            cu = 0x00;
            for (out_chunk_byte, chunk_byte) in out_chunk.iter_mut().zip(chunk) {
                *out_chunk_byte ^= *chunk_byte;
            }
            self.down(Some(out_chunk), 0x00);
        }
        Ok(())
    }

    pub fn encrypt_in_place(&mut self, in_out: &mut [u8]) -> Result<(), Error> {
        if self.mode() != Mode::Keyed {
            return Err(Error::KeyRequired);
        }
        debug_assert_eq!(self.squeeze_rate, KEYED_SQUEEZE_RATE);
        let mut tmp = [0u8; KEYED_SQUEEZE_RATE];
        let mut cu = 0x80;
        for in_out_chunk in in_out.chunks_mut(KEYED_SQUEEZE_RATE) {
            self.up(Some(&mut tmp), cu);
            cu = 0x00;
            self.down(Some(in_out_chunk), 0x00);
            for (in_out_chunk_byte, tmp_byte) in in_out_chunk.iter_mut().zip(&tmp) {
                *in_out_chunk_byte ^= *tmp_byte;
            }
        }
        Ok(())
    }

    pub fn decrypt_in_place(&mut self, in_out: &mut [u8]) -> Result<(), Error> {
        if self.mode() != Mode::Keyed {
            return Err(Error::KeyRequired);
        }
        debug_assert_eq!(self.squeeze_rate, KEYED_SQUEEZE_RATE);
        let mut tmp = [0u8; KEYED_SQUEEZE_RATE];
        let mut cu = 0x80;
        for in_out_chunk in in_out.chunks_mut(KEYED_SQUEEZE_RATE) {
            self.up(Some(&mut tmp), cu);
            cu = 0x00;
            for (in_out_chunk_byte, tmp_byte) in in_out_chunk.iter_mut().zip(&tmp) {
                *in_out_chunk_byte ^= *tmp_byte;
            }
            self.down(Some(in_out_chunk), 0x00);
        }
        Ok(())
    }

    pub fn aead_encrypt_detached(
        &mut self,
        out: &mut [u8],
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
        bin: Option<&[u8]>,
    ) -> Result<Tag, Error> {
        if out.len() < bin.unwrap_or_default().len() {
            return Err(Error::InvalidLength);
        }
        self.absorb(nonce.unwrap_or_default());
        self.absorb(ad.unwrap_or_default());
        self.encrypt(out, bin.unwrap_or_default())?;
        let mut auth_tag = Tag::default();
        self.squeeze(auth_tag.inner_mut());
        Ok(auth_tag)
    }

    pub fn aead_encrypt(
        &mut self,
        out: &mut [u8],
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
        bin: Option<&[u8]>,
    ) -> Result<(), Error> {
        let ct_len = bin.unwrap_or_default().len();
        if out.len() < ct_len + AUTH_TAG_BYTES {
            return Err(Error::InvalidLength);
        }
        let auth_tag = self.aead_encrypt_detached(out, nonce, ad, bin)?;
        out[ct_len..ct_len + AUTH_TAG_BYTES].copy_from_slice(auth_tag.as_ref());
        Ok(())
    }

    pub fn aead_decrypt_detached(
        &mut self,
        out: &mut [u8],
        auth_tag: &Tag,
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
        bin: Option<&[u8]>,
    ) -> Result<(), Error> {
        if out.len() < bin.unwrap_or_default().len() {
            return Err(Error::InvalidLength);
        }
        self.absorb(nonce.unwrap_or_default());
        self.absorb(ad.unwrap_or_default());
        self.decrypt(out, bin.unwrap_or_default())?;
        let mut computed_tag = Tag::default();
        self.squeeze(computed_tag.inner_mut());
        if computed_tag == *auth_tag {
            return Ok(());
        }
        out.iter_mut().for_each(|x| *x = 0);
        Err(Error::TagMismatch)
    }

    pub fn aead_decrypt(
        &mut self,
        out: &mut [u8],
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
        bin: &[u8],
    ) -> Result<(), Error> {
        let ct_len = bin
            .len()
            .checked_sub(AUTH_TAG_BYTES)
            .ok_or(Error::InvalidLength)?;
        if bin.len() < ct_len {
            return Err(Error::InvalidLength);
        }
        let mut auth_tag_bin = [0u8; AUTH_TAG_BYTES];
        auth_tag_bin.copy_from_slice(&bin[ct_len..]);
        let auth_tag = Tag::from(auth_tag_bin);
        let ct = &bin[..ct_len];
        self.aead_decrypt_detached(out, &auth_tag, nonce, ad, Some(ct))?;
        Ok(())
    }

    pub fn aead_encrypt_in_place_detached(
        &mut self,
        in_out: &mut [u8],
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
    ) -> Result<Tag, Error> {
        self.absorb(nonce.unwrap_or_default());
        self.absorb(ad.unwrap_or_default());
        self.encrypt_in_place(in_out)?;
        let mut auth_tag = Tag::default();
        self.squeeze(auth_tag.inner_mut());
        Ok(auth_tag)
    }

    pub fn aead_encrypt_in_place(
        &mut self,
        in_out: &mut [u8],
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
    ) -> Result<(), Error> {
        let ct_len = in_out
            .len()
            .checked_sub(AUTH_TAG_BYTES)
            .ok_or(Error::InvalidLength)?;
        let auth_tag = self.aead_encrypt_in_place_detached(&mut in_out[..ct_len], nonce, ad)?;
        in_out[ct_len..].copy_from_slice(auth_tag.as_ref());
        Ok(())
    }

    pub fn aead_decrypt_in_place_detached(
        &mut self,
        in_out: &mut [u8],
        auth_tag: &Tag,
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
    ) -> Result<(), Error> {
        self.absorb(nonce.unwrap_or_default());
        self.absorb(ad.unwrap_or_default());
        self.decrypt_in_place(in_out)?;
        let mut computed_tag = Tag::default();
        self.squeeze(computed_tag.inner_mut());
        if computed_tag == *auth_tag {
            return Ok(());
        }
        in_out.iter_mut().for_each(|x| *x = 0);
        Err(Error::TagMismatch)
    }

    pub fn aead_decrypt_in_place<'t>(
        &mut self,
        in_out: &'t mut [u8],
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
    ) -> Result<&'t mut [u8], Error> {
        let ct_len = in_out
            .len()
            .checked_sub(AUTH_TAG_BYTES)
            .ok_or(Error::InvalidLength)?;
        let mut auth_tag_bin = [0u8; AUTH_TAG_BYTES];
        auth_tag_bin.copy_from_slice(&in_out[ct_len..]);
        let ct = &mut in_out[..ct_len];
        let auth_tag = Tag::from(auth_tag_bin);
        self.aead_decrypt_in_place_detached(ct, &auth_tag, nonce, ad)?;
        Ok(ct)
    }

    #[cfg(feature = "std")]
    pub fn encrypt_to_vec(&mut self, bin: &[u8]) -> Result<Vec<u8>, Error> {
        let mut out = vec![0u8; bin.len()];
        self.encrypt(&mut out, bin)?;
        Ok(out)
    }

    #[cfg(feature = "std")]
    pub fn decrypt_to_vec(&mut self, bin: &[u8]) -> Result<Vec<u8>, Error> {
        let mut out = vec![0u8; bin.len()];
        self.decrypt(&mut out, bin)?;
        Ok(out)
    }

    #[cfg(feature = "std")]
    pub fn aead_encrypt_to_vec_detached(
        &mut self,
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
        bin: Option<&[u8]>,
    ) -> Result<(Vec<u8>, Tag), Error> {
        let mut out = vec![0u8; bin.unwrap_or_default().len()];
        let auth_tag = self.aead_encrypt_detached(&mut out, nonce, ad, bin)?;
        Ok((out, auth_tag))
    }

    #[cfg(feature = "std")]
    pub fn aead_encrypt_to_vec(
        &mut self,
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
        bin: Option<&[u8]>,
    ) -> Result<Vec<u8>, Error> {
        let mut out = vec![0u8; bin.unwrap_or_default().len() + AUTH_TAG_BYTES];
        self.aead_encrypt(&mut out, nonce, ad, bin)?;
        Ok(out)
    }

    #[cfg(feature = "std")]
    pub fn aead_encrypt_in_place_to_vec(
        &mut self,
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
        mut in_out: Vec<u8>,
    ) -> Result<Vec<u8>, Error> {
        let ct_len = in_out.len();
        in_out.resize_with(ct_len + AUTH_TAG_BYTES, || 0);
        self.aead_encrypt_in_place(&mut in_out, nonce, ad)?;
        Ok(in_out)
    }

    #[cfg(feature = "std")]
    pub fn aead_decrypt_to_vec_detached(
        &mut self,
        auth_tag: Tag,
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
        bin: Option<&[u8]>,
    ) -> Result<Vec<u8>, Error> {
        let mut out = vec![0u8; bin.unwrap_or_default().len()];
        self.aead_decrypt_detached(&mut out, &auth_tag, nonce, ad, bin)?;
        Ok(out)
    }

    #[cfg(feature = "std")]
    pub fn aead_decrypt_to_vec(
        &mut self,
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
        bin: &[u8],
    ) -> Result<Vec<u8>, Error> {
        let ct_len = bin
            .len()
            .checked_sub(AUTH_TAG_BYTES)
            .ok_or(Error::InvalidLength)?;
        let mut out = vec![0u8; ct_len];
        self.aead_decrypt(&mut out, nonce, ad, bin)?;
        Ok(out)
    }

    #[cfg(feature = "std")]
    pub fn aead_decrypt_in_place_to_vec(
        &mut self,
        nonce: Option<&[u8]>,
        ad: Option<&[u8]>,
        mut in_out: Vec<u8>,
    ) -> Result<Vec<u8>, Error> {
        let ct_len = in_out
            .len()
            .checked_sub(AUTH_TAG_BYTES)
            .ok_or(Error::InvalidLength)?;
        self.aead_decrypt_in_place(&mut in_out, nonce, ad)?;
        in_out.truncate(ct_len);
        Ok(in_out)
    }
}