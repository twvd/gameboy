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
const VRAM_BANKS: usize = 2;

// Tile sizes
const TILE_BSIZE: usize = 16;
const TILE_W: isize = 8;
const TILE_H: isize = 8;

// Background/window size (in tiles)
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
const OAM_PALETTE_DMG_MASK: u8 = 1 << 4;
const OAM_PALETTE_DMG_SHIFT: u8 = 4;
const OAM_PALETTE_CGB_MASK: u8 = 0x07;
const OAM_PALETTE_CGB_SHIFT: u8 = 0;

// Gameboy Color register properties
const CRAM_ENTRIES: usize = 0x20;
const XCPS_ADDR_MASK: u8 = 0x3F;
const XCPS_AUTO_INC: u8 = 1 << 7;
const COLOR_MASK: Color = 0x7FFF;
const CGB_PALETTE_SIZE: usize = 4;

// BG map attributes (VRAM bank 1, CGB only)
const BGMAP_ATTR_PALETTE_MASK: u8 = 0x07;
const BGMAP_ATTR_PALETTE_SHIFT: u8 = 0;

/// Generic of the DMG and CGB palette types
#[derive(Copy, Clone)]
enum Palette {
    DMG(u8),
    CGB([Color; CGB_PALETTE_SIZE]),
}

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
    vram: [u8; VRAM_SIZE * VRAM_BANKS],

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

    /// Background/window palette (DMG)
    bgp: u8,

    /// Object Palettes (DMG)
    obp: [u8; 2],

    /// Background CRAM (CGB)
    cram_bg: [Color; CRAM_ENTRIES],

    /// Object CRAM (CGB)
    cram_obj: [Color; CRAM_ENTRIES],

    /// Background Color Palette Specification (BCPS) (CGB)
    /// Bit 7: auto increment
    /// Bit 5-0: address (XCPS_ADDR_MASK)
    bcps: u8,

    /// Object Color Palette Specification (OCPS)(CGB)
    /// Bit 7: auto increment
    /// Bit 5-0: address (XCPS_ADDR_MASK)
    ocps: u8,

    /// VRAM bank select
    vbk: u8,

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
            vram: [0; VRAM_SIZE * VRAM_BANKS],

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
            bcps: 0,
            ocps: 0,
            vbk: 0,
            cram_bg: [0x7F; CRAM_ENTRIES],
            cram_obj: [0; CRAM_ENTRIES],

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
    fn get_tile_attr(&self, tm_x: isize, tm_y: isize, selbit: u8) -> u8 {
        // VRAM offset = 8000 - 9FFF
        // BG tile map at 9800 - 9BFF or 9C00 - 9FFF
        // Window tile map at 9800 - 9BFF or 9C00 - 9FFF
        let offset: isize = if self.lcdc & selbit == selbit {
            0x9C00
        } else {
            0x9800
        };

        assert!(tm_x < BGW_W && tm_y < BGW_H);

        self.vram[VRAM_SIZE + (offset - 0x8000 + (tm_y * BGW_H) + tm_x) as usize]
    }

    #[inline(always)]
    fn get_bgw_tile(&self, tm_x: isize, tm_y: isize, selbit: u8) -> (&[u8], u8) {
        // VRAM offset = 8000 - 9FFF
        // BG/Win tile data at 8800 - 97FF and 8000 - 8FFF
        // BG/Win tiles always 8 x 8 pixels
        let tile_id = self.get_tile_id(tm_x, tm_y, selbit) as usize;
        let tile_attr = self.get_tile_attr(tm_x, tm_y, selbit);
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

        (&self.vram[tile_addr..tile_addr + TILE_BSIZE], tile_attr)
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

    fn write_xcpd(cram: &mut [Color], xcps: &mut u8, val: u8) {
        let addr: usize = (*xcps & XCPS_ADDR_MASK) as usize;
        let entry: usize = addr >> 1;
        let new_val: Color = if addr & 1 == 0 {
            // Write LSB
            cram[entry] & 0xFF00 | val as Color
        } else {
            // Write MSB
            cram[entry] & 0x00FF | ((val as Color) << 8)
        } & COLOR_MASK;
        cram[entry] = new_val;

        // Handle auto-increment
        if *xcps & XCPS_AUTO_INC == XCPS_AUTO_INC {
            let new_addr = (*xcps + 1) & XCPS_ADDR_MASK;
            *xcps = new_addr | XCPS_AUTO_INC;
        }
    }

    fn read_xcpd(cram: &[Color], xcps: &u8) -> u8 {
        let addr: usize = (*xcps & XCPS_ADDR_MASK) as usize;
        let entry: usize = addr >> 1;
        if addr & 1 == 0 {
            // Read LSB
            (cram[entry] & 0xFF) as u8
        } else {
            // Read MSB
            (cram[entry] >> 8) as u8
        }

        // Auto-increment has no effect on reads
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

            // OBPx - Object Palette
            0xFF48 => self.obp[0] = val,
            0xFF49 => self.obp[1] = val,

            // WY - Window Y register
            0xFF4A => self.wy = val,

            // WX - Window X register
            0xFF4B => self.wx = val,

            // VBK - VRAM bank select (CGB)
            0xFF4F => self.vbk = val & 1,

            // BCPS - Background Color Palette Specification
            0xFF68 => self.bcps = (val & XCPS_ADDR_MASK) | (val & XCPS_AUTO_INC),

            // BCPD - Background Color Palette Data
            0xFF69 => Self::write_xcpd(&mut self.cram_bg, &mut self.bcps, val),

            // OCPS - Object Color Palette Specification
            0xFF6A => self.ocps = (val & XCPS_ADDR_MASK) | (val & XCPS_AUTO_INC),

            // OCPD - Background Color Palette Data
            0xFF6B => Self::write_xcpd(&mut self.cram_obj, &mut self.ocps, val),

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

            // BGP - Background and window palette
            0xFF47 => self.bgp,

            // OBPx - Object palette
            0xFF48 => self.obp[0],
            0xFF49 => self.obp[1],

            // WY - Window Y register
            0xFF4A => self.wy,

            // WX - Window X register
            0xFF4B => self.wx,

            // VBK - VRAM bank select (CGB)
            0xFF4F => self.vbk,

            // BCPS - Background Color Palette Specification
            0xFF68 => self.bcps,

            // BCPD - Background Color Palette Data
            0xFF69 => Self::read_xcpd(&self.cram_bg, &self.bcps),

            // OCPS - Object Color Palette Specification
            0xFF6A => self.ocps,

            // OCPD - Background Color Palette Data
            0xFF6B => Self::read_xcpd(&self.cram_obj, &self.ocps),

            _ => 0,
        }
    }

    pub fn write_vram(&mut self, addr: usize, val: u8) {
        self.vram[addr + (VRAM_SIZE * self.vbk as usize)] = val;
    }

    pub fn write_oam(&mut self, addr: usize, val: u8) {
        self.oam.write(addr, val);
    }

    pub fn read_vram(&self, addr: usize) -> u8 {
        self.vram[addr + (VRAM_SIZE * self.vbk as usize)]
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

    fn palette_convert_cgb(cidx: u8, palette: &[Color]) -> Color {
        palette[cidx as usize]
    }

    fn draw_tile_at(
        &self,
        tile: &[u8],
        line: &mut [Color],
        x: isize,
        y: isize,
        palette: Palette,
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

                let color = match palette {
                    Palette::DMG(p) => Self::palette_convert_dmg(color_idx, p),
                    Palette::CGB(p) => Self::palette_convert_cgb(color_idx, &p),
                };

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
                let (p_tile, attr) = self.get_bgw_tile(t_x, t_y, LCDC_BG_TILEMAP).to_owned();
                let tile = p_tile.to_owned();

                let palette = if !self.cgb {
                    Palette::DMG(self.bgp)
                } else {
                    let palidx =
                        ((attr & BGMAP_ATTR_PALETTE_MASK) >> BGMAP_ATTR_PALETTE_SHIFT) as usize;
                    Palette::CGB(
                        self.cram_bg
                            [(palidx * CGB_PALETTE_SIZE)..((palidx + 1) * CGB_PALETTE_SIZE)]
                            .try_into()
                            .unwrap(),
                    )
                };

                self.draw_tile_at(
                    &tile,
                    &mut line,
                    (t_x as isize * TILE_W) - self.scx as isize,
                    (t_y as isize * TILE_H) - self.scy as isize,
                    palette,
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
                let (p_tile, attr) = self.get_bgw_tile(t_x, t_y, LCDC_WINDOW_TILEMAP).to_owned();
                let tile = p_tile.to_owned();

                let palette = if !self.cgb {
                    Palette::DMG(self.bgp)
                } else {
                    let palidx =
                        ((attr & BGMAP_ATTR_PALETTE_MASK) >> BGMAP_ATTR_PALETTE_SHIFT) as usize;
                    Palette::CGB(
                        self.cram_bg
                            [(palidx * CGB_PALETTE_SIZE)..((palidx + 1) * CGB_PALETTE_SIZE)]
                            .try_into()
                            .unwrap(),
                    )
                };

                self.draw_tile_at(
                    &tile,
                    &mut line,
                    (t_x as isize * TILE_W) + self.wx as isize - 7,
                    (t_y as isize * TILE_H) + (scanline - self.wly as isize),
                    palette,
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

                let palette = if !self.cgb {
                    Palette::DMG(
                        self.obp
                            [((e.flags & OAM_PALETTE_DMG_MASK) >> OAM_PALETTE_DMG_SHIFT) as usize],
                    )
                } else {
                    let palidx =
                        ((e.flags & OAM_PALETTE_CGB_MASK) >> OAM_PALETTE_CGB_SHIFT) as usize;
                    Palette::CGB(
                        self.cram_obj
                            [(palidx * CGB_PALETTE_SIZE)..((palidx + 1) * CGB_PALETTE_SIZE)]
                            .try_into()
                            .unwrap(),
                    )
                };

                self.draw_tile_at(
                    &sprite,
                    &mut line,
                    disp_x,
                    disp_y,
                    palette,
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
                        palette,
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

    fn test_cram(xcps_addr: u16, xcpd_addr: u16, lcd: &mut LCDController) {
        macro_rules! read_cram {
            ($entry:expr) => {
                match xcps_addr {
                    0xFF68 => lcd.cram_bg[$entry],
                    0xFF6A => lcd.cram_obj[$entry],
                    _ => unreachable!(),
                }
            };
        }

        // Test writes, byte order
        lcd.write_io(xcps_addr, 0);
        lcd.write_io(xcpd_addr, 0xBB);
        lcd.write_io(xcps_addr, 1);
        lcd.write_io(xcpd_addr, 0x7A);
        assert_eq!(read_cram!(0), 0x7ABB);

        // Test auto increment
        lcd.write_io(xcps_addr, 0x20 | XCPS_AUTO_INC);
        assert_eq!(lcd.read_io(xcps_addr), 0x20 | XCPS_AUTO_INC);
        lcd.write_io(xcpd_addr, 0x04);
        assert_eq!(lcd.read_io(xcps_addr), 0x21 | XCPS_AUTO_INC);
        lcd.write_io(xcpd_addr, 0x03);
        assert_eq!(read_cram!(0x10), 0x0304);
        assert_eq!(lcd.read_io(xcps_addr), 0x22 | XCPS_AUTO_INC);

        // Test auto increment overflow
        lcd.write_io(xcps_addr, 0x3F | XCPS_AUTO_INC);
        assert_eq!(lcd.read_io(xcps_addr), 0x3F | XCPS_AUTO_INC);
        lcd.write_io(xcpd_addr, 0x55);
        assert_eq!(lcd.read_io(xcps_addr), 0x00 | XCPS_AUTO_INC);
        lcd.write_io(xcpd_addr, 0x66);
        assert_eq!(read_cram!(0x3F >> 1) & 0xFF00, 0x5500);
        assert_eq!(read_cram!(0x00) & 0x00FF, 0x0066);

        // Test address mask
        lcd.write_io(xcps_addr, 0x40 | XCPS_AUTO_INC);
        assert_eq!(lcd.read_io(xcps_addr), 0x00 | XCPS_AUTO_INC);
        lcd.write_io(xcpd_addr, 0x0B);
        lcd.write_io(xcpd_addr, 0x0A);
        assert_eq!(read_cram!(0), 0x0A0B);
    }

    #[test]
    fn cram_bg() {
        let mut c = LCDController::new(Box::new(NullDisplay::new()), false);
        test_cram(0xFF68, 0xFF69, &mut c);
    }

    #[test]
    fn cram_obj() {
        let mut c = LCDController::new(Box::new(NullDisplay::new()), false);
        test_cram(0xFF6A, 0xFF6B, &mut c);
    }

    #[test]
    fn vram_bank_switching() {
        let mut c = LCDController::new(Box::new(NullDisplay::new()), false);

        c.write_vram(0, 0xAA);
        assert_eq!(c.vram[0], 0xAA);
        assert_eq!(c.read_vram(0), 0xAA);
        assert_ne!(c.vram[VRAM_SIZE], 0xAA);
        c.write_io(0xFF4F, 1);
        assert_ne!(c.read_vram(0), 0xAA);
        c.write_vram(0, 0xBB);
        assert_eq!(c.vram[0], 0xAA);
        assert_eq!(c.read_vram(0), 0xBB);
        assert_eq!(c.vram[VRAM_SIZE], 0xBB);
    }
}
