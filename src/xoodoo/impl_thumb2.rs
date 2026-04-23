use super::{Xoodoo, ROUND_KEYS};
#[cfg(all(target_arch = "arm", target_has_atomic = "32"))]
use core::arch::asm;

#[cfg(all(target_arch = "arm", target_has_atomic = "32"))]
impl Xoodoo {
    /// Optimized Xoodoo permutation for ARMv7-M (Thumb-2).
    #[allow(clippy::many_single_char_names)]
    pub fn permute(&mut self) {
        let rkeys = ROUND_KEYS.as_ptr();
        unsafe {
            let st_ptr = self.st.as_mut_ptr() as *mut u32;
            let mut a0 = *st_ptr.add(0);
            let mut a1 = *st_ptr.add(1);
            let mut a2 = *st_ptr.add(2);
            let mut a3 = *st_ptr.add(3);
            let mut b0 = *st_ptr.add(4);
            let mut b1 = *st_ptr.add(5);
            let mut b2 = *st_ptr.add(6);
            let mut b3 = *st_ptr.add(7);
            let mut rk = rkeys;

            asm!(
                "mov  r0, #12",                 // Round counter
                ".p2align 2",
                "3:",
                "push {{r0}}",                   // Save counter

                // === θ step ===
                // P[x] = A[x,0] ^ A[x,1] ^ A[x,2]
                // E[x] = rot(P[x-1], 5) ^ rot(P[x-1], 14)
                // A[x,y] = A[x,y] ^ E[x]

                "ldr  r0, [{st}, #32]", "eor  r0, r0, {a0}", "eor  r0, r0, {b0}", "push {{r0}}", // P0 at [sp, #0]
                "ldr  r0, [{st}, #36]", "eor  r0, r0, {a1}", "eor  r0, r0, {b1}", "push {{r0}}", // P1 at [sp, #0], P0 at [sp, #4]
                "ldr  r0, [{st}, #40]", "eor  r0, r0, {a2}", "eor  r0, r0, {b2}", "push {{r0}}", // P2 at [sp, #0], P1 at [sp, #4], P0 at [sp, #8]
                "ldr  r0, [{st}, #44]", "eor  r0, r0, {a3}", "eor  r0, r0, {b3}",                 // r0 = P3

                // Apply E0..E3
                "ror  r1, r0, #27", "eor  r1, r1, r0, ror #18", // r1 = E0 (from P3)
                "eor  {a0}, {a0}, r1", "eor  {b0}, {b0}, r1",
                "ldr  r0, [{st}, #32]", "eor  r0, r0, r1", "str  r0, [{st}, #32]",

                "ldr  r0, [sp, #8]",             // P0
                "ror  r1, r0, #27", "eor  r1, r1, r0, ror #18", // r1 = E1
                "eor  {a1}, {a1}, r1", "eor  {b1}, {b1}, r1",
                "ldr  r0, [{st}, #36]", "eor  r0, r0, r1", "str  r0, [{st}, #36]",

                "ldr  r0, [sp, #4]",             // P1
                "ror  r1, r0, #27", "eor  r1, r1, r0, ror #18", // r1 = E2
                "eor  {a2}, {a2}, r1", "eor  {b2}, {b2}, r1",
                "ldr  r0, [{st}, #40]", "eor  r0, r0, r1", "str  r0, [{st}, #40]",

                "ldr  r0, [sp, #0]",             // P2
                "ror  r1, r0, #27", "eor  r1, r1, r0, ror #18", // r1 = E3
                "eor  {a3}, {a3}, r1", "eor  {b3}, {b3}, r1",
                "ldr  r0, [{st}, #44]", "eor  r0, r0, r1", "str  r0, [{st}, #44]",

                "add  sp, sp, #12",              // Clean P results

                // === ρ West step ===
                // A[x,1] = A[x-1,1]
                // A[x,2] = rot(A[x,2], 11)

                "mov  r0, {b3}", "mov  {b3}, {b2}", "mov  {b2}, {b1}", "mov  {b1}, {b0}", "mov  {b0}, r0",
                "ldr  r0, [{st}, #32]", "ror  r0, r0, #21", "str  r0, [{st}, #32]",
                "ldr  r0, [{st}, #36]", "ror  r0, r0, #21", "str  r0, [{st}, #36]",
                "ldr  r0, [{st}, #40]", "ror  r0, r0, #21", "str  r0, [{st}, #40]",
                "ldr  r0, [{st}, #44]", "ror  r0, r0, #21", "str  r0, [{st}, #44]",

                // === ι step ===
                // A[0,0] = A[0,0] ^ RC

                "ldr  r1, [{rk}], #4",
                "eor  {a0}, {a0}, r1",

                // === χ step ===
                // A[x,y] = A[x,y] ^ ((not A[x,y+1]) and A[x,y+2])

                // Col 0
                "mov  r1, {a0}", "push {{{b0}}}",
                "ldr  r0, [{st}, #32]", "bic  r0, r0, {b0}", "eor  {a0}, {a0}, r0",
                "ldr  r0, [{st}, #32]", "bic  r0, r1, r0", "eor  {b0}, {b0}, r0",
                "pop  {{r0}}", "bic  r0, r0, r1",
                "ldr  r1, [{st}, #32]", "eor  r0, r0, r1", "str  r0, [{st}, #32]",

                // Col 1
                "mov  r1, {a1}", "push {{{b1}}}",
                "ldr  r0, [{st}, #36]", "bic  r0, r0, {b1}", "eor  {a1}, {a1}, r0",
                "ldr  r0, [{st}, #36]", "bic  r0, r1, r0", "eor  {b1}, {b1}, r0",
                "pop  {{r0}}", "bic  r0, r0, r1",
                "ldr  r1, [{st}, #36]", "eor  r0, r0, r1", "str  r0, [{st}, #36]",

                // Col 2
                "mov  r1, {a2}", "push {{{b2}}}",
                "ldr  r0, [{st}, #40]", "bic  r0, r0, {b2}", "eor  {a2}, {a2}, r0",
                "ldr  r0, [{st}, #40]", "bic  r0, r1, r0", "eor  {b2}, {b2}, r0",
                "pop  {{r0}}", "bic  r0, r0, r1",
                "ldr  r1, [{st}, #40]", "eor  r0, r0, r1", "str  r0, [{st}, #40]",

                // Col 3
                "mov  r1, {a3}", "push {{{b3}}}",
                "ldr  r0, [{st}, #44]", "bic  r0, r0, {b3}", "eor  {a3}, {a3}, r0",
                "ldr  r0, [{st}, #44]", "bic  r0, r1, r0", "eor  {b3}, {b3}, r0",
                "pop  {{r0}}", "bic  r0, r0, r1",
                "ldr  r1, [{st}, #44]", "eor  r0, r0, r1", "str  r0, [{st}, #44]",

                // === ρ East step ===
                // A[x,1] = rot(A[x,1], 1)
                // A[x,2] = rot(A[x-2,2], 8)

                "ror  {b0}, {b0}, #31", "ror  {b1}, {b1}, #31", "ror  {b2}, {b2}, #31", "ror  {b3}, {b3}, #31",
                "ldr  r0, [{st}, #32]", "ldr  r1, [{st}, #40]", "str  r0, [{st}, #40]", "str  r1, [{st}, #32]",
                "ldr  r0, [{st}, #36]", "ldr  r1, [{st}, #44]", "str  r0, [{st}, #44]", "str  r1, [{st}, #36]",
                "ldr  r0, [{st}, #32]", "ror  r0, r0, #24", "str  r0, [{st}, #32]",
                "ldr  r0, [{st}, #36]", "ror  r0, r0, #24", "str  r0, [{st}, #36]",
                "ldr  r0, [{st}, #40]", "ror  r0, r0, #24", "str  r0, [{st}, #40]",
                "ldr  r0, [{st}, #44]", "ror  r0, r0, #24", "str  r0, [{st}, #44]",

                "pop  {{r0}}",                   // Restore loop counter
                "subs r0, #1",
                "bne  3b",

                st = in(reg) st_ptr,
                rk = inout(reg) rk,
                a0 = inout(reg) a0, a1 = inout(reg) a1, a2 = inout(reg) a2, a3 = inout(reg) a3,
                b0 = inout(reg) b0, b1 = inout(reg) b1, b2 = inout(reg) b2, b3 = inout(reg) b3,
                out("r0") _,
                out("r1") _,
            );

            *st_ptr.add(0) = a0;
            *st_ptr.add(1) = a1;
            *st_ptr.add(2) = a2;
            *st_ptr.add(3) = a3;
            *st_ptr.add(4) = b0;
            *st_ptr.add(5) = b1;
            *st_ptr.add(6) = b2;
            *st_ptr.add(7) = b3;
            let _ = rk;
        }
    }
}
