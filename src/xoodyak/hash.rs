use super::internal::{Mode, Phase};
use super::*;

#[derive(Clone, Debug)]
pub struct XoodyakHash {
    state: Xoodoo,
    phase: Phase,
}

impl XoodyakHash {
    pub fn new() -> Self {
        XoodyakHash {
            state: Xoodoo::default(),
            phase: Phase::Up,
        }
    }
}

impl Default for XoodyakHash {
    #[inline]
    fn default() -> Self {
        XoodyakHash::new()
    }
}

impl internal::XoodyakCommon for XoodyakHash {
    #[inline(always)]
    fn state(&mut self) -> &mut Xoodoo {
        &mut self.state
    }

    #[inline(always)]
    fn mode(&self) -> Mode {
        Mode::Hash
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
        HASH_ABSORB_RATE
    }

    #[inline(always)]
    fn squeeze_rate(&self) -> usize {
        HASH_SQUEEZE_RATE
    }
}

impl XoodyakCommon for XoodyakHash {}
