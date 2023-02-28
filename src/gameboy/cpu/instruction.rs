use std::fmt;

use anyhow::Result;

use super::cpu::CPU;
use super::instructions::{INSTRUCTIONS, INSTRUCTIONS_CB};
use super::regs::Register;

/// A single operand to an instruction
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

/// A value of an immediate operand of an instruction
pub enum ImmediateVal {
    Immediate8(u8),
    Immediate16(u16),
}

impl fmt::Display for ImmediateVal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Immediate8(i) => write!(f, "${:02X}", i),
            Self::Immediate16(i) => write!(f, "${:04X}", i),
        }
    }
}

/// A definition for a single instruction of the CPU instruction set
pub struct InstructionDef {
    /// Mnemonic as string
    pub mnemonic: &'static str,

    /// Operand or Operand::None if not applicable for instruction.
    pub operands: [Operand; 2],

    /// Length of the complete instruction, including immediate values, in bytes.
    pub len: usize,

    /// Amount of cycles spent on executing the instruction.
    ///
    /// For conditional instructions, cycles[0] contains the
    /// cycles spent if the condition is met and cycles[1]
    /// if not.
    ///
    /// For unconditional instructions, use cycles[0].
    pub cycles: [u8; 2],

    /// CPU function that executes the instruction.
    pub func: fn(&mut CPU, &Instruction),
}

/// A decoded instruction.
pub struct Instruction<'a> {
    /// Reference to the definition.
    pub def: &'static InstructionDef,

    /// Vector of immediate values, if applicable.
    pub immediate: Vec<ImmediateVal>,

    /// Length of the full instruction.
    pub len: usize,

    /// Raw instruction bytes.
    pub raw: &'a [u8],
}

impl<'a> Instruction<'a> {
    /// Try to decode a single instruction from a u8 slice,
    /// starting from index 0.
    pub fn decode(stream: &'a [u8]) -> Result<Instruction> {
        let cb = stream[0] == 0xCB;
        let def: &InstructionDef = if cb {
            &INSTRUCTIONS_CB[stream[1] as usize]
        } else {
            &INSTRUCTIONS[stream[0] as usize]
        };
        let mut streampos = if cb { 2 } else { 1 };

        // Decode immediate values.
        let mut immediate: Vec<ImmediateVal> = vec![];
        for operand in &def.operands {
            match operand {
                Operand::Immediate8
                | Operand::ImmediatePtr8
                | Operand::Relative8
                | Operand::SPRelative8 => {
                    immediate.push(ImmediateVal::Immediate8(stream[streampos]));
                    streampos += 1;
                }
                Operand::Immediate16 | Operand::ImmediatePtr16 => {
                    immediate.push(ImmediateVal::Immediate16(
                        stream[streampos] as u16 | ((stream[streampos + 1] as u16) << 8),
                    ));
                    streampos += 2;
                }
                _ => {}
            }
        }

        // FIXME
        //assert_eq!(streampos, def.len);

        Ok(Instruction {
            def,
            immediate,
            len: def.len,
            raw: &stream[..def.len],
        })
    }
}

impl<'a> fmt::Display for Instruction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = self.def.mnemonic.to_string();
        let mut i = self.immediate.iter();

        // Fill in immediate values.
        for operand in &self.def.operands {
            s = match operand {
                Operand::Immediate8 => {
                    s.replacen("d8", format!("{}", i.next().ok_or(fmt::Error)?).as_str(), 1)
                }
                Operand::ImmediatePtr8 => {
                    s.replacen("a8", format!("{}", i.next().ok_or(fmt::Error)?).as_str(), 1)
                }
                Operand::Immediate16 => s.replacen(
                    "d16",
                    format!("{}", i.next().ok_or(fmt::Error)?).as_str(),
                    1,
                ),
                Operand::ImmediatePtr16 => s.replacen(
                    "a16",
                    format!("{}", i.next().ok_or(fmt::Error)?).as_str(),
                    1,
                ),
                Operand::Relative8 | Operand::SPRelative8 => {
                    s.replacen("r8", format!("{}", i.next().ok_or(fmt::Error)?).as_str(), 1)
                }
                _ => s,
            }
        }
        write!(f, "{:02X?} {}", self.raw, s.as_str())
    }
}
