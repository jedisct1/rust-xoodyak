use super::{Xoodoo, ROUND_KEYS};

impl Xoodoo {
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

    pub fn permute(&mut self) {
        for &round_key in &ROUND_KEYS {
            self.round(round_key)
        }
    }
}
