use super::super::cartridge::cartridge::Cartridge;
use super::super::cpu::cpu;
use super::super::joypad::Joypad;
use super::super::lcd::LCDController;
use super::super::timer::Timer;
use super::bus::{Bus, BusMember};
use crate::input::input::Input;
use crate::tickable::Tickable;

use anyhow::Result;

use std::cmp;
use std::fmt;
use std::io;
use std::io::Write;
use std::sync::mpsc;

const IF_MASK: u8 = 0x1F;

/// Multiplexer for the Gameboy address bus
pub struct Gameboybus {
    cgb: bool,

    cart: Box<dyn Cartridge>,
    boot_rom: [u8; 256],

    boot_rom_enabled: bool,

    wram: [u8; Self::WRAM_SIZE * Self::WRAM_BANKS],
    hram: [u8; u16::MAX as usize + 1],
    ie: u8,

    lcd: LCDController,
    timer: Timer,
    joypad: Joypad,

    /// IF register
    intflags: u8,

    /// Serial data buffer
    serialbuffer: u8,

    /// Output serial data to terminal
    serial_output: bool,

    /// Serial output channel (for tests)
    serial_channel: Option<mpsc::Sender<u8>>,

    /// WRAM bank select (CGB only)
    wram_banksel: u8,
}

impl Gameboybus {
    const WRAM_SIZE: usize = 0x1000;
    const WRAM_BANKS: usize = 8;

    pub fn new(
        cart: Box<dyn Cartridge>,
        bootrom: Option<&[u8]>,
        lcd: LCDController,
        input: Box<dyn Input>,
        cgb: bool,
    ) -> Self {
        let mut bus = Gameboybus {
            cgb,
            cart,
            boot_rom: [0; 256],
            boot_rom_enabled: false,

            wram: [0; Self::WRAM_SIZE * Self::WRAM_BANKS],
            wram_banksel: 1,
            hram: [0; u16::MAX as usize + 1],
            ie: 0,

            lcd,
            timer: Timer::from_div(0xAC), // Value after boot ROM
            joypad: Joypad::new(input),

            intflags: cpu::INT_VBLANK, // VBlank is set after boot ROM
            serialbuffer: 0,
            serial_output: false,
            serial_channel: None,
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

    pub fn enable_serial_channel(&mut self, tx: mpsc::Sender<u8>) {
        self.serial_channel = Some(tx);
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

impl Bus for Gameboybus {}

impl BusMember for Gameboybus {
    fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;

        match addr {
            // Boot ROM
            0x0000..=0x00FF if self.boot_rom_enabled => self.boot_rom[addr],

            // Cartridge ROM
            0x0000..=0x7FFF => self.cart.read(addr as u16),

            // Video RAM
            0x8000..=0x9FFF => self.lcd.read_vram(addr - 0x8000),

            // External (cartridge) RAM
            0xA000..=0xBFFF => self.cart.read(addr as u16),

            // Working RAM (bank 0)
            0xC000..=0xCFFF => self.wram[addr - 0xC000],

            // Working RAM (bank 1 (DMG) / bank 1-7 (CGB))
            0xD000..=0xDFFF => {
                self.wram[addr - 0xD000 + self.wram_banksel as usize * Self::WRAM_SIZE]
            }

            // Echo RAM
            0xE000..=0xEFFF => self.wram[addr - 0xE000],
            0xF000..=0xFDFF => {
                self.wram[addr - 0xF000 + self.wram_banksel as usize * Self::WRAM_SIZE]
            }

            // Sprite Attribute Table (OAM)
            0xFE00..=0xFE9F => self.lcd.read_oam(addr - 0xFE00),

            // Unusable segment
            0xFEA0..=0xFEFF => 0,

            // I/O - Joypad
            0xFF00 => self.joypad.read(),

            // I/O - Serial transfer data buffer
            0xFF01 => self.serialbuffer,

            // I/O - Serial transfer control
            0xFF02 => 0x7E,

            // I/O - Timer
            0xFF04..=0xFF07 => self.timer.read(addr as u16),

            // IF - interrupt flags
            0xFF0F => self.intflags | !IF_MASK,

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

            // CGB - SVBK / WRAM bank select
            0xFF70 if self.cgb => self.wram_banksel & 0x07,

            // Other I/O registers
            0xFF00..=0xFF7F => 0xFF,

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

            // External (cartridge) RAM
            0xA000..=0xBFFF => self.cart.write(addr as u16, val),

            // Working RAM (bank 0)
            0xC000..=0xCFFF => self.wram[addr - 0xC000] = val,

            // Working RAM (bank 1 (DMG) / bank 1-7 (CGB))
            0xD000..=0xDFFF => {
                self.wram[addr - 0xD000 + self.wram_banksel as usize * Self::WRAM_SIZE] = val
            }

            // Echo RAM
            0xE000..=0xEFFF => self.wram[addr - 0xE000] = val,
            0xF000..=0xFDFF => {
                self.wram[addr - 0xF000 + self.wram_banksel as usize * Self::WRAM_SIZE] = val
            }

            // Sprite Attribute Table (OAM)
            0xFE00..=0xFE9F => self.lcd.write_oam(addr - 0xFE00, val),

            // Unusable segment
            0xFEA0..=0xFEFF => (),

            // I/O - Joypad
            0xFF00 => self.joypad.write(val),

            // I/O - Serial transfer data buffer
            0xFF01 => self.serialbuffer = val,

            // I/O - Serial transfer control
            0xFF02 => {
                if val == 0x81 && self.serial_output {
                    print!("{}", self.serialbuffer as char);
                    io::stdout().flush().unwrap_or_default();
                }
                if let Some(ref mut tx) = &mut self.serial_channel {
                    tx.send(self.serialbuffer).unwrap_or_default();
                }
            }

            // Timer
            0xFF04..=0xFF07 => self.timer.write(addr as u16, val),

            // IF - Interrupt Flags
            0xFF0F => self.intflags = val & IF_MASK,

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

            // CGB - SVBK / WRAM bank select
            0xFF70 if self.cgb => self.wram_banksel = cmp::max(1, val) & 0x07,

            // Other I/O registers
            0xFF00..=0xFF7F => (),

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

impl fmt::Display for Gameboybus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.cart.dump_state())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::display::NullDisplay;
    use crate::gameboy::cartridge::romonly::RomOnly;
    use crate::gameboy::lcd::LCDController;
    use crate::input::input::NullInput;

    fn gbbus() -> Gameboybus {
        let cart = Box::new(RomOnly::new(&[0xAA_u8; 32 * 1024]));
        let lcd = LCDController::new(Box::new(NullDisplay::new()), false);
        let input = Box::new(NullInput::new());
        Gameboybus::new(cart, None, lcd, input, false)
    }

    fn gbbus_cgb() -> Gameboybus {
        let cart = Box::new(RomOnly::new(&[0xAA_u8; 32 * 1024]));
        let lcd = LCDController::new(Box::new(NullDisplay::new()), false);
        let input = Box::new(NullInput::new());
        Gameboybus::new(cart, None, lcd, input, true)
    }

    fn gbbus_bootrom() -> Gameboybus {
        let cart = Box::new(RomOnly::new(&[0xAA_u8; 32 * 1024]));
        let lcd = LCDController::new(Box::new(NullDisplay::new()), false);
        let bootrom = [0xBB_u8; 256];
        let input = Box::new(NullInput::new());
        Gameboybus::new(cart, Some(&bootrom), lcd, input, false)
    }

    #[test]
    fn bootrom() {
        let b = gbbus_bootrom();
        for i in 0..=0xFF {
            assert_eq!(b.read(i), 0xBB);
        }
        assert_eq!(b.read(0x0100), 0xAA);

        let b = gbbus();
        for i in 0..=0xFF {
            assert_eq!(b.read(i), 0xAA);
        }
        assert_eq!(b.read(0x0100), 0xAA);
    }

    #[test]
    fn bootrom_disable() {
        let mut b = gbbus_bootrom();
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

    #[test]
    fn wram() {
        for b in 0xC000..=0xDFFF {
            let mut bus = gbbus();
            bus.write(b, 0xAB);
            for n in 0xC000..=0xDFFF {
                if n == b {
                    assert_eq!(bus.read(n), 0xAB);
                } else {
                    assert_eq!(bus.read(n), 0x00);
                }
            }
        }
    }

    #[test]
    fn cgb_wram_bank0() {
        for b in 0xC000..=0xCFFF {
            let mut bus = gbbus_cgb();
            bus.write(b, 0xAB);
            for bank in 1..8 {
                bus.write(0xFF70, bank);
                for n in 0xC000..=0xDFFF {
                    if n == b {
                        assert_eq!(bus.read(n), 0xAB);
                    } else {
                        assert_eq!(bus.read(n), 0x00);
                    }
                }
            }
        }
    }

    #[test]
    fn dmg_wram_no_banks() {
        for b in 0xD000..=0xDFFF {
            let mut bus = gbbus();
            bus.write(b, 0xAB);
            for bank in 1..8 {
                bus.write(0xFF70, bank);
                for n in 0xC000..=0xDFFF {
                    if n == b {
                        assert_eq!(bus.read(n), 0xAB);
                    } else {
                        assert_eq!(bus.read(n), 0x00);
                    }
                }
            }
        }
    }

    #[test]
    fn cgb_wram_banks() {
        for test_bank in 1..8 {
            for b in 0xD000..=0xDFFF {
                let mut bus = gbbus_cgb();
                bus.write(0xFF70, test_bank);
                bus.write(b, 0xAB);
                for bank in 1..8 {
                    bus.write(0xFF70, bank);
                    for n in 0xC000..=0xDFFF {
                        if n == b && bank == test_bank {
                            assert_eq!(bus.read(n), 0xAB);
                        } else {
                            assert_eq!(bus.read(n), 0x00);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn dmg_echo_ram_read() {
        let mut b = gbbus();
        b.write(0xC000, 0xAB);
        b.write(0xD000, 0xCD);

        assert_eq!(b.read(0xE000), 0xAB);
        assert_eq!(b.read(0xF000), 0xCD);

        // Bank select has no effect on DMG
        b.write(0xFF70, 4);
        assert_eq!(b.read(0xF000), 0xCD);
    }

    #[test]
    fn dmg_echo_ram_write() {
        let mut b = gbbus();
        b.write(0xE000, 0xAB);
        b.write(0xF000, 0xCD);

        assert_eq!(b.read(0xC000), 0xAB);
        assert_eq!(b.read(0xD000), 0xCD);

        // Bank select has no effect on DMG
        b.write(0xFF70, 4);
        b.write(0xF001, 0xEF);
        b.write(0xFF70, 1);
        assert_eq!(b.read(0xD001), 0xEF);
    }

    #[test]
    fn cgb_echo_ram_read() {
        let mut b = gbbus_cgb();
        b.write(0xC000, 0xAB);
        b.write(0xD000, 0xCD);

        assert_eq!(b.read(0xE000), 0xAB);
        assert_eq!(b.read(0xF000), 0xCD);

        // Bank select
        b.write(0xFF70, 4);
        assert_ne!(b.read(0xF000), 0xCD);
    }

    #[test]
    fn cgb_echo_ram_write() {
        let mut b = gbbus_cgb();
        b.write(0xE000, 0xAB);
        b.write(0xF000, 0xCD);

        assert_eq!(b.read(0xC000), 0xAB);
        assert_eq!(b.read(0xD000), 0xCD);

        // Bank select
        b.write(0xFF70, 4);
        b.write(0xF001, 0xEF);
        assert_eq!(b.read(0xD001), 0xEF);
        b.write(0xFF70, 1);
        assert_ne!(b.read(0xD001), 0xEF);
    }
}
