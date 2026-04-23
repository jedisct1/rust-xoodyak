#![cfg(all(target_arch = "arm", not(target_has_atomic = "32")))]

use super::{Xoodoo, ROUND_KEYS};
use core::arch::asm;

impl Xoodoo {
    /// Optimized Xoodoo permutation for ARMv6-M (Cortex-M0).
    #[allow(clippy::many_single_char_names)]
    pub fn permute(&mut self) {
        let rkeys = ROUND_KEYS.as_ptr();
        unsafe {
            let st_ptr = self.st.as_mut_ptr() as *mut u32;

            asm!(
                // Preserve callee-saved registers
                "push {{r4-r7, lr}}",
                "mov  r0, r8",
                "mov  r1, r9",
                "mov  r2, r10",
                "mov  r3, r11",
                "push {{r0-r3}}",
                "mov  r0, r12",
                "push {{r0}}",

                // Stack frame: [0]=A03, [4]=rk, [8]=counter, [12]=st
                "sub  sp, sp, #16",
                "str  {rk}, [sp, #4]",
                "movs r0, #12",
                "str  r0, [sp, #8]",
                "str  {st}, [sp, #12]",

                // Load initial state
                "mov  r0, {st}",
                "ldm  r0!, {{r3, r4, r5, r6}}",
                "mov  r8, r4",
                "mov  r9, r5",
                "str  r6, [sp, #0]",                                // Row 0: A00..A03 in r3, r8, r9, [sp]
                "ldm  r0!, {{r4, r5, r6, r7}}",
                "mov  r10, r4",
                "mov  r11, r5",
                "mov  r12, r6",
                "mov  lr, r7",                                      // Row 1: A10..A13 in r10, r11, r12, lr
                "ldm  r0!, {{r4, r5, r6, r7}}",                     // Row 2: A20..A23 in r4, r5, r6, r7

                ".p2align 2",
                "0:", // Round loop

                // === θ step ===
                // P[x] = A[x,0] ^ A[x,1] ^ A[x,2]
                // E[x] = rot(P[x-1], 5) ^ rot(P[x-1], 14)          (calculated as (P ^ rot(P, 9)).rot(5))
                // A[x,y] = A[x,y] ^ E[x]

                "ldr  r0, [sp, #0]",
                "mov  r1, lr",
                "eors r0, r0, r1",
                "eors r0, r0, r7",                                  // r0 = P3
                "mov  r1, r0",
                "movs r2, #23",
                "rors r1, r1, r2",
                "eors r1, r1, r0",
                "movs r2, #27",
                "rors r1, r1, r2",                                  // r1 = E0
                "mov  r0, r3",
                "mov  r2, r10",
                "eors r0, r0, r2",
                "eors r0, r0, r4",                                  // r0 = P0
                "eors r3, r3, r1",
                "mov  r2, r10",
                "eors r2, r2, r1",
                "mov  r10, r2",
                "eors r4, r4, r1",

                "mov  r1, r0",
                "movs r2, #23",
                "rors r1, r1, r2",
                "eors r1, r1, r0",
                "movs r2, #27",
                "rors r1, r1, r2",                                  // r1 = E1
                "mov  r0, r8",
                "mov  r2, r11",
                "eors r0, r0, r2",
                "eors r0, r0, r5",                                  // r0 = P1
                "mov  r2, r8",
                "eors r2, r2, r1",
                "mov  r8, r2",
                "mov  r2, r11",
                "eors r2, r2, r1",
                "mov  r11, r2",
                "eors r5, r5, r1",

                "mov  r1, r0",
                "movs r2, #23",
                "rors r1, r1, r2",
                "eors r1, r1, r0",
                "movs r2, #27",
                "rors r1, r1, r2",                                  // r1 = E2
                "mov  r0, r9",
                "mov  r2, r12",
                "eors r0, r0, r2",
                "eors r0, r0, r6",                                  // r0 = P2
                "mov  r2, r9",
                "eors r2, r2, r1",
                "mov  r9, r2",
                "mov  r2, r12",
                "eors r2, r2, r1",
                "mov  r12, r2",
                "eors r6, r6, r1",

                "mov  r1, r0",
                "movs r2, #23",
                "rors r1, r1, r2",
                "eors r1, r1, r0",
                "movs r2, #27",
                "rors r1, r1, r2",                                  // r1 = E3
                "ldr  r0, [sp, #0]",
                "eors r0, r0, r1",
                "str  r0, [sp, #0]",
                "mov  r2, lr",
                "eors r2, r2, r1",
                "mov  lr, r2",
                "eors r7, r7, r1",

                // === ρ West step ===
                // A[x,1] = A[x-1,1]
                // A[x,2] = rot(A[x,2], 11)

                "mov  r0, lr",
                "mov  lr, r12",
                "mov  r12, r11",
                "mov  r11, r10",
                "mov  r10, r0",

                "movs r0, #21",
                "rors r4, r4, r0",
                "rors r5, r5, r0",
                "rors r6, r6, r0",
                "rors r7, r7, r0",

                // === ι step ===
                // A[0,0] = A[0,0] ^ RC

                "ldr  r0, [sp, #4]",
                "ldm  r0!, {{r1}}",
                "str  r0, [sp, #4]",
                "eors r3, r3, r1",

                // === χ step ===
                // A[x,y] = A[x,y] ^ ((not A[x,y+1]) and A[x,y+2])
                // Uses BIC(dest, src) -> dest = dest & ~src

                // Col 0
                "mov  r0, r3",
                "mov  r1, r10",
                "bics r2, r4, r1",
                "eors r3, r3, r2",
                "bics r2, r0, r4",
                "eors r10, r10, r2",
                "bics r2, r1, r0",
                "eors r4, r4, r2",
                // Col 1
                "mov  r0, r8",
                "mov  r1, r11",
                "bics r2, r5, r1",
                "eors r8, r8, r2",
                "bics r2, r0, r5",
                "eors r11, r11, r2",
                "bics r2, r1, r0",
                "eors r5, r5, r2",
                // Col 2
                "mov  r0, r9",
                "mov  r1, r12",
                "bics r2, r6, r1",
                "eors r9, r9, r2",
                "bics r2, r0, r6",
                "eors r12, r12, r2",
                "bics r2, r1, r0",
                "eors r6, r6, r2",
                // Col 3
                "ldr  r0, [sp, #0]",
                "mov  r1, lr",
                "bics r2, r7, r1",
                "eors r0, r0, r2",
                "str  r0, [sp, #0]",
                "bics r2, r0, r7",
                "eors lr, lr, r2",
                "bics r2, r1, r0",
                "eors r7, r7, r2",

                // === ρ East step ===
                // A[x,1] = rot(A[x,1], 1)
                // A[x,2] = rot(A[x-2,2], 8)

                "movs r0, #31",
                "mov  r1, r10", "rors r1, r1, r0", "mov  r10, r1",
                "mov  r1, r11", "rors r1, r1, r0", "mov  r11, r1",
                "mov  r1, r12", "rors r1, r1, r0", "mov  r12, r1",
                "mov  r1, lr",  "rors r1, r1, r0", "mov  lr, r1",

                "movs r0, #24",
                "rors r4, r4, r0",
                "rors r5, r5, r0",
                "rors r6, r6, r0",
                "rors r7, r7, r0",
                "mov  r0, r4",
                "mov  r4, r6",
                "mov  r6, r0",
                "mov  r0, r5",
                "mov  r5, r7",
                "mov  r7, r0",

                "ldr  r0, [sp, #8]",
                "subs r0, r0, #1",
                "str  r0, [sp, #8]",
                "beq  1f",
                "b    0b",
                "1:",

                // Save back state
                "ldr  r0, [sp, #12]",
                "stm  r0!, {{r3}}",
                "mov  r1, r8",
                "mov  r2, r9",
                "ldr  r3, [sp, #0]",
                "stm  r0!, {{r1-r3}}",
                "mov  r1, r10",
                "mov  r2, r11",
                "mov  r3, r12",
                "stm  r0!, {{r1-r3}}",
                "mov  r1, lr",
                "stm  r0!, {{r1, r4-r7}}",

                // Restore registers
                "add  sp, sp, #16",
                "pop  {{r0}}",
                "mov  r12, r0",
                "pop  {{r0-r3}}",
                "mov  r8, r0",
                "mov  r9, r1",
                "mov  r10, r2",
                "mov  r11, r3",
                "pop  {{r4-r7}}",
                "pop  {{r0}}",
                "mov  lr, r0",

                rk = in(reg) rkeys,
                st = in(reg) st_ptr,
                out("r0") _, out("r1") _, out("r2") _, out("r3") _,
            );
        }
    }
}
