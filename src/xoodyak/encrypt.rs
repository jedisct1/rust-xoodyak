use super::*;

impl Xoodyak {
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
}
