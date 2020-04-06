use super::internal::{Mode, Phase};
use super::*;

#[derive(Clone, Debug)]
pub struct XoodyakHash {
    state: Xoodoo,
    mode: Mode,
    phase: Phase,
    absorb_rate: usize,
    squeeze_rate: usize,
}

impl XoodyakHash {
    pub fn new() -> Self {
        XoodyakHash {
            state: Xoodoo::default(),
            phase: Phase::Up,
            mode: Mode::Hash,
            absorb_rate: HASH_ABSORB_RATE,
            squeeze_rate: HASH_SQUEEZE_RATE,
        }
    }
}

impl Default for XoodyakHash {
    #[inline]
    fn default() -> Self {
        XoodyakHash::new()
    }
}

impl internal::Xoodyak for XoodyakHash {
    #[inline(always)]
    fn state(&mut self) -> &mut Xoodoo {
        &mut self.state
    }

    #[inline(always)]
    fn mode(&self) -> Mode {
        self.mode
    }

    #[inline(always)]
    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode
    }

    #[inline(always)]
    fn phase(&self) -> Phase {
        self.phase
    }

    #[inline(always)]
    fn set_phase(&mut self, phase: Phase) {
        self.phase = phase
    }

    #[inline(always)]
    fn absorb_rate(&self) -> usize {
        self.absorb_rate
    }

    #[inline(always)]
    fn set_absorb_rate(&mut self, rate: usize) {
        self.absorb_rate = rate;
    }

    #[inline(always)]
    fn squeeze_rate(&self) -> usize {
        self.squeeze_rate
    }

    #[inline(always)]
    fn set_squeeze_rate(&mut self, rate: usize) {
        self.squeeze_rate = rate;
    }
}

impl Xoodyak for XoodyakHash {}
