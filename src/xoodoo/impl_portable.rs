use super::{Xoodoo, ROUND_KEYS};

impl Xoodoo {
    #[inline(always)]
    fn round(&mut self, round_key: u32) {
        let st = &mut self.st;

        let p = [
            st[0] ^ st[4] ^ st[8],
            st[1] ^ st[5] ^ st[9],
            st[2] ^ st[6] ^ st[10],
            st[3] ^ st[7] ^ st[11],
        ];

        let e = [
            p[3].rotate_left(5) ^ p[3].rotate_left(14),
            p[0].rotate_left(5) ^ p[0].rotate_left(14),
            p[1].rotate_left(5) ^ p[1].rotate_left(14),
            p[2].rotate_left(5) ^ p[2].rotate_left(14),
        ];

        let mut tmp = [0u32; 12];

        tmp[0] = e[0] ^ st[0] ^ round_key;
        tmp[1] = e[1] ^ st[1];
        tmp[2] = e[2] ^ st[2];
        tmp[3] = e[3] ^ st[3];

        tmp[4] = e[3] ^ st[7];
        tmp[5] = e[0] ^ st[4];
        tmp[6] = e[1] ^ st[5];
        tmp[7] = e[2] ^ st[6];

        tmp[8] = (e[0] ^ st[8]).rotate_left(11);
        tmp[9] = (e[1] ^ st[9]).rotate_left(11);
        tmp[10] = (e[2] ^ st[10]).rotate_left(11);
        tmp[11] = (e[3] ^ st[11]).rotate_left(11);

        st[0] = (!tmp[4] & tmp[8]) ^ tmp[0];
        st[1] = (!tmp[5] & tmp[9]) ^ tmp[1];
        st[2] = (!tmp[6] & tmp[10]) ^ tmp[2];
        st[3] = (!tmp[7] & tmp[11]) ^ tmp[3];

        st[4] = ((!tmp[8] & tmp[0]) ^ tmp[4]).rotate_left(1);
        st[5] = ((!tmp[9] & tmp[1]) ^ tmp[5]).rotate_left(1);
        st[6] = ((!tmp[10] & tmp[2]) ^ tmp[6]).rotate_left(1);
        st[7] = ((!tmp[11] & tmp[3]) ^ tmp[7]).rotate_left(1);

        st[8] = ((!tmp[2] & tmp[6]) ^ tmp[10]).rotate_left(8);
        st[9] = ((!tmp[3] & tmp[7]) ^ tmp[11]).rotate_left(8);
        st[10] = ((!tmp[0] & tmp[4]) ^ tmp[8]).rotate_left(8);
        st[11] = ((!tmp[1] & tmp[5]) ^ tmp[9]).rotate_left(8);
    }

    pub fn permute(&mut self) {
        for &round_key in &ROUND_KEYS {
            self.round(round_key)
        }
    }
}
