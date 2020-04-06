use core::convert::TryInto;
use rawbytes::RawBytes;
use unroll::unroll_for_loops;
use zeroize::Zeroize;

const ROUND_KEYS: [u32; 12] = [
    0x058, 0x038, 0x3c0, 0x0d0, 0x120, 0x014, 0x060, 0x02c, 0x380, 0x0f0, 0x1a0, 0x012,
];

#[derive(Clone, Debug, Default)]
pub struct Xoodoo {
    st: [u32; 12],
}

impl Xoodoo {
    #[inline(always)]
    fn bytes_view(&self) -> &[u8] {
        let view = RawBytes::bytes_view(&self.st);
        debug_assert_eq!(view.len(), 48);
        view
    }

    #[inline(always)]
    fn bytes_view_mut(&mut self) -> &mut [u8] {
        let view = RawBytes::bytes_view_mut(&mut self.st);
        debug_assert_eq!(view.len(), 48);
        view
    }

    #[allow(non_upper_case_globals)]
    #[unroll_for_loops]
    #[inline]
    fn round(&mut self, round_key: u32) {
        let st = &mut self.st;
        let mut e = [0u32; 4];
        for i in 0..4 {
            e[i] = (st[i] ^ st[i + 4] ^ st[i + 8]).rotate_right(18);
            e[i] ^= e[i].rotate_right(9);
        }
        for i in 0..12 {
            st[i] ^= e[(i.wrapping_sub(1)) & 3];
        }
        st.swap(7, 4);
        st.swap(7, 5);
        st.swap(7, 6);
        st[0] ^= round_key;
        for i in 0..4 {
            let a = st[i];
            let b = st[i + 4];
            let c = st[i + 8].rotate_right(21);
            st[i + 8] = ((b & !a) ^ c).rotate_right(24);
            st[i + 4] = ((a & !c) ^ b).rotate_right(31);
            st[i] ^= c & !b;
        }
        st.swap(8, 10);
        st.swap(9, 11);
    }

    #[unroll_for_loops]
    pub fn permute(&mut self) {
        for &round_key in &ROUND_KEYS {
            self.round(round_key)
        }
    }

    #[inline]
    pub fn from_bytes(bytes: [u8; 48]) -> Self {
        let mut st = [0u32; 12];
        for (word, st_word) in bytes
            .chunks_exact(4)
            .map(|x| u32::from_le_bytes(x.try_into().unwrap()))
            .zip(st.iter_mut())
        {
            *st_word = word
        }
        Xoodoo { st }
    }

    #[inline(always)]
    pub fn bytes(&self, out: &mut [u8; 48]) {
        let st_bytes = self.bytes_view();
        out.copy_from_slice(st_bytes);
    }

    #[inline(always)]
    pub fn add_byte(&mut self, byte: u8, offset: usize) {
        self.st[offset / 4] ^= (byte as u32) << ((offset & 3) * 8);
    }

    #[inline(always)]
    pub fn add_bytes(&mut self, bytes: &[u8]) {
        let st_bytes = self.bytes_view_mut();
        for (st_byte, byte) in st_bytes.iter_mut().zip(bytes) {
            *st_byte ^= byte;
        }
    }

    #[inline(always)]
    pub fn extract_bytes(&self, out: &mut [u8]) {
        let st_bytes = self.bytes_view();
        out.copy_from_slice(&st_bytes[..out.len()]);
    }
}

impl Drop for Xoodoo {
    fn drop(&mut self) {
        self.st.zeroize()
    }
}
