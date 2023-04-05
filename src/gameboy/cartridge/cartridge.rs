use crate::gameboy::bus::bus::Bus;
use crate::tickable::Tickable;

use super::mbc1::Mbc1;
use super::romonly::RomOnly;

use anyhow::Result;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use std::fmt;

const TITLE_OFFSET: usize = 0x134;
const TITLE_SIZE: usize = 16;
const CARTTYPE_OFFSET: usize = 0x147;

#[derive(Debug, FromPrimitive)]
pub enum CartridgeType {
    Rom = 0x00,
    Mbc1 = 0x01,
    Mbc1Ram = 0x02,
    Mbc1RamBat = 0x03,
}

pub trait Cartridge: Bus + Tickable {
    fn get_title(&self) -> String {
        String::from_utf8(
            self.read_vec(TITLE_OFFSET as u16, TITLE_SIZE)
                .into_iter()
                .take_while(|&c| c != 0)
                .collect(),
        )
        .unwrap()
    }

    fn get_type(&self) -> CartridgeType {
        CartridgeType::from_u8(self.read(CARTTYPE_OFFSET as u16)).unwrap()
    }
}

impl fmt::Display for dyn Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Title: {} - Type: {:?}",
            self.get_title(),
            self.get_type()
        )
    }
}

impl<T> Tickable for T
where
    T: Cartridge,
{
    fn tick(&mut self, cycles: usize) -> Result<usize> {
        Ok(cycles)
    }
}

pub fn load(rom: &[u8]) -> Box<dyn Cartridge> {
    assert!(rom.len() >= 32 * 1024);

    match CartridgeType::from_u8(rom[CARTTYPE_OFFSET]) {
        Some(CartridgeType::Rom) => Box::new(RomOnly::new(rom)),
        Some(CartridgeType::Mbc1) => Box::new(Mbc1::new(rom)),
        Some(CartridgeType::Mbc1Ram) => Box::new(Mbc1::new(rom)),
        Some(CartridgeType::Mbc1RamBat) => Box::new(Mbc1::new(rom)),
        _ => panic!("Unknown cartridge type {:02X}", rom[CARTTYPE_OFFSET]),
    }
}
