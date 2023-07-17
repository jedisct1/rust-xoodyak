use std::convert::TryInto;
use zeroize::Zeroize;

#[cfg(not(target_arch = "x86_64"))]
mod impl_portable;
#[cfg(target_arch = "x86_64")]
mod impl_x86_64;

const ROUND_KEYS: [u32; 12] = [
    0x058, 0x038, 0x3c0, 0x0d0, 0x120, 0x014, 0x060, 0x02c, 0x380, 0x0f0, 0x1a0, 0x012,
];

#[derive(Clone, Debug)]
pub struct Xoodoo {
    st: [u8; 48],
}

impl Default for Xoodoo {
    fn default() -> Self {
        Self { st: [0u8; 48] }
    }
}

impl Xoodoo {
    #[inline(always)]
    fn bytes_view(&self) -> &[u8] {
        &self.st
    }

    #[inline(always)]
    fn bytes_view_mut(&mut self) -> &mut [u8] {
        &mut self.st
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn to_words(&self) -> [u32; 12] {
        let mut st_words = [0u32; 12];
        for (st_word, bytes) in st_words.iter_mut().zip(self.st.chunks_exact(4)) {
            *st_word = u32::from_le_bytes(bytes.try_into().unwrap());
        }
        st_words
    }

    #[inline(always)]
    #[allow(dead_code)]
    fn init_from_words(&mut self, st_words: [u32; 12]) {
        for (bytes, st_word) in self.st.chunks_exact_mut(4).zip(st_words.iter()) {
            bytes.copy_from_slice(&st_word.to_le_bytes());
        }
    }

    #[cfg(not(target_endian = "little"))]
    #[inline(always)]
    fn endian_swap(&mut self) {
        let mut st_words = self.to_words();
        for st_word in &mut st_words {
            *st_word = (*st_word).to_le()
        }
        self.from_words(&st_words);
    }

    #[cfg(target_endian = "little")]
    #[inline(always)]
    fn endian_swap(&mut self) {
        _ = self
    }

    #[inline]
    pub fn from_bytes(bytes: [u8; 48]) -> Self {
        let mut st = Xoodoo::default();
        let st_bytes = st.bytes_view_mut();
        st_bytes.copy_from_slice(&bytes);
        st
    }

    #[inline(always)]
    pub fn bytes(&self, out: &mut [u8; 48]) {
        let st_bytes = self.bytes_view();
        out.copy_from_slice(st_bytes);
    }

    #[inline(always)]
    pub fn add_byte(&mut self, byte: u8, offset: usize) {
        self.endian_swap();
        let st_bytes = self.bytes_view_mut();
        st_bytes[offset] ^= byte;
        self.endian_swap();
    }

    #[inline(always)]
    pub fn add_bytes(&mut self, bytes: &[u8]) {
        self.endian_swap();
        let st_bytes = self.bytes_view_mut();
        for (st_byte, byte) in st_bytes.iter_mut().zip(bytes) {
            *st_byte ^= byte;
        }
        self.endian_swap();
    }

    #[inline(always)]
    pub fn extract_bytes(&mut self, out: &mut [u8]) {
        self.endian_swap();
        let st_bytes = self.bytes_view();
        out.copy_from_slice(&st_bytes[..out.len()]);
        self.endian_swap();
    }
}

impl Drop for Xoodoo {
    fn drop(&mut self) {
        self.st.zeroize()
    }
}
