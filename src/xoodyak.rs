use zeroize::Zeroize;

use crate::error::*;
use crate::xoodoo::*;

pub(crate) const HASH_ABSORB_RATE: usize = 16;
pub(crate) const HASH_SQUEEZE_RATE: usize = 16;
pub(crate) const KEYED_ABSORB_RATE: usize = 44;
pub(crate) const KEYED_SQUEEZE_RATE: usize = 24;
pub(crate) const RATCHET_RATE: usize = 16;
pub const AUTH_TAG_BYTES: usize = 16;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Mode {
    Hash,
    Keyed,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Phase {
    Up,
    Down,
}

#[derive(Clone, Debug)]
pub struct Xoodyak {
    st: Xoodoo,
    mode: Mode,
    phase: Phase,
    absorb_rate: usize,
    squeeze_rate: usize,
}

impl Xoodyak {
    #[inline(always)]
    fn permute(&mut self) {
        self.st.permute()
    }

    #[inline(always)]
    fn add_byte(&mut self, byte: u8, offset: usize) {
        self.st.add_byte(byte, offset);
    }

    #[inline(always)]
    fn add_bytes(&mut self, bytes: &[u8]) {
        self.st.add_bytes(bytes);
    }

    #[inline(always)]
    fn extract_bytes(&mut self, out: &mut [u8], offset: usize) {
        self.st.extract_bytes(out, offset);
    }

    #[inline(always)]
    fn up(&mut self, out: Option<&mut [u8]>, cu: u8) {
        debug_assert!(out.as_ref().map(|x| x.len()).unwrap_or(0) <= self.squeeze_rate);
        self.phase = Phase::Up;
        if self.mode != Mode::Hash {
            self.add_byte(cu, 47);
        }
        self.permute();
        if let Some(mut out) = out {
            self.extract_bytes(&mut out, 0);
        }
    }

    #[inline(always)]
    fn down(&mut self, bin: Option<&[u8]>, cd: u8) {
        debug_assert!(bin.as_ref().map(|x| x.len()).unwrap_or(0) <= self.absorb_rate);
        self.phase = Phase::Down;
        if let Some(bin) = bin {
            self.add_bytes(&bin);
            self.add_byte(0x01, bin.len());
        } else {
            self.add_byte(0x01, 0);
        }
        if self.mode == Mode::Hash {
            self.add_byte(cd & 0x01, 47);
        } else {
            self.add_byte(cd, 47);
        }
    }

    #[inline]
    fn absorb_any(&mut self, bin: &[u8], rate: usize, mut cd: u8) {
        for chunk in bin.chunks(rate) {
            if self.phase != Phase::Up {
                self.up(None, 0x00)
            }
            self.down(Some(chunk), cd);
            cd = 0
        }
    }

    fn absorb_key(
        &mut self,
        key: &[u8],
        key_id: Option<&[u8]>,
        counter: Option<&[u8]>,
    ) -> Result<(), Error> {
        if key.len() + key_id.unwrap_or_default().len() > KEYED_ABSORB_RATE {
            return Err(Error::KeyTooLong);
        }
        self.absorb_rate = KEYED_ABSORB_RATE;
        self.squeeze_rate = KEYED_SQUEEZE_RATE;
        self.mode = Mode::Keyed;
        let mut iv = [0u8; KEYED_ABSORB_RATE];
        let key_len = key.len();
        let mut key_id_len = 0;
        iv[..key_len].copy_from_slice(key);
        let mut iv_len = key_len;
        if let Some(key_id) = key_id {
            key_id_len = key_id.len();
            iv[iv_len..iv_len + key_id_len].copy_from_slice(key_id);
            iv_len += key_id_len;
        }
        iv[iv_len] = key_id_len as u8;
        iv_len += 1;
        self.absorb_any(&iv[..iv_len], KEYED_ABSORB_RATE, 0x02);
        if let Some(counter) = counter {
            self.absorb_any(counter, KEYED_ABSORB_RATE, 0x00)
        }
        Ok(())
    }

    #[inline]
    fn squeeze_any(&mut self, out: &mut [u8], cu: u8) {
        let mut chunks_it = out.chunks_mut(self.squeeze_rate);
        if let Some(chunk) = chunks_it.next() {
            self.up(Some(chunk), cu);
        }
        for chunk in chunks_it {
            self.down(None, 0x00);
            self.up(Some(chunk), 0x00);
        }
    }

    pub fn new(
        key: Option<&[u8]>,
        key_id: Option<&[u8]>,
        counter: Option<&[u8]>,
    ) -> Result<Self, Error> {
        let mut xoodyak = Xoodyak {
            st: Xoodoo::default(),
            phase: Phase::Up,
            mode: Mode::Hash,
            absorb_rate: HASH_ABSORB_RATE,
            squeeze_rate: HASH_SQUEEZE_RATE,
        };
        if let Some(key) = key {
            xoodyak.absorb_key(key, key_id, counter)?;
        }
        Ok(xoodyak)
    }

    #[inline(always)]
    pub fn absorb_rate(&self) -> usize {
        self.absorb_rate
    }

    #[inline(always)]
    pub fn squeeze_rate(&self) -> usize {
        self.squeeze_rate
    }

    #[inline(always)]
    pub fn absorb(&mut self, bin: &[u8]) {
        self.absorb_any(bin, self.absorb_rate, 0x03);
    }

    #[inline(always)]
    pub fn squeeze(&mut self, out: &mut [u8]) {
        self.squeeze_any(out, 0x40);
    }

    #[inline(always)]
    pub fn squeeze_key(&mut self, out: &mut [u8]) {
        self.squeeze_any(out, 0x20);
    }

    #[inline]
    pub fn squeeze_more(&mut self, out: &mut [u8]) {
        for chunk in out.chunks_mut(self.squeeze_rate) {
            self.down(None, 0x00);
            self.up(Some(chunk), 0x00);
        }
    }

    pub fn encrypt(&mut self, out: &mut [u8], bin: &[u8]) -> Result<(), Error> {
        if self.mode != Mode::Keyed {
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
        if self.mode != Mode::Keyed {
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
        if self.mode != Mode::Keyed {
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
        if self.mode != Mode::Keyed {
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

    pub fn ratchet(&mut self) -> Result<(), Error> {
        if self.mode != Mode::Keyed {
            return Err(Error::KeyRequired);
        }
        let mut rolled_key = [0u8; RATCHET_RATE];
        self.squeeze_any(&mut rolled_key, 0x10);
        self.absorb_any(&rolled_key, RATCHET_RATE, 0x00);
        Ok(())
    }

    #[cfg(feature = "std")]
    pub fn squeeze_to_vec(&mut self, len: usize) -> Vec<u8> {
        let mut out = vec![0u8; len];
        self.squeeze(&mut out);
        out
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

#[derive(Clone, Debug, Default, Eq)]
pub struct Tag([u8; AUTH_TAG_BYTES]);

impl Tag {
    #[inline(always)]
    pub(crate) fn inner_mut(&mut self) -> &mut [u8; AUTH_TAG_BYTES] {
        &mut self.0
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

impl Into<[u8; AUTH_TAG_BYTES]> for Tag {
    #[inline(always)]
    fn into(self) -> [u8; AUTH_TAG_BYTES] {
        self.0
    }
}

impl From<[u8; AUTH_TAG_BYTES]> for Tag {
    #[inline(always)]
    fn from(bin: [u8; AUTH_TAG_BYTES]) -> Self {
        Tag(bin)
    }
}
