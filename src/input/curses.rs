use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::input::{Button, Input};

use pancurses;
use strum::IntoEnumIterator;

pub struct CursesInput {
    window: Rc<pancurses::Window>,
    press_time: RefCell<BTreeMap<Button, Instant>>,
}

impl CursesInput {
    /// Time an input remains asserted after a key press
    const KEYDOWN_TIME: u128 = 200;

    pub fn new(window: Rc<pancurses::Window>) -> Self {
        pancurses::cbreak();
        pancurses::noecho();
        window.nodelay(true);
        window.keypad(true);

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

    fn map_key(k: pancurses::Input) -> Option<Button> {
        match k {
            pancurses::Input::KeyUp => Some(Button::DPadUp),
            pancurses::Input::KeyDown => Some(Button::DPadDown),
            pancurses::Input::KeyLeft => Some(Button::DPadLeft),
            pancurses::Input::KeyRight => Some(Button::DPadRight),
            pancurses::Input::Character('l') => Some(Button::A),
            pancurses::Input::Character('m') => Some(Button::B),
            pancurses::Input::Character('.') => Some(Button::Select),
            pancurses::Input::Character(' ') => Some(Button::Start),

            _ => None,
        }
    }

    fn process_input(&self) {
        while let Some(ch) = self.window.getch() {
            if let Some(btn) = Self::map_key(ch) {
                self.kick_btn(btn);
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
