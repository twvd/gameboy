use anyhow::Result;

pub const ONE_MCYCLE: usize = 4;

pub trait Tickable {
    fn tick(&mut self, ticks: usize) -> Result<usize>;
}
