use std::fmt;

use anyhow::{bail, Result};

/// Datatype of a single CPU register.
type Reg = u8;

/// Bit positions of the flags in the F register.
pub enum Flags {
    /// Zero
    Z = 7,
    /// Subtract
    N = 6,
    /// Half-carry
    H = 5,
    /// Carry
    C = 4,
}

/// Enumeration of registers
#[derive(Debug, Copy, Clone)]
pub enum Register {
    A,
    F,
    B,
    C,
    D,
    E,
    L,
    H,

    AF,
    BC,
    DE,
    HL,

    SP,
    PC,
}

/// Complete CPU register file
pub struct RegisterFile {
    /// A (accumulator) register.
    pub a: Reg,

    /// F (flags) register.
    pub f: Reg,

    pub b: Reg,
    pub c: Reg,
    pub d: Reg,
    pub e: Reg,

    /// H register, high byte of the 16-bit HL register.
    pub h: Reg,
    /// L register, low byte of the 16-bit HL register.
    pub l: Reg,

    /// Stack Pointer (SP)
    pub sp: u16,

    /// Program Counter (PC)
    pub pc: u16,
}

impl RegisterFile {
    pub fn new() -> Self {
        Self {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    /// Write a value to a register.
    /// Returns an error when attempting to write
    /// a 16-bit value to an 8-bit register.
    pub fn write(&mut self, reg: Register, val: u16) -> Result<()> {
        let reg8 = || {
            if val > u8::MAX.into() {
                bail!("Cannot write {} to 8-bit register {:?}", val, reg)
            } else {
                Ok(val as u8)
            }
        };
        let reg16 = || -> Result<(u8, u8)> {
            // MSB, LSB
            Ok(((val >> 8) as u8, (val & 0xFF) as u8))
        };

        match reg {
            // 8-bit single registers
            Register::A => self.a = reg8()?,
            Register::B => self.b = reg8()?,
            Register::C => self.c = reg8()?,
            Register::D => self.d = reg8()?,
            Register::E => self.e = reg8()?,
            Register::F => self.f = reg8()?,
            Register::H => self.h = reg8()?,
            Register::L => self.l = reg8()?,

            // 16-bit combination registers
            Register::AF => (self.a, self.f) = reg16()?,
            Register::BC => (self.b, self.c) = reg16()?,
            Register::DE => (self.d, self.e) = reg16()?,
            Register::HL => (self.h, self.l) = reg16()?,

            // 16-bit-only registers
            Register::SP => self.sp = val,
            Register::PC => self.pc = val,
        }

        Ok(())
    }

    fn read(&self, reg: Register) -> u16 {
        let reg16 = |msb: u8, lsb: u8| (msb as u16) << 8 | lsb as u16;

        match reg {
            // 8-bit single registers
            Register::A => self.a as u16,
            Register::B => self.b as u16,
            Register::C => self.c as u16,
            Register::D => self.d as u16,
            Register::E => self.e as u16,
            Register::F => self.f as u16,
            Register::H => self.h as u16,
            Register::L => self.l as u16,

            // 16-bit combination registers
            Register::AF => reg16(self.a, self.f),
            Register::BC => reg16(self.b, self.c),
            Register::DE => reg16(self.d, self.e),
            Register::HL => reg16(self.h, self.l),

            // 16-bit-only registers
            Register::SP => self.sp,
            Register::PC => self.pc,
        }
    }

    pub fn read8(&self, reg: Register) -> Result<u8> {
        match reg {
            Register::A
            | Register::B
            | Register::C
            | Register::D
            | Register::E
            | Register::F
            | Register::H
            | Register::L => Ok(self.read(reg) as u8),
            _ => bail!("Attempting 8-bit read on 16-bit register {:?}", reg),
        }
    }

    pub fn read16(&self, reg: Register) -> Result<u16> {
        match reg {
            Register::AF
            | Register::BC
            | Register::DE
            | Register::HL
            | Register::PC
            | Register::SP => Ok(self.read(reg)),
            _ => bail!("Attempting 16-bit read on 8-bit register {:?}", reg),
        }
    }
}

impl fmt::Display for RegisterFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "A: {:02X}  F: {:02X}", self.a, self.f)?;
        writeln!(
            f,
            "B: {:02X}  C: {:02X} D: {:02X} E: {:02X}",
            self.b, self.c, self.d, self.e
        )?;
        write!(
            f,
            "HL: {:04X}     SP: {:04X}  PC: {:04X}",
            self.read16(Register::HL).or(Err(fmt::Error))?,
            self.sp,
            self.pc
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_8bit() {
        let mut r = RegisterFile::new();
        r.write(Register::A, 0x12).unwrap();
        assert_eq!(r.a, 0x12);

        let mut r = RegisterFile::new();
        r.write(Register::B, 0x12).unwrap();
        assert_eq!(r.b, 0x12);

        let mut r = RegisterFile::new();
        r.write(Register::C, 0x12).unwrap();
        assert_eq!(r.c, 0x12);

        let mut r = RegisterFile::new();
        r.write(Register::D, 0x12).unwrap();
        assert_eq!(r.d, 0x12);

        let mut r = RegisterFile::new();
        r.write(Register::E, 0x12).unwrap();
        assert_eq!(r.e, 0x12);

        let mut r = RegisterFile::new();
        r.write(Register::F, 0x12).unwrap();
        assert_eq!(r.f, 0x12);

        let mut r = RegisterFile::new();
        r.write(Register::H, 0x12).unwrap();
        assert_eq!(r.h, 0x12);

        let mut r = RegisterFile::new();
        r.write(Register::L, 0x12).unwrap();
        assert_eq!(r.l, 0x12);
    }

    #[test]
    fn write_8bit_error() {
        let mut r = RegisterFile::new();
        assert!(matches!(r.write(Register::A, 0xFF), Ok(_)));
        assert_eq!(r.a, 0xFF);

        let mut r = RegisterFile::new();
        assert!(matches!(r.write(Register::A, 0x1FF), Err(_)));
        assert_eq!(r.a, 0);
    }

    #[test]
    fn write_comb16bit() {
        let mut r = RegisterFile::new();
        r.write(Register::AF, 0x1234).unwrap();
        assert_eq!((r.a, r.f), (0x12, 0x34));

        let mut r = RegisterFile::new();
        r.write(Register::BC, 0x1234).unwrap();
        assert_eq!((r.b, r.c), (0x12, 0x34));

        let mut r = RegisterFile::new();
        r.write(Register::DE, 0x1234).unwrap();
        assert_eq!((r.d, r.e), (0x12, 0x34));

        let mut r = RegisterFile::new();
        r.write(Register::HL, 0x1234).unwrap();
        assert_eq!((r.h, r.l), (0x12, 0x34));
    }

    #[test]
    fn write_16bit() {
        let mut r = RegisterFile::new();
        r.write(Register::SP, 0x1234).unwrap();
        assert_eq!(r.sp, 0x1234);

        let mut r = RegisterFile::new();
        r.write(Register::PC, 0x1234).unwrap();
        assert_eq!(r.pc, 0x1234);
    }

    #[test]
    fn read8() {
        let mut r = RegisterFile::new();
        r.a = 0x12;
        assert!(matches!(r.read8(Register::A), Ok(0x12)));

        let mut r = RegisterFile::new();
        r.b = 0x12;
        assert!(matches!(r.read8(Register::B), Ok(0x12)));

        let mut r = RegisterFile::new();
        r.c = 0x12;
        assert!(matches!(r.read8(Register::C), Ok(0x12)));

        let mut r = RegisterFile::new();
        r.d = 0x12;
        assert!(matches!(r.read8(Register::D), Ok(0x12)));

        let mut r = RegisterFile::new();
        r.e = 0x12;
        assert!(matches!(r.read8(Register::E), Ok(0x12)));

        let mut r = RegisterFile::new();
        r.f = 0x12;
        assert!(matches!(r.read8(Register::F), Ok(0x12)));

        let mut r = RegisterFile::new();
        r.h = 0x12;
        assert!(matches!(r.read8(Register::H), Ok(0x12)));

        let mut r = RegisterFile::new();
        r.l = 0x12;
        assert!(matches!(r.read8(Register::L), Ok(0x12)));
    }

    #[test]
    fn read8_error() {
        let r = RegisterFile::new();
        assert!(matches!(r.read8(Register::AF), Err(_)));
    }

    #[test]
    fn read16() {
        let mut r = RegisterFile::new();
        (r.a, r.f) = (0x12, 0x34);
        assert!(matches!(r.read16(Register::AF), Ok(0x1234)));

        let mut r = RegisterFile::new();
        (r.b, r.c) = (0x12, 0x34);
        assert!(matches!(r.read16(Register::BC), Ok(0x1234)));

        let mut r = RegisterFile::new();
        (r.d, r.e) = (0x12, 0x34);
        assert!(matches!(r.read16(Register::DE), Ok(0x1234)));

        let mut r = RegisterFile::new();
        (r.h, r.l) = (0x12, 0x34);
        assert!(matches!(r.read16(Register::HL), Ok(0x1234)));
    }

    #[test]
    fn read16_error() {
        let r = RegisterFile::new();
        assert!(matches!(r.read16(Register::A), Err(_)));
    }
}
