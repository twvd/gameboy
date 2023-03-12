use super::bus::bus::Bus;

/// Multiplexer for the I/O address segment
pub struct IOMux {}

impl Bus for IOMux {
    fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;

        match addr {
            // LY - LCD Y position register
            0xFF44 => 0x90,

            // Remaining I/O space
            0xFF00..=0xFF70 => 0,
            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        let addr = addr as usize;

        match addr {
            0xFF00..=0xFF70 => {}
            _ => unreachable!(),
        }
    }
}
