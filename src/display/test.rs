use super::display::Display;

use sha2::{Digest, Sha256};

use std::cell::Cell;
use std::cmp;
use std::rc::Rc;

/// A display that hashes the contents using SHA256.
pub struct TestDisplay {
    width: usize,
    height: usize,
    buffer: Vec<Vec<u8>>,
    state: TDS,
}

#[derive(Debug, Copy, Clone)]
pub struct TestDisplayState {
    pub stable_frames: u16,
    pub hash: [u8; 256 / 8],
}

pub type TDS = Rc<Cell<TestDisplayState>>;

impl TestDisplay {
    pub fn new(width: usize, height: usize) -> (Box<Self>, TDS) {
        let mut vs: Vec<Vec<u8>> = Vec::with_capacity(height);
        for _ in 0..height {
            let mut vline = Vec::<u8>::with_capacity(width);
            for _ in 0..width {
                vline.push(0);
            }
            vs.push(vline);
        }

        let state = Rc::new(Cell::new(TestDisplayState {
            stable_frames: 0,
            hash: [0; 256 / 8],
        }));

        (
            Box::new(TestDisplay {
                width,
                height,
                buffer: vs,
                state: Rc::clone(&state),
            }),
            state,
        )
    }
}

impl Display for TestDisplay {
    fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        assert!(x < self.width);
        assert!(y < self.height);

        self.buffer[y][x] = color;
    }

    fn clear(&mut self) {}

    fn render(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(
            self.buffer
                .iter()
                .flat_map(|v| v.clone().into_iter())
                .collect::<Vec<u8>>(),
        );
        let hash = hasher.finalize();

        let oldstate = self.state.get();
        let stable_frames = if oldstate.hash == hash[..] {
            cmp::min(oldstate.stable_frames + 1, u16::MAX - 1)
        } else {
            1
        };

        self.state.set(TestDisplayState {
            hash: hash.into(),
            stable_frames,
        });
    }
}
