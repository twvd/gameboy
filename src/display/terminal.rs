use std::thread::sleep;
use std::time::{Duration, Instant};

use super::display::{Color, Display};
use crate::input::terminal::TerminalInput;

use anyhow::Result;
use std::io::{Stdout, Write};
use terminal::{Action, Clear, Color as TerminalColor, Terminal};

const PX_BOT: char = '▄';
//const PX_TOP: char = '▀';
const PX_BOTH: char = '█';
const PX_NONE: char = ' ';

pub struct TerminalDisplay {
    width: usize,
    height: usize,
    buffer: Vec<Vec<Color>>,
    terminal: Terminal<Stdout>,
    updates: usize,
    last_frame: Instant,
    frametime: u64,
}

/// Flag to mark a pixel for redrawing
/// We use bit 15 as RGB555 only uses bits 0 - 14.
const DISP_DIRTY: u16 = 1 << 15;

fn unpack_rgb555(c: Color) -> (u8, u8, u8) {
    (
        ((c >> 10) & 0x1F) as u8,
        ((c >> 5) & 0x1F) as u8,
        (c & 0x1F) as u8,
    )
}

fn rgb555_to_rgb888((r, g, b): (u8, u8, u8)) -> (u8, u8, u8) {
    (
        (r as u16 * 255 / 31) as u8,
        (g as u16 * 255 / 31) as u8,
        (b as u16 * 255 / 31) as u8,
    )
}

fn rgb888_to_ansi((r, g, b): (u8, u8, u8)) -> u8 {
    if r == g && g == b {
        if r < 8 {
            return 0;
        }

        if r > 248 {
            return 231;
        }

        return (((r as u16 - 8) * 24) / 247) as u8 + 232;
    }

    16 + (36 * (r / 255 * 5)) + (6 * (g / 255 * 5)) + (b / 255 * 5)
}

impl TerminalDisplay {
    pub fn new(width: usize, height: usize, fps: u64) -> Self {
        let mut vs: Vec<Vec<Color>> = Vec::with_capacity(height);
        for _ in 0..height {
            let mut vline = Vec::<Color>::with_capacity(width);
            for _ in 0..width {
                vline.push(0 | DISP_DIRTY);
            }
            vs.push(vline);
        }

        let term = terminal::stdout();
        term.act(Action::ResetColor).unwrap();
        term.act(Action::HideCursor).unwrap();
        term.act(Action::DisableBlinking).unwrap();
        term.act(Action::ClearTerminal(Clear::All)).unwrap();
        term.act(Action::MoveCursorTo(0, 0)).unwrap();

        Self {
            width,
            height,
            buffer: vs,
            terminal: term,
            updates: 0,
            last_frame: Instant::now(),
            frametime: (1000000 / fps),
        }
    }

    pub fn create_input(&self) -> TerminalInput {
        TerminalInput::new(terminal::stdout())
    }

    /// Map a color from our internal color type to a terminal color
    fn map_color(&self, c: Color) -> TerminalColor {
        let ansi = rgb888_to_ansi(rgb555_to_rgb888(unpack_rgb555(c)));
        if ansi != 16 {
            //panic!("{}", ansi);
        }
        TerminalColor::AnsiValue(ansi)
    }

    /// Map a pair of two vertically adjacent pixels to a printable
    /// character.
    fn map_ch(&self, y1: Color, y2: Color) -> char {
        if y1 == 3 && y2 == 3 {
            // Both black
            PX_NONE
        } else if y1 == y2 {
            // Same color
            PX_BOTH
        } else {
            // Different colors
            PX_BOT
        }
    }

    /// Render a pair of two vertically adjacent pixels, specifying the cpordinates of the
    /// top pixel.
    fn render_pair(&mut self, x: usize, y: usize, top: Color, bottom: Color) -> Result<()> {
        assert_eq!(y & 1, 0);

        self.terminal
            .batch(Action::MoveCursorTo(x as u16, y as u16 / 2))?;

        let ch = self.map_ch(top, bottom);
        if ch == PX_BOT || ch == PX_BOTH {
            self.terminal
                .batch(Action::SetForegroundColor(self.map_color(bottom)))?;
        }
        if ch == PX_BOT || ch == PX_NONE {
            self.terminal
                .batch(Action::SetBackgroundColor(self.map_color(top)))?;
        }

        write!(self.terminal, "{}", ch)?;

        Ok(())
    }

    /// Render changed pixels since last redraw, or entire frame if
    /// 'full' is set.
    fn render_partial(&mut self, full: bool) -> Result<()> {
        for y in (0..self.height).step_by(2) {
            for x in 0..self.width {
                if (self.buffer[y][x] | self.buffer[y + 1][x]) & DISP_DIRTY == DISP_DIRTY || full {
                    self.buffer[y][x] &= !DISP_DIRTY;
                    self.buffer[y + 1][x] &= !DISP_DIRTY;

                    let y1 = self.buffer[y][x];
                    let y2 = self.buffer[y + 1][x];

                    self.render_pair(x, y, y1, y2)?;
                }
            }
        }
        self.terminal.flush_batch()?;

        Ok(())
    }
}

impl Display for TerminalDisplay {
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        assert!(x < self.width);
        assert!(y < self.height);

        if self.buffer[y][x] & !DISP_DIRTY != color {
            self.buffer[y][x] = DISP_DIRTY | color;
        }
    }

    fn clear(&mut self) {}

    fn render(&mut self) {
        // Full redraw every 300 frames
        self.render_partial(self.updates == 0).unwrap();
        self.updates = (self.updates + 1) % 300;

        // Limit the framerate
        let framelen = self.last_frame.elapsed().as_micros() as u64;
        if framelen < self.frametime {
            sleep(Duration::from_micros(self.frametime - framelen));
        }
        self.last_frame = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb555_to_rgb888() {
        assert_eq!(rgb555_to_rgb888((0, 0, 0)), (0, 0, 0));
        assert_eq!(rgb555_to_rgb888((0x1F, 0x1F, 0x1F)), (255, 255, 255));

        assert_eq!(rgb555_to_rgb888((0b01000, 0b01000, 0b01000)), (65, 65, 65));
        assert_eq!(
            rgb555_to_rgb888((0b10000, 0b10000, 0b10000)),
            (131, 131, 131)
        );
    }

    #[test]
    fn test_unpack_rgb555() {
        assert_eq!(unpack_rgb555(0x7FFF), (0x1F, 0x1F, 0x1F));
        assert_eq!(unpack_rgb555(0), (0, 0, 0));
        assert_eq!(unpack_rgb555(0b01000_01000_01000), (8, 8, 8));
        assert_eq!(unpack_rgb555(0b10000_10000_10000), (16, 16, 16));

        assert_eq!(unpack_rgb555(0b11111_00000_00000), (0x1F, 0, 0));
        assert_eq!(unpack_rgb555(0b00000_11111_00000), (0, 0x1F, 0));
        assert_eq!(unpack_rgb555(0b00000_00000_11111), (0, 0, 0x1F));
    }

    #[test]
    fn test_rgb888_to_ansi() {
        assert_eq!(rgb888_to_ansi((0, 0, 0)), 0);
        assert_eq!(rgb888_to_ansi((65, 65, 65)), 237);
        assert_eq!(rgb888_to_ansi((131, 131, 131)), 243);
        assert_eq!(rgb888_to_ansi((255, 255, 255)), 231);
    }
}
