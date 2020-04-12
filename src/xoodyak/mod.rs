#[allow(clippy::unit_arg)]
mod any;
mod hash;
mod keyed;
mod tag;

use crate::error::*;
use crate::xoodoo::*;

pub use any::*;
pub use hash::*;
pub use keyed::*;
pub use tag::*;

pub(crate) const HASH_ABSORB_RATE: usize = 16;
pub(crate) const HASH_SQUEEZE_RATE: usize = 16;
pub(crate) const KEYED_ABSORB_RATE: usize = 44;
pub(crate) const KEYED_SQUEEZE_RATE: usize = 24;
pub(crate) const RATCHET_RATE: usize = 16;

mod internal {
    use super::*;

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Mode {
        Hash,
        Keyed,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Phase {
        Up,
        Down,
    }

    pub trait XoodyakCommon {
        fn state(&mut self) -> &mut Xoodoo;
        fn mode(&self) -> Mode;
        fn phase(&self) -> Phase;
        fn set_phase(&mut self, phase: Phase);
        fn absorb_rate(&self) -> usize;
        fn squeeze_rate(&self) -> usize;

        #[inline(always)]
        fn permute(&mut self) {
            self.state().permute()
        }

        #[inline(always)]
        fn add_byte(&mut self, byte: u8, offset: usize) {
            self.state().add_byte(byte, offset);
        }

        #[inline(always)]
        fn add_bytes(&mut self, bytes: &[u8]) {
            self.state().add_bytes(bytes);
        }

        #[inline(always)]
        fn extract_bytes(&mut self, out: &mut [u8]) {
            self.state().extract_bytes(out);
        }

        #[inline(always)]
        fn up(&mut self, out: Option<&mut [u8]>, cu: u8) {
            debug_assert!(out.as_ref().map(|x| x.len()).unwrap_or(0) <= self.squeeze_rate());
            self.set_phase(Phase::Up);
            if self.mode() != Mode::Hash {
                self.add_byte(cu, 47);
            }
            self.permute();
            if let Some(mut out) = out {
                self.extract_bytes(&mut out);
            }
        }

        #[inline(always)]
        fn down(&mut self, bin: Option<&[u8]>, cd: u8) {
            debug_assert!(bin.as_ref().map(|x| x.len()).unwrap_or(0) <= self.absorb_rate());
            self.set_phase(Phase::Down);
            if let Some(bin) = bin {
                self.add_bytes(&bin);
                self.add_byte(0x01, bin.len());
            } else {
                self.add_byte(0x01, 0);
            }
            if self.mode() == Mode::Hash {
                self.add_byte(cd & 0x01, 47);
            } else {
                self.add_byte(cd, 47);
            }
        }

        #[inline]
        fn absorb_any(&mut self, bin: &[u8], rate: usize, cd: u8) {
            let mut chunks_it = bin.chunks(rate);
            if self.phase() != Phase::Up {
                self.up(None, 0x00)
            }
            self.down(chunks_it.next(), cd);
            for chunk in chunks_it {
                self.up(None, 0x00);
                self.down(Some(chunk), 0x00);
            }
        }

        #[inline]
        fn squeeze_any(&mut self, out: &mut [u8], cu: u8) {
            let mut chunks_it = out.chunks_mut(self.squeeze_rate());
            self.up(chunks_it.next(), cu);
            for chunk in chunks_it {
                self.down(None, 0x00);
                self.up(Some(chunk), 0x00);
            }
        }
    }
}

pub trait XoodyakCommon: internal::XoodyakCommon {
    #[inline(always)]
    fn absorb(&mut self, bin: &[u8]) {
        self.absorb_any(bin, self.absorb_rate(), 0x03);
    }

    #[inline]
    fn absorb_more(&mut self, bin: &[u8], rate: usize) {
        for chunk in bin.chunks(rate) {
            self.up(None, 0x00);
            self.down(Some(chunk), 0x00);
        }
    }

    #[inline(always)]
    fn squeeze(&mut self, out: &mut [u8]) {
        self.squeeze_any(out, 0x40);
    }

    #[inline(always)]
    fn squeeze_key(&mut self, out: &mut [u8]) {
        self.squeeze_any(out, 0x20);
    }

    #[inline]
    fn squeeze_more(&mut self, out: &mut [u8]) {
        for chunk in out.chunks_mut(self.squeeze_rate()) {
            self.down(None, 0x00);
            self.up(Some(chunk), 0x00);
        }
    }

    #[cfg(feature = "std")]
    fn squeeze_to_vec(&mut self, len: usize) -> Vec<u8> {
        let mut out = vec![0u8; len];
        self.squeeze(&mut out);
        out
    }
}
