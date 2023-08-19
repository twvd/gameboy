use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::display::{Color, Display};

use sixel_rs::encoder::{Encoder, QuickFrameBuilder};
use sixel_rs::sys::PixelFormat;

pub struct SixelDisplay {
    width: usize,
    height: usize,
    depth: usize,
    scale: usize,
    buffer: Vec<u8>,
    encoder: Encoder,
    updates: usize,
    last_frame: Instant,
    frametime: u64,
    stdout: io::Stdout,
}

impl SixelDisplay {
    pub fn new(width: usize, height: usize, fps: u64) -> Self {
        let encoder = Encoder::new().unwrap();

        Self {
            width,
            height,
            depth: 3,
            scale: 4,
            buffer: vec![0; width * height * 3 * 4 * 4],
            encoder,
            updates: 0,
            last_frame: Instant::now(),
            frametime: (1000000 / fps),
            stdout: std::io::stdout(),
        }
    }

    fn move_cursor(&mut self, x: usize, y: usize) {
        let s = format!("\x1b[{};{}H", x + 1, y + 1);
        self.stdout.write_all(s.as_bytes()).unwrap();
    }

    fn clear_screen(&mut self) {
        let s = format!("\x1b[2J");
        self.stdout.write_all(s.as_bytes()).unwrap();
        self.stdout.flush().unwrap();
    }
}

impl Display for SixelDisplay {
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let (r, g, b) = match color {
            3 => (0, 0, 0),
            2 => (90, 90, 90),
            1 => (160, 160, 160),
            0 => (255, 255, 255),
            _ => {
                println!("{}", color);
                unreachable!()
            }
        };

        for px in (x * self.scale)..((x + 1) * self.scale) {
            for py in (y * self.scale)..((y + 1) * self.scale) {
                self.buffer[px * self.depth + py * self.depth * self.width * self.scale + 0] = r;
                self.buffer[px * self.depth + py * self.depth * self.width * self.scale + 1] = g;
                self.buffer[px * self.depth + py * self.depth * self.width * self.scale + 2] = b;
            }
        }
    }

    fn clear(&mut self) {
        self.clear_screen();
    }

    fn render(&mut self) {
        self.updates += 1;

        // 75% frame skip
        if self.updates % 4 == 0 {
            let frame = QuickFrameBuilder::new()
                .width(self.width * self.scale)
                .height(self.height * self.scale)
                .format(PixelFormat::RGB888)
                .pixels(self.buffer.clone());

            self.move_cursor(0, 0);
            self.encoder.encode_bytes(frame).unwrap();
            self.stdout.flush().unwrap();
        }

        // Limit the framerate
        let framelen = self.last_frame.elapsed().as_micros() as u64;
        if framelen < self.frametime {
            sleep(Duration::from_micros(self.frametime - framelen));
        }
        self.last_frame = Instant::now();
    }
}
