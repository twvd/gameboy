pub trait Bus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);

    fn write_slice(&mut self, from: &[u8], offset: u16) {
        for (i, b) in from.into_iter().enumerate() {
            self.write(offset.wrapping_add(i as u16), *b);
        }
    }

    /// Write 16-bits to addr and addr + 1,
    /// in little endian.
    fn write16(&mut self, addr: u16, val: u16) {
        self.write_slice(&u16::to_le_bytes(val), addr);
    }

    /// Read 16-bits from addr and addr + 1,
    /// from little endian.
    fn read16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read(addr), self.read(addr.wrapping_add(1))])
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

#[cfg(test)]
mod tests {
    use super::super::testbus::Testbus;
    use super::*;

    fn testbus() -> Testbus {
        let mut b = Testbus::new();
        for a in 0..=u16::MAX {
            b.write(a, a as u8);
        }
        b
    }

    #[test]
    fn busiterator_new() {
        let b: Box<dyn Bus> = Box::new(testbus());
        let mut i = BusIterator::new(&b);

        for a in 0..=u16::MAX {
            assert_eq!(i.next(), Some(a as u8));
        }
        assert_eq!(i.next(), None);
    }

    #[test]
    fn busiterator_new_from() {
        let b: Box<dyn Bus> = Box::new(testbus());
        let mut i = BusIterator::new_from(&b, 5);

        for a in 5..=u16::MAX {
            assert_eq!(i.next(), Some(a as u8));
        }
        assert_eq!(i.next(), None);
    }

    #[test]
    fn bus_write16() {
        let mut b: Box<dyn Bus> = Box::new(testbus());
        b.write16(0x1000, 0x55AA);
        assert_eq!(b.read(0x1000), 0xAA);
        assert_eq!(b.read(0x1001), 0x55);
    }

    #[test]
    fn bus_read16() {
        let mut b: Box<dyn Bus> = Box::new(testbus());
        b.write(0x1000, 0xAA);
        b.write(0x1001, 0x55);
        assert_eq!(b.read16(0x1000), 0x55AA);
    }
}
