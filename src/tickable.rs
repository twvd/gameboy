use anyhow::Result;

pub trait Tickable {
    fn tick(&mut self, ticks: usize) -> Result<usize>;
}
