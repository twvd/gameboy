use anyhow::Result;

use super::bus::{Bus, BusMember};
use crate::tickable::Tickable;

use std::fmt;

pub struct Testbus {
    mem: [u8; u16::MAX as usize + 1],
}

impl Testbus {
    pub fn new() -> Self {
        Testbus {
            mem: [0; u16::MAX as usize + 1],
        }
    }

    pub fn from(data: &[u8]) -> Self {
        let mut ret = Testbus::new();
        ret.write_slice(data, 0);
        ret
    }
}

impl Bus for Testbus {}

impl BusMember for Testbus {
    fn read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.mem[addr as usize] = val;
    }
}

impl Tickable for Testbus {
    fn tick(&mut self, ticks: usize) -> Result<usize> {
        Ok(ticks)
    }
}

impl fmt::Display for Testbus {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Result::Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testbus() {
        let mut b = Testbus::new();

        for a in 0..=u16::MAX {
            assert_eq!(b.read(a), 0);
        }
        for a in 0..=u16::MAX {
            b.write(a, a as u8);
        }
        for a in 0..=u16::MAX {
            assert_eq!(b.read(a), a as u8);
        }
    }
}
