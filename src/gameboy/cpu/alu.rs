pub struct ALUResult<T> {
    pub result: T,
    pub halfcarry: bool,
    pub carry: bool,
}

/// 8-bit add with flags
pub fn add_8b(a: u8, b: u8) -> ALUResult<u8> {
    let result: u16 = a as u16 + b as u16;
    ALUResult {
        result: result as u8,
        carry: (result > u8::MAX.into()),
        halfcarry: (((a & 0x0F) + (b & 0x0F)) & 0x10) == 0x10,
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
}
