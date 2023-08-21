use crate::display::display::Display;
use crate::gameboy::lcd_oam::OAMTable;
use crate::tickable::Tickable;

use anyhow::Result;
use num_derive::ToPrimitive;
use num_traits::ToPrimitive;

pub const LCD_W: usize = 160;
pub const LCD_H: usize = 144;

/// Type of a color definition (RGB555)
pub type Color = u16;

const VRAM_SIZE: usize = 0x2000;

// Tile sizes
const TILE_BSIZE: usize = 16;
const TILE_W: isize = 8;
const TILE_H: isize = 8;

// Backgtound/window size (in tiles)
const BGW_H: isize = 32;
const BGW_W: isize = 32;

// LCDC flags
const LCDC_ENABLE: u8 = 1 << 7;
const LCDC_WINDOW_TILEMAP: u8 = 1 << 6;
const LCDC_WINDOW_ENABLE: u8 = 1 << 5;
const LCDC_BGW_TILEDATA: u8 = 1 << 4;
const LCDC_BG_TILEMAP: u8 = 1 << 3;
const LCDC_OBJ_SIZE: u8 = 1 << 2;
const LCDC_OBJ_ENABLE: u8 = 1 << 1;
const LCDC_BGW_ENABLE: u8 = 1 << 0;

// Writable LCDS bits
const LCDS_MASK: u8 = 0x78;

const LCDS_STATMODE_MASK: u8 = 0x03;

// LCDS bits
const LCDS_INT_LYC: u8 = 1 << 6;
const LCDS_INT_STAT_OAM: u8 = 1 << 5;
const LCDS_INT_STAT_VBLANK: u8 = 1 << 4;
const LCDS_INT_STAT_HBLANK: u8 = 1 << 3;
const LCDS_LYC: u8 = 1 << 2;

// OAM flags
const OAM_BGW_PRIORITY: u8 = 1 << 7;
const OAM_FLIP_Y: u8 = 1 << 6;
const OAM_FLIP_X: u8 = 1 << 5;

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
    oam: OAMTable,

    /// Gameboy Color mode
    cgb: bool,

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

    /// WX - Window X register
    wx: u8,

    /// WY - Window Y register
    wy: u8,

    /// Current scanline
    ly: u8,

    /// Current window scanline
    wly: u8,

    /// LY compare register
    lyc: u8,

    /// Background/window palette
    bgp: u8,

    /// Object Palettes
    obp: [u8; 2],

    /// Output display needs updsting
    redraw_pending: bool,

    /// Dot refresh position
    dots: u128,

    /// STAT interrupt request
    intreq_stat: bool,

    /// VBlank interrupt request
    intreq_vblank: bool,
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

    pub fn new(display: Box<dyn Display>, cgb: bool) -> Self {
        Self {
            output: display,
            cgb,
            oam: OAMTable::new(),
            vram: [0; VRAM_SIZE],

            lcdc: LCDC_ENABLE,
            lcds: 0,
            scy: 0,
            scx: 0,
            wx: 0,
            wy: 0,
            wly: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            obp: [0, 0],

            redraw_pending: false,

            dots: 0,

            intreq_stat: false,
            intreq_vblank: false,
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
        self.dots >= (Self::VBLANK_START * Self::DOTS_PER_LINE)
    }

    /// Tests all conditions for the window to be drawn and the counter
    /// running
    fn is_window_active(&self) -> bool {
        self.lcdc & (LCDC_WINDOW_ENABLE | LCDC_BGW_ENABLE) == (LCDC_WINDOW_ENABLE | LCDC_BGW_ENABLE)
            && (0u8..=166).contains(&self.wx)
            && (0u8..=143).contains(&self.wy)
    }

    #[inline(always)]
    fn get_tile_id(&self, tm_x: isize, tm_y: isize, selbit: u8) -> u8 {
        // VRAM offset = 8000 - 9FFF
        // BG tile map at 9800 - 9BFF or 9C00 - 9FFF
        // Window tile map at 9800 - 9BFF or 9C00 - 9FFF
        let offset: isize = if self.lcdc & selbit == selbit {
            0x9C00
        } else {
            0x9800
        };

        assert!(tm_x < BGW_W && tm_y < BGW_H);

        self.vram[(offset - 0x8000 + (tm_y * BGW_H) + tm_x) as usize]
    }

    #[inline(always)]
    fn get_bgw_tile(&self, tm_x: isize, tm_y: isize, selbit: u8) -> &[u8] {
        // VRAM offset = 8000 - 9FFF
        // BG/Win tile data at 8800 - 97FF and 8000 - 8FFF
        // BG/Win tiles always 8 x 8 pixels
        let tile_id = self.get_tile_id(tm_x, tm_y, selbit) as usize;
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
        let tile_addr = (tile_addr - 0x8000) as usize;

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
            0xFF41 => self.lcds = (self.lcds & !LCDS_MASK) | (val & LCDS_MASK),

            // SCY - Background scrolling viewport Y
            0xFF42 => self.scy = val,

            // SCX - Background scrolling viewport X
            0xFF43 => self.scx = val,

            // LYC - LY compare
            0xFF45 => self.lyc = val,

            // BGP - Background and window palette
            0xFF47 => self.bgp = val,

            // WY - Window Y register
            0xFF4A => self.wy = val,

            // WX - Window X register
            0xFF4B => self.wx = val,

            // OBPx - Object Palette
            0xFF48 => self.obp[0] = val,
            0xFF49 => self.obp[1] = val,

            _ => (),
        }
    }

    pub fn read_io(&self, addr: u16) -> u8 {
        match addr {
            // LCDC - LCD control register
            0xFF40 => self.lcdc,

            // LCDS - LCD status register
            0xFF41 => self.lcds,

            // SCY - Background scrolling viewport Y
            0xFF42 => self.scy,

            // SCX - Background scrolling viewport X
            0xFF43 => self.scx,

            // LY - LCD update Y position
            0xFF44 => self.ly,

            // LYC - LY compare
            0xFF45 => self.lyc,

            // BGP - Bsckground and window palette
            0xFF47 => self.bgp,

            // OBPx - Object palette
            0xFF48 => self.obp[0],
            0xFF49 => self.obp[1],

            // WY - Window Y register
            0xFF4A => self.wy,

            // WX - Window X register
            0xFF4B => self.wx,

            _ => 0,
        }
    }

    pub fn write_vram(&mut self, addr: usize, val: u8) {
        self.vram[addr] = val;
    }

    pub fn write_oam(&mut self, addr: usize, val: u8) {
        self.oam.write(addr, val);
    }

    pub fn read_vram(&self, addr: usize) -> u8 {
        self.vram[addr]
    }

    pub fn read_oam(&self, addr: usize) -> u8 {
        self.oam.read(addr)
    }

    /// Converts a color index to a color from the
    /// BG/OBJ palettes
    /// (DMG-mode only)
    fn palette_convert_dmg(cidx: u8, palette: u8) -> Color {
        match (palette >> (cidx * 2)) & 3 {
            3 => 0,
            2 => 0b01000_01000_01000,
            1 => 0b11000_11000_11000,
            0 => 0b11111_11111_11111,
            _ => unreachable!(),
        }
    }

    fn draw_tile_at(
        &self,
        tile: &[u8],
        line: &mut [Color],
        x: isize,
        y: isize,
        palette: u8,
        obj_flags: Option<u8>,
        scanline: isize,
        wrap_x: Option<isize>,
        wrap_y: Option<isize>,
    ) {
        for ty in 0..TILE_H {
            // Y-axis wrap around (scrolling)
            let disp_y = if let Some(wy) = wrap_y {
                (y + ty).rem_euclid(wy)
            } else {
                y + ty
            };

            if disp_y != scanline {
                continue;
            }

            for tx in 0..TILE_W {
                // X-axis wrap around (scrolling)
                let disp_x = if let Some(wx) = wrap_x {
                    (x + tx).rem_euclid(wx)
                } else {
                    x + tx
                };

                if disp_x < 0 || disp_x >= LCD_W as isize {
                    continue;
                }

                let color_idx = Self::tile_decode(
                    &tile,
                    if obj_flags.unwrap_or(0) & OAM_FLIP_X == OAM_FLIP_X {
                        // Mirror along X axis
                        7 - tx as usize
                    } else {
                        tx as usize
                    },
                    if obj_flags.unwrap_or(0) & OAM_FLIP_Y == OAM_FLIP_Y {
                        // Mirror along Y axis
                        7 - ty as usize
                    } else {
                        ty as usize
                    },
                );

                // Objects blend into background
                if obj_flags.is_some() {
                    if color_idx == 0 {
                        continue;
                    }

                    // BG priority
                    if obj_flags.unwrap() & OAM_BGW_PRIORITY == OAM_BGW_PRIORITY
                        && line[disp_x as usize] > 0
                    {
                        continue;
                    }
                }

                let color = Self::palette_convert_dmg(color_idx, palette);

                line[disp_x as usize] = color;
            }
        }
    }

    pub fn draw_scanline(&mut self, scanline: isize) {
        if self.lcdc & LCDC_ENABLE != LCDC_ENABLE {
            return;
        }

        let mut line = [0; LCD_W];

        // Background
        if self.lcdc & LCDC_BGW_ENABLE == LCDC_BGW_ENABLE {
            let t_y = (scanline + self.scy as isize).rem_euclid(BGW_H * TILE_H) / TILE_H;
            for t_x in 0..BGW_W {
                let tile = self.get_bgw_tile(t_x, t_y, LCDC_BG_TILEMAP).to_owned();
                self.draw_tile_at(
                    &tile,
                    &mut line,
                    (t_x as isize * TILE_W) - self.scx as isize,
                    (t_y as isize * TILE_H) - self.scy as isize,
                    self.bgp,
                    None,
                    scanline,
                    Some(BGW_W * TILE_W),
                    Some(BGW_H * TILE_H),
                );
            }
        }

        // The window
        if self.is_window_active() && scanline >= self.wy as isize {
            let t_y = self.wly as isize / TILE_H;
            for t_x in 0..BGW_W {
                let tile = self.get_bgw_tile(t_x, t_y, LCDC_WINDOW_TILEMAP).to_owned();
                self.draw_tile_at(
                    &tile,
                    &mut line,
                    (t_x as isize * TILE_W) + self.wx as isize - 7,
                    (t_y as isize * TILE_H) + (scanline - self.wly as isize),
                    self.bgp,
                    None,
                    scanline,
                    None,
                    None,
                );
            }
        }

        // Object sprites
        if self.lcdc & LCDC_OBJ_ENABLE == LCDC_OBJ_ENABLE {
            for e in self.oam.iter_scanline(
                scanline,
                if self.lcdc & LCDC_OBJ_SIZE == LCDC_OBJ_SIZE {
                    TILE_H * 2
                } else {
                    TILE_H
                },
            ) {
                let disp_y = e.y as isize - 16;
                let mut tile_idx = e.tile_idx;
                if self.lcdc & LCDC_OBJ_SIZE == LCDC_OBJ_SIZE {
                    tile_idx &= !0x01;
                    if e.flags & OAM_FLIP_Y == OAM_FLIP_Y {
                        // Also rotate the tiles for 8x16
                        tile_idx |= 0x01;
                    }
                }

                let disp_x = e.x as isize - 8;

                let sprite = self.get_sprite(tile_idx as usize).to_owned();

                self.draw_tile_at(
                    &sprite,
                    &mut line,
                    disp_x,
                    disp_y,
                    self.obp[((e.flags & 0x10) >> 4) as usize],
                    Some(e.flags),
                    scanline,
                    None,
                    None,
                );
                if self.lcdc & LCDC_OBJ_SIZE == LCDC_OBJ_SIZE {
                    // 8x16
                    let tile_idx2 = if e.flags & OAM_FLIP_Y == OAM_FLIP_Y {
                        tile_idx & !0x01
                    } else {
                        tile_idx | 0x01
                    };
                    let sprite2 = self.get_sprite(tile_idx2 as usize).to_owned();
                    self.draw_tile_at(
                        &sprite2,
                        &mut line,
                        disp_x,
                        disp_y + TILE_H,
                        self.obp[((e.flags & 0x10) >> 4) as usize],
                        Some(e.flags),
                        scanline,
                        None,
                        None,
                    );
                }
            }
        }

        for (x, c) in line.into_iter().enumerate() {
            self.output.set_pixel(x, scanline as usize, c.into());
        }
    }

    pub fn get_clr_intreq_stat(&mut self) -> bool {
        let b = self.intreq_stat;
        self.intreq_stat = false;
        b
    }

    pub fn get_clr_intreq_vblank(&mut self) -> bool {
        let b = self.intreq_vblank;
        self.intreq_vblank = false;
        b
    }
}

impl Tickable for LCDController {
    fn tick(&mut self, ticks: usize) -> Result<usize> {
        if self.lcdc & LCDC_ENABLE == 0 {
            // PPU disabled, restart frame
            self.dots = 0;
            self.ly = 0;
            return Ok(ticks);
        }

        let old_mode = self.get_stat_mode();

        // TODO this may skip interrupts on many ticks?
        assert!(ticks < Self::SEARCH_PERIOD as usize);

        self.dots = (self.dots + ticks as u128) % (Self::DOTS_PER_LINE * Self::SCANLINES);

        let newly = self.calc_ly();
        let new_mode = self.get_stat_mode();

        if newly != self.ly {
            self.ly = newly;

            // Check LY compare + interrupt
            if self.ly == self.lyc {
                self.lcds |= LCDS_LYC;
                if self.lcds & LCDS_INT_LYC == LCDS_INT_LYC {
                    self.intreq_stat = true;
                }
            } else {
                self.lcds &= !LCDS_LYC;
            }

            // Check VBlank / VBlank STAT interrupts
            if old_mode != LCDStatMode::VBlank && new_mode == LCDStatMode::VBlank {
                self.intreq_vblank = true;
                if self.lcds & LCDS_INT_STAT_VBLANK == LCDS_INT_STAT_VBLANK {
                    self.intreq_stat = true;
                }

                // Reset window line counter
                self.wly = 0;
            }
        }

        // Check HBlank STAT interrupt
        if old_mode != LCDStatMode::HBlank && new_mode == LCDStatMode::HBlank {
            if self.lcds & LCDS_INT_STAT_HBLANK == LCDS_INT_STAT_HBLANK {
                self.intreq_stat = true;
            }
        }

        // Check OAM STAT interrupt
        if old_mode != LCDStatMode::Search && new_mode == LCDStatMode::Search {
            if self.lcds & LCDS_INT_STAT_OAM == LCDS_INT_STAT_OAM {
                self.intreq_stat = true;
            }
        }

        // Draw when in transfer mode
        if old_mode != LCDStatMode::Transfer
            && new_mode == LCDStatMode::Transfer
            && !self.in_vblank()
        {
            self.draw_scanline(self.ly as isize);

            // Window line counter
            if self.is_window_active() && !self.in_vblank() && self.ly >= self.wy {
                self.wly += 1;
            }
        }

        // Update mode register
        self.lcds = (self.lcds & !LCDS_STATMODE_MASK) | self.get_stat_mode().to_u8().unwrap();

        if self.in_vblank() {
            if self.redraw_pending {
                self.redraw_pending = false;
                self.output.render();
            }
        } else {
            self.redraw_pending = true;
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

        let mut c = LCDController::new(Box::new(NullDisplay::new()), false);
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

    #[test]
    fn vblank() {
        let mut c = LCDController::new(Box::new(NullDisplay::new()), false);
        for _ in 0..(LCD_H * LCDController::DOTS_PER_LINE as usize) {
            assert!(!c.in_vblank());
            c.tick(1).unwrap();
        }
        assert!(c.in_vblank());
    }

    #[test]
    fn int_stat_lyc() {
        let mut c = LCDController::new(Box::new(NullDisplay::new()), false);
        c.write_io(0xFF45, 10);
        c.write_io(0xFF41, LCDS_INT_LYC);

        c.tick(1).unwrap();
        assert!(!c.get_clr_intreq_stat());
        assert!(c.read_io(0xFF41) & LCDS_LYC != LCDS_LYC);

        while c.ly != 10 {
            c.tick(1).unwrap();
        }
        assert!(c.read_io(0xFF41) & LCDS_LYC == LCDS_LYC);
        assert!(c.get_clr_intreq_stat());
        assert!(!c.get_clr_intreq_stat());

        c.tick(1).unwrap();
        assert!(c.read_io(0xFF41) & LCDS_LYC == LCDS_LYC);
        assert!(!c.get_clr_intreq_stat());
    }

    #[test]
    fn int_stat_vblank() {
        let mut c = LCDController::new(Box::new(NullDisplay::new()), false);
        c.write_io(0xFF41, LCDS_INT_STAT_VBLANK);

        c.tick(1).unwrap();
        assert!(!c.get_clr_intreq_stat());

        while !c.in_vblank() {
            c.tick(1).unwrap();
        }
        assert!(c.get_clr_intreq_stat());
        assert!(!c.get_clr_intreq_stat());

        c.tick(1).unwrap();
        assert!(!c.get_clr_intreq_stat());
    }

    #[test]
    fn int_stat_hblank() {
        let mut c = LCDController::new(Box::new(NullDisplay::new()), false);
        c.write_io(0xFF41, LCDS_INT_STAT_HBLANK);

        c.tick(1).unwrap();
        assert!(!c.get_clr_intreq_stat());

        while c.get_stat_mode() != LCDStatMode::HBlank {
            c.tick(1).unwrap();
        }
        assert!(c.get_clr_intreq_stat());
        assert!(!c.get_clr_intreq_stat());

        c.tick(1).unwrap();
        assert!(!c.get_clr_intreq_stat());
    }

    #[test]
    fn int_stat_oam() {
        let mut c = LCDController::new(Box::new(NullDisplay::new()), false);

        while c.get_stat_mode() == LCDStatMode::Search {
            c.tick(1).unwrap();
        }

        c.write_io(0xFF41, LCDS_INT_STAT_OAM);

        c.tick(1).unwrap();
        assert!(!c.get_clr_intreq_stat());

        while c.get_stat_mode() != LCDStatMode::Search {
            c.tick(1).unwrap();
        }
        assert!(c.get_clr_intreq_stat());
        assert!(!c.get_clr_intreq_stat());

        c.tick(1).unwrap();
        assert!(!c.get_clr_intreq_stat());
    }

    #[test]
    fn int_vblank() {
        let mut c = LCDController::new(Box::new(NullDisplay::new()), false);

        c.tick(1).unwrap();
        assert!(!c.get_clr_intreq_vblank());

        while !c.in_vblank() {
            c.tick(1).unwrap();
        }
        assert!(c.get_clr_intreq_vblank());
        assert!(!c.get_clr_intreq_vblank());

        c.tick(1).unwrap();
        assert!(!c.get_clr_intreq_vblank());
    }

    #[test]
    fn int_vblank_lcdc_disable() {
        let mut c = LCDController::new(Box::new(NullDisplay::new()), false);

        c.write_io(0xFF40, LCDC_ENABLE);
        c.tick(1).unwrap();
        assert!(!c.get_clr_intreq_vblank());

        for _ in 0..(LCDController::DOTS_PER_LINE * LCDController::SCANLINES) {
            c.tick(1).unwrap();
        }
        assert!(c.get_clr_intreq_vblank());
        assert!(!c.get_clr_intreq_vblank());

        c.write_io(0xFF40, 0);
        c.tick(1).unwrap();
        assert!(!c.get_clr_intreq_vblank());

        for _ in 0..(LCDController::DOTS_PER_LINE * LCDController::SCANLINES) {
            c.tick(1).unwrap();
        }
        assert!(!c.get_clr_intreq_vblank());
    }
}
