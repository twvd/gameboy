use super::super::display::display::Display;

pub const LCD_W: usize = 160;
pub const LCD_H: usize = 144;

const OAM_SIZE: usize = 0x9F;
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

pub struct LCDController {
    output: Box<dyn Display>,
    oam: [u8; OAM_SIZE],
    vram: [u8; VRAM_SIZE],

    lcdc: u8,
    scy: u8,
    scx: u8,
}

impl LCDController {
    pub fn new(display: Box<dyn Display>) -> Self {
        Self {
            output: display,
            oam: [0; OAM_SIZE],
            vram: [0; VRAM_SIZE],

            lcdc: 0,
            scy: 0,
            scx: 0,
        }
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
                self.redraw()
            }

            // SCX - Background scrolling viewport X
            0xFF43 => self.scx = val,

            _ => println!("Write to unknown LCD address: {:04X}", addr),
        }
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
            0xFF44 => 0x90,
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

    fn draw_tile_at(&mut self, tile: &[u8], x: usize, y: usize) {
        for tx in 0..TILE_W {
            for ty in 0..TILE_H {
                let color = Self::tile_decode(&tile, tx, ty);
                let disp_x = x + tx;
                let disp_y = y + ty;

                if disp_x >= LCD_W || disp_y >= LCD_H {
                    continue;
                }
                self.output.set_pixel(disp_x, disp_y, color);
            }
        }
    }

    pub fn redraw(&mut self) {
        self.output.clear();

        // Background
        for x in 0..32 {
            for y in 0..32 {
                let tile = self.get_bg_tile(x, y).to_owned();
                self.draw_tile_at(&tile, x * TILE_W, y * TILE_H);
            }
        }

        self.output.render();

        std::thread::sleep(std::time::Duration::from_millis(100));
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
}
