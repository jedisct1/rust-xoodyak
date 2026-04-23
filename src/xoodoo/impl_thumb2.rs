#![cfg(all(target_arch = "arm", target_has_atomic = "32"))]

use super::{Xoodoo, ROUND_KEYS};
use core::arch::asm;

impl Xoodoo {
    /// Highly optimized Xoodoo permutation for ARMv7-M (Thumb-2).
    /// Uses all available registers to eliminate memory spills.
    #[allow(clippy::many_single_char_names)]
    pub fn permute(&mut self) {
        let rkeys = ROUND_KEYS.as_ptr();
        unsafe {
            let st_ptr = self.st.as_mut_ptr() as *mut u32;

            asm!(
                "push {{r4-r11, lr}}",
                "sub  sp, sp, #12",              // [sp, #0]:rk, [sp, #4]:count, [sp, #8]:st
                "str  {rk}, [sp, #0]",
                "mov  r0, #12",
                "str  r0, [sp, #4]",
                "str  {st}, [sp, #8]",

                // Load entire 12-word state into registers
                // r2-r5: Row 0
                // r6-r9: Row 1
                // r10-r12, lr: Row 2
                "ldr  r0, [sp, #8]",
                "ldmia r0, {{r2-r12, lr}}",

                ".p2align 2",
                "0:",
                // === θ step ===
                // P[x] = A[x,0] ^ A[x,1] ^ A[x,2]
                "eor  r0, r2, r6",  "eor  r0, r0, r10", // r0 = P0
                "eor  r1, r3, r7",  "eor  r1, r1, r11", // r1 = P1
                "push {{r0, r1}}",                      // Save P0, P1 to stack
                "eor  r0, r4, r8",  "eor  r0, r0, r12", // r0 = P2
                "eor  r1, r5, r9",  "eor  r1, r1, lr",  // r1 = P3

                // E[x] = (P[x-1] ^ rot(P[x-1], 9)).rot(5)

                // E0 depends on P3 (r1)
                "eor  r1, r1, r1, ror #23", "ror r1, r1, #27",
                "eor  r2, r2, r1", "eor  r6, r6, r1", "eor  r10, r10, r1",

                // E1 depends on P0 (stack offset 0)
                "ldr  r1, [sp, #0]",
                "eor  r1, r1, r1, ror #23", "ror r1, r1, #27",
                "eor  r3, r3, r1",  "eor  r7, r7, r1",  "eor  r11, r11, r1",

                // E2 depends on P1 (stack offset 4)
                "ldr  r1, [sp, #4]",
                "eor  r1, r1, r1, ror #23", "ror r1, r1, #27",
                "eor  r4, r4, r1",  "eor  r8, r8, r1",  "eor  r12, r12, r1",

                // E3 depends on P2 (r0)
                "eor  r0, r0, r0, ror #23",  "ror r0, r0, #27",
                "eor  r5, r5, r0",  "eor  r9, r9, r0",  "eor  lr, lr, r0",

                "add  sp, sp, #8",                      // Clean P0, P1 from stack

                // === ρ West step ===
                // Row 1: cyclic shift right 1
                "mov  r0, r9", "mov  r9, r8", "mov  r8, r7", "mov  r7, r6", "mov  r6, r0",
                // Row 2: bit rotation 11
                "ror  r10, r10, #21", "ror  r11, r11, #21", "ror  r12, r12, #21", "ror  lr, lr, #21",

                // === ι step ===
                "ldr  r0, [sp, #0]",                    // load rk pointer
                "ldr  r1, [r0], #4",                    // load rc and increment rk
                "str  r0, [sp, #0]",                    // save rk pointer
                "eor  r2, r2, r1",

                // === χ step ===
                "bic  r0, r10, r6",  "eor  r2, r2, r0",  "bic  r0, r2, r10",  "eor  r6, r6, r0",  "bic  r0, r6, r2",   "eor  r10, r10, r0",
                "bic  r0, r11, r7",  "eor  r3, r3, r0",  "bic  r0, r3, r11",  "eor  r7, r7, r0",  "bic  r0, r7, r3",   "eor  r11, r11, r0",
                "bic  r0, r12, r8",  "eor  r4, r4, r0",  "bic  r0, r4, r12",  "eor  r8, r8, r0",  "bic  r0, r8, r4",   "eor  r12, r12, r0",
                "bic  r0, lr, r9",   "eor  r5, r5, r0",  "bic  r0, r5, lr",   "eor  r9, r9, r0",  "bic  r0, r9, r5",   "eor  lr, lr, r0",

                // === ρ East step ===
                // Row 1: bit rotation 1
                "ror  r6, r6, #31", "ror  r7, r7, #31", "ror  r8, r8, #31", "ror  r9, r9, #31",
                // Row 2: cyclic shift right 2 + bit rotation 8
                "mov  r0, r10", "mov  r1, r11", "mov  r10, r12", "mov  r11, lr", "mov  r12, r0", "mov  lr, r1",
                "ror  r10, r10, #24", "ror  r11, r11, #24", "ror  r12, r12, #24", "ror  lr, lr, #24",

                "ldr  r0, [sp, #4]",                    // load counter
                "subs r0, r0, #1",
                "str  r0, [sp, #4]",                    // save counter
                "bne  0b",

                "ldr  r0, [sp, #8]",                    // load initial st pointer
                "stmia r0, {{r2-r12, lr}}",             // Save back state

                "add  sp, sp, #12",
                "pop  {{r4-r11, lr}}",                  // Restore and return

                rk = in(reg) rkeys,
                st = in(reg) st_ptr,
                out("r0") _, out("r1") _, out("r2") _, out("r3") _,
                out("r12") _,
            );
        }
    }
}
