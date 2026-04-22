use super::{Xoodoo, ROUND_KEYS};
#[cfg(all(target_arch = "arm", target_feature = "thumb2"))]
use core::arch::asm;

#[cfg(all(target_arch = "arm", target_feature = "thumb2"))]
impl Xoodoo {
    #[allow(clippy::many_single_char_names)]
    pub fn permute(&mut self) {
        let st_ptr = self.st.as_mut_ptr() as *mut u32;
        let mut rkeys = ROUND_KEYS.as_ptr();
        let rkeys_end = unsafe { rkeys.add(12) };
        let c_ptr = unsafe { st_ptr.add(8) };

        unsafe {
            let mut a0 = *st_ptr.add(0);
            let mut a1 = *st_ptr.add(1);
            let mut a2 = *st_ptr.add(2);
            let mut a3 = *st_ptr.add(3);
            let mut b0 = *st_ptr.add(4);
            let mut b1 = *st_ptr.add(5);
            let mut b2 = *st_ptr.add(6);
            let mut b3 = *st_ptr.add(7);

            asm!(
                ".p2align 2",
                "2:", // .Lround_loop

                // === THETA ===
                // P0..3 are computed into the b registers temporarily,
                // so we must save the real B plane to the stack.
                // We use generic SUB/STR instead of PUSH {b0..b3} because
                // rustc register allocation may not map b0..b3 to contiguous ascending registers.
                "sub sp, sp, #16",
                "str {b0}, [sp, #0]",
                "str {b1}, [sp, #4]",
                "str {b2}, [sp, #8]",
                "str {b3}, [sp, #12]",

                // Theta: Compute P in b0..b3
                // P0 = a0 ^ b0 ^ c0
                "ldr {t1}, [{c_ptr}, #0]",
                "eor {b0}, {a0}, {b0}", "eor {b0}, {b0}, {t1}",
                // P1
                "ldr {t1}, [{c_ptr}, #4]",
                "eor {b1}, {a1}, {b1}", "eor {b1}, {b1}, {t1}",
                // P2
                "ldr {t1}, [{c_ptr}, #8]",
                "eor {b2}, {a2}, {b2}", "eor {b2}, {b2}, {t1}",
                // P3
                "ldr {t1}, [{c_ptr}, #12]",
                "eor {b3}, {a3}, {b3}", "eor {b3}, {b3}, {t1}",

                // Theta: Compute E. E0 = P3.rotr(27) ^ P3.rotr(18)
                "ror {t1}, {b2}, #27", "eor {t1}, {t1}, {b2}, ror #18", // t1 = E3
                "ror {t2}, {b1}, #27", "eor {t2}, {t2}, {b1}, ror #18", // t2 = E2
                "ror {b1}, {b0}, #27", "eor {b1}, {b1}, {b0}, ror #18", // b1 = E1 (P0 is now consumed)
                "ror {b2}, {b3}, #27", "eor {b2}, {b2}, {b3}, ror #18", // b2 = E0 (P3 is now consumed)

                // Apply E to A completely in registers
                "eor {a0}, {a0}, {b2}",
                "eor {a1}, {a1}, {b1}",
                "eor {a2}, {a2}, {t2}",
                "eor {a3}, {a3}, {t1}",

                // Apply E to B (which currently lives on the stack)
                "ldr {b0}, [sp, #0]", "eor {b0}, {b0}, {b2}", "str {b0}, [sp, #0]",
                "ldr {b0}, [sp, #4]", "eor {b0}, {b0}, {b1}", "str {b0}, [sp, #4]",
                "ldr {b0}, [sp, #8]", "eor {b0}, {b0}, {t2}", "str {b0}, [sp, #8]",
                "ldr {b0}, [sp, #12]","eor {b0}, {b0}, {t1}", "str {b0}, [sp, #12]",

                // Apply E to C (lives in main `st` array)
                "ldr {b0}, [{c_ptr}, #0]", "eor {b0}, {b0}, {b2}", "str {b0}, [{c_ptr}, #0]",
                "ldr {b0}, [{c_ptr}, #4]", "eor {b0}, {b0}, {b1}", "str {b0}, [{c_ptr}, #4]",
                "ldr {b0}, [{c_ptr}, #8]", "eor {b0}, {b0}, {t2}", "str {b0}, [{c_ptr}, #8]",
                "ldr {b0}, [{c_ptr}, #12]","eor {b0}, {b0}, {t1}", "str {b0}, [{c_ptr}, #12]",

                // === RHO WEST ===
                // B is shifted Left by 1 = Right by 31.
                // We load B from stack with the column-shifted registry mapping:
                // Old B3 goes to New B0, Old B0 to New B1, etc.
                "ldr {b1}, [sp, #0]", "ror {b1}, {b1}, #31",
                "ldr {b2}, [sp, #4]", "ror {b2}, {b2}, #31",
                "ldr {b3}, [sp, #8]", "ror {b3}, {b3}, #31",
                "ldr {b0}, [sp, #12]","ror {b0}, {b0}, #31",
                // We purposely DO NOT free the sp stack layer here,
                // so we can seamlessly reuse it as tmp scratch for Chi!

                // === IOTA ===
                "ldr {t1}, [{rkeys}], #4", // Load Round key & auto increment
                "eor {a0}, {a0}, {t1}",

                // === CHI and RHO EAST (combined) ===
                // To minimize scratch register dependencies securely, we process columns one by one
                // and use the allocated stack layer to temporarily hold `tmpA` across the calculations.

                // Col 0
                "ldr {t1}, [{c_ptr}, #0]",
                "ror {t1}, {t1}, #21", // t1 = C0
                "bic {t2}, {t1}, {b0}", "eor {t2}, {a0}, {t2}", // t2 = tmpA0
                "str {t2}, [sp, #0]", // Save tmpA0
                "bic {t2}, {a0}, {t1}", "eor {t2}, {b0}, {t2}", // t2 = tmpB0
                "bic {a0}, {b0}, {a0}", "eor {t1}, {t1}, {a0}", // t1 = tmpC0
                "ror {t2}, {t2}, #31", "mov {b0}, {t2}", // rotB = 1L (31R), save B0
                "ror {t1}, {t1}, #24", "str {t1}, [{c_ptr}, #8]", // rotC = 8L (24R), save to C2!
                "ldr {a0}, [sp, #0]", // restore A0

                // Col 1
                "ldr {t1}, [{c_ptr}, #4]",
                "ror {t1}, {t1}, #21", // t1 = C1
                "bic {t2}, {t1}, {b1}", "eor {t2}, {a1}, {t2}", // t2 = tmpA1
                "str {t2}, [sp, #4]", // Save tmpA1
                "bic {t2}, {a1}, {t1}", "eor {t2}, {b1}, {t2}", // t2 = tmpB1
                "bic {a1}, {b1}, {a1}", "eor {t1}, {t1}, {a1}", // t1 = tmpC1
                "ror {t2}, {t2}, #31", "mov {b1}, {t2}", // rotB, save B1
                "ror {t1}, {t1}, #24", "str {t1}, [{c_ptr}, #12]", // rotC, save to C3!
                "ldr {a1}, [sp, #4]", // restore A1

                // Col 2
                "ldr {t1}, [{c_ptr}, #8]",
                "ror {t1}, {t1}, #21", // t1 = C2
                "bic {t2}, {t1}, {b2}", "eor {t2}, {a2}, {t2}", // t2 = tmpA2
                "str {t2}, [sp, #8]", // Save tmpA2
                "bic {t2}, {a2}, {t1}", "eor {t2}, {b2}, {t2}", // t2 = tmpB2
                "bic {a2}, {b2}, {a2}", "eor {t1}, {t1}, {a2}", // t1 = tmpC2
                "ror {t2}, {t2}, #31", "mov {b2}, {t2}", // rotB, save B2
                "ror {t1}, {t1}, #24", "str {t1}, [{c_ptr}, #0]", // rotC, save to C0!
                "ldr {a2}, [sp, #8]", // restore A2

                // Col 3
                "ldr {t1}, [{c_ptr}, #12]",
                "ror {t1}, {t1}, #21", // t1 = C3
                "bic {t2}, {t1}, {b3}", "eor {t2}, {a3}, {t2}", // t2 = tmpA3
                "str {t2}, [sp, #12]", // Save tmpA3
                "bic {t2}, {a3}, {t1}", "eor {t2}, {b3}, {t2}", // t2 = tmpB3
                "bic {a3}, {b3}, {a3}", "eor {t1}, {t1}, {a3}", // t1 = tmpC3
                "ror {t2}, {t2}, #31", "mov {b3}, {t2}", // rotB, save B3
                "ror {t1}, {t1}, #24", "str {t1}, [{c_ptr}, #4]", // rotC, save to C1!
                "ldr {a3}, [sp, #12]", // restore A3

                "add sp, sp, #16", // Now we free the scratch stack correctly

                "cmp {rkeys}, {rkeys_end}",
                "bne 2b",

                a0 = inout(reg) a0,
                a1 = inout(reg) a1,
                a2 = inout(reg) a2,
                a3 = inout(reg) a3,
                b0 = inout(reg) b0,
                b1 = inout(reg) b1,
                b2 = inout(reg) b2,
                b3 = inout(reg) b3,
                rkeys = inout(reg) rkeys,
                rkeys_end = in(reg) rkeys_end,
                c_ptr = in(reg) c_ptr,
                t1 = out(reg) _,
                t2 = out(reg) _,
            );

            *st_ptr.add(0) = a0;
            *st_ptr.add(1) = a1;
            *st_ptr.add(2) = a2;
            *st_ptr.add(3) = a3;
            *st_ptr.add(4) = b0;
            *st_ptr.add(5) = b1;
            *st_ptr.add(6) = b2;
            *st_ptr.add(7) = b3;
        }
    }
}
