use std::fmt;

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

    pub fn hl(&self) -> u16 {
        self.l as u16 | (self.h as u16) << 8
    }

    pub fn set_hl(&mut self, hl: u16) {
        self.l = (hl & 0xFF) as u8;
        self.h = ((hl >> 8) & 0xFF) as u8;
    }
}

impl fmt::Display for RegisterFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SP: {:04X}  PC: {:04X}", self.sp, self.pc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hl() {
        let mut r = RegisterFile::new();

        assert_eq!(r.h, 0);
        assert_eq!(r.l, 0);
        assert_eq!(r.hl(), 0);

        r.l = 0x34;
        assert_eq!(r.hl(), 0x34);

        r.h = 0x12;
        assert_eq!(r.hl(), 0x1234);
    }

    #[test]
    fn set_hl() {
        let mut r = RegisterFile::new();

        assert_eq!(r.h, 0);
        assert_eq!(r.l, 0);
        assert_eq!(r.hl(), 0);

        r.set_hl(0x1234);
        assert_eq!(r.h, 0x12);
        assert_eq!(r.l, 0x34);
        assert_eq!(r.hl(), 0x1234);
    }
}
