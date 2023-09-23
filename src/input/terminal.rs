use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::input::{Button, Input};

use strum::IntoEnumIterator;
use terminal::{KeyCode, KeyEvent};

pub struct TerminalInput {
    receiver: mpsc::Receiver<KeyEvent>,
    press_time: RefCell<BTreeMap<Button, Instant>>,
}

impl TerminalInput {
    /// Time an input remains asserted after a key press
    const KEYDOWN_TIME: u128 = 200;

    pub fn new(receiver: mpsc::Receiver<KeyEvent>) -> Self {
        Self {
            receiver,
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

    fn map_key(k: KeyCode) -> Option<Button> {
        match k {
            KeyCode::Up => Some(Button::DPadUp),
            KeyCode::Down => Some(Button::DPadDown),
            KeyCode::Left => Some(Button::DPadLeft),
            KeyCode::Right => Some(Button::DPadRight),

            KeyCode::Char('l') => Some(Button::A),
            KeyCode::Char('m') => Some(Button::B),
            KeyCode::Char('.') => Some(Button::Select),
            KeyCode::Char(' ') => Some(Button::Start),

            _ => None,
        }
    }

    fn process_input(&self) {
        if let Ok(keyevent) = self.receiver.try_recv() {
            if let Some(btn) = Self::map_key(keyevent.code) {
                self.kick_btn(btn);
            }
        }
    }
}

impl Input for TerminalInput {
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
