/// Type of a color definition (RGB555)
pub type Color = u16;

/// Base trait for a display output
pub trait Display: std::any::Any {
    fn set_pixel(&mut self, x: usize, y: usize, color: Color);
    fn clear(&mut self);
    fn render(&mut self);
}

/// A display thst doesn't do anything.
pub struct NullDisplay {}

impl NullDisplay {
    pub fn new() -> Self {
        Self {}
    }
}

impl Display for NullDisplay {
    fn set_pixel(&mut self, _x: usize, _y: usize, _color: Color) {}
    fn clear(&mut self) {}
    fn render(&mut self) {}
}
