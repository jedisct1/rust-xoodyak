use super::{Xoodoo, ROUND_KEYS};

impl Xoodoo {
    #[inline(always)]
    fn round(st_words: &mut [u32; 12], round_key: u32) {
        let p = [
            st_words[0] ^ st_words[4] ^ st_words[8],
            st_words[1] ^ st_words[5] ^ st_words[9],
            st_words[2] ^ st_words[6] ^ st_words[10],
            st_words[3] ^ st_words[7] ^ st_words[11],
        ];

        let e = [
            p[3].rotate_left(5) ^ p[3].rotate_left(14),
            p[0].rotate_left(5) ^ p[0].rotate_left(14),
            p[1].rotate_left(5) ^ p[1].rotate_left(14),
            p[2].rotate_left(5) ^ p[2].rotate_left(14),
        ];

        let mut tmp = [0u32; 12];

        tmp[0] = e[0] ^ st_words[0] ^ round_key;
        tmp[1] = e[1] ^ st_words[1];
        tmp[2] = e[2] ^ st_words[2];
        tmp[3] = e[3] ^ st_words[3];

        tmp[4] = e[3] ^ st_words[7];
        tmp[5] = e[0] ^ st_words[4];
        tmp[6] = e[1] ^ st_words[5];
        tmp[7] = e[2] ^ st_words[6];

        tmp[8] = (e[0] ^ st_words[8]).rotate_left(11);
        tmp[9] = (e[1] ^ st_words[9]).rotate_left(11);
        tmp[10] = (e[2] ^ st_words[10]).rotate_left(11);
        tmp[11] = (e[3] ^ st_words[11]).rotate_left(11);

        st_words[0] = (!tmp[4] & tmp[8]) ^ tmp[0];
        st_words[1] = (!tmp[5] & tmp[9]) ^ tmp[1];
        st_words[2] = (!tmp[6] & tmp[10]) ^ tmp[2];
        st_words[3] = (!tmp[7] & tmp[11]) ^ tmp[3];

        st_words[4] = ((!tmp[8] & tmp[0]) ^ tmp[4]).rotate_left(1);
        st_words[5] = ((!tmp[9] & tmp[1]) ^ tmp[5]).rotate_left(1);
        st_words[6] = ((!tmp[10] & tmp[2]) ^ tmp[6]).rotate_left(1);
        st_words[7] = ((!tmp[11] & tmp[3]) ^ tmp[7]).rotate_left(1);

        st_words[8] = ((!tmp[2] & tmp[6]) ^ tmp[10]).rotate_left(8);
        st_words[9] = ((!tmp[3] & tmp[7]) ^ tmp[11]).rotate_left(8);
        st_words[10] = ((!tmp[0] & tmp[4]) ^ tmp[8]).rotate_left(8);
        st_words[11] = ((!tmp[1] & tmp[5]) ^ tmp[9]).rotate_left(8);
    }

    pub fn permute(&mut self) {
        let mut st_words = self.to_words();
        for &round_key in &ROUND_KEYS {
            Self::round(&mut st_words, round_key)
        }
        self.init_from_words(st_words);
    }
}
