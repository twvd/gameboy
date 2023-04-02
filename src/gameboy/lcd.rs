use crate::display::display::Display;
use crate::tickable::Tickable;

use anyhow::Result;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;

pub const LCD_W: usize = 160;
pub const LCD_H: usize = 144;

const OAM_SIZE: usize = 0xA0;
const OAM_ENTRY_SIZE: usize = 4;
const VRAM_SIZE: usize = 0x2000;

//const OAM_ENTRIES: usize = 40;

const TILE_BSIZE: usize = 16;
const TILE_W: usize = 8;
const TILE_H: usize = 8;

// LCDC flags
const LCDC_ENABLE: u8 = 1 << 7;
const LCDC_BGW_TILEDATA: u8 = 1 << 4;
const LCDC_BGW_TILEMAP: u8 = 1 << 3;
const LCDC_OBJ_SIZE: u8 = 1 << 2;
const LCDC_OBJ_ENABLE: u8 = 1 << 1;
const LCDC_BGW_ENABLE: u8 = 1 << 0;

// Writable LCDS bits
const LCDS_MASK: u8 = 0x78;

#[derive(Debug, Copy, Clone, Eq, PartialEq, ToPrimitive)]
enum LCDStatMode {
    Search = 2,
    Transfer = 3,
    HBlank = 0,
    VBlank = 1,
}

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

    /// LCDS - LCD Status register
    lcds: u8,

    /// SCY - Scroll Y register
    scy: u8,

    /// SCX - Scroll X register
    scx: u8,

    /// Current scanline
    ly: u8,

    /// Background/window palette
    bgp: u8,

    /// Object Palettes
    obp: [u8; 2],

    /// Output display needs updsting
    redraw_pending: bool,

    /// Dot refresh position
    dots: u128,
}

impl LCDController {
    /// Dots per scanline (including HBlank)
    const DOTS_PER_LINE: u128 = 456;

    /// Amount of vertical scanlines (including VBlank)
    const SCANLINES: u128 = 154;

    /// Start scanline of VBLANK period
    const VBLANK_START: u128 = 144;

    /// Amount of dots in 'search' mode
    const SEARCH_PERIOD: u128 = 80;

    /// Amount of dots in 'transfer' mode (actually 168 to 291)
    const TRANSFER_PERIOD: u128 = 200;

    pub fn new(display: Box<dyn Display>) -> Self {
        Self {
            output: display,
            oam: [0; OAM_SIZE],
            vram: [0; VRAM_SIZE],

            lcdc: 0,
            lcds: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            bgp: 0,
            obp: [0, 0],

            redraw_pending: false,

            dots: 0,
        }
    }

    fn get_stat_mode(&self) -> LCDStatMode {
        // Mode 2  2_____2_____2_____2_____2_____2___________________2____
        // Mode 3  _33____33____33____33____33____33__________________3___
        // Mode 0  ___000___000___000___000___000___000________________000
        // Mode 1  ____________________________________11111111111111_____
        if self.in_vblank() {
            LCDStatMode::VBlank
        } else {
            let hpos = self.dots % Self::DOTS_PER_LINE;
            if hpos < Self::SEARCH_PERIOD {
                LCDStatMode::Search
            } else if hpos < Self::SEARCH_PERIOD + Self::TRANSFER_PERIOD {
                LCDStatMode::Transfer
            } else {
                LCDStatMode::HBlank
            }
        }
    }

    /// Calculate LY based on current timed LCD scan
    fn calc_ly(&self) -> u8 {
        Self::calc_scanline(self.dots)
    }

    fn calc_scanline(dots: u128) -> u8 {
        let lines_scanned = dots / Self::DOTS_PER_LINE;
        (lines_scanned % Self::SCANLINES) as u8
    }

    pub fn in_vblank(&self) -> bool {
        self.ly >= Self::VBLANK_START as u8
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
        let tile_id = self.get_bg_tile_id(tm_x, tm_y) as usize;
        let tile_addr = if self.lcdc & LCDC_BGW_TILEDATA == LCDC_BGW_TILEDATA {
            // 0x8000 base offset, contiguous blocks
            0x8000 + tile_id * TILE_BSIZE
        } else {
            // 0-127 from 0x9000, 128-255 from 0x8800
            if tile_id < 128 {
                0x9000 + tile_id * TILE_BSIZE
            } else {
                0x8800 + (tile_id - 128) * TILE_BSIZE
            }
        };

        // Correct for our VRAM array
        let tile_addr = tile_addr - 0x8000;

        &self.vram[tile_addr..tile_addr + TILE_BSIZE]
    }

    #[inline(always)]
    fn get_sprite(&self, tile_idx: usize) -> &[u8] {
        // VRAM offset = 8000 - 9FFF
        // Sprites always start from 8000 (tile_idx 0)
        // Sprites can be 8x8 or 8x16 (LCDC_OBJ_SIZE)
        // In 8x16 mode, the least significant bit of tile_idx
        // is ignored.
        let offset = 0x8000;
        let tile_addr = offset - 0x8000 + tile_idx * TILE_BSIZE;

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

            // LCDS - LCD status register
            0xFF41 => self.lcds = val & LCDS_MASK,

            // SCY - Background scrolling viewport Y
            0xFF42 => {
                self.scy = val;
            }

            // SCX - Background scrolling viewport X
            0xFF43 => self.scx = val,

            // BGP - Background and window palette
            0xFF47 => self.bgp = val,

            // OBPx - Object Palette
            0xFF48 => self.obp[0] = val,
            0xFF49 => self.obp[1] = val,

            _ => (), //println!("Write to unknown LCD address: {:04X} = {:02X}", addr, val),
        }
    }

    pub fn read_io(&self, addr: u16) -> u8 {
        match addr {
            // LCDC - LCD control register
            0xFF40 => self.lcdc,

            // LCDS - LCD status register
            0xFF41 => (self.lcds & LCDS_MASK) | self.get_stat_mode().to_u8().unwrap(),

            // SCY - Background scrolling viewport Y
            0xFF42 => self.scy,

            // SCX - Background scrolling viewport X
            0xFF43 => self.scx,

            // OAM DMA start
            0xFF46 => 0,

            // LY - LCD update Y position
            0xFF44 => self.ly,

            // BGP - Bsckground and window palette
            0xFF47 => self.bgp,

            // OBPx - Object palette
            0xFF48 => self.obp[0],
            0xFF49 => self.obp[1],

            _ => {
                //println!("Read from unknown LCD address: {:04X}", addr);
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

    /// Converts a color index to a color from the
    /// BG/OBJ palettes
    /// (DMG-mode only)
    fn palette_convert(cidx: u8, palette: u8) -> u8 {
        (palette >> (cidx * 2)) & 3
    }

    fn draw_tile_at(&mut self, tile: &[u8], x: isize, y: isize, palette: u8) {
        for tx in 0..TILE_W {
            for ty in 0..TILE_H {
                let color = Self::palette_convert(Self::tile_decode(&tile, tx, ty), palette);
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

        if self.lcdc & LCDC_ENABLE != LCDC_ENABLE {
            return;
        }

        // Background
        if self.lcdc & LCDC_BGW_ENABLE == LCDC_BGW_ENABLE {
            for x in 0..32 {
                for y in 0..32 {
                    let tile = self.get_bg_tile(x, y).to_owned();
                    self.draw_tile_at(
                        &tile,
                        (x * TILE_W) as isize - self.scx as isize,
                        (y * TILE_H) as isize - self.scy as isize,
                        self.bgp,
                    );
                }
            }
        }

        // Object sprites
        if self.lcdc & LCDC_OBJ_ENABLE == LCDC_OBJ_ENABLE {
            if self.lcdc & LCDC_OBJ_SIZE == LCDC_OBJ_SIZE {
                // 16px wide sprites
                todo!();
            }

            for obj_idx in 0..(OAM_SIZE / OAM_ENTRY_SIZE) {
                let entry =
                    self.oam[(obj_idx * OAM_ENTRY_SIZE)..(obj_idx + 1) * OAM_ENTRY_SIZE].to_owned();
                let (y, x, tile_idx, flags) = (entry[0], entry[1], entry[2], entry[3]);
                let disp_y = y as isize - 16;
                let disp_x = x as isize - 8;

                let sprite = self.get_sprite(tile_idx as usize).to_owned();

                self.draw_tile_at(
                    &sprite,
                    disp_x,
                    disp_y,
                    self.obp[((flags & 0x10) >> 4) as usize],
                );
            }
        }

        self.output.render();
        self.redraw_pending = false;
    }
}

impl Tickable for LCDController {
    fn tick(&mut self, ticks: usize) -> Result<usize> {
        self.dots = (self.dots + ticks as u128) % (Self::DOTS_PER_LINE * Self::SCANLINES);
        self.ly = self.calc_ly();

        if self.in_vblank() {
            self.redraw_pending = true;
        } else if self.redraw_pending && self.ly >= 10 {
            // Wait 10 lines to give the CPU some time
            self.redraw();
        }

        Ok(ticks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::display::display::NullDisplay;

    #[test]
    fn tile_decode() {
        let tile = [0x3C, 0x7E];
        let result = [0, 2, 3, 3, 3, 3, 2, 0];

        for x in 0..result.len() {
            assert_eq!(LCDController::tile_decode(&tile, x, 0), result[x]);
        }
    }

    #[test]
    fn statmode() {
        fn next(l: &mut LCDController) {
            let val = l.get_stat_mode();
            while l.get_stat_mode() == val {
                l.tick(1).unwrap();
            }
        }

        let mut c = LCDController::new(Box::new(NullDisplay::new()));
        assert_eq!(c.get_stat_mode(), LCDStatMode::Search);

        for _ in 0..LCDController::VBLANK_START {
            assert_eq!(c.get_stat_mode(), LCDStatMode::Search);
            next(&mut c);
            assert_eq!(c.get_stat_mode(), LCDStatMode::Transfer);
            next(&mut c);
            assert_eq!(c.get_stat_mode(), LCDStatMode::HBlank);
            next(&mut c);
        }
        assert_eq!(c.get_stat_mode(), LCDStatMode::VBlank);
    }
}
