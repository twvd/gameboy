use super::super::cartridge::cartridge::Cartridge;
use super::super::cpu::cpu;
use super::super::lcd::LCDController;
use super::super::timer::Timer;
use super::bus::Bus;
use crate::tickable::Tickable;

use anyhow::Result;

use std::io;
use std::io::Write;

/// Multiplexer for the Gameboy address bus
pub struct Gameboybus {
    cart: Box<dyn Cartridge>,
    boot_rom: [u8; 256],

    boot_rom_enabled: bool,

    ext_ram: [u8; u16::MAX as usize + 1],
    wram: [u8; u16::MAX as usize + 1],
    hram: [u8; u16::MAX as usize + 1],
    ie: u8,

    lcd: LCDController,
    timer: Timer,

    /// IF register
    intflags: u8,

    /// Serial data buffer
    serialbuffer: u8,

    /// Output serial data to terminal
    serial_output: bool,
}

impl Gameboybus {
    pub fn new(cart: Box<dyn Cartridge>, bootrom: Option<&[u8]>, lcd: LCDController) -> Self {
        let mut bus = Gameboybus {
            cart,
            boot_rom: [0; 256],
            boot_rom_enabled: false,

            ext_ram: [0; u16::MAX as usize + 1],
            wram: [0; u16::MAX as usize + 1],
            hram: [0; u16::MAX as usize + 1],
            ie: 0,

            lcd,
            timer: Timer::new(),

            intflags: 0,
            serialbuffer: 0,
            serial_output: false,
        };

        if let Some(br) = bootrom {
            bus.boot_rom.copy_from_slice(br);
            bus.boot_rom_enabled = true;
        }

        bus
    }

    pub fn enable_serial_output(&mut self) {
        self.serial_output = true;
    }

    fn update_intflags(&mut self) {
        if self.lcd.get_clr_intreq_vblank() {
            self.intflags |= cpu::INT_VBLANK;
        }
        if self.lcd.get_clr_intreq_stat() {
            self.intflags |= cpu::INT_LCDSTAT;
        }
        if self.timer.get_clr_intreq() {
            self.intflags |= cpu::INT_TIMER;
        }
    }
}

impl Bus for Gameboybus {
    fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;

        match addr {
            // Boot ROM
            0x0000..=0x00FF if self.boot_rom_enabled => self.boot_rom[addr],

            // Cartridge ROM
            0x0000..=0x7FFF => self.cart.read(addr as u16),

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

            // I/O - Joypad
            0xFF00 => 0xFF,

            // I/O - Serial transfer data buffer
            0xFF01 => self.serialbuffer,

            // I/O - Serial transfer control
            0xFF02 => 0,

            // I/O - Timer
            0xFF04..=0xFF07 => self.timer.read(addr as u16),

            // IF - interrupt flags
            0xFF0F => self.intflags,

            // I/O - Audio control + wave pattern (ignore)
            0xFF10..=0xFF3F => 0,

            // I/O - Boot ROM disable
            0xFF50 if self.boot_rom_enabled => 0,
            0xFF50 => 1,

            // I/O - LCD OAM DMA start
            // Handled here because we need to access source memory
            0xFF46 => 0,

            // I/O - LCD I/O
            0xFF40..=0xFF4B | 0xFF51..=0xFF55 | 0xFF68..=0xFF69 => self.lcd.read_io(addr as u16),

            // Other I/O registers
            0xFF00..=0xFF7F => {
                //println!("Read from unknown I/O address {:04X}", addr);
                0xFF
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
            // Cartridge ROM
            0x0000..=0x7FFF => self.cart.write(addr as u16, val),

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

            // I/O - Serial transfer data buffer
            0xFF01 => self.serialbuffer = val,

            // I/O - Serial transfer control
            0xFF02 => {
                if val == 0x81 && self.serial_output {
                    print!("{}", self.serialbuffer as char);
                    io::stdout().flush().unwrap_or_default();
                }
            }

            // Timer
            0xFF04..=0xFF07 => self.timer.write(addr as u16, val),

            // IF - Interrupt Flags
            0xFF0F => self.intflags = val,

            // I/O - Audio control + wave pattern (ignore)
            0xFF10..=0xFF3F => (),

            // I/O - Boot ROM disable
            0xFF50 => {
                if val > 0 && self.boot_rom_enabled {
                    self.boot_rom_enabled = false;
                }
            }

            // I/O - LCD OAM DMA start
            // Handled here because we need to access source memory
            0xFF46 => {
                for i in 0u16..=0x9F {
                    self.write(0xFE00 | i, self.read(((val as u16) << 8) | i));
                }
            }

            // I/O - LCD I/O
            0xFF40..=0xFF4B | 0xFF51..=0xFF55 | 0xFF68..=0xFF69 => {
                self.lcd.write_io(addr as u16, val)
            }
            // Other I/O registers
            0xFF00..=0xFF7F => (), //println!("Write to unknown I/O address {:04X}", addr),

            // High RAM
            0xFF80..=0xFFFE => self.hram[addr] = val,

            // Interrupt Enable (IE) register
            0xFFFF => self.ie = val,

            _ => unreachable!(),
        }
    }
}

impl Tickable for Gameboybus {
    fn tick(&mut self, ticks: usize) -> Result<usize> {
        self.lcd.tick(ticks)?;
        self.timer.tick(ticks)?;
        self.update_intflags();

        Ok(ticks)
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
