use std::io;
use std::io::Write;

use crate::gameboy::bus::bus::BusMember;

/// Serial (link cable) controller
pub struct Serial {
    /// Serial data buffer
    serialbuffer: u8,

    /// Serial input stream
    serial_in: Box<dyn io::Read>,

    /// Serial output stream
    serial_out: Box<dyn io::Write>,
}

impl Serial {
    pub fn new(serial_in: Box<dyn io::Read>, serial_out: Box<dyn io::Write>) -> Self {
        Self {
            serial_in,
            serial_out,
            serialbuffer: 0,
        }
    }
}

impl BusMember for Serial {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // I/O - Serial transfer data buffer
            0xFF01 => self.serialbuffer,

            // I/O - Serial transfer control
            0xFF02 => 0x7E,

            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // I/O - Serial transfer data buffer
            0xFF01 => self.serialbuffer = val,

            // I/O - Serial transfer control
            0xFF02 => {
                if val == 0x81 {
                    self.serial_out.write(&[self.serialbuffer]).unwrap();
                }
            }

            _ => unreachable!(),
        }
    }
}
