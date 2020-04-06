use super::*;

impl Xoodyak {
    pub fn squeeze_to_vec(&mut self, len: usize) -> Vec<u8> {
        let mut out = vec![0u8; len];
        self.squeeze(&mut out);
        out
    }

    pub fn encrypt_to_vec(&mut self, bin: &[u8]) -> Result<Vec<u8>, Error> {
        let mut out = vec![0u8; bin.len()];
        self.encrypt(&mut out, bin)?;
        Ok(out)
    }

    pub fn decrypt_to_vec(&mut self, bin: &[u8]) -> Result<Vec<u8>, Error> {
        let mut out = vec![0u8; bin.len()];
        self.decrypt(&mut out, bin)?;
        Ok(out)
    }

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
