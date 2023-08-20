use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use super::input::{Button, Input};

use ncurses;
use strum::IntoEnumIterator;

pub struct CursesInput {
    window: ncurses::WINDOW,
    press_time: RefCell<BTreeMap<Button, Instant>>,
}

impl CursesInput {
    /// Time an input remains asserted after a key press
    const KEYDOWN_TIME: u128 = 200;

    const KEY_A: i32 = b'l' as i32;
    const KEY_B: i32 = b'm' as i32;
    const KEY_START: i32 = b' ' as i32;
    const KEY_SELECT: i32 = b'.' as i32;

    pub fn new(window: ncurses::WINDOW) -> Self {
        ncurses::cbreak();
        ncurses::noecho();
        ncurses::nodelay(window, true);
        ncurses::keypad(window, true);

        Self {
            window,
            press_time: RefCell::new(BTreeMap::from_iter(Button::iter().map(|e| {
                (
                    e,
                    Instant::now()
                        .checked_sub(Duration::from_millis(Self::KEYDOWN_TIME as u64))
                        .unwrap(),
                )
            }))),
        }
    }

    fn kick_btn(&self, b: Button) {
        self.press_time.borrow_mut().insert(b, Instant::now());
    }

    fn map_key(k: i32) -> Option<Button> {
        match k {
            ncurses::KEY_UP => Some(Button::DPadUp),
            ncurses::KEY_DOWN => Some(Button::DPadDown),
            ncurses::KEY_LEFT => Some(Button::DPadLeft),
            ncurses::KEY_RIGHT => Some(Button::DPadRight),

            Self::KEY_A => Some(Button::A),
            Self::KEY_B => Some(Button::B),
            Self::KEY_SELECT => Some(Button::Select),
            Self::KEY_START => Some(Button::Start),

            _ => None,
        }
    }

    fn process_input(&self) {
        loop {
            match ncurses::wgetch(self.window) {
                ncurses::ERR => break,
                res => {
                    if let Some(btn) = Self::map_key(res) {
                        self.kick_btn(btn);
                    }
                }
            }
        }
    }
}

impl Input for CursesInput {
    fn is_pressed(&self, b: Button) -> bool {
        self.process_input();
        self.press_time
            .borrow()
            .get(&b)
            .unwrap()
            .elapsed()
            .as_millis()
            < Self::KEYDOWN_TIME
    }
}
