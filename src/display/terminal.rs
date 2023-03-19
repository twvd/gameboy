use super::display::Display;

const PX_BOT: char = '▄';
const PX_TOP: char = '▀';
const PX_BOTH: char = '█';
const PX_NONE: char = ' ';

pub struct TermDisplay {
    width: usize,
    height: usize,
    buffer: Vec<Vec<u8>>,
}

impl TermDisplay {
    pub fn new(width: usize, height: usize) -> Self {
        let mut vs: Vec<Vec<u8>> = Vec::with_capacity(height);
        for _ in 0..height {
            let mut vline = Vec::<u8>::with_capacity(width);
            for _ in 0..width {
                vline.push(0);
            }
            vs.push(vline);
        }

        TermDisplay {
            width,
            height,
            buffer: vs,
        }
    }

    #[inline(always)]
    fn ch(&self, x: usize, y: usize) -> char {
        let y1: u8 = self.buffer[y][x];
        let y2: u8 = self.buffer[y + 1][x];

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
}

impl Display for TermDisplay {
    fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        assert!(x < self.width);
        assert!(y < self.height);

        self.buffer[y][x] = color;
    }

    fn clear(&mut self) {}

    fn render(&self) {
        let mut s = String::from("\x1B[2J\x1B[1;1H");

        for y in (0..self.height).step_by(2) {
            for x in 0..self.width {
                s.push(self.ch(x, y));
            }
            s.push('\n');
        }
        print!("{}", s);
    }
}
