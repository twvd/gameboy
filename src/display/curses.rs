use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::display::{Color, Display};
use crate::input::curses::CursesInput;

use itertools::Itertools;
use pancurses;
use pancurses::COLOR_PAIR;

const PX_BOT: &'static str = "▄";
//const PX_TOP: &'static str = "▀";
const PX_BOTH: &'static str = "█";
const PX_NONE: &'static str = " ";

pub struct CursesDisplay {
    width: usize,
    height: usize,
    buffer: Vec<Vec<Color>>,
    window: Rc<pancurses::Window>,
    updates: usize,
    last_frame: Instant,
    frametime: u64,
}

/// Flag to mark a pixel for redrawing
const DISP_DIRTY: u32 = 1 << 31;

const COLORS: [i16; 4] = [
    pancurses::COLOR_WHITE,
    pancurses::COLOR_CYAN,
    pancurses::COLOR_BLUE,
    pancurses::COLOR_BLACK,
];

impl CursesDisplay {
    pub fn new(width: usize, height: usize, fps: u64) -> Self {
        let mut vs: Vec<Vec<Color>> = Vec::with_capacity(height);
        for _ in 0..height {
            let mut vline = Vec::<Color>::with_capacity(width);
            for _ in 0..width {
                vline.push(0 | DISP_DIRTY);
            }
            vs.push(vline);
        }

        let mut win = pancurses::initscr();
        win.resize(height as i32 / 2, width as i32);
        pancurses::curs_set(0);

        pancurses::start_color();

        for v in [0, 0, 1, 1, 2, 2, 3, 3].into_iter().permutations(2) {
            let (a, b) = (v[0], v[1]);
            let pair: i16 = (a << 4) | b;
            assert!((pair as i32) < pancurses::COLOR_PAIRS());
            pancurses::init_pair(pair, COLORS[a as usize], COLORS[b as usize]);
        }

        Self {
            width,
            height,
            buffer: vs,
            window: Rc::new(win),
            updates: 0,
            last_frame: Instant::now(),
            frametime: (1000000 / fps),
        }
    }

    pub fn create_input(&self) -> CursesInput {
        CursesInput::new(self.window.clone())
    }

    fn render_partial(&mut self, full: bool) {
        for y in (0..self.height).step_by(2) {
            for x in 0..self.width {
                if (self.buffer[y][x] | self.buffer[y + 1][x]) & DISP_DIRTY == DISP_DIRTY || full {
                    let y1: u32 = self.buffer[y][x] as u32 & 3;
                    let y2: u32 = self.buffer[y + 1][x] as u32 & 3;
                    let colors: u32 = (y2 << 4) | y1;
                    let ch = if y1 == 3 && y2 == 3 {
                        // Both black
                        PX_NONE
                    } else if y1 == y2 {
                        // Same color
                        PX_BOTH
                    } else {
                        // Different colors
                        PX_BOT
                    };

                    self.window.attron(COLOR_PAIR(colors));
                    self.window.mvaddstr(y as i32 / 2, x as i32, ch);
                    self.window.attroff(COLOR_PAIR(colors));

                    self.buffer[y][x] &= !DISP_DIRTY;
                    self.buffer[y + 1][x] &= !DISP_DIRTY;
                }
            }
        }
        self.window.refresh();
    }
}

impl Display for CursesDisplay {
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        assert!(x < self.width);
        assert!(y < self.height);

        if self.buffer[y][x] & !DISP_DIRTY != color {
            self.buffer[y][x] = DISP_DIRTY | color;
        }
    }

    fn clear(&mut self) {}

    fn render(&mut self) {
        // Full redraw every 60 frames
        self.render_partial(self.updates == 0);
        self.updates = (self.updates + 1) % 60;

        // Limit the framerate
        let framelen = self.last_frame.elapsed().as_micros() as u64;
        if framelen < self.frametime {
            sleep(Duration::from_micros(self.frametime - framelen));
        }
        self.last_frame = Instant::now();
    }
}
