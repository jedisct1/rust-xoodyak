use core::arch::aarch64::*;

use super::{Xoodoo, ROUND_KEYS};

impl Xoodoo {
    #[allow(
        non_upper_case_globals,
        clippy::many_single_char_names,
        clippy::cast_ptr_alignment
    )]
    pub fn permute(&mut self) {
        macro_rules! rol32in128 {
            ($x:ident, $b:literal) => {
                vsriq_n_u32::<{ 32 - $b }>(vshlq_n_u32::<$b>($x), $x)
            };
        }

        unsafe {
            let mut a = vld1q_u32(self.st.as_ptr().add(0));
            let mut b = vld1q_u32(self.st.as_ptr().add(4));
            let mut c = vld1q_u32(self.st.as_ptr().add(8));

            for &round_key in &ROUND_KEYS {
                let mut d = veorq_u32(veorq_u32(a, b), c);
                d = vextq_u32::<3>(d, d);
                let mut e = rol32in128!(d, 5);
                let mut f = rol32in128!(d, 14);
                e = veorq_u32(e, f);
                a = veorq_u32(a, e);
                b = veorq_u32(b, e);
                f = veorq_u32(c, e);
                c = rol32in128!(f, 11);
                b = vextq_u32::<3>(b, b);
                a = veorq_u32(a, vsetq_lane_u32::<0>(round_key, vmovq_n_u32(0)));
                e = vbicq_u32(c, b);
                d = vbicq_u32(a, c);
                f = vbicq_u32(b, a);
                a = veorq_u32(a, e);
                d = veorq_u32(b, d);
                c = veorq_u32(c, f);
                f = vextq_u32::<2>(c, c);
                b = rol32in128!(d, 1);
                c = rol32in128!(f, 8);
            }

            vst1q_u32(self.st.as_ptr().add(0) as *mut _, a);
            vst1q_u32(self.st.as_ptr().add(4) as *mut _, b);
            vst1q_u32(self.st.as_ptr().add(8) as *mut _, c);
        }
    }
}
