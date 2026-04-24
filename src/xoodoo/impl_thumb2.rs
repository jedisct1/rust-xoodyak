use core::arch::asm;

use super::{Xoodoo, ROUND_KEYS};

impl Xoodoo {
    /// Highly optimized Xoodoo permutation for ARMv7-M (Thumb-2).
    /// Uses all available registers to eliminate memory spills.
    #[allow(clippy::many_single_char_names)]
    pub fn permute(&mut self) {
        let rkeys = ROUND_KEYS.as_ptr();
        unsafe {
            let st_ptr = self.st.as_mut_ptr() as *mut u32;

            asm!(
                // Save callee-saved registers
                "push    {{r4-r11, lr}}",
                // Stack frame: [sp, #4]=st, [sp, #0]=rk
                "push    {{{st}}}",
                "push    {{{rk}}}",
                "mov     r1, #12",                // r1 = round counter

                // Load 12-word state into registers
                // r2-r5: Row 0 (A[0,0]-A[3,0])
                // r6-r9: Row 1 (A[0,1]-A[3,1])
                // r10-r12, lr: Row 2 (A[0,2]-A[3,2])
                "ldr     r0, [sp, #4]",
                "ldmia   r0, {{r2-r12, lr}}",

                ".p2align 2",
                "0:",

                // === θ (Theta) step ===
                // P[x] = A[x,0] ^ A[x,1] ^ A[x,2]
                // OPTIMIZATION: "Parity Correction". Instead of storing P[x] on stack,
                // we update columns sequentially. Original parity of a modified column
                // is recovered by P_orig = P_updated ^ E_applied.

                "eor     r0, r2, r6",   "eor r0, r0, r10",  // r0 = P0
                "eor     r0, r0, r0, ror #23", "ror r0, r0, #27", // r0 = E1
                "eor     r3, r3, r0",   "eor r7, r7, r0",   "eor r11, r11, r0", // Update Col 1

                "eor     r0, r0, r3",   "eor r0, r0, r7",   "eor r0, r0, r11",  // r0 = P1
                "eor     r0, r0, r0, ror #23", "ror r0, r0, #27", // r0 = E2
                "eor     r4, r4, r0",   "eor r8, r8, r0",   "eor r12, r12, r0", // Update Col 2

                "eor     r0, r0, r4",   "eor r0, r0, r8",   "eor r0, r0, r12",  // r0 = P2
                "eor     r0, r0, r0, ror #23", "ror r0, r0, #27", // r0 = E3
                "eor     r5, r5, r0",   "eor r9, r9, r0",   "eor lr, lr, r0",   // Update Col 3

                "eor     r0, r0, r5",   "eor r0, r0, r9",   "eor r0, r0, lr",   // r0 = P3
                "eor     r0, r0, r0, ror #23", "ror r0, r0, #27", // r0 = E0
                "eor     r2, r2, r0",   "eor r6, r6, r0",   "eor r10, r10, r0", // Update Col 0

                // === ρ (Rho) West step ===
                // A[x,1] = A[x-1,1] (Cyclic shift Row 1)
                "mov     r0, r9",
                "mov     r9, r8",
                "mov     r8, r7",
                "mov     r7, r6",
                "mov     r6, r0",

                // A[x,2] = rot(A[x,2], 11)
                "ror     r10, r10, #21",
                "ror     r11, r11, #21",
                "ror     r12, r12, #21",
                "ror     lr, lr, #21",

                // === ι (Iota) step ===
                // A[0,0] = A[0,0] ^ RC
                "ldr     r0, [sp, #0]",                    // Get rk_ptr
                "add     r0, r0, #4",                      // Increment
                "str     r0, [sp, #0]",                    // Save incremented pointer back
                "ldr     r0, [r0, #-4]",                   // Load RC (post-incremented access)
                "eor     r2, r2, r0",                      // A[0,0] ^= RC

                // === χ (Chi) step ===
                // A[x,y] ^= (~A[x,y+1]) & A[x,y+2]
                "bic     r0, r10, r6",  "eor r2, r2, r0",
                "bic     r0, r2, r10",  "eor r6, r6, r0",
                "bic     r0, r6, r2",   "eor r10, r10, r0",
                "bic     r0, r11, r7",  "eor r3, r3, r0",
                "bic     r0, r3, r11",  "eor r7, r7, r0",
                "bic     r0, r7, r3",   "eor r11, r11, r0",
                "bic     r0, r12, r8",  "eor r4, r4, r0",
                "bic     r0, r4, r12",  "eor r8, r8, r0",
                "bic     r0, r8, r4",   "eor r12, r12, r0",
                "bic     r0, lr, r9",   "eor r5, r5, r0",
                "bic     r0, r5, lr",   "eor r9, r9, r0",
                "bic     r0, r9, r5",   "eor lr, lr, r0",

                // === ρ (Rho) East step ===
                // A[x,1] = rot(A[x,1], 1)
                "ror     r6, r6, #31",
                "ror     r7, r7, #31",
                "ror     r8, r8, #31",
                "ror     r9, r9, #31",

                // A[x,2] = rot(A[x-2,2], 8)
                // OPTIMIZATION: Folded rotation into register move using barrel shifter.
                "mov     r0, r10",
                "mov     r10, r12, ror #24",
                "mov     r12, r0,  ror #24",
                "mov     r0, r11",
                "mov     r11, lr,  ror #24",
                "mov     lr,  r0,  ror #24",

                // Loop control
                "subs    r1, r1, #1",
                "bne     0b",

                // Store state back
                "ldr     r0, [sp, #4]",
                "stmia   r0, {{r2-r12, lr}}",

                "pop     {{r0, r1}}",                       // Discard rk, st
                "pop     {{r4-r11, lr}}",

                rk = in(reg) rkeys,
                st = in(reg) st_ptr,
                out("r0") _, out("r1") _, out("r2") _, out("r3") _,
                out("r12") _,
            );
        }
    }
}
