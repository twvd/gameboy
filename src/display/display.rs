/// Base trait for a display output
pub trait Display {
    fn set_pixel(&mut self, x: usize, y: usize, color: u8);
    fn clear(&mut self);
    fn render(&self);
}
