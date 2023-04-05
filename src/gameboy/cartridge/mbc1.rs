use super::cartridge::Cartridge;
use crate::gameboy::bus::bus::Bus;

use std::cmp;

const ROM_BANK_SIZE: usize = 16 * 1024;
const ROM_BANK_COUNT: usize = ROM_BANKS_MAX + 1;
const ROM_BANKS_MAX: usize = 0x1F;

const RAM_BANK_SIZE: usize = 8 * 1024;
const RAM_BANK_COUNT: usize = RAM_BANKS_MAX + 1;
const RAM_BANKS_MAX: usize = 0x03;

pub struct Mbc1 {
    rom: [u8; ROM_BANK_COUNT * ROM_BANK_SIZE],
    rom_banksel: u8,
    ram: [u8; RAM_BANK_COUNT * RAM_BANK_SIZE],
    ram_banksel: u8,
}

impl Mbc1 {
    pub fn new(rom: &[u8]) -> Self {
        let mut cart = Self {
            rom: [0; ROM_BANK_COUNT * ROM_BANK_SIZE],
            ram: [0; RAM_BANK_COUNT * RAM_BANK_SIZE],
            rom_banksel: 1,
            ram_banksel: 0,
        };
        cart.rom[0..rom.len()].copy_from_slice(rom);
        cart
    }

    fn rom_translate(&self, addr: u16) -> usize {
        assert!(addr >= 0x4000);

        let bankaddr: usize = ROM_BANK_SIZE * (self.rom_banksel as usize);
        bankaddr + (addr as usize - 0x4000)
    }

    fn ram_translate(&self, addr: u16) -> usize {
        assert!(addr >= 0xA000);

        let bankaddr: usize = RAM_BANK_SIZE * (self.ram_banksel as usize);
        bankaddr + (addr as usize - 0xA000)
    }
}

impl Cartridge for Mbc1 {}

impl Bus for Mbc1 {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // ROM - Always bank 0
            0x0000..=0x3FFF => self.rom[addr as usize],
            // ROM - Bank 1..=31
            0x4000..=0x7FFF => self.rom[self.rom_translate(addr)],
            // RAM - Bank 0..=3
            0xA000..=0xBFFF => self.ram[self.ram_translate(addr)],

            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // RAM enable
            0x0000..=0x1FFF => (),
            // ROM bank select
            0x2000..=0x3FFF => self.rom_banksel = cmp::max(val, 1) & ROM_BANKS_MAX as u8,
            // RAM/upper ROM bank select
            0x4000..=0x5FFF => self.ram_banksel = val & RAM_BANKS_MAX as u8,
            // Banking mode select
            0x6000..=0x7FFF => todo!(),
            // RAM - Bank 0..=3
            0xA000..=0xBFFF => self.ram[self.ram_translate(addr)] = val,

            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use itertools::repeat_n;

    #[test]
    fn rom_bank_switching() {
        let rom: Vec<u8> = (0u8..=(ROM_BANKS_MAX as u8))
            .flat_map(|i| repeat_n(i, ROM_BANK_SIZE))
            .collect();
        assert_eq!(rom.len(), ROM_BANK_COUNT * ROM_BANK_SIZE);

        let mut c = Mbc1::new(&rom);

        // Bank 0
        for i in 0u16..(ROM_BANK_SIZE as u16) {
            assert_eq!(c.read(i), 0);
        }
        // Bank n default (1)
        for i in 0u16..(ROM_BANK_SIZE as u16) {
            assert_eq!(c.read(0x4000 + i), 1);
        }

        // Test each bank n
        for b in 1u8..=(ROM_BANKS_MAX as u8) {
            c.write(0x2000, b);
            // Bank 0
            for i in 0..(ROM_BANK_SIZE as u16) {
                assert_eq!(c.read(i), 0);
            }
            // Bank n
            for i in 0u16..(ROM_BANK_SIZE as u16) {
                assert_eq!(c.read(0x4000 + i), b);
            }
        }

        // Test masking
        c.write(0x2000, 0x3F);
        for i in 0u16..(ROM_BANK_SIZE as u16) {
            assert_eq!(c.read(0x4000 + i), 0x1F);
        }

        // Selecting bank 0 should select bank 1
        c.write(0x2000, 0);
        for i in 0u16..(ROM_BANK_SIZE as u16) {
            assert_eq!(c.read(0x4000 + i), 0x01);
        }
    }

    #[test]
    fn ram_bank_switching() {
        let mut c = Mbc1::new(&[]);

        for b in 0u8..(RAM_BANK_COUNT as u8) {
            c.write(0x4000, b);
            for n in 0u16..(RAM_BANK_SIZE as u16) {
                assert_eq!(c.read(0xA000 + n), 0);
                c.write(0xA000 + n, b + 1);
            }
        }

        for b in 0u8..(RAM_BANK_COUNT as u8) {
            c.write(0x4000, b);
            for n in 0u16..(RAM_BANK_SIZE as u16) {
                assert_eq!(c.read(0xA000 + n), b + 1);
            }
        }

        // Masking
        c.write(0x4000, RAM_BANK_COUNT as u8);
        assert_eq!(c.read(0xA000 as u16), 1);
    }
}
