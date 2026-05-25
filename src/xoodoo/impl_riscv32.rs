use core::arch::asm;

use super::{Xoodoo, ROUND_KEYS};

impl Xoodoo {
    /// Dynamically binds the 12-word state to registers for optimal compiler allocation.
    #[allow(clippy::many_single_char_names)]
    pub fn permute(&mut self) {
        let st_words = unsafe { &mut *(self.st.as_mut_ptr() as *mut [u32; 12]) };
        let rkeys = ROUND_KEYS.as_ptr();
        let rkeys_end = unsafe { rkeys.add(12) };
        let rk = rkeys;

        unsafe {
            asm!(
                "0:",
                // === θ (Theta) step ===
                "xor     t0, {s0}, {s4}",
                "xor     t0, t0, {s8}",          // t0 = P0
                "xor     t1, {s1}, {s5}",
                "xor     t1, t1, {s9}",          // t1 = P1
                "xor     t2, {s2}, {s6}",
                "xor     t2, t2, {s10}",         // t2 = P2
                "xor     t3, {s3}, {s7}",
                "xor     t3, t3, {s11}",         // t3 = P3

                // Compute and apply E0 to Column 0
                "slli    t4, t3, 5",
                "srli    t5, t3, 27",
                "or      t4, t4, t5",
                "slli    t5, t3, 14",
                "srli    t6, t3, 18",
                "or      t5, t5, t6",
                "xor     t4, t4, t5",
                "xor     {s0}, {s0}, t4",
                "xor     {s4}, {s4}, t4",
                "xor     {s8}, {s8}, t4",

                // Compute and apply E1 to Column 1
                "slli    t4, t0, 5",
                "srli    t5, t0, 27",
                "or      t4, t4, t5",
                "slli    t5, t0, 14",
                "srli    t6, t0, 18",
                "or      t5, t5, t6",
                "xor     t4, t4, t5",
                "xor     {s1}, {s1}, t4",
                "xor     {s5}, {s5}, t4",
                "xor     {s9}, {s9}, t4",

                // Compute and apply E2 to Column 2
                "slli    t4, t1, 5",
                "srli    t5, t1, 27",
                "or      t4, t4, t5",
                "slli    t5, t1, 14",
                "srli    t6, t1, 18",
                "or      t5, t5, t6",
                "xor     t4, t4, t5",
                "xor     {s2}, {s2}, t4",
                "xor     {s6}, {s6}, t4",
                "xor     {s10}, {s10}, t4",

                // Compute and apply E3 to Column 3
                "slli    t4, t2, 5",
                "srli    t5, t2, 27",
                "or      t4, t4, t5",
                "slli    t5, t2, 14",
                "srli    t6, t2, 18",
                "or      t5, t5, t6",
                "xor     t4, t4, t5",
                "xor     {s3}, {s3}, t4",
                "xor     {s7}, {s7}, t4",
                "xor     {s11}, {s11}, t4",

                // === ρ (Rho) west step ===
                "mv      t4, {s7}",
                "mv      {s7}, {s6}",
                "mv      {s6}, {s5}",
                "mv      {s5}, {s4}",
                "mv      {s4}, t4",

                "slli    t4, {s8}, 11",
                "srli    {s8}, {s8}, 21",
                "or      {s8}, {s8}, t4",

                "slli    t4, {s9}, 11",
                "srli    {s9}, {s9}, 21",
                "or      {s9}, {s9}, t4",

                "slli    t4, {s10}, 11",
                "srli    {s10}, {s10}, 21",
                "or      {s10}, {s10}, t4",

                "slli    t4, {s11}, 11",
                "srli    {s11}, {s11}, 21",
                "or      {s11}, {s11}, t4",

                // === ι (Iota) step ===
                "lw      t4, 0({rk})",
                "addi    {rk}, {rk}, 4",
                "xor     {s0}, {s0}, t4",

                // === χ (Chi) step ===
                "not     t4, {s4}",
                "and     t4, t4, {s8}",
                "xor     {s0}, {s0}, t4",
                "not     t4, {s8}",
                "and     t4, t4, {s0}",
                "xor     {s4}, {s4}, t4",
                "not     t4, {s0}",
                "and     t4, t4, {s4}",
                "xor     {s8}, {s8}, t4",

                "not     t4, {s5}",
                "and     t4, t4, {s9}",
                "xor     {s1}, {s1}, t4",
                "not     t4, {s9}",
                "and     t4, t4, {s1}",
                "xor     {s5}, {s5}, t4",
                "not     t4, {s1}",
                "and     t4, t4, {s5}",
                "xor     {s9}, {s9}, t4",

                "not     t4, {s6}",
                "and     t4, t4, {s10}",
                "xor     {s2}, {s2}, t4",
                "not     t4, {s10}",
                "and     t4, t4, {s2}",
                "xor     {s6}, {s6}, t4",
                "not     t4, {s2}",
                "and     t4, t4, {s6}",
                "xor     {s10}, {s10}, t4",

                "not     t4, {s7}",
                "and     t4, t4, {s11}",
                "xor     {s3}, {s3}, t4",
                "not     t4, {s11}",
                "and     t4, t4, {s3}",
                "xor     {s7}, {s7}, t4",
                "not     t4, {s3}",
                "and     t4, t4, {s7}",
                "xor     {s11}, {s11}, t4",

                // === ρ (Rho) east step ===
                "slli    t4, {s4}, 1",
                "srli    {s4}, {s4}, 31",
                "or      {s4}, {s4}, t4",

                "slli    t4, {s5}, 1",
                "srli    {s5}, {s5}, 31",
                "or      {s5}, {s5}, t4",

                "slli    t4, {s6}, 1",
                "srli    {s6}, {s6}, 31",
                "or      {s6}, {s6}, t4",

                "slli    t4, {s7}, 1",
                "srli    {s7}, {s7}, 31",
                "or      {s7}, {s7}, t4",

                "slli    t4, {s8}, 8",
                "srli    t5, {s8}, 24",
                "or      t5, t5, t4",

                "slli    t4, {s9}, 8",
                "srli    t6, {s9}, 24",
                "or      t6, t6, t4",

                "slli    t4, {s10}, 8",
                "srli    {s8}, {s10}, 24",
                "or      {s8}, {s8}, t4",

                "slli    t4, {s11}, 8",
                "srli    {s9}, {s11}, 24",
                "or      {s9}, {s9}, t4",

                "mv      {s10}, t5",
                "mv      {s11}, t6",

                // === Loop check ===
                "bne     {rk}, {rk_end}, 0b",

                s0 = inout(reg) st_words[0],
                s1 = inout(reg) st_words[1],
                s2 = inout(reg) st_words[2],
                s3 = inout(reg) st_words[3],
                s4 = inout(reg) st_words[4],
                s5 = inout(reg) st_words[5],
                s6 = inout(reg) st_words[6],
                s7 = inout(reg) st_words[7],
                s8 = inout(reg) st_words[8],
                s9 = inout(reg) st_words[9],
                s10 = inout(reg) st_words[10],
                s11 = inout(reg) st_words[11],
                rk = inout(reg) rk => _,
                rk_end = in(reg) rkeys_end,
                out("t0") _, out("t1") _, out("t2") _, out("t3") _,
                out("t4") _, out("t5") _, out("t6") _,
                options(nostack),
            );
        }
    }
}
