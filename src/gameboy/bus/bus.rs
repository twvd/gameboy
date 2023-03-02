pub trait Bus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);

    fn write_slice(&mut self, from: &[u8], offset: usize) {
        for (i, b) in from.into_iter().enumerate() {
            self.write((offset + i) as u16, *b);
        }
    }
}

pub struct BusIterator<'a> {
    bus: &'a Box<dyn Bus>,
    next: u16,
    finished: bool,
}

impl<'a> BusIterator<'a> {
    pub fn new_from(bus: &'a Box<dyn Bus>, offset: u16) -> BusIterator {
        BusIterator {
            bus,
            next: offset,
            finished: false,
        }
    }

    pub fn new(bus: &'a Box<dyn Bus>) -> BusIterator {
        Self::new_from(bus, 0)
    }
}

impl<'a> Iterator for BusIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let curr = self.next;
        if self.next == u16::MAX {
            self.finished = true;
        } else {
            self.next += 1;
        }

        Some(self.bus.read(curr))
    }
}
