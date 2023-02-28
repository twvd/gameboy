pub trait Bus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);

    fn write_slice(&mut self, from: &[u8], offset: usize) {
        for (i, b) in from.into_iter().enumerate() {
            self.write((offset + i) as u16, *b);
        }
    }
}
