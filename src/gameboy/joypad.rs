use crate::input::input::{Button, Input};

const JOYPAD_UNUSED: u8 = (1 << 7) | (1 << 6);
const JOYPAD_SELECT_MASK: u8 = 0x30;
const JOYPAD_SELECT_ACTION: u8 = 1 << 5;
const JOYPAD_SELECT_DIRECTION: u8 = 1 << 4;
const JOYPAD_IN_DOWN_START: u8 = 1 << 3;
const JOYPAD_IN_UP_SELECT: u8 = 1 << 2;
const JOYPAD_IN_LEFT_B: u8 = 1 << 1;
const JOYPAD_IN_RIGHT_A: u8 = 1 << 0;

pub struct Joypad {
    input: Box<dyn Input>,

    /// Joypad select bits
    select: u8,
}

impl Joypad {
    pub fn new(input: Box<dyn Input>) -> Self {
        Self { input, select: 0 }
    }

    fn read_bit(&self, b: Button, bit: u8) -> u8 {
        if self.input.is_pressed(b) {
            0
        } else {
            bit
        }
    }

    pub fn read(&self) -> u8 {
        JOYPAD_UNUSED
            | self.select
            | match !self.select & JOYPAD_SELECT_MASK {
                JOYPAD_SELECT_ACTION => {
                    self.read_bit(Button::Start, JOYPAD_IN_DOWN_START)
                        | self.read_bit(Button::Select, JOYPAD_IN_UP_SELECT)
                        | self.read_bit(Button::A, JOYPAD_IN_RIGHT_A)
                        | self.read_bit(Button::B, JOYPAD_IN_LEFT_B)
                }
                JOYPAD_SELECT_DIRECTION => {
                    self.read_bit(Button::DPadDown, JOYPAD_IN_DOWN_START)
                        | self.read_bit(Button::DPadUp, JOYPAD_IN_UP_SELECT)
                        | self.read_bit(Button::DPadRight, JOYPAD_IN_RIGHT_A)
                        | self.read_bit(Button::DPadLeft, JOYPAD_IN_LEFT_B)
                }
                _ => 0x0F,
            }
    }

    pub fn write(&mut self, val: u8) {
        self.select = val & JOYPAD_SELECT_MASK;
    }
}
