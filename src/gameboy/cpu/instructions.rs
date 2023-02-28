use super::cpu::CPU;
use super::instruction::{InstructionDef, Operand};
use super::regs::Register;

/// Base instruction table, parsed from https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
pub const INSTRUCTIONS: [InstructionDef; 256] = [
    // NOP (1), - - - -
    InstructionDef {
        mnemonic: "NOP",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_nop,
    },
    // LD BC,d16 (3), - - - -
    InstructionDef {
        mnemonic: "LD BC,d16",
        operands: [Operand::Register(Register::BC), Operand::Immediate16],
        len: 3,
        cycles: [12, 12],
        func: CPU::op_ld,
    },
    // LD (BC),A (1), - - - -
    InstructionDef {
        mnemonic: "LD (BC),A",
        operands: [
            Operand::RegisterPtr(Register::BC),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // INC BC (1), - - - -
    InstructionDef {
        mnemonic: "INC BC",
        operands: [Operand::Register(Register::BC), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_inc,
    },
    // INC B (1), Z 0 H -
    InstructionDef {
        mnemonic: "INC B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_inc,
    },
    // DEC B (1), Z 1 H -
    InstructionDef {
        mnemonic: "DEC B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_dec,
    },
    // LD B,d8 (2), - - - -
    InstructionDef {
        mnemonic: "LD B,d8",
        operands: [Operand::Register(Register::B), Operand::Immediate8],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // RLCA (1), 0 0 0 C
    InstructionDef {
        mnemonic: "RLCA",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_rlca,
    },
    // LD (a16),SP (3), - - - -
    InstructionDef {
        mnemonic: "LD (a16),SP",
        operands: [Operand::ImmediatePtr16, Operand::Register(Register::SP)],
        len: 3,
        cycles: [20, 20],
        func: CPU::op_ld,
    },
    // ADD HL,BC (1), - 0 H C
    InstructionDef {
        mnemonic: "ADD HL,BC",
        operands: [
            Operand::Register(Register::HL),
            Operand::Register(Register::BC),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_add,
    },
    // LD A,(BC) (1), - - - -
    InstructionDef {
        mnemonic: "LD A,(BC)",
        operands: [
            Operand::Register(Register::A),
            Operand::RegisterPtr(Register::BC),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // DEC BC (1), - - - -
    InstructionDef {
        mnemonic: "DEC BC",
        operands: [Operand::Register(Register::BC), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_dec,
    },
    // INC C (1), Z 0 H -
    InstructionDef {
        mnemonic: "INC C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_inc,
    },
    // DEC C (1), Z 1 H -
    InstructionDef {
        mnemonic: "DEC C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_dec,
    },
    // LD C,d8 (2), - - - -
    InstructionDef {
        mnemonic: "LD C,d8",
        operands: [Operand::Register(Register::C), Operand::Immediate8],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // RRCA (1), 0 0 0 C
    InstructionDef {
        mnemonic: "RRCA",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_rrca,
    },
    // STOP 0 (2), - - - -
    InstructionDef {
        mnemonic: "STOP 0",
        operands: [Operand::Constant(0), Operand::None],
        len: 2,
        cycles: [4, 4],
        func: CPU::op_stop,
    },
    // LD DE,d16 (3), - - - -
    InstructionDef {
        mnemonic: "LD DE,d16",
        operands: [Operand::Register(Register::DE), Operand::Immediate16],
        len: 3,
        cycles: [12, 12],
        func: CPU::op_ld,
    },
    // LD (DE),A (1), - - - -
    InstructionDef {
        mnemonic: "LD (DE),A",
        operands: [
            Operand::RegisterPtr(Register::DE),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // INC DE (1), - - - -
    InstructionDef {
        mnemonic: "INC DE",
        operands: [Operand::Register(Register::DE), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_inc,
    },
    // INC D (1), Z 0 H -
    InstructionDef {
        mnemonic: "INC D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_inc,
    },
    // DEC D (1), Z 1 H -
    InstructionDef {
        mnemonic: "DEC D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_dec,
    },
    // LD D,d8 (2), - - - -
    InstructionDef {
        mnemonic: "LD D,d8",
        operands: [Operand::Register(Register::D), Operand::Immediate8],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // RLA (1), 0 0 0 C
    InstructionDef {
        mnemonic: "RLA",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_rla,
    },
    // JR r8 (2), - - - -
    InstructionDef {
        mnemonic: "JR r8",
        operands: [Operand::Relative8, Operand::None],
        len: 2,
        cycles: [12, 12],
        func: CPU::op_jr,
    },
    // ADD HL,DE (1), - 0 H C
    InstructionDef {
        mnemonic: "ADD HL,DE",
        operands: [
            Operand::Register(Register::HL),
            Operand::Register(Register::DE),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_add,
    },
    // LD A,(DE) (1), - - - -
    InstructionDef {
        mnemonic: "LD A,(DE)",
        operands: [
            Operand::Register(Register::A),
            Operand::RegisterPtr(Register::DE),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // DEC DE (1), - - - -
    InstructionDef {
        mnemonic: "DEC DE",
        operands: [Operand::Register(Register::DE), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_dec,
    },
    // INC E (1), Z 0 H -
    InstructionDef {
        mnemonic: "INC E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_inc,
    },
    // DEC E (1), Z 1 H -
    InstructionDef {
        mnemonic: "DEC E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_dec,
    },
    // LD E,d8 (2), - - - -
    InstructionDef {
        mnemonic: "LD E,d8",
        operands: [Operand::Register(Register::E), Operand::Immediate8],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // RRA (1), 0 0 0 C
    InstructionDef {
        mnemonic: "RRA",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_rra,
    },
    // JR NZ,r8 (2), - - - -
    InstructionDef {
        mnemonic: "JR NZ,r8",
        operands: [Operand::Relative8, Operand::None],
        len: 2,
        cycles: [12, 8],
        func: CPU::op_jr_nz,
    },
    // LD HL,d16 (3), - - - -
    InstructionDef {
        mnemonic: "LD HL,d16",
        operands: [Operand::Register(Register::HL), Operand::Immediate16],
        len: 3,
        cycles: [12, 12],
        func: CPU::op_ld,
    },
    // LD (HL+),A (1), - - - -
    InstructionDef {
        mnemonic: "LD (HL+),A",
        operands: [
            Operand::RegisterPtrInc(Register::HL),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // INC HL (1), - - - -
    InstructionDef {
        mnemonic: "INC HL",
        operands: [Operand::Register(Register::HL), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_inc,
    },
    // INC H (1), Z 0 H -
    InstructionDef {
        mnemonic: "INC H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_inc,
    },
    // DEC H (1), Z 1 H -
    InstructionDef {
        mnemonic: "DEC H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_dec,
    },
    // LD H,d8 (2), - - - -
    InstructionDef {
        mnemonic: "LD H,d8",
        operands: [Operand::Register(Register::H), Operand::Immediate8],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // DAA (1), Z - 0 C
    InstructionDef {
        mnemonic: "DAA",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_daa,
    },
    // JR Z,r8 (2), - - - -
    InstructionDef {
        mnemonic: "JR Z,r8",
        operands: [Operand::Relative8, Operand::None],
        len: 2,
        cycles: [12, 8],
        func: CPU::op_jr_z,
    },
    // ADD HL,HL (1), - 0 H C
    InstructionDef {
        mnemonic: "ADD HL,HL",
        operands: [
            Operand::Register(Register::HL),
            Operand::Register(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_add,
    },
    // LD A,(HL+) (1), - - - -
    InstructionDef {
        mnemonic: "LD A,(HL+)",
        operands: [
            Operand::Register(Register::A),
            Operand::RegisterPtrInc(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // DEC HL (1), - - - -
    InstructionDef {
        mnemonic: "DEC HL",
        operands: [Operand::Register(Register::HL), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_dec,
    },
    // INC L (1), Z 0 H -
    InstructionDef {
        mnemonic: "INC L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_inc,
    },
    // DEC L (1), Z 1 H -
    InstructionDef {
        mnemonic: "DEC L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_dec,
    },
    // LD L,d8 (2), - - - -
    InstructionDef {
        mnemonic: "LD L,d8",
        operands: [Operand::Register(Register::L), Operand::Immediate8],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // CPL (1), - 1 1 -
    InstructionDef {
        mnemonic: "CPL",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_cpl,
    },
    // JR NC,r8 (2), - - - -
    InstructionDef {
        mnemonic: "JR NC,r8",
        operands: [Operand::Relative8, Operand::None],
        len: 2,
        cycles: [12, 8],
        func: CPU::op_jr_nc,
    },
    // LD SP,d16 (3), - - - -
    InstructionDef {
        mnemonic: "LD SP,d16",
        operands: [Operand::Register(Register::SP), Operand::Immediate16],
        len: 3,
        cycles: [12, 12],
        func: CPU::op_ld,
    },
    // LD (HL-),A (1), - - - -
    InstructionDef {
        mnemonic: "LD (HL-),A",
        operands: [
            Operand::RegisterPtrDec(Register::HL),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // INC SP (1), - - - -
    InstructionDef {
        mnemonic: "INC SP",
        operands: [Operand::Register(Register::SP), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_inc,
    },
    // INC (HL) (1), Z 0 H -
    InstructionDef {
        mnemonic: "INC (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 1,
        cycles: [12, 12],
        func: CPU::op_inc,
    },
    // DEC (HL) (1), Z 1 H -
    InstructionDef {
        mnemonic: "DEC (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 1,
        cycles: [12, 12],
        func: CPU::op_dec,
    },
    // LD (HL),d8 (2), - - - -
    InstructionDef {
        mnemonic: "LD (HL),d8",
        operands: [Operand::RegisterPtr(Register::HL), Operand::Immediate8],
        len: 2,
        cycles: [12, 12],
        func: CPU::op_ld,
    },
    // SCF (1), - 0 0 1
    InstructionDef {
        mnemonic: "SCF",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_scf,
    },
    // JR C,r8 (2), - - - -
    InstructionDef {
        mnemonic: "JR C,r8",
        operands: [Operand::Register(Register::C), Operand::Relative8],
        len: 2,
        cycles: [12, 8],
        func: CPU::op_jr,
    },
    // ADD HL,SP (1), - 0 H C
    InstructionDef {
        mnemonic: "ADD HL,SP",
        operands: [
            Operand::Register(Register::HL),
            Operand::Register(Register::SP),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_add,
    },
    // LD A,(HL-) (1), - - - -
    InstructionDef {
        mnemonic: "LD A,(HL-)",
        operands: [
            Operand::Register(Register::A),
            Operand::RegisterPtrDec(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // DEC SP (1), - - - -
    InstructionDef {
        mnemonic: "DEC SP",
        operands: [Operand::Register(Register::SP), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_dec,
    },
    // INC A (1), Z 0 H -
    InstructionDef {
        mnemonic: "INC A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_inc,
    },
    // DEC A (1), Z 1 H -
    InstructionDef {
        mnemonic: "DEC A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_dec,
    },
    // LD A,d8 (2), - - - -
    InstructionDef {
        mnemonic: "LD A,d8",
        operands: [Operand::Register(Register::A), Operand::Immediate8],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // CCF (1), - 0 0 C
    InstructionDef {
        mnemonic: "CCF",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ccf,
    },
    // LD B,B (1), - - - -
    InstructionDef {
        mnemonic: "LD B,B",
        operands: [
            Operand::Register(Register::B),
            Operand::Register(Register::B),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD B,C (1), - - - -
    InstructionDef {
        mnemonic: "LD B,C",
        operands: [
            Operand::Register(Register::B),
            Operand::Register(Register::C),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD B,D (1), - - - -
    InstructionDef {
        mnemonic: "LD B,D",
        operands: [
            Operand::Register(Register::B),
            Operand::Register(Register::D),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD B,E (1), - - - -
    InstructionDef {
        mnemonic: "LD B,E",
        operands: [
            Operand::Register(Register::B),
            Operand::Register(Register::E),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD B,H (1), - - - -
    InstructionDef {
        mnemonic: "LD B,H",
        operands: [
            Operand::Register(Register::B),
            Operand::Register(Register::H),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD B,L (1), - - - -
    InstructionDef {
        mnemonic: "LD B,L",
        operands: [
            Operand::Register(Register::B),
            Operand::Register(Register::L),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD B,(HL) (1), - - - -
    InstructionDef {
        mnemonic: "LD B,(HL)",
        operands: [
            Operand::Register(Register::B),
            Operand::RegisterPtr(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD B,A (1), - - - -
    InstructionDef {
        mnemonic: "LD B,A",
        operands: [
            Operand::Register(Register::B),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD C,B (1), - - - -
    InstructionDef {
        mnemonic: "LD C,B",
        operands: [
            Operand::Register(Register::C),
            Operand::Register(Register::B),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD C,C (1), - - - -
    InstructionDef {
        mnemonic: "LD C,C",
        operands: [
            Operand::Register(Register::C),
            Operand::Register(Register::C),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD C,D (1), - - - -
    InstructionDef {
        mnemonic: "LD C,D",
        operands: [
            Operand::Register(Register::C),
            Operand::Register(Register::D),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD C,E (1), - - - -
    InstructionDef {
        mnemonic: "LD C,E",
        operands: [
            Operand::Register(Register::C),
            Operand::Register(Register::E),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD C,H (1), - - - -
    InstructionDef {
        mnemonic: "LD C,H",
        operands: [
            Operand::Register(Register::C),
            Operand::Register(Register::H),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD C,L (1), - - - -
    InstructionDef {
        mnemonic: "LD C,L",
        operands: [
            Operand::Register(Register::C),
            Operand::Register(Register::L),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD C,(HL) (1), - - - -
    InstructionDef {
        mnemonic: "LD C,(HL)",
        operands: [
            Operand::Register(Register::C),
            Operand::RegisterPtr(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD C,A (1), - - - -
    InstructionDef {
        mnemonic: "LD C,A",
        operands: [
            Operand::Register(Register::C),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD D,B (1), - - - -
    InstructionDef {
        mnemonic: "LD D,B",
        operands: [
            Operand::Register(Register::D),
            Operand::Register(Register::B),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD D,C (1), - - - -
    InstructionDef {
        mnemonic: "LD D,C",
        operands: [
            Operand::Register(Register::D),
            Operand::Register(Register::C),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD D,D (1), - - - -
    InstructionDef {
        mnemonic: "LD D,D",
        operands: [
            Operand::Register(Register::D),
            Operand::Register(Register::D),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD D,E (1), - - - -
    InstructionDef {
        mnemonic: "LD D,E",
        operands: [
            Operand::Register(Register::D),
            Operand::Register(Register::E),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD D,H (1), - - - -
    InstructionDef {
        mnemonic: "LD D,H",
        operands: [
            Operand::Register(Register::D),
            Operand::Register(Register::H),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD D,L (1), - - - -
    InstructionDef {
        mnemonic: "LD D,L",
        operands: [
            Operand::Register(Register::D),
            Operand::Register(Register::L),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD D,(HL) (1), - - - -
    InstructionDef {
        mnemonic: "LD D,(HL)",
        operands: [
            Operand::Register(Register::D),
            Operand::RegisterPtr(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD D,A (1), - - - -
    InstructionDef {
        mnemonic: "LD D,A",
        operands: [
            Operand::Register(Register::D),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD E,B (1), - - - -
    InstructionDef {
        mnemonic: "LD E,B",
        operands: [
            Operand::Register(Register::E),
            Operand::Register(Register::B),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD E,C (1), - - - -
    InstructionDef {
        mnemonic: "LD E,C",
        operands: [
            Operand::Register(Register::E),
            Operand::Register(Register::C),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD E,D (1), - - - -
    InstructionDef {
        mnemonic: "LD E,D",
        operands: [
            Operand::Register(Register::E),
            Operand::Register(Register::D),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD E,E (1), - - - -
    InstructionDef {
        mnemonic: "LD E,E",
        operands: [
            Operand::Register(Register::E),
            Operand::Register(Register::E),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD E,H (1), - - - -
    InstructionDef {
        mnemonic: "LD E,H",
        operands: [
            Operand::Register(Register::E),
            Operand::Register(Register::H),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD E,L (1), - - - -
    InstructionDef {
        mnemonic: "LD E,L",
        operands: [
            Operand::Register(Register::E),
            Operand::Register(Register::L),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD E,(HL) (1), - - - -
    InstructionDef {
        mnemonic: "LD E,(HL)",
        operands: [
            Operand::Register(Register::E),
            Operand::RegisterPtr(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD E,A (1), - - - -
    InstructionDef {
        mnemonic: "LD E,A",
        operands: [
            Operand::Register(Register::E),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD H,B (1), - - - -
    InstructionDef {
        mnemonic: "LD H,B",
        operands: [
            Operand::Register(Register::H),
            Operand::Register(Register::B),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD H,C (1), - - - -
    InstructionDef {
        mnemonic: "LD H,C",
        operands: [
            Operand::Register(Register::H),
            Operand::Register(Register::C),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD H,D (1), - - - -
    InstructionDef {
        mnemonic: "LD H,D",
        operands: [
            Operand::Register(Register::H),
            Operand::Register(Register::D),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD H,E (1), - - - -
    InstructionDef {
        mnemonic: "LD H,E",
        operands: [
            Operand::Register(Register::H),
            Operand::Register(Register::E),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD H,H (1), - - - -
    InstructionDef {
        mnemonic: "LD H,H",
        operands: [
            Operand::Register(Register::H),
            Operand::Register(Register::H),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD H,L (1), - - - -
    InstructionDef {
        mnemonic: "LD H,L",
        operands: [
            Operand::Register(Register::H),
            Operand::Register(Register::L),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD H,(HL) (1), - - - -
    InstructionDef {
        mnemonic: "LD H,(HL)",
        operands: [
            Operand::Register(Register::H),
            Operand::RegisterPtr(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD H,A (1), - - - -
    InstructionDef {
        mnemonic: "LD H,A",
        operands: [
            Operand::Register(Register::H),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD L,B (1), - - - -
    InstructionDef {
        mnemonic: "LD L,B",
        operands: [
            Operand::Register(Register::L),
            Operand::Register(Register::B),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD L,C (1), - - - -
    InstructionDef {
        mnemonic: "LD L,C",
        operands: [
            Operand::Register(Register::L),
            Operand::Register(Register::C),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD L,D (1), - - - -
    InstructionDef {
        mnemonic: "LD L,D",
        operands: [
            Operand::Register(Register::L),
            Operand::Register(Register::D),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD L,E (1), - - - -
    InstructionDef {
        mnemonic: "LD L,E",
        operands: [
            Operand::Register(Register::L),
            Operand::Register(Register::E),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD L,H (1), - - - -
    InstructionDef {
        mnemonic: "LD L,H",
        operands: [
            Operand::Register(Register::L),
            Operand::Register(Register::H),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD L,L (1), - - - -
    InstructionDef {
        mnemonic: "LD L,L",
        operands: [
            Operand::Register(Register::L),
            Operand::Register(Register::L),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD L,(HL) (1), - - - -
    InstructionDef {
        mnemonic: "LD L,(HL)",
        operands: [
            Operand::Register(Register::L),
            Operand::RegisterPtr(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD L,A (1), - - - -
    InstructionDef {
        mnemonic: "LD L,A",
        operands: [
            Operand::Register(Register::L),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD (HL),B (1), - - - -
    InstructionDef {
        mnemonic: "LD (HL),B",
        operands: [
            Operand::RegisterPtr(Register::HL),
            Operand::Register(Register::B),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD (HL),C (1), - - - -
    InstructionDef {
        mnemonic: "LD (HL),C",
        operands: [
            Operand::RegisterPtr(Register::HL),
            Operand::Register(Register::C),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD (HL),D (1), - - - -
    InstructionDef {
        mnemonic: "LD (HL),D",
        operands: [
            Operand::RegisterPtr(Register::HL),
            Operand::Register(Register::D),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD (HL),E (1), - - - -
    InstructionDef {
        mnemonic: "LD (HL),E",
        operands: [
            Operand::RegisterPtr(Register::HL),
            Operand::Register(Register::E),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD (HL),H (1), - - - -
    InstructionDef {
        mnemonic: "LD (HL),H",
        operands: [
            Operand::RegisterPtr(Register::HL),
            Operand::Register(Register::H),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD (HL),L (1), - - - -
    InstructionDef {
        mnemonic: "LD (HL),L",
        operands: [
            Operand::RegisterPtr(Register::HL),
            Operand::Register(Register::L),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // HALT (1), - - - -
    InstructionDef {
        mnemonic: "HALT",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_halt,
    },
    // LD (HL),A (1), - - - -
    InstructionDef {
        mnemonic: "LD (HL),A",
        operands: [
            Operand::RegisterPtr(Register::HL),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD A,B (1), - - - -
    InstructionDef {
        mnemonic: "LD A,B",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::B),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD A,C (1), - - - -
    InstructionDef {
        mnemonic: "LD A,C",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::C),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD A,D (1), - - - -
    InstructionDef {
        mnemonic: "LD A,D",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::D),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD A,E (1), - - - -
    InstructionDef {
        mnemonic: "LD A,E",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::E),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD A,H (1), - - - -
    InstructionDef {
        mnemonic: "LD A,H",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::H),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD A,L (1), - - - -
    InstructionDef {
        mnemonic: "LD A,L",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::L),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // LD A,(HL) (1), - - - -
    InstructionDef {
        mnemonic: "LD A,(HL)",
        operands: [
            Operand::Register(Register::A),
            Operand::RegisterPtr(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD A,A (1), - - - -
    InstructionDef {
        mnemonic: "LD A,A",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ld,
    },
    // ADD A,B (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADD A,B",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::B),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_add,
    },
    // ADD A,C (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADD A,C",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::C),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_add,
    },
    // ADD A,D (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADD A,D",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::D),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_add,
    },
    // ADD A,E (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADD A,E",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::E),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_add,
    },
    // ADD A,H (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADD A,H",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::H),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_add,
    },
    // ADD A,L (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADD A,L",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::L),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_add,
    },
    // ADD A,(HL) (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADD A,(HL)",
        operands: [
            Operand::Register(Register::A),
            Operand::RegisterPtr(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_add,
    },
    // ADD A,A (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADD A,A",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_add,
    },
    // ADC A,B (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADC A,B",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::B),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_adc,
    },
    // ADC A,C (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADC A,C",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::C),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_adc,
    },
    // ADC A,D (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADC A,D",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::D),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_adc,
    },
    // ADC A,E (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADC A,E",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::E),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_adc,
    },
    // ADC A,H (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADC A,H",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::H),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_adc,
    },
    // ADC A,L (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADC A,L",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::L),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_adc,
    },
    // ADC A,(HL) (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADC A,(HL)",
        operands: [
            Operand::Register(Register::A),
            Operand::RegisterPtr(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_adc,
    },
    // ADC A,A (1), Z 0 H C
    InstructionDef {
        mnemonic: "ADC A,A",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_adc,
    },
    // SUB B (1), Z 1 H C
    InstructionDef {
        mnemonic: "SUB B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sub,
    },
    // SUB C (1), Z 1 H C
    InstructionDef {
        mnemonic: "SUB C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sub,
    },
    // SUB D (1), Z 1 H C
    InstructionDef {
        mnemonic: "SUB D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sub,
    },
    // SUB E (1), Z 1 H C
    InstructionDef {
        mnemonic: "SUB E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sub,
    },
    // SUB H (1), Z 1 H C
    InstructionDef {
        mnemonic: "SUB H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sub,
    },
    // SUB L (1), Z 1 H C
    InstructionDef {
        mnemonic: "SUB L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sub,
    },
    // SUB (HL) (1), Z 1 H C
    InstructionDef {
        mnemonic: "SUB (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_sub,
    },
    // SUB A (1), Z 1 H C
    InstructionDef {
        mnemonic: "SUB A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sub,
    },
    // SBC A,B (1), Z 1 H C
    InstructionDef {
        mnemonic: "SBC A,B",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::B),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sbc,
    },
    // SBC A,C (1), Z 1 H C
    InstructionDef {
        mnemonic: "SBC A,C",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::C),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sbc,
    },
    // SBC A,D (1), Z 1 H C
    InstructionDef {
        mnemonic: "SBC A,D",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::D),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sbc,
    },
    // SBC A,E (1), Z 1 H C
    InstructionDef {
        mnemonic: "SBC A,E",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::E),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sbc,
    },
    // SBC A,H (1), Z 1 H C
    InstructionDef {
        mnemonic: "SBC A,H",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::H),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sbc,
    },
    // SBC A,L (1), Z 1 H C
    InstructionDef {
        mnemonic: "SBC A,L",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::L),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sbc,
    },
    // SBC A,(HL) (1), Z 1 H C
    InstructionDef {
        mnemonic: "SBC A,(HL)",
        operands: [
            Operand::Register(Register::A),
            Operand::RegisterPtr(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_sbc,
    },
    // SBC A,A (1), Z 1 H C
    InstructionDef {
        mnemonic: "SBC A,A",
        operands: [
            Operand::Register(Register::A),
            Operand::Register(Register::A),
        ],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_sbc,
    },
    // AND B (1), Z 0 1 0
    InstructionDef {
        mnemonic: "AND B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_and,
    },
    // AND C (1), Z 0 1 0
    InstructionDef {
        mnemonic: "AND C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_and,
    },
    // AND D (1), Z 0 1 0
    InstructionDef {
        mnemonic: "AND D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_and,
    },
    // AND E (1), Z 0 1 0
    InstructionDef {
        mnemonic: "AND E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_and,
    },
    // AND H (1), Z 0 1 0
    InstructionDef {
        mnemonic: "AND H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_and,
    },
    // AND L (1), Z 0 1 0
    InstructionDef {
        mnemonic: "AND L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_and,
    },
    // AND (HL) (1), Z 0 1 0
    InstructionDef {
        mnemonic: "AND (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_and,
    },
    // AND A (1), Z 0 1 0
    InstructionDef {
        mnemonic: "AND A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_and,
    },
    // XOR B (1), Z 0 0 0
    InstructionDef {
        mnemonic: "XOR B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_xor,
    },
    // XOR C (1), Z 0 0 0
    InstructionDef {
        mnemonic: "XOR C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_xor,
    },
    // XOR D (1), Z 0 0 0
    InstructionDef {
        mnemonic: "XOR D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_xor,
    },
    // XOR E (1), Z 0 0 0
    InstructionDef {
        mnemonic: "XOR E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_xor,
    },
    // XOR H (1), Z 0 0 0
    InstructionDef {
        mnemonic: "XOR H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_xor,
    },
    // XOR L (1), Z 0 0 0
    InstructionDef {
        mnemonic: "XOR L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_xor,
    },
    // XOR (HL) (1), Z 0 0 0
    InstructionDef {
        mnemonic: "XOR (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_xor,
    },
    // XOR A (1), Z 0 0 0
    InstructionDef {
        mnemonic: "XOR A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_xor,
    },
    // OR B (1), Z 0 0 0
    InstructionDef {
        mnemonic: "OR B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_or,
    },
    // OR C (1), Z 0 0 0
    InstructionDef {
        mnemonic: "OR C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_or,
    },
    // OR D (1), Z 0 0 0
    InstructionDef {
        mnemonic: "OR D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_or,
    },
    // OR E (1), Z 0 0 0
    InstructionDef {
        mnemonic: "OR E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_or,
    },
    // OR H (1), Z 0 0 0
    InstructionDef {
        mnemonic: "OR H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_or,
    },
    // OR L (1), Z 0 0 0
    InstructionDef {
        mnemonic: "OR L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_or,
    },
    // OR (HL) (1), Z 0 0 0
    InstructionDef {
        mnemonic: "OR (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_or,
    },
    // OR A (1), Z 0 0 0
    InstructionDef {
        mnemonic: "OR A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_or,
    },
    // CP B (1), Z 1 H C
    InstructionDef {
        mnemonic: "CP B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_cp,
    },
    // CP C (1), Z 1 H C
    InstructionDef {
        mnemonic: "CP C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_cp,
    },
    // CP D (1), Z 1 H C
    InstructionDef {
        mnemonic: "CP D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_cp,
    },
    // CP E (1), Z 1 H C
    InstructionDef {
        mnemonic: "CP E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_cp,
    },
    // CP H (1), Z 1 H C
    InstructionDef {
        mnemonic: "CP H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_cp,
    },
    // CP L (1), Z 1 H C
    InstructionDef {
        mnemonic: "CP L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_cp,
    },
    // CP (HL) (1), Z 1 H C
    InstructionDef {
        mnemonic: "CP (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_cp,
    },
    // CP A (1), Z 1 H C
    InstructionDef {
        mnemonic: "CP A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_cp,
    },
    // RET NZ (1), - - - -
    InstructionDef {
        mnemonic: "RET NZ",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [20, 8],
        func: CPU::op_ret_nz,
    },
    // POP BC (1), - - - -
    InstructionDef {
        mnemonic: "POP BC",
        operands: [Operand::Register(Register::BC), Operand::None],
        len: 1,
        cycles: [12, 12],
        func: CPU::op_pop,
    },
    // JP NZ,a16 (3), - - - -
    InstructionDef {
        mnemonic: "JP NZ,a16",
        operands: [Operand::ImmediatePtr16, Operand::None],
        len: 3,
        cycles: [16, 12],
        func: CPU::op_jp_nz,
    },
    // JP a16 (3), - - - -
    InstructionDef {
        mnemonic: "JP a16",
        operands: [Operand::ImmediatePtr16, Operand::None],
        len: 3,
        cycles: [16, 16],
        func: CPU::op_jp,
    },
    // CALL NZ,a16 (3), - - - -
    InstructionDef {
        mnemonic: "CALL NZ,a16",
        operands: [Operand::ImmediatePtr16, Operand::None],
        len: 3,
        cycles: [24, 12],
        func: CPU::op_call_nz,
    },
    // PUSH BC (1), - - - -
    InstructionDef {
        mnemonic: "PUSH BC",
        operands: [Operand::Register(Register::BC), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_push,
    },
    // ADD A,d8 (2), Z 0 H C
    InstructionDef {
        mnemonic: "ADD A,d8",
        operands: [Operand::Register(Register::A), Operand::Immediate8],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_add,
    },
    // RST 00H (1), - - - -
    InstructionDef {
        mnemonic: "RST 00H",
        operands: [Operand::Constant(0x00), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_rst,
    },
    // RET Z (1), - - - -
    InstructionDef {
        mnemonic: "RET Z",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [20, 8],
        func: CPU::op_ret_z,
    },
    // RET (1), - - - -
    InstructionDef {
        mnemonic: "RET",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_ret,
    },
    // JP Z,a16 (3), - - - -
    InstructionDef {
        mnemonic: "JP Z,a16",
        operands: [Operand::ImmediatePtr16, Operand::None],
        len: 3,
        cycles: [16, 12],
        func: CPU::op_jp_z,
    },
    // PREFIX CB (1), - - - -
    InstructionDef {
        mnemonic: "PREFIX CB",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_prefix_cb,
    },
    // CALL Z,a16 (3), - - - -
    InstructionDef {
        mnemonic: "CALL Z,a16",
        operands: [Operand::ImmediatePtr16, Operand::None],
        len: 3,
        cycles: [24, 12],
        func: CPU::op_call_z,
    },
    // CALL a16 (3), - - - -
    InstructionDef {
        mnemonic: "CALL a16",
        operands: [Operand::ImmediatePtr16, Operand::None],
        len: 3,
        cycles: [24, 24],
        func: CPU::op_call,
    },
    // ADC A,d8 (2), Z 0 H C
    InstructionDef {
        mnemonic: "ADC A,d8",
        operands: [Operand::Register(Register::A), Operand::Immediate8],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_adc,
    },
    // RST 08H (1), - - - -
    InstructionDef {
        mnemonic: "RST 08H",
        operands: [Operand::Constant(0x08), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_rst,
    },
    // RET NC (1), - - - -
    InstructionDef {
        mnemonic: "RET NC",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [20, 8],
        func: CPU::op_ret_nc,
    },
    // POP DE (1), - - - -
    InstructionDef {
        mnemonic: "POP DE",
        operands: [Operand::Register(Register::DE), Operand::None],
        len: 1,
        cycles: [12, 12],
        func: CPU::op_pop,
    },
    // JP NC,a16 (3), - - - -
    InstructionDef {
        mnemonic: "JP NC,a16",
        operands: [Operand::ImmediatePtr16, Operand::None],
        len: 3,
        cycles: [16, 12],
        func: CPU::op_jp_nc,
    },
    // INVALID
    InstructionDef {
        mnemonic: "INVALID",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [0, 0],
        func: CPU::op_invalid,
    },
    // CALL NC,a16 (3), - - - -
    InstructionDef {
        mnemonic: "CALL NC,a16",
        operands: [Operand::ImmediatePtr16, Operand::None],
        len: 3,
        cycles: [24, 12],
        func: CPU::op_call_nc,
    },
    // PUSH DE (1), - - - -
    InstructionDef {
        mnemonic: "PUSH DE",
        operands: [Operand::Register(Register::DE), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_push,
    },
    // SUB d8 (2), Z 1 H C
    InstructionDef {
        mnemonic: "SUB d8",
        operands: [Operand::Immediate8, Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sub,
    },
    // RST 10H (1), - - - -
    InstructionDef {
        mnemonic: "RST 10H",
        operands: [Operand::Constant(0x10), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_rst,
    },
    // RET C (1), - - - -
    InstructionDef {
        mnemonic: "RET C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 1,
        cycles: [20, 8],
        func: CPU::op_ret,
    },
    // RETI (1), - - - -
    InstructionDef {
        mnemonic: "RETI",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_reti,
    },
    // JP C,a16 (3), - - - -
    InstructionDef {
        mnemonic: "JP C,a16",
        operands: [Operand::Register(Register::C), Operand::ImmediatePtr16],
        len: 3,
        cycles: [16, 12],
        func: CPU::op_jp,
    },
    // INVALID
    InstructionDef {
        mnemonic: "INVALID",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [0, 0],
        func: CPU::op_invalid,
    },
    // CALL C,a16 (3), - - - -
    InstructionDef {
        mnemonic: "CALL C,a16",
        operands: [Operand::Register(Register::C), Operand::ImmediatePtr16],
        len: 3,
        cycles: [24, 12],
        func: CPU::op_call,
    },
    // INVALID
    InstructionDef {
        mnemonic: "INVALID",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [0, 0],
        func: CPU::op_invalid,
    },
    // SBC A,d8 (2), Z 1 H C
    InstructionDef {
        mnemonic: "SBC A,d8",
        operands: [Operand::Register(Register::A), Operand::Immediate8],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sbc,
    },
    // RST 18H (1), - - - -
    InstructionDef {
        mnemonic: "RST 18H",
        operands: [Operand::Constant(0x18), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_rst,
    },
    // LDH (a8),A (2), - - - -
    InstructionDef {
        mnemonic: "LDH (a8),A",
        operands: [Operand::ImmediatePtr8, Operand::Register(Register::A)],
        len: 2,
        cycles: [12, 12],
        func: CPU::op_ldh,
    },
    // POP HL (1), - - - -
    InstructionDef {
        mnemonic: "POP HL",
        operands: [Operand::Register(Register::HL), Operand::None],
        len: 1,
        cycles: [12, 12],
        func: CPU::op_pop,
    },
    // LD (C),A (2), - - - -
    InstructionDef {
        mnemonic: "LD (C),A",
        operands: [
            Operand::RegisterPtr(Register::C),
            Operand::Register(Register::A),
        ],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // INVALID
    InstructionDef {
        mnemonic: "INVALID",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [0, 0],
        func: CPU::op_invalid,
    },
    // INVALID
    InstructionDef {
        mnemonic: "INVALID",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [0, 0],
        func: CPU::op_invalid,
    },
    // PUSH HL (1), - - - -
    InstructionDef {
        mnemonic: "PUSH HL",
        operands: [Operand::Register(Register::HL), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_push,
    },
    // AND d8 (2), Z 0 1 0
    InstructionDef {
        mnemonic: "AND d8",
        operands: [Operand::Immediate8, Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_and,
    },
    // RST 20H (1), - - - -
    InstructionDef {
        mnemonic: "RST 20H",
        operands: [Operand::Constant(0x20), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_rst,
    },
    // ADD SP,r8 (2), 0 0 H C
    InstructionDef {
        mnemonic: "ADD SP,r8",
        operands: [Operand::Register(Register::SP), Operand::Relative8],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_add,
    },
    // JP (HL) (1), - - - -
    InstructionDef {
        mnemonic: "JP (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_jp,
    },
    // LD (a16),A (3), - - - -
    InstructionDef {
        mnemonic: "LD (a16),A",
        operands: [Operand::ImmediatePtr16, Operand::Register(Register::A)],
        len: 3,
        cycles: [16, 16],
        func: CPU::op_ld,
    },
    // INVALID
    InstructionDef {
        mnemonic: "INVALID",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [0, 0],
        func: CPU::op_invalid,
    },
    // INVALID
    InstructionDef {
        mnemonic: "INVALID",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [0, 0],
        func: CPU::op_invalid,
    },
    // INVALID
    InstructionDef {
        mnemonic: "INVALID",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [0, 0],
        func: CPU::op_invalid,
    },
    // XOR d8 (2), Z 0 0 0
    InstructionDef {
        mnemonic: "XOR d8",
        operands: [Operand::Immediate8, Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_xor,
    },
    // RST 28H (1), - - - -
    InstructionDef {
        mnemonic: "RST 28H",
        operands: [Operand::Constant(0x28), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_rst,
    },
    // LDH A,(a8) (2), - - - -
    InstructionDef {
        mnemonic: "LDH A,(a8)",
        operands: [Operand::Register(Register::A), Operand::ImmediatePtr8],
        len: 2,
        cycles: [12, 12],
        func: CPU::op_ldh,
    },
    // POP AF (1), Z N H C
    InstructionDef {
        mnemonic: "POP AF",
        operands: [Operand::Register(Register::AF), Operand::None],
        len: 1,
        cycles: [12, 12],
        func: CPU::op_pop,
    },
    // LD A,(C) (2), - - - -
    InstructionDef {
        mnemonic: "LD A,(C)",
        operands: [
            Operand::Register(Register::A),
            Operand::RegisterPtr(Register::C),
        ],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // DI (1), - - - -
    InstructionDef {
        mnemonic: "DI",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_di,
    },
    // INVALID
    InstructionDef {
        mnemonic: "INVALID",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [0, 0],
        func: CPU::op_invalid,
    },
    // PUSH AF (1), - - - -
    InstructionDef {
        mnemonic: "PUSH AF",
        operands: [Operand::Register(Register::AF), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_push,
    },
    // OR d8 (2), Z 0 0 0
    InstructionDef {
        mnemonic: "OR d8",
        operands: [Operand::Immediate8, Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_or,
    },
    // RST 30H (1), - - - -
    InstructionDef {
        mnemonic: "RST 30H",
        operands: [Operand::Constant(0x30), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_rst,
    },
    // LD HL,SP+r8 (2), 0 0 H C
    InstructionDef {
        mnemonic: "LD HL,SP+r8",
        operands: [Operand::Register(Register::HL), Operand::SPRelative8],
        len: 2,
        cycles: [12, 12],
        func: CPU::op_ld,
    },
    // LD SP,HL (1), - - - -
    InstructionDef {
        mnemonic: "LD SP,HL",
        operands: [
            Operand::Register(Register::SP),
            Operand::Register(Register::HL),
        ],
        len: 1,
        cycles: [8, 8],
        func: CPU::op_ld,
    },
    // LD A,(a16) (3), - - - -
    InstructionDef {
        mnemonic: "LD A,(a16)",
        operands: [Operand::Register(Register::A), Operand::ImmediatePtr16],
        len: 3,
        cycles: [16, 16],
        func: CPU::op_ld,
    },
    // EI (1), - - - -
    InstructionDef {
        mnemonic: "EI",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [4, 4],
        func: CPU::op_ei,
    },
    // INVALID
    InstructionDef {
        mnemonic: "INVALID",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [0, 0],
        func: CPU::op_invalid,
    },
    // INVALID
    InstructionDef {
        mnemonic: "INVALID",
        operands: [Operand::None, Operand::None],
        len: 1,
        cycles: [0, 0],
        func: CPU::op_invalid,
    },
    // CP d8 (2), Z 1 H C
    InstructionDef {
        mnemonic: "CP d8",
        operands: [Operand::Immediate8, Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_cp,
    },
    // RST 38H (1), - - - -
    InstructionDef {
        mnemonic: "RST 38H",
        operands: [Operand::Constant(0x38), Operand::None],
        len: 1,
        cycles: [16, 16],
        func: CPU::op_rst,
    },
];

/// 0xCB prefix instruction table, parsed from https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
pub const INSTRUCTIONS_CB: [InstructionDef; 256] = [
    // RLC B (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RLC B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rlc,
    },
    // RLC C (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RLC C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rlc,
    },
    // RLC D (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RLC D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rlc,
    },
    // RLC E (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RLC E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rlc,
    },
    // RLC H (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RLC H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rlc,
    },
    // RLC L (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RLC L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rlc,
    },
    // RLC (HL) (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RLC (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_rlc,
    },
    // RLC A (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RLC A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rlc,
    },
    // RRC B (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RRC B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rrc,
    },
    // RRC C (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RRC C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rrc,
    },
    // RRC D (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RRC D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rrc,
    },
    // RRC E (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RRC E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rrc,
    },
    // RRC H (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RRC H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rrc,
    },
    // RRC L (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RRC L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rrc,
    },
    // RRC (HL) (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RRC (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_rrc,
    },
    // RRC A (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RRC A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rrc,
    },
    // RL B (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RL B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rl,
    },
    // RL C (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RL C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rl,
    },
    // RL D (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RL D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rl,
    },
    // RL E (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RL E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rl,
    },
    // RL H (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RL H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rl,
    },
    // RL L (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RL L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rl,
    },
    // RL (HL) (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RL (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_rl,
    },
    // RL A (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RL A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rl,
    },
    // RR B (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RR B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rr,
    },
    // RR C (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RR C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rr,
    },
    // RR D (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RR D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rr,
    },
    // RR E (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RR E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rr,
    },
    // RR H (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RR H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rr,
    },
    // RR L (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RR L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rr,
    },
    // RR (HL) (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RR (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_rr,
    },
    // RR A (2), Z 0 0 C
    InstructionDef {
        mnemonic: "RR A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_rr,
    },
    // SLA B (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SLA B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sla,
    },
    // SLA C (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SLA C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sla,
    },
    // SLA D (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SLA D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sla,
    },
    // SLA E (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SLA E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sla,
    },
    // SLA H (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SLA H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sla,
    },
    // SLA L (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SLA L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sla,
    },
    // SLA (HL) (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SLA (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_sla,
    },
    // SLA A (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SLA A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sla,
    },
    // SRA B (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SRA B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sra,
    },
    // SRA C (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SRA C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sra,
    },
    // SRA D (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SRA D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sra,
    },
    // SRA E (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SRA E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sra,
    },
    // SRA H (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SRA H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sra,
    },
    // SRA L (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SRA L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sra,
    },
    // SRA (HL) (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SRA (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_sra,
    },
    // SRA A (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SRA A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_sra,
    },
    // SWAP B (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SWAP B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_swap,
    },
    // SWAP C (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SWAP C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_swap,
    },
    // SWAP D (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SWAP D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_swap,
    },
    // SWAP E (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SWAP E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_swap,
    },
    // SWAP H (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SWAP H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_swap,
    },
    // SWAP L (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SWAP L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_swap,
    },
    // SWAP (HL) (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SWAP (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_swap,
    },
    // SWAP A (2), Z 0 0 0
    InstructionDef {
        mnemonic: "SWAP A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_swap,
    },
    // SRL B (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SRL B",
        operands: [Operand::Register(Register::B), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_srl,
    },
    // SRL C (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SRL C",
        operands: [Operand::Register(Register::C), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_srl,
    },
    // SRL D (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SRL D",
        operands: [Operand::Register(Register::D), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_srl,
    },
    // SRL E (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SRL E",
        operands: [Operand::Register(Register::E), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_srl,
    },
    // SRL H (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SRL H",
        operands: [Operand::Register(Register::H), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_srl,
    },
    // SRL L (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SRL L",
        operands: [Operand::Register(Register::L), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_srl,
    },
    // SRL (HL) (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SRL (HL)",
        operands: [Operand::RegisterPtr(Register::HL), Operand::None],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_srl,
    },
    // SRL A (2), Z 0 0 C
    InstructionDef {
        mnemonic: "SRL A",
        operands: [Operand::Register(Register::A), Operand::None],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_srl,
    },
    // BIT 0,B (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 0,B",
        operands: [Operand::Constant(0), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 0,C (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 0,C",
        operands: [Operand::Constant(0), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 0,D (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 0,D",
        operands: [Operand::Constant(0), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 0,E (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 0,E",
        operands: [Operand::Constant(0), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 0,H (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 0,H",
        operands: [Operand::Constant(0), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 0,L (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 0,L",
        operands: [Operand::Constant(0), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 0,(HL) (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 0,(HL)",
        operands: [Operand::Constant(0), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_bit,
    },
    // BIT 0,A (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 0,A",
        operands: [Operand::Constant(0), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 1,B (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 1,B",
        operands: [Operand::Constant(1), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 1,C (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 1,C",
        operands: [Operand::Constant(1), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 1,D (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 1,D",
        operands: [Operand::Constant(1), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 1,E (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 1,E",
        operands: [Operand::Constant(1), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 1,H (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 1,H",
        operands: [Operand::Constant(1), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 1,L (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 1,L",
        operands: [Operand::Constant(1), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 1,(HL) (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 1,(HL)",
        operands: [Operand::Constant(1), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_bit,
    },
    // BIT 1,A (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 1,A",
        operands: [Operand::Constant(1), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 2,B (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 2,B",
        operands: [Operand::Constant(2), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 2,C (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 2,C",
        operands: [Operand::Constant(2), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 2,D (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 2,D",
        operands: [Operand::Constant(2), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 2,E (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 2,E",
        operands: [Operand::Constant(2), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 2,H (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 2,H",
        operands: [Operand::Constant(2), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 2,L (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 2,L",
        operands: [Operand::Constant(2), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 2,(HL) (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 2,(HL)",
        operands: [Operand::Constant(2), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_bit,
    },
    // BIT 2,A (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 2,A",
        operands: [Operand::Constant(2), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 3,B (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 3,B",
        operands: [Operand::Constant(3), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 3,C (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 3,C",
        operands: [Operand::Constant(3), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 3,D (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 3,D",
        operands: [Operand::Constant(3), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 3,E (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 3,E",
        operands: [Operand::Constant(3), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 3,H (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 3,H",
        operands: [Operand::Constant(3), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 3,L (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 3,L",
        operands: [Operand::Constant(3), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 3,(HL) (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 3,(HL)",
        operands: [Operand::Constant(3), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_bit,
    },
    // BIT 3,A (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 3,A",
        operands: [Operand::Constant(3), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 4,B (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 4,B",
        operands: [Operand::Constant(4), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 4,C (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 4,C",
        operands: [Operand::Constant(4), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 4,D (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 4,D",
        operands: [Operand::Constant(4), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 4,E (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 4,E",
        operands: [Operand::Constant(4), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 4,H (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 4,H",
        operands: [Operand::Constant(4), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 4,L (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 4,L",
        operands: [Operand::Constant(4), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 4,(HL) (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 4,(HL)",
        operands: [Operand::Constant(4), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_bit,
    },
    // BIT 4,A (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 4,A",
        operands: [Operand::Constant(4), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 5,B (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 5,B",
        operands: [Operand::Constant(5), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 5,C (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 5,C",
        operands: [Operand::Constant(5), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 5,D (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 5,D",
        operands: [Operand::Constant(5), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 5,E (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 5,E",
        operands: [Operand::Constant(5), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 5,H (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 5,H",
        operands: [Operand::Constant(5), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 5,L (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 5,L",
        operands: [Operand::Constant(5), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 5,(HL) (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 5,(HL)",
        operands: [Operand::Constant(5), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_bit,
    },
    // BIT 5,A (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 5,A",
        operands: [Operand::Constant(5), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 6,B (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 6,B",
        operands: [Operand::Constant(6), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 6,C (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 6,C",
        operands: [Operand::Constant(6), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 6,D (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 6,D",
        operands: [Operand::Constant(6), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 6,E (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 6,E",
        operands: [Operand::Constant(6), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 6,H (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 6,H",
        operands: [Operand::Constant(6), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 6,L (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 6,L",
        operands: [Operand::Constant(6), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 6,(HL) (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 6,(HL)",
        operands: [Operand::Constant(6), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_bit,
    },
    // BIT 6,A (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 6,A",
        operands: [Operand::Constant(6), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 7,B (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 7,B",
        operands: [Operand::Constant(7), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 7,C (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 7,C",
        operands: [Operand::Constant(7), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 7,D (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 7,D",
        operands: [Operand::Constant(7), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 7,E (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 7,E",
        operands: [Operand::Constant(7), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 7,H (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 7,H",
        operands: [Operand::Constant(7), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 7,L (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 7,L",
        operands: [Operand::Constant(7), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // BIT 7,(HL) (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 7,(HL)",
        operands: [Operand::Constant(7), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_bit,
    },
    // BIT 7,A (2), Z 0 1 -
    InstructionDef {
        mnemonic: "BIT 7,A",
        operands: [Operand::Constant(7), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_bit,
    },
    // RES 0,B (2), - - - -
    InstructionDef {
        mnemonic: "RES 0,B",
        operands: [Operand::Constant(0), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 0,C (2), - - - -
    InstructionDef {
        mnemonic: "RES 0,C",
        operands: [Operand::Constant(0), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 0,D (2), - - - -
    InstructionDef {
        mnemonic: "RES 0,D",
        operands: [Operand::Constant(0), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 0,E (2), - - - -
    InstructionDef {
        mnemonic: "RES 0,E",
        operands: [Operand::Constant(0), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 0,H (2), - - - -
    InstructionDef {
        mnemonic: "RES 0,H",
        operands: [Operand::Constant(0), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 0,L (2), - - - -
    InstructionDef {
        mnemonic: "RES 0,L",
        operands: [Operand::Constant(0), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 0,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "RES 0,(HL)",
        operands: [Operand::Constant(0), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_res,
    },
    // RES 0,A (2), - - - -
    InstructionDef {
        mnemonic: "RES 0,A",
        operands: [Operand::Constant(0), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 1,B (2), - - - -
    InstructionDef {
        mnemonic: "RES 1,B",
        operands: [Operand::Constant(1), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 1,C (2), - - - -
    InstructionDef {
        mnemonic: "RES 1,C",
        operands: [Operand::Constant(1), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 1,D (2), - - - -
    InstructionDef {
        mnemonic: "RES 1,D",
        operands: [Operand::Constant(1), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 1,E (2), - - - -
    InstructionDef {
        mnemonic: "RES 1,E",
        operands: [Operand::Constant(1), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 1,H (2), - - - -
    InstructionDef {
        mnemonic: "RES 1,H",
        operands: [Operand::Constant(1), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 1,L (2), - - - -
    InstructionDef {
        mnemonic: "RES 1,L",
        operands: [Operand::Constant(1), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 1,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "RES 1,(HL)",
        operands: [Operand::Constant(1), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_res,
    },
    // RES 1,A (2), - - - -
    InstructionDef {
        mnemonic: "RES 1,A",
        operands: [Operand::Constant(1), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 2,B (2), - - - -
    InstructionDef {
        mnemonic: "RES 2,B",
        operands: [Operand::Constant(2), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 2,C (2), - - - -
    InstructionDef {
        mnemonic: "RES 2,C",
        operands: [Operand::Constant(2), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 2,D (2), - - - -
    InstructionDef {
        mnemonic: "RES 2,D",
        operands: [Operand::Constant(2), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 2,E (2), - - - -
    InstructionDef {
        mnemonic: "RES 2,E",
        operands: [Operand::Constant(2), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 2,H (2), - - - -
    InstructionDef {
        mnemonic: "RES 2,H",
        operands: [Operand::Constant(2), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 2,L (2), - - - -
    InstructionDef {
        mnemonic: "RES 2,L",
        operands: [Operand::Constant(2), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 2,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "RES 2,(HL)",
        operands: [Operand::Constant(2), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_res,
    },
    // RES 2,A (2), - - - -
    InstructionDef {
        mnemonic: "RES 2,A",
        operands: [Operand::Constant(2), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 3,B (2), - - - -
    InstructionDef {
        mnemonic: "RES 3,B",
        operands: [Operand::Constant(3), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 3,C (2), - - - -
    InstructionDef {
        mnemonic: "RES 3,C",
        operands: [Operand::Constant(3), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 3,D (2), - - - -
    InstructionDef {
        mnemonic: "RES 3,D",
        operands: [Operand::Constant(3), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 3,E (2), - - - -
    InstructionDef {
        mnemonic: "RES 3,E",
        operands: [Operand::Constant(3), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 3,H (2), - - - -
    InstructionDef {
        mnemonic: "RES 3,H",
        operands: [Operand::Constant(3), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 3,L (2), - - - -
    InstructionDef {
        mnemonic: "RES 3,L",
        operands: [Operand::Constant(3), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 3,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "RES 3,(HL)",
        operands: [Operand::Constant(3), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_res,
    },
    // RES 3,A (2), - - - -
    InstructionDef {
        mnemonic: "RES 3,A",
        operands: [Operand::Constant(3), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 4,B (2), - - - -
    InstructionDef {
        mnemonic: "RES 4,B",
        operands: [Operand::Constant(4), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 4,C (2), - - - -
    InstructionDef {
        mnemonic: "RES 4,C",
        operands: [Operand::Constant(4), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 4,D (2), - - - -
    InstructionDef {
        mnemonic: "RES 4,D",
        operands: [Operand::Constant(4), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 4,E (2), - - - -
    InstructionDef {
        mnemonic: "RES 4,E",
        operands: [Operand::Constant(4), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 4,H (2), - - - -
    InstructionDef {
        mnemonic: "RES 4,H",
        operands: [Operand::Constant(4), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 4,L (2), - - - -
    InstructionDef {
        mnemonic: "RES 4,L",
        operands: [Operand::Constant(4), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 4,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "RES 4,(HL)",
        operands: [Operand::Constant(4), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_res,
    },
    // RES 4,A (2), - - - -
    InstructionDef {
        mnemonic: "RES 4,A",
        operands: [Operand::Constant(4), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 5,B (2), - - - -
    InstructionDef {
        mnemonic: "RES 5,B",
        operands: [Operand::Constant(5), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 5,C (2), - - - -
    InstructionDef {
        mnemonic: "RES 5,C",
        operands: [Operand::Constant(5), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 5,D (2), - - - -
    InstructionDef {
        mnemonic: "RES 5,D",
        operands: [Operand::Constant(5), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 5,E (2), - - - -
    InstructionDef {
        mnemonic: "RES 5,E",
        operands: [Operand::Constant(5), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 5,H (2), - - - -
    InstructionDef {
        mnemonic: "RES 5,H",
        operands: [Operand::Constant(5), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 5,L (2), - - - -
    InstructionDef {
        mnemonic: "RES 5,L",
        operands: [Operand::Constant(5), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 5,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "RES 5,(HL)",
        operands: [Operand::Constant(5), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_res,
    },
    // RES 5,A (2), - - - -
    InstructionDef {
        mnemonic: "RES 5,A",
        operands: [Operand::Constant(5), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 6,B (2), - - - -
    InstructionDef {
        mnemonic: "RES 6,B",
        operands: [Operand::Constant(6), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 6,C (2), - - - -
    InstructionDef {
        mnemonic: "RES 6,C",
        operands: [Operand::Constant(6), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 6,D (2), - - - -
    InstructionDef {
        mnemonic: "RES 6,D",
        operands: [Operand::Constant(6), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 6,E (2), - - - -
    InstructionDef {
        mnemonic: "RES 6,E",
        operands: [Operand::Constant(6), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 6,H (2), - - - -
    InstructionDef {
        mnemonic: "RES 6,H",
        operands: [Operand::Constant(6), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 6,L (2), - - - -
    InstructionDef {
        mnemonic: "RES 6,L",
        operands: [Operand::Constant(6), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 6,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "RES 6,(HL)",
        operands: [Operand::Constant(6), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_res,
    },
    // RES 6,A (2), - - - -
    InstructionDef {
        mnemonic: "RES 6,A",
        operands: [Operand::Constant(6), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 7,B (2), - - - -
    InstructionDef {
        mnemonic: "RES 7,B",
        operands: [Operand::Constant(7), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 7,C (2), - - - -
    InstructionDef {
        mnemonic: "RES 7,C",
        operands: [Operand::Constant(7), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 7,D (2), - - - -
    InstructionDef {
        mnemonic: "RES 7,D",
        operands: [Operand::Constant(7), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 7,E (2), - - - -
    InstructionDef {
        mnemonic: "RES 7,E",
        operands: [Operand::Constant(7), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 7,H (2), - - - -
    InstructionDef {
        mnemonic: "RES 7,H",
        operands: [Operand::Constant(7), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 7,L (2), - - - -
    InstructionDef {
        mnemonic: "RES 7,L",
        operands: [Operand::Constant(7), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // RES 7,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "RES 7,(HL)",
        operands: [Operand::Constant(7), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_res,
    },
    // RES 7,A (2), - - - -
    InstructionDef {
        mnemonic: "RES 7,A",
        operands: [Operand::Constant(7), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_res,
    },
    // SET 0,B (2), - - - -
    InstructionDef {
        mnemonic: "SET 0,B",
        operands: [Operand::Constant(0), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 0,C (2), - - - -
    InstructionDef {
        mnemonic: "SET 0,C",
        operands: [Operand::Constant(0), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 0,D (2), - - - -
    InstructionDef {
        mnemonic: "SET 0,D",
        operands: [Operand::Constant(0), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 0,E (2), - - - -
    InstructionDef {
        mnemonic: "SET 0,E",
        operands: [Operand::Constant(0), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 0,H (2), - - - -
    InstructionDef {
        mnemonic: "SET 0,H",
        operands: [Operand::Constant(0), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 0,L (2), - - - -
    InstructionDef {
        mnemonic: "SET 0,L",
        operands: [Operand::Constant(0), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 0,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "SET 0,(HL)",
        operands: [Operand::Constant(0), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_set,
    },
    // SET 0,A (2), - - - -
    InstructionDef {
        mnemonic: "SET 0,A",
        operands: [Operand::Constant(0), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 1,B (2), - - - -
    InstructionDef {
        mnemonic: "SET 1,B",
        operands: [Operand::Constant(1), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 1,C (2), - - - -
    InstructionDef {
        mnemonic: "SET 1,C",
        operands: [Operand::Constant(1), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 1,D (2), - - - -
    InstructionDef {
        mnemonic: "SET 1,D",
        operands: [Operand::Constant(1), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 1,E (2), - - - -
    InstructionDef {
        mnemonic: "SET 1,E",
        operands: [Operand::Constant(1), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 1,H (2), - - - -
    InstructionDef {
        mnemonic: "SET 1,H",
        operands: [Operand::Constant(1), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 1,L (2), - - - -
    InstructionDef {
        mnemonic: "SET 1,L",
        operands: [Operand::Constant(1), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 1,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "SET 1,(HL)",
        operands: [Operand::Constant(1), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_set,
    },
    // SET 1,A (2), - - - -
    InstructionDef {
        mnemonic: "SET 1,A",
        operands: [Operand::Constant(1), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 2,B (2), - - - -
    InstructionDef {
        mnemonic: "SET 2,B",
        operands: [Operand::Constant(2), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 2,C (2), - - - -
    InstructionDef {
        mnemonic: "SET 2,C",
        operands: [Operand::Constant(2), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 2,D (2), - - - -
    InstructionDef {
        mnemonic: "SET 2,D",
        operands: [Operand::Constant(2), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 2,E (2), - - - -
    InstructionDef {
        mnemonic: "SET 2,E",
        operands: [Operand::Constant(2), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 2,H (2), - - - -
    InstructionDef {
        mnemonic: "SET 2,H",
        operands: [Operand::Constant(2), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 2,L (2), - - - -
    InstructionDef {
        mnemonic: "SET 2,L",
        operands: [Operand::Constant(2), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 2,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "SET 2,(HL)",
        operands: [Operand::Constant(2), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_set,
    },
    // SET 2,A (2), - - - -
    InstructionDef {
        mnemonic: "SET 2,A",
        operands: [Operand::Constant(2), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 3,B (2), - - - -
    InstructionDef {
        mnemonic: "SET 3,B",
        operands: [Operand::Constant(3), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 3,C (2), - - - -
    InstructionDef {
        mnemonic: "SET 3,C",
        operands: [Operand::Constant(3), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 3,D (2), - - - -
    InstructionDef {
        mnemonic: "SET 3,D",
        operands: [Operand::Constant(3), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 3,E (2), - - - -
    InstructionDef {
        mnemonic: "SET 3,E",
        operands: [Operand::Constant(3), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 3,H (2), - - - -
    InstructionDef {
        mnemonic: "SET 3,H",
        operands: [Operand::Constant(3), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 3,L (2), - - - -
    InstructionDef {
        mnemonic: "SET 3,L",
        operands: [Operand::Constant(3), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 3,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "SET 3,(HL)",
        operands: [Operand::Constant(3), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_set,
    },
    // SET 3,A (2), - - - -
    InstructionDef {
        mnemonic: "SET 3,A",
        operands: [Operand::Constant(3), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 4,B (2), - - - -
    InstructionDef {
        mnemonic: "SET 4,B",
        operands: [Operand::Constant(4), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 4,C (2), - - - -
    InstructionDef {
        mnemonic: "SET 4,C",
        operands: [Operand::Constant(4), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 4,D (2), - - - -
    InstructionDef {
        mnemonic: "SET 4,D",
        operands: [Operand::Constant(4), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 4,E (2), - - - -
    InstructionDef {
        mnemonic: "SET 4,E",
        operands: [Operand::Constant(4), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 4,H (2), - - - -
    InstructionDef {
        mnemonic: "SET 4,H",
        operands: [Operand::Constant(4), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 4,L (2), - - - -
    InstructionDef {
        mnemonic: "SET 4,L",
        operands: [Operand::Constant(4), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 4,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "SET 4,(HL)",
        operands: [Operand::Constant(4), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_set,
    },
    // SET 4,A (2), - - - -
    InstructionDef {
        mnemonic: "SET 4,A",
        operands: [Operand::Constant(4), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 5,B (2), - - - -
    InstructionDef {
        mnemonic: "SET 5,B",
        operands: [Operand::Constant(5), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 5,C (2), - - - -
    InstructionDef {
        mnemonic: "SET 5,C",
        operands: [Operand::Constant(5), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 5,D (2), - - - -
    InstructionDef {
        mnemonic: "SET 5,D",
        operands: [Operand::Constant(5), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 5,E (2), - - - -
    InstructionDef {
        mnemonic: "SET 5,E",
        operands: [Operand::Constant(5), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 5,H (2), - - - -
    InstructionDef {
        mnemonic: "SET 5,H",
        operands: [Operand::Constant(5), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 5,L (2), - - - -
    InstructionDef {
        mnemonic: "SET 5,L",
        operands: [Operand::Constant(5), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 5,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "SET 5,(HL)",
        operands: [Operand::Constant(5), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_set,
    },
    // SET 5,A (2), - - - -
    InstructionDef {
        mnemonic: "SET 5,A",
        operands: [Operand::Constant(5), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 6,B (2), - - - -
    InstructionDef {
        mnemonic: "SET 6,B",
        operands: [Operand::Constant(6), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 6,C (2), - - - -
    InstructionDef {
        mnemonic: "SET 6,C",
        operands: [Operand::Constant(6), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 6,D (2), - - - -
    InstructionDef {
        mnemonic: "SET 6,D",
        operands: [Operand::Constant(6), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 6,E (2), - - - -
    InstructionDef {
        mnemonic: "SET 6,E",
        operands: [Operand::Constant(6), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 6,H (2), - - - -
    InstructionDef {
        mnemonic: "SET 6,H",
        operands: [Operand::Constant(6), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 6,L (2), - - - -
    InstructionDef {
        mnemonic: "SET 6,L",
        operands: [Operand::Constant(6), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 6,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "SET 6,(HL)",
        operands: [Operand::Constant(6), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_set,
    },
    // SET 6,A (2), - - - -
    InstructionDef {
        mnemonic: "SET 6,A",
        operands: [Operand::Constant(6), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 7,B (2), - - - -
    InstructionDef {
        mnemonic: "SET 7,B",
        operands: [Operand::Constant(7), Operand::Register(Register::B)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 7,C (2), - - - -
    InstructionDef {
        mnemonic: "SET 7,C",
        operands: [Operand::Constant(7), Operand::Register(Register::C)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 7,D (2), - - - -
    InstructionDef {
        mnemonic: "SET 7,D",
        operands: [Operand::Constant(7), Operand::Register(Register::D)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 7,E (2), - - - -
    InstructionDef {
        mnemonic: "SET 7,E",
        operands: [Operand::Constant(7), Operand::Register(Register::E)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 7,H (2), - - - -
    InstructionDef {
        mnemonic: "SET 7,H",
        operands: [Operand::Constant(7), Operand::Register(Register::H)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 7,L (2), - - - -
    InstructionDef {
        mnemonic: "SET 7,L",
        operands: [Operand::Constant(7), Operand::Register(Register::L)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
    // SET 7,(HL) (2), - - - -
    InstructionDef {
        mnemonic: "SET 7,(HL)",
        operands: [Operand::Constant(7), Operand::RegisterPtr(Register::HL)],
        len: 2,
        cycles: [16, 16],
        func: CPU::op_set,
    },
    // SET 7,A (2), - - - -
    InstructionDef {
        mnemonic: "SET 7,A",
        operands: [Operand::Constant(7), Operand::Register(Register::A)],
        len: 2,
        cycles: [8, 8],
        func: CPU::op_set,
    },
];
