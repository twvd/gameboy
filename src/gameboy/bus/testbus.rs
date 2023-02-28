use super::bus::Bus;

pub struct Testbus {
    mem: [u8; u16::MAX as usize],
}

impl Testbus {
    fn new() -> Self {
        Testbus {
            mem: [0; u16::MAX as usize],
        }
    }
}

impl Bus for Testbus {
    fn read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, val: u8) {
        self.mem[addr as usize] = val;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testbus() {
        let mut b = Testbus::new();

        for a in 0..u16::MAX {
            assert_eq!(b.read(a), 0);
        }
        for a in 0..u16::MAX {
            b.write(a, a as u8);
        }
        for a in 0..u16::MAX {
            assert_eq!(b.read(a), a as u8);
        }
    }
}
