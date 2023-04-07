pub enum Button {
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    A,
    B,
    Start,
    Select,
}

pub trait Input {
    fn is_pressed(&self, b: Button) -> bool;
}

pub struct NullInput {}

impl NullInput {
    pub fn new() -> Self {
        Self {}
    }
}

impl Input for NullInput {
    fn is_pressed(&self, _b: Button) -> bool {
        false
    }
}
