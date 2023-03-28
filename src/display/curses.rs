use super::display::Display;

use pancurses;

const PX_BOT: &'static str = "▄";
const PX_TOP: &'static str = "▀";
const PX_BOTH: &'static str = "█";
const PX_NONE: &'static str = " ";

pub struct CursesDisplay {
    width: usize,
    height: usize,
    buffer: Vec<Vec<u8>>,
    window: pancurses::Window,
}

const DISP_DIRTY: u8 = 1 << 7;

impl CursesDisplay {
    pub fn new(width: usize, height: usize) -> Self {
        let mut vs: Vec<Vec<u8>> = Vec::with_capacity(height);
        for _ in 0..height {
            let mut vline = Vec::<u8>::with_capacity(width);
            for _ in 0..width {
                vline.push(0);
            }
            vs.push(vline);
        }

        let mut win = pancurses::initscr();
        win.resize(height as i32 / 2, width as i32);

        Self {
            width,
            height,
            buffer: vs,
            window: win,
        }
    }

    #[inline(always)]
    fn ch(&self, x: usize, y: usize) -> &'static str {
        let y1: u8 = self.buffer[y][x] & !DISP_DIRTY;
        let y2: u8 = self.buffer[y + 1][x] & !DISP_DIRTY;

        if y1 > 0 && y2 > 0 {
            PX_BOTH
        } else if y1 > 0 {
            PX_TOP
        } else if y2 > 0 {
            PX_BOT
        } else {
            PX_NONE
        }
    }

    fn render_partial(&mut self) {
        for y in (0..self.height).step_by(2) {
            for x in 0..self.width {
                if (self.buffer[y][x] | self.buffer[y + 1][x]) & DISP_DIRTY == DISP_DIRTY {
                    self.window.mvaddstr(y as i32 / 2, x as i32, self.ch(x, y));
                    self.window.delch();
                    self.buffer[y][x] &= !DISP_DIRTY;
                    self.buffer[y + 1][x] &= !DISP_DIRTY;
                }
            }
        }
        self.window.refresh();
    }

    #[allow(dead_code)]
    fn render_full(&self) {
        self.window.clear();

        for y in (0..self.height).step_by(2) {
            self.window.mv((y / 2) as i32, 0);
            for x in 0..self.width {
                self.window.addstr(self.ch(x, y));
            }
        }
        self.window.refresh();
    }
}

impl Display for CursesDisplay {
    fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        assert!(x < self.width);
        assert!(y < self.height);

        self.buffer[y][x] = DISP_DIRTY | color;
    }

    fn clear(&mut self) {}

    fn render(&mut self) {
        self.render_partial();
    }
}
