use super::cartridge::Cartridge;
use crate::gameboy::bus::bus::BusMember;

pub struct RomOnly {
    rom: [u8; 32 * 1024],
}

impl RomOnly {
    pub fn new(rom: &[u8]) -> Self {
        let mut cart = Self {
            rom: [0; 32 * 1024],
        };
        cart.rom.copy_from_slice(rom);
        cart
    }
}

impl Cartridge for RomOnly {
    fn dump_state(&self) -> String {
        "".to_string()
    }
}

impl BusMember for RomOnly {
    fn read(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }

    fn write(&mut self, _addr: u16, _val: u8) {}
}
