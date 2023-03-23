use std::cell::Cell;
use std::time::SystemTime;

use super::super::display::display::Display;

pub const LCD_W: usize = 160;
pub const LCD_H: usize = 144;

const OAM_SIZE: usize = 0xA0;
const VRAM_SIZE: usize = 0x2000;

//const OAM_ENTRIES: usize = 40;

const TILE_BSIZE: usize = 16;
const TILE_W: usize = 8;
const TILE_H: usize = 8;

// LCDC flags
const LCDC_BGW_TILEDATA: u8 = 1 << 4;
const LCDC_BGW_TILEMAP: u8 = 1 << 3;

#[derive(Copy, Clone)]
#[repr(C)]
struct OAMEntry {
    y: u8,
    x: u8,
    tile_idx: u8,
    flags: u8,
}

impl OAMEntry {}

/// LCD controller state
pub struct LCDController {
    /// Display output
    output: Box<dyn Display>,

    /// OAM memory
    oam: [u8; OAM_SIZE],

    /// VRAM memory
    vram: [u8; VRAM_SIZE],

    /// LCDC - LCD Control register
    lcdc: u8,

    /// SCY - Scroll Y register
    scy: u8,

    /// SCX - Scroll X register
    scx: u8,

    /// Time base for timing stuff
    timebase: SystemTime,

    /// Current scanline
    scanline: Cell<u8>,
}

impl LCDController {
    /// LCD dot clock frequency in Hz
    const DOTCLOCK_HZ: u128 = 4_194_000;

    /// Dots per scanline (including HBlank)
    const DOTS_PER_LINE: u128 = 456;

    /// Amount of vertical scanlines (including VBlank)
    const SCANLINES: u128 = 154;

    pub fn new(display: Box<dyn Display>) -> Self {
        Self {
            output: display,
            oam: [0; OAM_SIZE],
            vram: [0; VRAM_SIZE],

            lcdc: 0,
            scy: 0,
            scx: 0,

            timebase: SystemTime::now(),
            scanline: Cell::new(0),
        }
    }

    /// Calculate LY based on current timed LCD scan
    fn get_ly(&self) -> u8 {
        let elapsed = SystemTime::now()
            .duration_since(self.timebase)
            .expect("Time error");

        // Make sure LY always increments by 1. Because games tend
        // to check LY against absolute numbers, skipping over the
        // number its being compared to (i.e. because of scheduling),
        // we'd delay the game by a whole frame.
        if Self::calc_scanline(elapsed.as_micros()) != self.scanline.get() {
            self.scanline
                .set((self.scanline.get() + 1) % Self::SCANLINES as u8);
        }
        self.scanline.get()
    }

    fn calc_scanline(elapsed: u128) -> u8 {
        let dots_scanned = Self::DOTCLOCK_HZ * elapsed / 1_000_000u128;
        let lines_scanned = dots_scanned / Self::DOTS_PER_LINE;
        (lines_scanned % Self::SCANLINES) as u8
    }

    #[inline(always)]
    fn get_bg_tile_id(&self, tm_x: usize, tm_y: usize) -> u8 {
        // VRAM offset = 8000 - 9FFF
        // BG tile map at 9800 - 9BFF and 9C00 - 9FFF
        let offset = if self.lcdc & LCDC_BGW_TILEMAP == LCDC_BGW_TILEMAP {
            0x9C00
        } else {
            0x9800
        };

        self.vram[offset - 0x8000 + (tm_y * 32) + tm_x]
    }

    #[inline(always)]
    fn get_bg_tile(&self, tm_x: usize, tm_y: usize) -> &[u8] {
        // VRAM offset = 8000 - 9FFF
        // BG tile data at 8800 - 97FF and 8000 - 8FFF
        // BG tiles always 8 x 8 pixels
        let offset = if self.lcdc & LCDC_BGW_TILEDATA == LCDC_BGW_TILEDATA {
            0x8000
        } else {
            0x8800
        };
        let tile_addr = offset - 0x8000 + self.get_bg_tile_id(tm_x, tm_y) as usize * TILE_BSIZE;

        &self.vram[tile_addr..tile_addr + TILE_BSIZE]
    }

    #[inline(always)]
    fn tile_decode(tile: &[u8], x: usize, y: usize) -> u8 {
        // Least significant bit in the odd bytes,
        // most significant bit in the even bytes.
        let x = 7 - x;
        let lsb = (tile[y * 2] & (1 << x)) >> x;
        let msb = (tile[y * 2 + 1] & (1 << x)) >> x;

        lsb | msb << 1
    }

    pub fn write_io(&mut self, addr: u16, val: u8) {
        match addr {
            // LCDC - LCD control register
            0xFF40 => self.lcdc = val,

            // SCY - Background scrolling viewport Y
            0xFF42 => {
                self.scy = val;
            }

            // SCX - Background scrolling viewport X
            0xFF43 => self.scx = val,

            _ => println!("Write to unknown LCD address: {:04X}", addr),
        }
        self.redraw();
    }

    pub fn read_io(&self, addr: u16) -> u8 {
        match addr {
            // LCDC - LCD control register
            0xFF40 => self.lcdc,

            // SCY - Background scrolling viewport Y
            0xFF42 => self.scy,

            // SCX - Background scrolling viewport X
            0xFF43 => self.scx,

            // LY - LCD update Y position
            0xFF44 => self.get_ly(),
            _ => {
                println!("Read from unknown LCD address: {:04X}", addr);
                0
            }
        }
    }

    pub fn write_vram(&mut self, addr: usize, val: u8) {
        self.vram[addr] = val;
    }

    pub fn write_oam(&mut self, addr: usize, val: u8) {
        self.oam[addr] = val;
    }

    pub fn read_vram(&self, addr: usize) -> u8 {
        self.vram[addr]
    }

    fn draw_tile_at(&mut self, tile: &[u8], x: isize, y: isize) {
        for tx in 0..TILE_W {
            for ty in 0..TILE_H {
                let color = Self::tile_decode(&tile, tx, ty);
                let disp_x = x + tx as isize;
                let disp_y = y + ty as isize;

                if disp_x < 0 || disp_y < 0 || disp_x >= LCD_W as isize || disp_y >= LCD_H as isize
                {
                    continue;
                }
                self.output
                    .set_pixel(disp_x as usize, disp_y as usize, color);
            }
        }
    }

    pub fn redraw(&mut self) {
        self.output.clear();

        // Background
        for x in 0..32 {
            for y in 0..32 {
                let tile = self.get_bg_tile(x, y).to_owned();
                self.draw_tile_at(
                    &tile,
                    (x * TILE_W) as isize - self.scx as isize,
                    (y * TILE_H) as isize - self.scy as isize,
                );
            }
        }

        self.output.render();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_decode() {
        let tile = [0x3C, 0x7E];
        let result = [0, 2, 3, 3, 3, 3, 2, 0];

        for x in 0..result.len() {
            assert_eq!(LCDController::tile_decode(&tile, x, 0), result[x]);
        }
    }

    #[test]
    fn calc_scanline() {
        // One full frame = 16.74ms
        assert_eq!(LCDController::calc_scanline(0u128), 0);
        assert_eq!(LCDController::calc_scanline(1_674_0u128), 153);
        assert_eq!(LCDController::calc_scanline(1_675_0u128), 0);
    }
}
