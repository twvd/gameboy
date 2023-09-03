use super::super::apu::APU;
use super::super::cartridge::cartridge::Cartridge;
use super::super::cpu::cpu;
use super::super::joypad::Joypad;
use super::super::lcd::{LCDController, LCDStatMode};
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

#[allow(dead_code)]
const BOOTROM_SIZE_DMG: usize = 0x100;
const BOOTROM_SIZE_CGB: usize = 0x900;

const IF_MASK: u8 = 0x1F;

const VRAMDMA_HBLANK_MODE: u8 = 1 << 7;
const VRAMDMA_LEN_MASK: u8 = 0x7F;
const VRAMDMA_IDLE: u8 = 0xFF;
const VRAMDMA_BLOCK_SIZE: usize = 0x10;

/// Multiplexer for the Gameboy address bus
pub struct Gameboybus {
    cgb: bool,

    cart: Box<dyn Cartridge>,
    boot_rom: [u8; BOOTROM_SIZE_CGB],

    boot_rom_enabled: bool,

    wram: [u8; Self::WRAM_SIZE * Self::WRAM_BANKS],
    hram: [u8; u16::MAX as usize + 1],
    ie: u8,

    lcd: LCDController,
    timer: Timer,
    joypad: Joypad,
    apu: APU,

    /// IF register
    intflags: u8,

    /// Serial data buffer
    serialbuffer: u8,

    /// Output serial data to terminal
    serial_output: bool,

    /// Serial output channel (for tests)
    serial_channel: Option<mpsc::Sender<u8>>,

    /// CGB - WRAM bank select
    wram_banksel: u8,

    /// CGB - VRAM DMA source address
    vramdma_src: u16,

    /// CGB - VRAM DMA destination address
    vramdma_dest: u16,

    /// CGB - VRAM DMA len + HBlank mode
    /// If Some(), then DMA is in progress. This is required
    /// because the value for 'idle' is shared with a transfer
    /// of 0x7F in length in HBlank mode.
    vramdma_len: Option<u8>,

    /// CGB - Track HBlank STAT mode
    vramdma_hb_seen: bool,
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
            boot_rom: [0; BOOTROM_SIZE_CGB],
            boot_rom_enabled: false,

            wram: [0; Self::WRAM_SIZE * Self::WRAM_BANKS],
            wram_banksel: 1,
            hram: [0; u16::MAX as usize + 1],
            ie: 0,

            lcd,
            timer: Timer::from_div(0xAC), // Value after boot ROM
            joypad: Joypad::new(input),
            apu: APU::new(),

            intflags: cpu::INT_VBLANK, // VBlank is set after boot ROM
            serialbuffer: 0,
            serial_output: false,
            serial_channel: None,

            vramdma_src: 0,
            vramdma_dest: 0,
            vramdma_len: None,
            vramdma_hb_seen: false,
        };

        if let Some(br) = bootrom {
            bus.boot_rom[0..br.len()].copy_from_slice(br);
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

    fn do_vramdma(&mut self, written_len: Option<u8>) {
        if let Some(start_len) = written_len {
            self.vramdma_src = self.vramdma_src & !0x000F;
            self.vramdma_dest = (self.vramdma_dest & 0x9FF0) | 0x8000;

            if let Some(len) = self.vramdma_len {
                if len & !VRAMDMA_HBLANK_MODE == start_len {
                    // Pause transaction
                    self.vramdma_len = Some(start_len);
                    return;
                }
            }

            // Start transaction
            self.vramdma_len = Some(start_len);

            if start_len & VRAMDMA_HBLANK_MODE == VRAMDMA_HBLANK_MODE {
                // Do the first transaction once HBlank status is observed.
                return;
            }
        }

        let Some(len) = self.vramdma_len else {
            // Nothing to do..
            return;
        };
        let transfer_len;

        if !written_len.is_none() && len & VRAMDMA_HBLANK_MODE == 0 {
            // Normal mode, transfer full length immediately.
            transfer_len = (len as usize + 1) * VRAMDMA_BLOCK_SIZE;
            self.vramdma_len = None;
        } else if written_len.is_none() && len & VRAMDMA_HBLANK_MODE == VRAMDMA_HBLANK_MODE {
            // HBlank mode, transfer one block
            transfer_len = VRAMDMA_BLOCK_SIZE;
            if (len & VRAMDMA_LEN_MASK) == 0 {
                // Last block
                self.vramdma_len = None;
            } else {
                self.vramdma_len = Some(((len & VRAMDMA_LEN_MASK) - 1) | VRAMDMA_HBLANK_MODE);
            }
        } else {
            // Paused
            return;
        }

        for _ in 0..transfer_len {
            self.write(self.vramdma_dest, self.read(self.vramdma_src));

            if self.vramdma_dest >= 0x9FFF {
                // Destination overflow means completion
                self.vramdma_len = None;
                break;
            }

            self.vramdma_src = self.vramdma_src.wrapping_add(1);
            self.vramdma_dest = self.vramdma_dest.wrapping_add(1);
        }
    }
}

impl Bus for Gameboybus {}

impl BusMember for Gameboybus {
    fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;

        match addr {
            // DMG/CGB (lower part) Boot ROM
            0x0000..=0x00FF if self.boot_rom_enabled => self.boot_rom[addr],

            // CGB (upper part) Boot ROM
            0x0200..=0x08FF if self.boot_rom_enabled && self.cgb => self.boot_rom[addr],

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

            // I/O - APU
            0xFF10..=0xFF3F => self.apu.read(addr as u16),

            // I/O - LCD OAM DMA start
            // Handled here because we need to access source memory
            0xFF46 => 0,

            // I/O - LCD I/O
            0xFF40..=0xFF4B | 0xFF4F | 0xFF68..=0xFF6B => self.lcd.read_io(addr as u16),

            // CGB - KEY1 - Prepare speed switch
            0xFF4D if self.cgb => unreachable!(), // Handled by CPU

            // I/O - Boot ROM disable
            0xFF50 if self.boot_rom_enabled => 0,
            0xFF50 => 1,

            // CGB - HDMA1 - VRAM DMA source (MSB)
            0xFF51 if self.cgb => (self.vramdma_src >> 8) as u8,

            // CGB - HDMA2 - VRAM DMA source (LSB)
            0xFF52 if self.cgb => (self.vramdma_src & 0xFF) as u8,

            // CGB - HDMA3 - VRAM DMA destination (MSB)
            0xFF53 if self.cgb => (self.vramdma_dest >> 8) as u8,

            // CGB - HDMA4 - VRAM DMA destination (LSB)
            0xFF54 if self.cgb => (self.vramdma_dest & 0xFF) as u8,

            // CGB - HDMA5 - VRAM DMA length/mode/start
            0xFF55 if self.cgb => self.vramdma_len.unwrap_or(VRAMDMA_IDLE),

            // CGB - SVBK - WRAM bank select
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

            // I/O - APU
            0xFF10..=0xFF3F => self.apu.write(addr as u16, val),

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
            0xFF40..=0xFF4B | 0xFF4F | 0xFF68..=0xFF6B => self.lcd.write_io(addr as u16, val),

            // CGB - KEY1 - Prepare speed switch
            0xFF4D if self.cgb => unreachable!(), // Handled by CPU

            // CGB - HDMA1 - VRAM DMA source (MSB)
            0xFF51 if self.cgb => self.vramdma_src = ((val as u16) << 8) | self.vramdma_src & 0xFF,

            // CGB - HDMA2 - VRAM DMA source (LSB)
            0xFF52 if self.cgb => self.vramdma_src = self.vramdma_src & 0xFF00 | val as u16,

            // CGB - HDMA3 - VRAM DMA destination (MSB)
            0xFF53 if self.cgb => {
                self.vramdma_dest = ((val as u16) << 8) | self.vramdma_dest & 0xFF
            }

            // CGB - HDMA4 - VRAM DMA destination (LSB)
            0xFF54 if self.cgb => self.vramdma_dest = self.vramdma_dest & 0xFF00 | val as u16,

            // CGB - HDMA5 - VRAM DMA length/mode/start
            0xFF55 if self.cgb => self.do_vramdma(Some(val)),

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

        // Progress HBlank DMA if active
        let statmode = self.lcd.get_stat_mode();
        if statmode == LCDStatMode::HBlank && !self.vramdma_hb_seen {
            self.vramdma_hb_seen = true;

            self.do_vramdma(None);
        } else if statmode != LCDStatMode::HBlank {
            self.vramdma_hb_seen = false;
        }

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

    use num_traits::ToPrimitive;

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

    #[test]
    fn cgb_vram_dma() {
        let testex = |len, start: u16, end, src: u16, dest: u16| {
            let mut b = gbbus_cgb();

            // Pattern to WRAM
            for i in 0xC000_u16..0xD000 {
                b.write(i, 0xAB);
            }

            // Check initial VRAM
            for i in 0x8000..=0x9FFF {
                assert_eq!(b.read(i), 0);
            }

            // DMA idle
            assert_eq!(b.read(0xFF55), 0xFF);

            // Run transfer
            b.write16(0xFF51, src.to_be());
            b.write16(0xFF53, dest.to_be());
            b.write16(0xFF55, len);

            // Transfer completed, DMA idle
            assert_eq!(b.read(0xFF55), 0xFF);

            assert_ne!(b.read(start - 1), 0xAB);
            for a in start..=end {
                assert_eq!(b.read(a), 0xAB);
            }
            assert_ne!(b.read(end + 1), 0xAB);
        };
        let test = |len, start, end| testex(len, start, end, 0xC000, start);

        test(0, 0x8010, 0x801F);
        test(10, 0x8010, 0x80BF);
        test(0x7F, 0x8000, 0x87FF);
        test(0x7F, 0x8800, 0x8FFF);
        test(10, 0x9010, 0x90BF);
        test(0x7F, 0x9000, 0x97FF);
        test(0x7F, 0x9800, 0x9FFF);
    }

    #[test]
    fn cgb_vram_dma_masking_src() {
        let mut b = gbbus_cgb();

        // Pattern to WRAM
        for i in 0xC000_u16..0xC010 {
            b.write(i, 0xAB);
        }

        // Run transfer. This is supposed to copy
        // 0xC000 - 0xC00F.
        b.write16(0xFF51, 0xC00F_u16.to_be());
        b.write16(0xFF53, 0x8000_u16.to_be());
        b.write(0xFF55, 0); // 0x10

        // Transfer completed, DMA idle
        assert_eq!(b.read(0xFF55), 0xFF);

        for a in 0x8000_u16..=0x800F_u16 {
            assert_eq!(b.read(a), 0xAB);
        }
        assert_ne!(b.read(0x8010), 0xAB);
    }

    #[test]
    fn cgb_vram_dma_masking_dest() {
        let mut b = gbbus_cgb();

        // Pattern to WRAM
        for i in 0xC000_u16..0xC010 {
            b.write(i, 0xAB);
        }

        // Run transfer. This is supposed to copy
        // to 0x8000 - 0x800F.
        b.write16(0xFF51, 0xC000_u16.to_be());
        b.write16(0xFF53, 0x000F_u16.to_be());
        b.write(0xFF55, 0); // 0x10

        // Transfer completed, DMA idle
        assert_eq!(b.read(0xFF55), 0xFF);

        for a in 0x8000_u16..=0x800F_u16 {
            assert_eq!(b.read(a), 0xAB);
        }
        assert_ne!(b.read(0x8010), 0xAB);
        assert_ne!(b.read(0x000F), 0xAB);
    }

    #[test]
    fn cgb_vram_dma_dest_overflow() {
        let mut b = gbbus_cgb();

        // Pattern to WRAM
        for i in 0xC000_u16..0xC100 {
            b.write(i, 0xAB);
        }

        // Run transfer. This is supposed to stop
        // after one block.
        b.write16(0xFF51, 0xC000_u16.to_be());
        b.write16(0xFF53, 0x9FF0_u16.to_be());
        b.write(0xFF55, 0x7F);

        // Transfer completed, DMA idle
        assert_eq!(b.read(0xFF55), 0xFF);

        for a in 0x9FF0_u16..=0x9FFF_u16 {
            println!("{:X}", a);
            assert_eq!(b.read(a), 0xAB);
        }
        assert_ne!(b.read(0x8000), 0xAB);
        assert_ne!(b.read(0xA000), 0xAB);
    }

    #[test]
    fn cgb_vram_dma_hblank() {
        let testex = |len: u8, start: u16, end, src: u16, dest: u16| {
            let mut b = gbbus_cgb();

            let lcd_to_stat_mode = |b: &mut Gameboybus, mode: LCDStatMode| {
                while b.read(0xFF41) & 0x03 != mode.to_u8().unwrap() {
                    b.tick(1).unwrap();
                }
            };

            // Pattern to WRAM
            for i in 0xC000_u16..0xD000 {
                b.write(i, 0xAB);
            }

            // Check initial VRAM
            for i in 0x8000..=0x9FFF {
                assert_eq!(b.read(i), 0);
            }

            // DMA idle
            assert_eq!(b.read(0xFF55), 0xFF);

            // Run transfer
            b.write16(0xFF51, src.to_be());
            b.write16(0xFF53, dest.to_be());
            b.write(0xFF55, len | 0x80u8);

            // Transfer started
            assert_eq!(b.read(0xFF55), len | 0x80);
            assert_ne!(b.read(start), 0xAB);

            for block in 0..=(len as u16) {
                assert_ne!(b.read(start + block * 0x10), 0xAB);

                lcd_to_stat_mode(&mut b, LCDStatMode::HBlank);

                // Hit HBlank, now next block should have
                // been transfered.
                for c in 0..0x10 {
                    assert_eq!(b.read(start + (block * 0x10) + c), 0xAB);
                }
                assert_ne!(b.read(start + ((block + 1) * 0x10)), 0xAB);

                // End HBlank
                lcd_to_stat_mode(&mut b, LCDStatMode::Search);

                if block != len as u16 {
                    // Check progress indication
                    assert_eq!(b.read(0xFF55), (len - block as u8 - 1) | 0x80);
                }
            }

            // Transfer completed, DMA idle
            assert_eq!(b.read(0xFF55), 0xFF);

            assert_ne!(b.read(start - 1), 0xAB);
            for a in start..=end {
                assert_eq!(b.read(a), 0xAB);
            }
            assert_ne!(b.read(end + 1), 0xAB);
        };
        let test = |len, start, end| testex(len, start, end, 0xC000, start);

        test(0, 0x8010, 0x801F);
        test(10, 0x8010, 0x80BF);
        test(0x7F, 0x8000, 0x87FF);
        test(0x7F, 0x8800, 0x8FFF);
        test(10, 0x9010, 0x90BF);
        test(0x7F, 0x9000, 0x97FF);
        test(0x7F, 0x9800, 0x9FFF);
    }

    #[test]
    fn cgb_vram_dma_hblank_pause() {
        let testex = |len: u8, start: u16, end, src: u16, dest: u16, pause_interval: u8| {
            println!(
                "testex len: {:X} start: {:X} pauseint: {:X}",
                len, start, pause_interval
            );
            let mut b = gbbus_cgb();

            let lcd_to_stat_mode = |b: &mut Gameboybus, mode: LCDStatMode| {
                while b.read(0xFF41) & 0x03 != mode.to_u8().unwrap() {
                    b.tick(1).unwrap();
                }
            };

            // Pattern to WRAM
            for i in 0xC000_u16..0xD000 {
                b.write(i, 0xAB);
            }

            // Check initial VRAM
            for i in 0x8000..=0x9FFF {
                assert_eq!(b.read(i), 0);
            }

            // DMA idle
            assert_eq!(b.read(0xFF55), 0xFF);

            // Run transfer
            b.write16(0xFF51, src.to_be());
            b.write16(0xFF53, dest.to_be());
            b.write(0xFF55, len | 0x80u8);

            // Transfer started
            assert_eq!(b.read(0xFF55), len | 0x80);
            assert_ne!(b.read(start), 0xAB);

            for block in 0..=(len as u16) {
                println!("{}", block);

                if pause_interval == 0 || block % pause_interval as u16 == 0 {
                    println!("pausing at block {}", block);
                    // Test pause
                    b.write(0xFF55, b.read(0xFF55) & !0x80);
                    assert_eq!(b.read(0xFF55), (len - block as u8));

                    lcd_to_stat_mode(&mut b, LCDStatMode::HBlank);
                    assert_eq!(b.read(0xFF55), (len - block as u8));

                    assert_ne!(b.read(start + block * 0x10), 0xAB);

                    // Resume
                    lcd_to_stat_mode(&mut b, LCDStatMode::Search);
                    b.write(0xFF55, b.read(0xFF55) | 0x80);

                    // Should not have written anything now until
                    // next HBlank.
                }

                assert_ne!(b.read(start + block * 0x10), 0xAB);

                lcd_to_stat_mode(&mut b, LCDStatMode::HBlank);

                // Hit HBlank, now next block should have
                // been transfered.
                for c in 0..0x10 {
                    assert_eq!(b.read(start + (block * 0x10) + c), 0xAB);
                }
                assert_ne!(b.read(start + ((block + 1) * 0x10)), 0xAB);

                // End HBlank
                lcd_to_stat_mode(&mut b, LCDStatMode::Search);

                if block != len as u16 {
                    // Check progress indication
                    assert_eq!(b.read(0xFF55), (len - block as u8 - 1) | 0x80);
                }
            }

            // Transfer completed, DMA idle
            assert_eq!(b.read(0xFF55), 0xFF);

            assert_ne!(b.read(start - 1), 0xAB);
            for a in start..=end {
                assert_eq!(b.read(a), 0xAB);
            }
            assert_ne!(b.read(end + 1), 0xAB);
        };
        let test = |len, start, end| {
            for pause_interval in 0..len {
                testex(len, start, end, 0xC000, start, pause_interval);
            }
        };

        test(0, 0x8010, 0x801F);
        test(10, 0x8010, 0x80BF);
        test(0x7F, 0x8000, 0x87FF);
        test(0x7F, 0x8800, 0x8FFF);
        test(10, 0x9010, 0x90BF);
        test(0x7F, 0x9000, 0x97FF);
        test(0x7F, 0x9800, 0x9FFF);
    }
}
