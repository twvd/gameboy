use super::cpu::CPU;
use super::regs::Register;

pub enum Operand {
    None,
    Constant(u8),
    Register(Register),
    RegisterPtr(Register),
    RegisterPtrInc(Register),
    RegisterPtrDec(Register),
    Immediate8,
    Immediate16,
    ImmediatePtr8,
    ImmediatePtr16,
    Relative8,
    SPRelative8,
}

pub struct Instruction {
    pub mnemonic: &'static str,
    pub operands: [Operand; 2],
    pub len: u8,
    pub cycles: [u8; 2],
    pub func: fn(&mut CPU, &Instruction),
}
