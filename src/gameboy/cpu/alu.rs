pub struct ALUResult<T> {
    pub result: T,
    pub halfcarry: bool,
    pub carry: bool,
}

/// 8-bit add
pub fn add_8b(a: u8, b: u8) -> ALUResult<u8> {
    let result: u16 = a as u16 + b as u16;
    ALUResult {
        result: result as u8,
        carry: (result > u8::MAX.into()),
        halfcarry: (((a & 0x0F) + (b & 0x0F)) & 0x10) == 0x10,
    }
}

/// 16-bit add
pub fn add_16b(a: u16, b: u16) -> ALUResult<u16> {
    let result: u32 = a as u32 + b as u32;
    ALUResult {
        result: result as u16,
        carry: (result > u16::MAX.into()),
        halfcarry: (((a & 0x0FFF) + (b & 0x0FFF)) & 0x1000) == 0x1000,
    }
}

/// 8-bit subtract
pub fn sub_8b(a: u8, b: u8) -> ALUResult<u8> {
    let result: i16 = a as i16 - b as i16;
    ALUResult {
        result: result as u8,
        carry: result < 0,
        halfcarry: (result as u8 & 0x0F) > (a & 0x0F),
    }
}

/// Rotate left with carry
pub fn rotleft_9b(a: u8, carry: bool) -> ALUResult<u8> {
    let mut result = (a as u16) << 1;
    if carry {
        result |= 1;
    }

    ALUResult {
        result: result as u8,
        carry: result & 0x100 == 0x100,
        halfcarry: false,
    }
}

/// Rotate left, copy to carry
pub fn rotleft_8b(a: u8) -> ALUResult<u8> {
    let result = a.rotate_left(1);
    ALUResult {
        result: result as u8,
        carry: a & 0x80 == 0x80,
        halfcarry: false,
    }
}

/// Rotate right with carry
pub fn rotright_9b(a: u8, carry: bool) -> ALUResult<u8> {
    let mut result = (a as u16) >> 1;
    if carry {
        result |= 0x80;
    }

    ALUResult {
        result: result as u8,
        carry: a & 0x01 == 0x01,
        halfcarry: false,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn add_8b() {
        assert_eq!(super::add_8b(10, 2).result, 12);
    }

    #[test]
    fn add_8b_halfcarry() {
        let r = super::add_8b(0, 1);
        assert_eq!(r.result, 1);
        assert!(!r.halfcarry);

        let r = super::add_8b(0x0F, 1);
        assert_eq!(r.result, 0x10);
        assert!(r.halfcarry);

        let r = super::add_8b(0x10, 0);
        assert_eq!(r.result, 0x10);
        assert!(!r.halfcarry);

        let r = super::add_8b(0xFF, 1);
        assert_eq!(r.result, 0);
        assert!(r.halfcarry);

        let r = super::add_8b(0xFF, 0xFF);
        assert_eq!(r.result, 0xFE);
        assert!(r.halfcarry);
    }

    #[test]
    fn add_8b_carry() {
        let r = super::add_8b(0, 1);
        assert_eq!(r.result, 1);
        assert!(!r.carry);

        let r = super::add_8b(0xFF, 1);
        assert_eq!(r.result, 0);
        assert!(r.carry);

        let r = super::add_8b(0xFF, 0xFF);
        assert_eq!(r.result, 0xFE);
        assert!(r.carry);
    }

    #[test]
    fn rotleft_8b() {
        let r = super::rotleft_8b(0b01010101);
        assert_eq!(r.result, 0b10101010);
        assert_eq!(r.carry, false);

        let r = super::rotleft_8b(0x80);
        assert_eq!(r.result, 0x01);
        assert_eq!(r.carry, true);

        let r = super::rotleft_8b(0x11);
        assert_eq!(r.result, 0x22);
        assert_eq!(r.carry, false);

        let r = super::rotleft_8b(0x85);
        assert_eq!(r.result, 0x0B);
        assert_eq!(r.carry, true);
    }

    #[test]
    fn rotleft_9b() {
        let r = super::rotleft_9b(0b01010101, false);
        assert_eq!(r.result, 0b10101010);
        assert_eq!(r.carry, false);

        let r = super::rotleft_9b(0x80, false);
        assert_eq!(r.result, 0);
        assert_eq!(r.carry, true);

        let r = super::rotleft_9b(0x11, false);
        assert_eq!(r.result, 0x22);
        assert_eq!(r.carry, false);
    }

    #[test]
    fn rotright_9b() {
        let r = super::rotright_9b(0b01010101, true);
        assert_eq!(r.result, 0b10101010);
        assert_eq!(r.carry, true);

        let r = super::rotright_9b(0x01, false);
        assert_eq!(r.result, 0);
        assert_eq!(r.carry, true);

        let r = super::rotright_9b(0x22, false);
        assert_eq!(r.result, 0x11);
        assert_eq!(r.carry, false);
    }

    #[test]
    fn sub_8b() {
        let r = super::sub_8b(0x3E, 0x3E);
        assert_eq!(r.result, 0);
        assert!(!r.carry);
        assert!(!r.halfcarry);

        let r = super::sub_8b(0x3E, 0x0F);
        assert_eq!(r.result, 0x2F);
        assert!(!r.carry);
        assert!(r.halfcarry);

        let r = super::sub_8b(0x3E, 0x40);
        assert_eq!(r.result, 0xFE);
        assert!(r.carry);
        assert!(!r.halfcarry);
    }

    #[test]
    fn add_16b() {
        let r = super::add_16b(0x8A23, 0x0605);
        assert_eq!(r.result, 0x9028);
        assert!(!r.carry);
        assert!(r.halfcarry);

        let r = super::add_16b(0x8A23, 0x8A23);
        assert_eq!(r.result, 0x1446);
        assert!(r.carry);
        assert!(r.halfcarry);
    }
}
