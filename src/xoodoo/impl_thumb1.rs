use core::arch::asm;

use super::{Xoodoo, ROUND_KEYS};

impl Xoodoo {
    /// Optimized Xoodoo permutation for ARMv6-M (Cortex-M0).
    #[allow(clippy::many_single_char_names)]
    pub fn permute(&mut self) {
        let rkeys = ROUND_KEYS.as_ptr();
        unsafe {
            let st_ptr = self.st.as_mut_ptr() as *mut u32;

            asm!(
                // Preserve callee-saved registers
                "push    {{r4-r7, lr}}",
                "mov     r0, r8",
                "mov     r1, r9",
                "mov     r2, r10",
                "mov     r3, r11",
                "push    {{r0-r3}}",
                "mov     r0, r12",
                "push    {{r0}}",

                // Stack frame: [0]=A30, [4]=rk, [8]=counter, [12]=st
                "push    {{{st}}}",                 // [sp, #12]:st
                "movs    r0, #12",
                "push    {{r0}}",                    // [sp, #8]:counter
                "push    {{{rk}}}",                  // [sp, #4]:rk
                "movs    r0, #0",
                "push    {{r0}}",                    // [sp, #0]:A30 placeholder

                // Load initial state
                // Row 0: r3, r8, r9, [sp, #0]   (A[0,0], A[1,0], A[2,0], A[3,0])
                // Row 1: r10, r11, r12, lr     (A[0,1], A[1,1], A[2,1], A[3,1])
                // Row 2: r4, r5, r6, r7        (A[0,2], A[1,2], A[2,2], A[3,2])
                "mov     r0, {st}",
                "ldm     r0!, {{r3, r4, r5, r6}}",
                "mov     r8, r4",
                "mov     r9, r5",
                "str     r6, [sp, #0]",
                "ldm     r0!, {{r4, r5, r6, r7}}",
                "mov     r10, r4",
                "mov     r11, r5",
                "mov     r12, r6",
                "mov     lr, r7",
                "ldm     r0!, {{r4, r5, r6, r7}}",

                ".p2align 2",
                "0:",

                // === θ (Theta) step ===
                // P[x] = A[x,0] ^ A[x,1] ^ A[x,2]
                // E[x] = rot(P[x-1], 5) ^ rot(P[x-1], 14)

                // --- Calculate E0 from P3 ---
                // P3 = A[3,0] ^ A[3,1] ^ A[3,2]
                "ldr     r0, [sp, #0]",
                "mov     r1, lr",
                "eors    r0, r1",
                "eors    r0, r7",                   // r0 = P3
                "mov     r1, r0",
                "movs    r2, #23",                  // rot 9
                "rors    r1, r2",
                "eors    r1, r0",
                "movs    r2, #27",                  // rot 5
                "rors    r1, r2",                   // r1 = E0

                // Apply E0 to Col 0
                "mov     r0, r3",                   // P0 calculation starts here
                "mov     r2, r10",
                "eors    r0, r2",
                "eors    r0, r4",                   // r0 = P0
                "eors    r3, r1",                   // A[0,0] ^= E0
                "mov     r2, r10",
                "eors    r2, r1",
                "mov     r10, r2",                  // A[0,1] ^= E0
                "eors    r4, r1",                   // A[0,2] ^= E0

                // --- Calculate E1 from P0 ---
                "mov     r1, r0",
                "movs    r2, #23",
                "rors    r1, r2",
                "eors    r1, r0",
                "movs    r2, #27",
                "rors    r1, r2",               // r1 = E1

                // Apply E1 to Col 1
                "mov     r0, r8",                   // P1 calculation
                "mov     r2, r11",
                "eors    r0, r2",
                "eors    r0, r5",               // r0 = P1
                "mov     r2, r8",
                "eors    r2, r1",
                "mov     r8, r2",                   // A[1,0] ^= E1
                "mov     r2, r11",
                "eors    r2, r1",
                "mov     r11, r2",                  // A[1,1] ^= E1
                "eors    r5, r1",                   // A[1,2] ^= E1

                // --- Calculate E2 from P1 ---
                "mov     r1, r0",
                "movs    r2, #23",
                "rors    r1, r2",
                "eors    r1, r0",
                "movs    r2, #27",
                "rors    r1, r2",               // r1 = E2

                // Apply E2 to Col 2
                "mov     r0, r9",                   // P2 calculation
                "mov     r2, r12",
                "eors    r0, r2",
                "eors    r0, r6",               // r0 = P2
                "mov     r2, r9",
                "eors    r2, r1",
                "mov     r9, r2",                   // A[2,0] ^= E2
                "mov     r2, r12",
                "eors    r2, r1",
                "mov     r12, r2",                  // A[2,1] ^= E2
                "eors    r6, r1",                   // A[2,2] ^= E2

                // --- Calculate E3 from P2 ---
                "mov     r1, r0",
                "movs    r2, #23",
                "rors    r1, r2",
                "eors    r1, r0",
                "movs    r2, #27",
                "rors    r1, r2",               // r1 = E3

                // Apply E3 to Col 3
                "ldr     r0, [sp, #0]",
                "eors    r0, r1",
                "str     r0, [sp, #0]",             // A[3,0] ^= E3
                "mov     r2, lr",
                "eors    r2, r1",
                "mov     lr, r2",                   // A[3,1] ^= E3
                "eors    r7, r1",                   // A[3,2] ^= E3

                // === ρ (Rho) West step ===
                // A[x,1] = A[x-1,1] (Cyclic shift Row 1)
                "mov     r0, lr",
                "mov     lr, r12",
                "mov     r12, r11",
                "mov     r11, r10",
                "mov     r10, r0",

                // A[x,2] = rot(A[x,2], 11)
                "movs    r0, #21",                  // rot 11
                "rors    r4, r0",
                "rors    r5, r0",
                "rors    r6, r0",
                "rors    r7, r0",

                // === ι (Iota) step ===
                // A[0,0] ^= RC
                "ldr     r0, [sp, #4]",
                "ldm     r0!, {{r1}}",
                "str     r0, [sp, #4]",
                "eors    r3, r1",

                // === χ (Chi) step ===
                // A[x,0] ^= (~A[x,1]) & A[x,2]
                // A[x,1] ^= (~A[x,2]) & A[x,0]
                // A[x,2] ^= (~A[x,0]) & A[x,1]

                // x=0 (Col 0)
                "mov     r1, r10",
                "mov     r2, r4",
                "bics    r2, r1",
                "eors    r3, r2",                   // a0 = a0_new
                "mov     r2, r3",
                "bics    r2, r4",
                "eors    r2, r1",
                "mov     r10, r2",                  // a1 = a1_new
                "bics    r2, r3",
                "eors    r4, r2",                   // a2 = a2_new

                // x=1 (Col 1)
                "mov     r1, r11",
                "mov     r2, r5",
                "bics    r2, r1",
                "mov     r0, r8",
                "eors    r2, r0",
                "mov     r8, r2",                   // a0 = a0_new
                "bics    r2, r5",
                "eors    r2, r1",
                "mov     r11, r2",                  // a1 = a1_new
                "mov     r0, r8",
                "bics    r2, r0",
                "eors    r5, r2",                   // a2 = a2_new

                // x=2 (Col 2)
                "mov     r1, r12",
                "mov     r2, r6",
                "bics    r2, r1",
                "mov     r0, r9",
                "eors    r2, r0",
                "mov     r9, r2",                   // a0 = a0_new
                "bics    r2, r6",
                "eors    r2, r1",
                "mov     r12, r2",                  // a1 = a1_new
                "mov     r0, r9",
                "bics    r2, r0",
                "eors    r6, r2",                   // a2 = a2_new

                // x=3 (Col 3)
                "ldr     r0, [sp, #0]",
                "mov     r1, lr",
                "mov     r2, r7",
                "bics    r2, r1",
                "eors    r0, r2",
                "str     r0, [sp, #0]",             // a0 = a0_new
                "mov     r2, r0",
                "bics    r2, r7",
                "eors    r2, r1",
                "mov     lr, r2",                   // a1 = a1_new
                "bics    r2, r0",
                "eors    r7, r2",                   // a2 = a2_new

                // === ρ (Rho) East step ===
                // A[x,1] = rot(A[x,1], 1)
                "movs    r0, #31",                  // rot 1
                "mov     r1, r10",  "rors r1, r0",  "mov r10, r1",
                "mov     r1, r11",  "rors r1, r0",  "mov r11, r1",
                "mov     r1, r12",  "rors r1, r0",  "mov r12, r1",
                "mov     r1, lr",   "rors r1, r0",  "mov lr, r1",

                // A[x,2] = rot(A[x-2,2], 8)
                "movs    r0, #24",                  // rot 8
                "rors    r4, r0",
                "rors    r5, r0",
                "rors    r6, r0",
                "rors    r7, r0",
                // Cyclic shift Row 2 by 2
                "mov     r0, r4",   "mov r4, r6",   "mov r6, r0",
                "mov     r0, r5",   "mov r5, r7",   "mov r7, r0",

                // Loop control
                "ldr     r0, [sp, #8]",
                "subs    r0, r0, #1",
                "str     r0, [sp, #8]",
                "bne     0b",

                // Save back state
                "ldr     r0, [sp, #12]",
                "stm     r0!, {{r3}}",
                "mov     r1, r8",
                "mov     r2, r9",
                "ldr     r3, [sp, #0]",
                "stm     r0!, {{r1-r3}}",
                "mov     r1, r10",
                "mov     r2, r11",
                "mov     r3, r12",
                "stm     r0!, {{r1-r3}}",
                "mov     r1, lr",
                "stm     r0!, {{r1, r4-r7}}",

                // Restore registers
                "pop     {{r0, r1, r2, r3}}",               // Discard frame
                "pop     {{r0}}",
                "mov     r12, r0",
                "pop     {{r0-r3}}",
                "mov     r8, r0",
                "mov     r9, r1",
                "mov     r10, r2",
                "mov     r11, r3",
                "pop     {{r4-r7}}",
                "pop     {{r0}}",
                "mov     lr, r0",

                rk = in(reg) rkeys,
                st = in(reg) st_ptr,
                out("r0") _, out("r1") _, out("r2") _, out("r3") _,
            );
        }
    }
}
