use super::super::lcd::LCDController;
use super::bus::Bus;
use crate::tickable::Tickable;

use anyhow::Result;

const CART_ROM_BANK_SIZE: usize = 16 * 1024;

type CartROMBank = [u8; CART_ROM_BANK_SIZE];

/// Multiplexer for the Gameboy address bus
pub struct Gameboybus {
    boot_rom: [u8; 256],

    cart_rom: [CartROMBank; 2],

    boot_rom_enabled: bool,

    ext_ram: [u8; u16::MAX as usize + 1],
    wram: [u8; u16::MAX as usize + 1],
    hram: [u8; u16::MAX as usize + 1],
    ie: u8,

    lcd: LCDController,
}

impl Gameboybus {
    pub fn new(cart: &[u8], bootrom: Option<&[u8]>, lcd: LCDController) -> Self {
        let mut bus = Gameboybus {
            cart_rom: [[0; CART_ROM_BANK_SIZE]; 2],
            boot_rom: [0; 256],
            boot_rom_enabled: false,

            ext_ram: [0; u16::MAX as usize + 1],
            wram: [0; u16::MAX as usize + 1],
            hram: [0; u16::MAX as usize + 1],
            ie: 0,

            lcd,
        };

        if let Some(br) = bootrom {
            bus.boot_rom.copy_from_slice(br);
            bus.boot_rom_enabled = true;
        }

        // TODO support cartridges with different
        // amount of banks etc.
        assert_eq!(cart.len(), 32 * 1024);
        bus.cart_rom[0].copy_from_slice(&cart[0..(16 * 1024)]);
        bus.cart_rom[1].copy_from_slice(&cart[(16 * 1024)..]);

        bus
    }
}

impl Bus for Gameboybus {
    fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;

        match addr {
            // Boot ROM
            0x0000..=0x00FF if self.boot_rom_enabled => self.boot_rom[addr],

            // Cartridge bank 0
            0x0000..=0x3FFF => self.cart_rom[0][addr],

            // Cartridge bank 1
            // TODO bank switching
            0x4000..=0x7FFF => self.cart_rom[1][addr - 0x4000],

            // Video RAM
            0x8000..=0x9FFF => self.lcd.read_vram(addr - 0x8000),

            // External RAM
            // TODO bank switching
            0xA000..=0xBFFF => self.ext_ram[addr],

            // Working RAM (fixed portion)
            0xC000..=0xCFFF => self.wram[addr],

            // Working RAM (switchable on CGB)
            // TODO bank switching
            0xD000..=0xDFFF => self.wram[addr],

            // Echo RAM
            0xE000..=0xFDFF => panic!("Read from Echo RAM"),

            // Sprite Attribute Table (OAM)
            0xFE00..=0xFE9F => todo!(),

            // Unusable segment
            0xFEA0..=0xFEFF => 0,

            // Boot ROM disable
            0xFF50 if self.boot_rom_enabled => 0,
            0xFF50 => 1,

            // LCD I/O
            0xFF40..=0xFF4B | 0xFF51..=0xFF55 | 0xFF68..=0xFF69 => self.lcd.read_io(addr as u16),
            // Other I/O registers
            0xFF00..=0xFF7F => {
                println!("Read from unknown I/O address {:04X}", addr);
                0
            }

            // High RAM
            0xFF80..=0xFFFE => self.hram[addr],

            // Interrupt Enable (IE) register
            0xFFFF => self.ie,

            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;

        match addr {
            // Cartridge bank 0
            0x0000..=0x3FFF |
            // Cartridge bank 1
            0x4000..=0x7FFF => println!("Write to read-only address {:04X}", addr),

            // Video RAM
            0x8000..=0x9FFF => self.lcd.write_vram(addr - 0x8000, val),

            // External RAM
            // TODO bank switching
            0xA000..=0xBFFF => self.ext_ram[addr] = val,

            // Working RAM (fixed portion)
            0xC000..=0xCFFF => self.wram[addr] = val,

            // Working RAM (switchable on CGB)
            // TODO bank switching
            0xD000..=0xDFFF => self.wram[addr] = val,

            // Echo RAM
            0xE000..=0xFDFF => panic!("Write to Echo RAM"),

            // Sprite Attribute Table (OAM)
            0xFE00..=0xFE9F => self.lcd.write_oam(addr - 0xFE00, val),

            // Unusable segment
            0xFEA0..=0xFEFF => (),

            // Boot ROM disable
            0xFF50 => if val > 0 && self.boot_rom_enabled {
                println!("Boot ROM disabled!");
                self.boot_rom_enabled = false;
            },

            // LCD I/O
            0xFF40..=0xFF4B |
                0xFF51..=0xFF55 |
                0xFF68..=0xFF69 => self.lcd.write_io(addr as u16, val),
            // Other I/O registers
            0xFF00..=0xFF7F => println!("Write to unknown I/O address {:04X}", addr),

            // High RAM
            0xFF80..=0xFFFE => self.hram[addr] = val,

            // Interrupt Enable (IE) register
            0xFFFF => {
                println!("IE = {:02X}", val);
                self.ie = val
            },

            _ => unreachable!(),
        }
    }
}

impl Tickable for Gameboybus {
    fn tick(&mut self) -> Result<()> {
        self.lcd.tick()?;

        Ok(())
    }
}

// TODO fix broken tests
#[cfg(ignore)]
mod tests {
    use super::*;

    #[test]
    fn bootrom() {
        let cart = [0xAA_u8; 32 * 1024];
        let bootrom = [0xBB_u8; 256];

        let b = Gameboybus::new(&cart, Some(&bootrom));
        for i in 0..=0xFF {
            assert_eq!(b.read(i), 0xBB);
        }
        assert_eq!(b.read(0x0100), 0xAA);

        let b = Gameboybus::new(&cart, None);
        for i in 0..=0xFF {
            assert_eq!(b.read(i), 0xAA);
        }
        assert_eq!(b.read(0x0100), 0xAA);
    }

    #[test]
    fn bootrom_disable() {
        let cart = [0xAA_u8; 32 * 1024];
        let bootrom = [0xBB_u8; 256];

        let mut b = Gameboybus::new(&cart, Some(&bootrom));
        for i in 0..=0xFF {
            assert_eq!(b.read(i), 0xBB);
        }
        assert_eq!(b.read(0x0100), 0xAA);

        // Writing 0 should do nothing
        b.write(0xFF50, 0); // Boot ROM disable
        for i in 0..=0xFF {
            assert_eq!(b.read(i), 0xBB);
        }
        assert_eq!(b.read(0x0100), 0xAA);

        // Disable boot ROM
        b.write(0xFF50, 1); // Boot ROM disable
        for i in 0..=0xFF {
            assert_eq!(b.read(i), 0xAA);
        }
        assert_eq!(b.read(0x0100), 0xAA);

        // Must not be able to reset boot ROM
        b.write(0xFF50, 0); // Boot ROM disable
        for i in 0..=0xFF {
            assert_eq!(b.read(i), 0xAA);
        }
        assert_eq!(b.read(0x0100), 0xAA);
    }
}
