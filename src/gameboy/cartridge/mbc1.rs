use super::cartridge::Cartridge;
use crate::gameboy::bus::bus::Bus;

use std::cmp;

const BANK_SIZE: usize = 16 * 1024;
const BANK_COUNT: usize = BANKS_MAX + 1;
const BANKS_MAX: usize = 0x1F;

pub struct Mbc1 {
    rom: [u8; BANK_COUNT * BANK_SIZE],
    banksel: u8,
}

impl Mbc1 {
    pub fn new(rom: &[u8]) -> Self {
        let mut cart = Self {
            rom: [0; BANK_COUNT * BANK_SIZE],
            banksel: 1,
        };
        cart.rom[0..rom.len()].copy_from_slice(rom);
        cart
    }

    fn translate(&self, addr: u16) -> usize {
        assert!(addr >= 0x4000);

        let bankaddr: usize = 0x4000 * (self.banksel as usize);
        bankaddr + (addr as usize - 0x4000)
    }
}

impl Cartridge for Mbc1 {}

impl Bus for Mbc1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // Always bank 0
            0x0000..=0x3FFF => self.rom[addr as usize],
            // Bank 1..31
            0x4000..=0x7FFF => self.rom[self.translate(addr)],

            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // RAM enable
            0x0000..=0x1FFF => (),
            // ROM bank select
            0x2000..=0x3FFF => self.banksel = cmp::max(val, 1) & BANKS_MAX as u8,
            // RAM/upper ROM bank select
            0x4000..=0x5FFF => todo!(),
            // Banking mode select
            0x6000..=0x7FFF => todo!(),
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::repeat_n;

    #[test]
    fn bank_switching() {
        let rom: Vec<u8> = (0u8..=(BANKS_MAX as u8))
            .flat_map(|i| repeat_n(i, BANK_SIZE))
            .collect();
        assert_eq!(rom.len(), BANK_COUNT * BANK_SIZE);

        let mut c = Mbc1::new(&rom);

        // Bank 0
        for i in 0u16..(BANK_SIZE as u16) {
            assert_eq!(c.read(i), 0);
        }
        // Bank n default (1)
        for i in 0u16..(BANK_SIZE as u16) {
            assert_eq!(c.read(0x4000 + i), 1);
        }

        // Test each bank n
        for b in 1u8..=(BANKS_MAX as u8) {
            c.write(0x2000, b);
            // Bank 0
            for i in 0..(BANK_SIZE as u16) {
                assert_eq!(c.read(i), 0);
            }
            // Bank n
            for i in 0u16..(BANK_SIZE as u16) {
                assert_eq!(c.read(0x4000 + i), b);
            }
        }

        // Test masking
        c.write(0x2000, 0x3F);
        for i in 0u16..(BANK_SIZE as u16) {
            assert_eq!(c.read(0x4000 + i), 0x1F);
        }

        // Selecting bank 0 should select bank 1
        c.write(0x2000, 0);
        for i in 0u16..(BANK_SIZE as u16) {
            assert_eq!(c.read(0x4000 + i), 0x01);
        }
    }
}
