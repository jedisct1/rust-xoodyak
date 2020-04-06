mod hash;
mod keyed;

use crate::error::*;
use crate::tag::*;
use crate::xoodoo::*;

pub use hash::*;
pub use keyed::*;

pub(crate) const HASH_ABSORB_RATE: usize = 16;
pub(crate) const HASH_SQUEEZE_RATE: usize = 16;
pub(crate) const KEYED_ABSORB_RATE: usize = 44;
pub(crate) const KEYED_SQUEEZE_RATE: usize = 24;
pub(crate) const RATCHET_RATE: usize = 16;
pub const AUTH_TAG_BYTES: usize = 16;

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

mod internal {
    use super::*;

    pub trait Xoodyak {
        fn state(&mut self) -> &mut Xoodoo;
        fn mode(&self) -> Mode;
        fn set_mode(&mut self, mode: Mode);
        fn phase(&self) -> Phase;
        fn set_phase(&mut self, phase: Phase);
        fn absorb_rate(&self) -> usize;
        fn set_absorb_rate(&mut self, rate: usize);
        fn squeeze_rate(&self) -> usize;
        fn set_squeeze_rate(&mut self, rate: usize);

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
        fn extract_bytes(&mut self, out: &mut [u8], offset: usize) {
            self.state().extract_bytes(out, offset);
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
                self.extract_bytes(&mut out, 0);
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
        fn absorb_any(&mut self, bin: &[u8], rate: usize, mut cd: u8) {
            for chunk in bin.chunks(rate) {
                if self.phase() != Phase::Up {
                    self.up(None, 0x00)
                }
                self.down(Some(chunk), cd);
                cd = 0
            }
        }

        fn absorb_key(
            &mut self,
            key: &[u8],
            key_id: Option<&[u8]>,
            counter: Option<&[u8]>,
        ) -> Result<(), Error> {
            if key.len() + key_id.unwrap_or_default().len() > KEYED_ABSORB_RATE {
                return Err(Error::KeyTooLong);
            }
            self.set_absorb_rate(KEYED_ABSORB_RATE);
            self.set_squeeze_rate(KEYED_SQUEEZE_RATE);
            self.set_mode(Mode::Keyed);
            let mut iv = [0u8; KEYED_ABSORB_RATE];
            let key_len = key.len();
            let mut key_id_len = 0;
            iv[..key_len].copy_from_slice(key);
            let mut iv_len = key_len;
            if let Some(key_id) = key_id {
                key_id_len = key_id.len();
                iv[iv_len..iv_len + key_id_len].copy_from_slice(key_id);
                iv_len += key_id_len;
            }
            iv[iv_len] = key_id_len as u8;
            iv_len += 1;
            self.absorb_any(&iv[..iv_len], KEYED_ABSORB_RATE, 0x02);
            if let Some(counter) = counter {
                self.absorb_any(counter, 1, 0x00)
            }
            Ok(())
        }

        #[inline]
        fn squeeze_any(&mut self, out: &mut [u8], cu: u8) {
            let mut chunks_it = out.chunks_mut(self.squeeze_rate());
            if let Some(chunk) = chunks_it.next() {
                self.up(Some(chunk), cu);
            }
            for chunk in chunks_it {
                self.down(None, 0x00);
                self.up(Some(chunk), 0x00);
            }
        }
    }
}

pub trait Xoodyak: internal::Xoodyak {
    #[inline(always)]
    fn absorb(&mut self, bin: &[u8]) {
        self.absorb_any(bin, self.absorb_rate(), 0x03);
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
