use super::{Xoodoo, ROUND_KEYS};

use core::arch::x86_64::*;

impl Xoodoo {
    #[allow(
        non_upper_case_globals,
        clippy::many_single_char_names,
        clippy::cast_ptr_alignment
    )]
    pub fn permute(&mut self) {
        let st = &mut self.st;
        unsafe {
            let rho_east_2 = _mm_set_epi32(0x0605_0407, 0x0201_0003, 0x0e0d_0c0f, 0x0a09_080b);
            let mut a = _mm_loadu_si128(st.as_ptr().add(0) as *const _);
            let mut b = _mm_loadu_si128(st.as_ptr().add(4) as *const _);
            let mut c = _mm_loadu_si128(st.as_ptr().add(8) as *const _);
            for &round_key in &ROUND_KEYS {
                let mut p = _mm_shuffle_epi32(_mm_xor_si128(_mm_xor_si128(a, b), c), 0x93);
                let mut e = _mm_or_si128(_mm_slli_epi32(p, 5), _mm_srli_epi32(p, 32 - 5));
                p = _mm_or_si128(_mm_slli_epi32(p, 14), _mm_srli_epi32(p, 32 - 14));
                e = _mm_xor_si128(e, p);
                a = _mm_xor_si128(a, e);
                b = _mm_xor_si128(b, e);
                c = _mm_xor_si128(c, e);
                b = _mm_shuffle_epi32(b, 0x93);
                c = _mm_or_si128(_mm_slli_epi32(c, 11), _mm_srli_epi32(c, 32 - 11));
                a = _mm_xor_si128(a, _mm_set_epi32(0, 0, 0, round_key as _));
                a = _mm_xor_si128(a, _mm_andnot_si128(b, c));
                b = _mm_xor_si128(b, _mm_andnot_si128(c, a));
                c = _mm_xor_si128(c, _mm_andnot_si128(a, b));
                b = _mm_or_si128(_mm_slli_epi32(b, 1), _mm_srli_epi32(b, 32 - 1));
                c = _mm_shuffle_epi8(c, rho_east_2);
            }
            _mm_storeu_si128(st.as_mut_ptr().add(0) as *mut _, a);
            _mm_storeu_si128(st.as_mut_ptr().add(4) as *mut _, b);
            _mm_storeu_si128(st.as_mut_ptr().add(8) as *mut _, c);
        }
    }
}
