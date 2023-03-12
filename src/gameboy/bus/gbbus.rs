use super::super::iomux::IOMux;
use super::bus::Bus;

const CART_ROM_BANK_SIZE: usize = 16 * 1024;

type CartROMBank = [u8; CART_ROM_BANK_SIZE];

/// Multiplexer for the Gameboy address bus
pub struct Gameboybus {
    boot_rom: [u8; 256],

    cart_rom: [CartROMBank; 2],

    boot_rom_enabled: bool,

    ext_ram: [u8; u16::MAX as usize + 1],
    wram: [u8; u16::MAX as usize + 1],
    vram: [u8; u16::MAX as usize + 1],
    hram: [u8; u16::MAX as usize + 1],

    io: IOMux,
}

impl Gameboybus {
    pub fn new(cart: &[u8], bootrom: Option<&[u8]>) -> Self {
        let mut bus = Gameboybus {
            cart_rom: [[0; CART_ROM_BANK_SIZE]; 2],
            boot_rom: [0; 256],
            boot_rom_enabled: false,

            ext_ram: [0; u16::MAX as usize + 1],
            wram: [0; u16::MAX as usize + 1],
            hram: [0; u16::MAX as usize + 1],
            vram: [0; u16::MAX as usize + 1],

            io: IOMux {},
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
            // Boot ROM (or cartridge after disable)
            0x0000..=0x00FF => {
                if self.boot_rom_enabled {
                    self.boot_rom[addr]
                } else {
                    self.cart_rom[0][addr]
                }
            }

            // Cartridge bank 0
            0x0000..=0x3FFF => self.cart_rom[0][addr],

            // Cartridge bank 1
            // TODO bank switching
            0x4000..=0x7FFF => self.cart_rom[1][addr],

            // Video RAM
            0x8000..=0x9FFF => self.vram[addr],

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
            0xFEA0..=0xFEFF => panic!("Read from unusable segment"),

            // I/O registers
            0xFF00..=0xFF7F => self.io.read(addr as u16),

            // High RAM
            0xFF80..=0xFFFE => self.hram[addr],

            // Interrupt Enable (IE) register
            0xFFFF => todo!(),

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
            0x8000..=0x9FFF => self.vram[addr] = val,

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
            0xFE00..=0xFE9F => todo!(),

            // Unusable segment
            0xFEA0..=0xFEFF => panic!("Write to unusable segment"),

            // I/O registers
            0xFF00..=0xFF7F => self.io.write(addr as u16, val),

            // High RAM
            0xFF80..=0xFFFE => self.hram[addr] = val,

            // Interrupt Enable (IE) register
            0xFFFF => todo!(),

            _ => unreachable!(),
        }
    }
}
