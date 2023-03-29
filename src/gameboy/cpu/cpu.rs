use anyhow::{bail, Result};
use std::borrow::Borrow;

use super::super::bus::bus::{Bus, BusIterator};
use super::alu;
use super::instruction::{Instruction, Operand};
use super::regs::{Flag, Register, RegisterFile, RegisterWidth};
use crate::tickable::Tickable;

// Interrupt flags
pub const INT_VBLANK: u8 = 1 << 0;
pub const INT_LCDSTAT: u8 = 1 << 1;
pub const INT_TIMER: u8 = 1 << 2;
pub const INT_SERIAL: u8 = 1 << 3;
pub const INT_JOYPAD: u8 = 1 << 4;

/// Return type of CPU::op_* functions
type CPUOpResult = Result<OpOk>;

/// Function signature of CPU::op_* functions
pub type CPUOpFn = fn(&mut CPU, &Instruction) -> CPUOpResult;

/// Result of a successful CPU::op_* function.
pub struct OpOk {
    /// New program counter
    pc: u16,

    /// Cycles taken
    cycles: usize,
}

impl OpOk {
    /// Normal successful op, moves PC to next
    /// instruction and always fixed cycle count.
    #[inline(always)]
    fn ok(cpu: &CPU, instr: &Instruction) -> Self {
        Self {
            pc: cpu.regs.pc + instr.len as u16,
            cycles: instr.def.cycles[0].into(),
        }
    }

    /// Branch op: successful op, branch not taken.
    #[inline(always)]
    fn no_branch(cpu: &CPU, instr: &Instruction) -> Self {
        Self {
            pc: cpu.regs.pc + instr.len as u16,
            cycles: instr.def.cycles[1].into(),
        }
    }

    /// Branch op: successful op, branch taken.
    #[inline(always)]
    fn branch(_cpu: &CPU, instr: &Instruction, pc: u16) -> Self {
        Self {
            pc,
            cycles: instr.def.cycles[0].into(),
        }
    }
}

/// Gameboy CPU
pub struct CPU {
    /// Address bus
    pub bus: Box<dyn Bus>,

    /// Register file
    pub regs: RegisterFile,

    /// Total amount of cycles
    cycles: usize,

    /// Interrupt Master Enable
    ime: bool,

    /// HALT instruction pauses CPU
    halted: bool,
}

impl CPU {
    /// IE register address on address bus
    const BUS_IE: u16 = 0xFFFF;

    /// IF register address on address bus
    const BUS_IF: u16 = 0xFF0F;

    pub fn new(bus: Box<dyn Bus>) -> Self {
        Self {
            bus,
            regs: RegisterFile::new(),
            cycles: 0,
            ime: false,
            halted: false,
        }
    }

    pub fn peek_next_instr(&self) -> Result<Instruction> {
        let mut busiter = BusIterator::new_from(self.bus.borrow(), self.regs.pc);
        Instruction::decode(&mut busiter)
    }

    fn service_interrupts(&mut self) {
        if !self.ime {
            return;
        }

        let inte = self.bus.read(Self::BUS_IE);
        let intf = self.bus.read(Self::BUS_IF);
        let service = intf & inte;

        let mut calli = |addr, flag: u8| {
            // 2 wait states
            self.cycles += 2;
            self.halted = false;

            self.bus.write(Self::BUS_IF, intf & !flag);
            self.stack_push(self.regs.pc);
            self.ime = false;
            self.regs.pc = addr;
        };

        if service & INT_VBLANK == INT_VBLANK {
            return calli(0x40, INT_VBLANK);
        }
    }

    pub fn step(&mut self) -> Result<usize> {
        self.service_interrupts();

        if self.halted {
            return Ok(1);
        }

        let instr = self.peek_next_instr()?;
        let result = (instr.def.func)(self, &instr)?;
        self.regs.pc = result.pc;
        self.cycles += result.cycles;
        Ok(result.cycles)
    }

    pub fn get_cycles(&self) -> usize {
        self.cycles
    }

    /// Pushes 16-bits onto the stack.
    fn stack_push(&mut self, val: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        self.bus.write16(self.regs.sp, val);
    }

    /// Pops 16-bits from the stack.
    fn stack_pop(&mut self) -> u16 {
        let val = self.bus.read16(self.regs.sp);
        self.regs.sp = self.regs.sp.wrapping_add(2);
        val
    }

    /// SET/RES generic implementation
    fn op_set_res(&mut self, instr: &Instruction, set: bool) -> CPUOpResult {
        // SET/RES const, _
        let Operand::Constant(bit) = instr.def.operands[0]
            else { bail!("Unknown first operand {:?}", instr.def.operands[0]) };

        // This is always an 8-bit operation.
        assert!((0..8).contains(&bit));

        let val = match instr.def.operands[1] {
            // SET/RES _, reg
            Operand::Register(reg) => self.regs.read8(reg)?,
            // SET/RES _, (reg)
            Operand::RegisterIndirect(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);
                self.bus.read(self.regs.read16(reg)?)
            }
            _ => todo!(),
        };

        let val = if set {
            val | (1 << bit)
        } else {
            val & !(1 << bit)
        };

        match instr.def.operands[1] {
            // SET/RES _, reg
            Operand::Register(reg) => self.regs.write8(reg, val)?,
            // SET/RES _, (reg)
            Operand::RegisterIndirect(reg) => self.bus.write(self.regs.read16(reg)?, val),
            _ => todo!(),
        }

        Ok(OpOk::ok(self, instr))
    }

    /// SET b,n - Set bit 'b' in 'n'
    pub fn op_set(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_set_res(instr, true)
    }

    /// RES b,n - Clear bit 'b' in 'n'
    pub fn op_res(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_set_res(instr, false)
    }

    /// SRL - Shift right
    pub fn op_srl(&mut self, instr: &Instruction) -> CPUOpResult {
        let result = match instr.def.operands[0] {
            Operand::Register(reg) => {
                let val = self.regs.read8(reg)?;
                let result = alu::shright_8b(val);
                self.regs.write8(reg, result.result)?;
                result
            }
            Operand::RegisterIndirect(reg) => {
                let addr = self.regs.read16(reg)?;
                let val = self.bus.read(addr);
                let result = alu::shright_8b(val);
                self.bus.write(addr, result.result);
                result
            }
            _ => unreachable!(),
        };

        self.regs.write_flags(&[
            (Flag::C, result.carry),
            (Flag::H, false),
            (Flag::N, false),
            (Flag::Z, (result.result == 0)),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// SWAP - Swap nibbles
    pub fn op_swap(&mut self, instr: &Instruction) -> CPUOpResult {
        let val = match instr.def.operands[0] {
            Operand::Register(reg) => {
                assert_eq!(reg.width(), RegisterWidth::EightBit);
                self.regs.read8(reg)?
            }
            Operand::RegisterIndirect(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);
                self.bus.read(self.regs.read16(reg)?)
            }
            _ => unreachable!(),
        };

        let val = (val >> 4) | ((val & 0x0F) << 4);

        match instr.def.operands[0] {
            Operand::Register(reg) => self.regs.write8(reg, val)?,
            Operand::RegisterIndirect(reg) => self.bus.write(self.regs.read16(reg)?, val),
            _ => unreachable!(),
        };

        self.regs.write_flags(&[
            (Flag::Z, val == 0),
            (Flag::N, false),
            (Flag::H, false),
            (Flag::C, false),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// SLA - Shift left
    pub fn op_sla(&mut self, instr: &Instruction) -> CPUOpResult {
        let result = match instr.def.operands[0] {
            Operand::Register(reg) => {
                let result = alu::shleft_8b(self.regs.read8(reg)?);
                self.regs.write8(reg, result.result)?;
                result
            }
            Operand::RegisterIndirect(reg) => {
                let addr = self.regs.read16(reg)?;
                let val = self.bus.read(addr);
                let result = alu::shleft_8b(val);
                self.bus.write(addr, result.result);
                result
            }
            _ => unreachable!(),
        };

        self.regs.write_flags(&[
            (Flag::C, result.carry),
            (Flag::H, false),
            (Flag::N, false),
            (Flag::Z, (result.result == 0)),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// SRA - Shift right (most significant bit unchanged)
    pub fn op_sra(&mut self, instr: &Instruction) -> CPUOpResult {
        let result = match instr.def.operands[0] {
            Operand::Register(reg) => {
                let val = self.regs.read8(reg)?;
                let result = alu::shright_8b(val);
                self.regs.write8(reg, result.result | (val & 0x80))?;
                result
            }
            Operand::RegisterIndirect(reg) => {
                let addr = self.regs.read16(reg)?;
                let val = self.bus.read(addr);
                let result = alu::shright_8b(val);
                self.bus.write(addr, result.result | (val & 0x80));
                result
            }
            _ => unreachable!(),
        };

        self.regs.write_flags(&[
            (Flag::C, result.carry),
            (Flag::H, false),
            (Flag::N, false),
            (Flag::Z, (result.result == 0)),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// BIT b,n - Test for bit 'b' in 'n'
    pub fn op_bit(&mut self, instr: &Instruction) -> CPUOpResult {
        // BIT const, _
        let Operand::Constant(bit) = instr.def.operands[0]
            else { bail!("Unknown first operand {:?}", instr.def.operands[0]) };

        // This is always an 8-bit operation.
        assert!((0..8).contains(&bit));

        let val = match instr.def.operands[1] {
            // BIT _, reg
            Operand::Register(reg) => self.regs.read8(reg)?,
            // BIT _, (reg)
            Operand::RegisterIndirect(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);
                self.bus.read(self.regs.read16(reg)?)
            }
            _ => unreachable!(),
        };

        self.regs.write_flags(&[
            (Flag::Z, (val & (1 << bit) == 0)),
            (Flag::H, true),
            (Flag::N, false),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// RL - Rotate left (carry = bit 9)
    pub fn op_rl(&mut self, instr: &Instruction) -> CPUOpResult {
        let result = match instr.def.operands[0] {
            Operand::Register(reg) => {
                let result = alu::rotleft_9b(self.regs.read8(reg)?, self.regs.test_flag(Flag::C));
                self.regs.write8(reg, result.result)?;
                result
            }
            Operand::RegisterIndirect(reg) => {
                let addr = self.regs.read16(reg)?;
                let val = self.bus.read(addr);
                let result = alu::rotleft_9b(val, self.regs.test_flag(Flag::C));
                self.bus.write(addr, result.result);
                result
            }
            _ => unreachable!(),
        };

        self.regs.write_flags(&[
            (Flag::C, result.carry),
            (Flag::H, false),
            (Flag::N, false),
            (Flag::Z, (result.result == 0)),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// RLA - Rotate Left, A register
    pub fn op_rla(&mut self, instr: &Instruction) -> CPUOpResult {
        let result = alu::rotleft_9b(self.regs.read8(Register::A)?, self.regs.test_flag(Flag::C));
        self.regs.write8(Register::A, result.result)?;

        self.regs.write_flags(&[
            (Flag::C, result.carry),
            (Flag::H, false),
            (Flag::N, false),
            // This seems weird?
            (Flag::Z, false),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// RLC - Rotate left (copy to carry)
    pub fn op_rlc(&mut self, instr: &Instruction) -> CPUOpResult {
        let result = match instr.def.operands[0] {
            Operand::Register(reg) => {
                let result = alu::rotleft_8b(self.regs.read8(reg)?);
                self.regs.write8(reg, result.result)?;
                result
            }
            Operand::RegisterIndirect(reg) => {
                let addr = self.regs.read16(reg)?;
                let val = self.bus.read(addr);
                let result = alu::rotleft_8b(val);
                self.bus.write(addr, result.result);
                result
            }
            _ => unreachable!(),
        };

        self.regs.write_flags(&[
            (Flag::C, result.carry),
            (Flag::H, false),
            (Flag::N, false),
            (Flag::Z, (result.result == 0)),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    pub fn op_rlca(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    /// RR - Rotate Right (bit 9 = carry)
    pub fn op_rr(&mut self, instr: &Instruction) -> CPUOpResult {
        let result = match instr.def.operands[0] {
            Operand::Register(reg) => {
                let result = alu::rotright_9b(self.regs.read8(reg)?, self.regs.test_flag(Flag::C));
                self.regs.write8(reg, result.result)?;
                result
            }
            Operand::RegisterIndirect(reg) => {
                let addr = self.regs.read16(reg)?;
                let val = self.bus.read(addr);
                let result = alu::rotright_9b(val, self.regs.test_flag(Flag::C));
                self.bus.write(addr, result.result);
                result
            }
            _ => unreachable!(),
        };

        self.regs.write_flags(&[
            (Flag::C, result.carry),
            (Flag::H, false),
            (Flag::N, false),
            (Flag::Z, (result.result == 0)),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    pub fn op_rra(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    /// RRC - Rotate Right (copy to carry)
    pub fn op_rrc(&mut self, instr: &Instruction) -> CPUOpResult {
        let result = match instr.def.operands[0] {
            Operand::Register(reg) => {
                let result = alu::rotright_8b(self.regs.read8(reg)?);
                self.regs.write8(reg, result.result)?;
                result
            }
            Operand::RegisterIndirect(reg) => {
                let addr = self.regs.read16(reg)?;
                let val = self.bus.read(addr);
                let result = alu::rotright_8b(val);
                self.bus.write(addr, result.result);
                result
            }
            _ => unreachable!(),
        };

        self.regs.write_flags(&[
            (Flag::C, result.carry),
            (Flag::H, false),
            (Flag::N, false),
            (Flag::Z, (result.result == 0)),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    pub fn op_rrca(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    /// EI - Enable Interrupts
    pub fn op_ei(&mut self, instr: &Instruction) -> CPUOpResult {
        self.ime = true;
        Ok(OpOk::ok(self, instr))
    }

    /// DI - Disable Interrupts
    pub fn op_di(&mut self, instr: &Instruction) -> CPUOpResult {
        self.ime = false;
        Ok(OpOk::ok(self, instr))
    }

    /// RST - Call
    pub fn op_rst(&mut self, instr: &Instruction) -> CPUOpResult {
        let next_addr = self.regs.pc.wrapping_add(instr.len as u16);
        self.stack_push(next_addr);

        let Operand::Constant(new_addr) = instr.def.operands[0]
            else { unreachable!() };

        Ok(OpOk::branch(self, instr, new_addr.into()))
    }

    /// NOP - No Operation
    pub fn op_nop(&mut self, instr: &Instruction) -> CPUOpResult {
        Ok(OpOk::ok(self, instr))
    }

    pub fn op_stop(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    /// HALT - Halts execution until interrupt or reset
    pub fn op_halt(&mut self, instr: &Instruction) -> CPUOpResult {
        self.halted = true;

        Ok(OpOk::ok(self, instr))
    }

    /// LD - Load Register
    pub fn op_ld(&mut self, instr: &Instruction) -> CPUOpResult {
        let indreg = |r: Register, regval| match r.width() {
            RegisterWidth::SixteenBit => regval,
            RegisterWidth::EightBit => {
                assert_eq!(regval & !0xFF, 0);
                0xFF00 | regval
            }
        };

        // Source operand
        let val: u16 = match &instr.def.operands[1] {
            // LD _, imm8
            Operand::Immediate8 => instr.imm8(1)?.into(),
            // LD _, (imm8)
            Operand::ImmediateIndirect8 => self.bus.read(0xFF00_u16 | instr.imm8(1)? as u16).into(),
            // LD _, (imm16)
            Operand::ImmediateIndirect16 => self.bus.read(instr.imm16(1)?).into(),
            // LD _, imm16
            Operand::Immediate16 => instr.imm16(1)?,
            // LD _, reg
            Operand::Register(reg) => self.regs.read(*reg),
            // LD _, (reg)
            Operand::RegisterIndirect(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);
                self.bus.read(self.regs.read(*reg)).into()
            }
            // LD _, (reg+)
            Operand::RegisterIndirectInc(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);
                self.bus.read(self.regs.read_inc(*reg)?).into()
            }
            // LD _, (reg-)
            Operand::RegisterIndirectDec(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);
                self.bus.read(self.regs.read_dec(*reg)?).into()
            }
            _ => todo!(),
        };

        // Destination operand
        match instr.def.operands[0] {
            // LD reg, _
            Operand::Register(dest) => self.regs.write(dest, val.try_into()?)?,
            // LD (reg), _
            Operand::RegisterIndirect(dest) => {
                let addr = self.regs.read(dest);
                self.bus.write(indreg(dest, addr), val.try_into()?)
            }
            // LD (reg-), _
            Operand::RegisterIndirectDec(dest) => {
                let addr = self.regs.read_dec(dest)?;
                self.bus.write(indreg(dest, addr), val.try_into()?)
            }
            // LD (reg+), _
            Operand::RegisterIndirectInc(dest) => {
                let addr = self.regs.read_inc(dest)?;
                self.bus.write(indreg(dest, addr), val.try_into()?)
            }
            // LDH (a8), _
            Operand::ImmediateIndirect8 => {
                let addr = 0xFF00_u16 + instr.imm8(0)? as u16;
                self.bus.write(addr, val.try_into()?)
            }
            // LDH (a16), _
            Operand::ImmediateIndirect16 => self.bus.write(instr.imm16(0)?, val.try_into()?),
            _ => bail!("Invalid first operand: {:?}", instr.def.operands[0]),
        }

        Ok(OpOk::ok(self, instr))
    }

    /// SCF - Set carry flag
    pub fn op_scf(&mut self, instr: &Instruction) -> CPUOpResult {
        self.regs.write_flags(&[(Flag::C, true)]);

        Ok(OpOk::ok(self, instr))
    }

    /// CCF - Flip carry flag
    pub fn op_ccf(&mut self, instr: &Instruction) -> CPUOpResult {
        self.regs
            .write_flags(&[(Flag::C, !self.regs.test_flag(Flag::C))]);

        Ok(OpOk::ok(self, instr))
    }

    /// CP - Compare
    pub fn op_cp(&mut self, instr: &Instruction) -> CPUOpResult {
        let val = match instr.def.operands[0] {
            // CP imm8
            Operand::Immediate8 => instr.imm8(0)?,
            // CP reg8
            Operand::Register(reg) => {
                assert_eq!(reg.width(), RegisterWidth::EightBit);
                self.regs.read8(reg)?
            }
            // CP (reg16)
            Operand::RegisterIndirect(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);
                self.bus.read(self.regs.read16(reg)?)
            }
            _ => unreachable!(),
        };

        let result = alu::sub_8b(self.regs.read8(Register::A)?, val);
        self.regs.write_flags(&[
            (Flag::Z, result.result == 0),
            (Flag::H, result.halfcarry),
            (Flag::C, result.carry),
            (Flag::N, true),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// CPL - One's complement (of A)
    pub fn op_cpl(&mut self, instr: &Instruction) -> CPUOpResult {
        self.regs
            .write8(Register::A, !self.regs.read8(Register::A)?)?;
        self.regs.write_flags(&[(Flag::H, true), (Flag::N, true)]);

        Ok(OpOk::ok(self, instr))
    }

    /// OR - Bitwise OR
    pub fn op_or(&mut self, instr: &Instruction) -> CPUOpResult {
        let a = self.regs.read8(Register::A)?;
        let val = match instr.def.operands[0] {
            // OR reg
            Operand::Register(r) => self.regs.read8(r)?,
            // OR (reg)
            Operand::RegisterIndirect(r) => {
                assert_eq!(r.width(), RegisterWidth::SixteenBit);
                self.bus.read(self.regs.read16(r)?)
            }
            // OR imm8
            Operand::Immediate8 => instr.imm8(0)?,
            _ => todo!(),
        };
        let result = a | val;
        self.regs.write(Register::A, result.into())?;
        self.regs.write_flags(&[
            (Flag::Z, result == 0),
            (Flag::C, false),
            (Flag::H, false),
            (Flag::N, false),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// XOR - Bitwise XOR
    pub fn op_xor(&mut self, instr: &Instruction) -> CPUOpResult {
        let a = self.regs.read8(Register::A)?;
        let val = match instr.def.operands[0] {
            // XOR reg
            Operand::Register(r) => self.regs.read8(r)?,
            // XOR imm8
            Operand::Immediate8 => instr.imm8(0)?,
            // XOR (reg)
            Operand::RegisterIndirect(r) => {
                assert_eq!(r.width(), RegisterWidth::SixteenBit);
                self.bus.read(self.regs.read16(r)?)
            }
            _ => todo!(),
        };
        let result = a ^ val;
        self.regs.write(Register::A, result.into())?;
        self.regs.write_flags(&[
            (Flag::Z, result == 0),
            (Flag::C, false),
            (Flag::H, false),
            (Flag::N, false),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// AND - Bitwise AND
    pub fn op_and(&mut self, instr: &Instruction) -> CPUOpResult {
        let a = self.regs.read8(Register::A)?;
        let val = match instr.def.operands[0] {
            // AND reg
            Operand::Register(r) => self.regs.read8(r)?,
            // AND (reg)
            Operand::RegisterIndirect(r) => {
                assert_eq!(r.width(), RegisterWidth::SixteenBit);
                self.bus.read(self.regs.read16(r)?)
            }
            // AND imm8
            Operand::Immediate8 => instr.imm8(0)?,
            _ => todo!(),
        };
        let result = a & val;
        self.regs.write(Register::A, result.into())?;
        self.regs.write_flags(&[
            (Flag::Z, result == 0),
            (Flag::C, false),
            (Flag::H, true),
            (Flag::N, false),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// PUSH - Push register onto stack
    pub fn op_push(&mut self, instr: &Instruction) -> CPUOpResult {
        let Operand::Register(reg) = instr.def.operands[0]
            else { unreachable!() };
        assert_eq!(reg.width(), RegisterWidth::SixteenBit);

        self.stack_push(self.regs.read16(reg)?);
        Ok(OpOk::ok(self, instr))
    }

    /// POP - Pop register from stack
    pub fn op_pop(&mut self, instr: &Instruction) -> CPUOpResult {
        let Operand::Register(reg) = instr.def.operands[0]
            else { unreachable!() };
        assert_eq!(reg.width(), RegisterWidth::SixteenBit);

        let val = self.stack_pop();
        self.regs.write(reg, val)?;
        Ok(OpOk::ok(self, instr))
    }

    pub fn op_adc(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_daa(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    /// ADD - Add (8-bit)
    pub fn op_add(&mut self, instr: &Instruction) -> CPUOpResult {
        // First operand is always A
        let left = match instr.def.operands[0] {
            Operand::Register(reg) => {
                assert_eq!(reg, Register::A);
                self.regs.read8(reg)?
            }
            _ => unreachable!(),
        };

        let right = match instr.def.operands[1] {
            Operand::RegisterIndirect(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);
                self.bus.read(self.regs.read16(reg)?)
            }
            Operand::Register(reg) => {
                assert_eq!(reg.width(), RegisterWidth::EightBit);
                self.regs.read8(reg)?
            }
            Operand::Immediate8 => instr.imm8(1)?,
            _ => unreachable!(),
        };

        let result = alu::add_8b(left, right);
        self.regs.write8(Register::A, result.result)?;
        self.regs.write_flags(&[
            (Flag::Z, result.result == 0),
            (Flag::C, result.carry),
            (Flag::H, result.halfcarry),
            (Flag::N, false),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// ADD - Add (16-bit)
    pub fn op_add_16b(&mut self, instr: &Instruction) -> CPUOpResult {
        let left = match instr.def.operands[0] {
            Operand::Register(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);
                self.regs.read16(reg)?
            }
            _ => unreachable!(),
        };

        let right = match instr.def.operands[1] {
            Operand::Register(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);
                self.regs.read16(reg)?
            }
            _ => unreachable!(),
        };

        let result = alu::add_16b(left, right);

        let Operand::Register(destreg) = instr.def.operands[0]
            else { unreachable!() };
        self.regs.write(destreg, result.result)?;
        self.regs.write_flags(&[
            (Flag::Z, result.result == 0),
            (Flag::C, result.carry),
            (Flag::H, result.halfcarry),
            (Flag::N, false),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// ADD SP,r8 - Add to SP
    pub fn op_add_sp_r8(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    /// SUB - Subtract (8-bit)
    pub fn op_sub(&mut self, instr: &Instruction) -> CPUOpResult {
        let val: u8 = match instr.def.operands[0] {
            Operand::Register(reg) => {
                assert_eq!(reg.width(), RegisterWidth::EightBit);
                self.regs.read8(reg)?
            }
            Operand::RegisterIndirect(reg) => self.bus.read(self.regs.read16(reg)?),
            Operand::Immediate8 => instr.imm8(0)?,
            _ => unreachable!(),
        };

        let res = alu::sub_8b(self.regs.read8(Register::A)?, val);

        self.regs.write8(Register::A, res.result)?;
        self.regs.write_flags(&[
            (Flag::Z, (res.result == 0)),
            (Flag::N, true),
            (Flag::C, res.carry),
            (Flag::H, res.halfcarry),
        ]);

        Ok(OpOk::ok(self, instr))
    }

    /// DEC - Decrement (8-bit)
    pub fn op_dec_8b(&mut self, instr: &Instruction) -> CPUOpResult {
        match instr.def.operands[0] {
            Operand::Register(reg) => {
                let res = alu::sub_8b(self.regs.read8(reg)?, 1);
                self.regs.write8(reg, res.result)?;
                self.regs.write_flags(&[
                    (Flag::H, res.halfcarry),
                    (Flag::N, true),
                    (Flag::Z, (res.result == 0)),
                    // Carry not used
                ]);
            }
            Operand::RegisterIndirect(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);

                let addr = self.regs.read16(reg)?;
                let res = alu::sub_8b(self.bus.read(addr), 1);
                self.bus.write(addr, res.result);
                self.regs.write_flags(&[
                    (Flag::H, res.halfcarry),
                    (Flag::N, true),
                    (Flag::Z, (res.result == 0)),
                    // Carry not used
                ]);
            }
            _ => unreachable!(),
        }

        Ok(OpOk::ok(self, instr))
    }

    /// DEC - Decrement (16-bit)
    pub fn op_dec_16b(&mut self, instr: &Instruction) -> CPUOpResult {
        let Operand::Register(reg) = instr.def.operands[0]
            else { unreachable!() };

        assert_eq!(reg.width(), RegisterWidth::SixteenBit);
        self.regs
            .write(reg, self.regs.read16(reg)?.wrapping_sub(1))?;

        Ok(OpOk::ok(self, instr))
    }

    /// INC - Increment (8-bit)
    pub fn op_inc_8b(&mut self, instr: &Instruction) -> CPUOpResult {
        match instr.def.operands[0] {
            Operand::Register(reg) => {
                let res = alu::add_8b(self.regs.read8(reg)?, 1);
                self.regs.write8(reg, res.result)?;
                self.regs.write_flags(&[
                    (Flag::H, res.halfcarry),
                    (Flag::N, false),
                    (Flag::Z, (res.result == 0)),
                    // Carry not used
                ]);
            }
            Operand::RegisterIndirect(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);

                let addr = self.regs.read16(reg)?;
                let res = alu::add_8b(self.bus.read(addr), 1);
                self.bus.write(addr, res.result);
                self.regs.write_flags(&[
                    (Flag::H, res.halfcarry),
                    (Flag::N, false),
                    (Flag::Z, (res.result == 0)),
                    // Carry not used
                ]);
            }
            _ => unreachable!(),
        }

        Ok(OpOk::ok(self, instr))
    }

    /// INC - Increment (16-bit)
    pub fn op_inc_16b(&mut self, instr: &Instruction) -> CPUOpResult {
        let Operand::Register(reg) = instr.def.operands[0]
            else { unreachable!() };

        assert_eq!(reg.width(), RegisterWidth::SixteenBit);
        self.regs
            .write(reg, self.regs.read16(reg)?.wrapping_add(1))?;

        Ok(OpOk::ok(self, instr))
    }

    /// JR _, s8 - Jump Relative (conditional/unconditional)
    fn op_jr_cc(&mut self, instr: &Instruction, cc: bool) -> CPUOpResult {
        if !cc {
            return Ok(OpOk::no_branch(self, instr));
        }

        // value = address - 2.
        let rel_addr = instr.imms8(0)? + 2;
        let new_pc = self.regs.pc.wrapping_add_signed(rel_addr.into());
        Ok(OpOk::branch(self, instr, new_pc))
    }

    /// JR s8 - Jump Relative (unconditionally)
    pub fn op_jr(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_jr_cc(instr, true)
    }

    /// JR NC s8 - Jump Relative (if not carry)
    pub fn op_jr_nc(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_jr_cc(instr, !self.regs.test_flag(Flag::C))
    }

    /// JR C s8 - Jump Relative (if carry)
    pub fn op_jr_c(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_jr_cc(instr, self.regs.test_flag(Flag::C))
    }

    /// JR NZ s8 - Jump Relative (if not zero)
    pub fn op_jr_nz(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_jr_cc(instr, !self.regs.test_flag(Flag::Z))
    }

    /// JR Z s8 - Jump Relative (if zero)
    pub fn op_jr_z(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_jr_cc(instr, self.regs.test_flag(Flag::Z))
    }

    /// JP _ - Jump Absolute (conditional/unconditional)
    fn op_jp_cc(&mut self, instr: &Instruction, cc: bool) -> CPUOpResult {
        if !cc {
            return Ok(OpOk::no_branch(self, instr));
        }

        let new_pc = match instr.def.operands[0] {
            Operand::ImmediateIndirect16 => instr.imm16(0)?,
            Operand::RegisterIndirect(reg) => {
                assert_eq!(reg.width(), RegisterWidth::SixteenBit);
                self.regs.read(reg)
            }
            _ => todo!(),
        };
        Ok(OpOk::branch(self, instr, new_pc))
    }

    /// JP _ - Jump Absolute (unconditionally)
    pub fn op_jp(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_jp_cc(instr, true)
    }

    /// JP NC _ - Jump Absolute (if not carry)
    pub fn op_jp_nc(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_jp_cc(instr, !self.regs.test_flag(Flag::C))
    }

    /// JP C _ - Jump Absolute (if carry)
    pub fn op_jp_c(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_jp_cc(instr, self.regs.test_flag(Flag::C))
    }

    /// JP NZ _ - Jump Absolute (if not zero)
    pub fn op_jp_nz(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_jp_cc(instr, !self.regs.test_flag(Flag::Z))
    }

    /// JP Z _ - Jump Absolute (if zero)
    pub fn op_jp_z(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_jp_cc(instr, self.regs.test_flag(Flag::Z))
    }

    /// CALL cc - Call (conditional/unconditional)
    fn op_call_cc(&mut self, instr: &Instruction, cc: bool) -> CPUOpResult {
        if !cc {
            return Ok(OpOk::no_branch(self, instr));
        }

        let next_addr = self.regs.pc.wrapping_add(instr.len as u16);
        self.stack_push(next_addr);

        Ok(OpOk::branch(self, instr, instr.imm16(0)?))
    }

    /// CALL - Call (unconditional)
    pub fn op_call(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_call_cc(instr, true)
    }

    /// CALL - Call (if carry)
    pub fn op_call_c(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_call_cc(instr, self.regs.test_flag(Flag::C))
    }

    /// CALL - Call (if not carry)
    pub fn op_call_nc(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_call_cc(instr, !self.regs.test_flag(Flag::C))
    }

    /// CALL - Call (if not zero)
    pub fn op_call_nz(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_call_cc(instr, !self.regs.test_flag(Flag::Z))
    }

    /// CALL - Call (if zero)
    pub fn op_call_z(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_call_cc(instr, self.regs.test_flag(Flag::Z))
    }

    /// RET cc - Return (conditional/unconditional)
    fn op_ret_cc(&mut self, instr: &Instruction, cc: bool) -> CPUOpResult {
        if !cc {
            return Ok(OpOk::no_branch(self, instr));
        }

        let ret_addr = self.stack_pop();
        Ok(OpOk::branch(self, instr, ret_addr))
    }

    /// RET - Return (unconditional)
    pub fn op_ret(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_ret_cc(instr, true)
    }

    /// RET NC - Return (if not carry)
    pub fn op_ret_nc(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_ret_cc(instr, !self.regs.test_flag(Flag::C))
    }

    /// RET C - Return (if zero)
    pub fn op_ret_c(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_ret_cc(instr, self.regs.test_flag(Flag::C))
    }

    /// RET NZ - Return (if not zero)
    pub fn op_ret_nz(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_ret_cc(instr, !self.regs.test_flag(Flag::Z))
    }

    /// RET Z - Return (if zero)
    pub fn op_ret_z(&mut self, instr: &Instruction) -> CPUOpResult {
        self.op_ret_cc(instr, self.regs.test_flag(Flag::Z))
    }

    /// RETI - Return from interrupt
    pub fn op_reti(&mut self, instr: &Instruction) -> CPUOpResult {
        self.ime = true;
        self.op_ret_cc(instr, true)
    }

    pub fn op_sbc(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_prefix_cb(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_invalid(&mut self, _instr: &Instruction) -> CPUOpResult {
        panic!("Invalid opcode");
    }
}

impl Tickable for CPU {
    fn tick(&mut self, ticks: usize) -> Result<usize> {
        let cycles = self.step()?;

        self.bus.tick(ticks * cycles)?;

        Ok(ticks * cycles)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::bus::testbus::Testbus;
    use super::*;

    fn cpu(code: &[u8]) -> CPU {
        let bus = Testbus::from(code);
        CPU::new(Box::new(bus))
    }

    fn cpu_run(cpu: &mut CPU) {
        cpu.step().unwrap();
    }

    fn run(code: &[u8]) -> CPU {
        let mut cpu = cpu(code);
        cpu_run(&mut cpu);
        cpu
    }

    fn run_reg(code: &[u8], reg: Register, val: u16) -> CPU {
        let mut cpu = cpu(code);
        cpu.regs.write(reg, val).unwrap();
        cpu_run(&mut cpu);
        cpu
    }

    fn run_flags(code: &[u8], flags: &[Flag]) -> CPU {
        let mut cpu = cpu(code);
        cpu.regs.write_flags(
            &flags
                .into_iter()
                .map(|&f| (f, true))
                .collect::<Vec<(Flag, bool)>>(),
        );
        cpu_run(&mut cpu);
        cpu
    }

    fn run_reg_flags(code: &[u8], reg: Register, val: u16, flags: &[Flag]) -> CPU {
        let mut cpu = cpu(code);
        cpu.regs.write(reg, val).unwrap();
        cpu.regs.write_flags(
            &flags
                .into_iter()
                .map(|&f| (f, true))
                .collect::<Vec<(Flag, bool)>>(),
        );
        cpu_run(&mut cpu);
        cpu
    }

    #[test]
    fn op_ld_reg_imm16() {
        let cpu = run(&[0x31, 0x34, 0x12]); // LD SP,0x1234
        assert_eq!(cpu.regs.sp, 0x1234);
    }

    #[test]
    fn op_ld_reg_imm8() {
        let cpu = run(&[0x3E, 0x12]); // LD A,0x12
        assert_eq!(cpu.regs.a, 0x12);
    }

    #[test]
    fn op_xor_reg() {
        let mut c = cpu(&[0xA8]); // XOR B
        c.regs.a = 0x55;
        c.regs.b = 0xAA;
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0xFF);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let mut c = cpu(&[0xA8]); // XOR B
        c.regs.a = 0xAA;
        c.regs.b = 0xAA;
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_ld_indreg8_reg() {
        let mut c = cpu(&[0xE2]); // LD (C),A
        c.regs.c = 0x11;
        c.regs.a = 0x5A;
        cpu_run(&mut c);
        assert_eq!(c.bus.read(0xFF11), 0x5A);
    }

    #[test]
    fn op_ld_indreg16_reg() {
        let mut c = cpu(&[0x70]); // LD (HL),B
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.regs.b = 0x5A;
        cpu_run(&mut c);
        assert_eq!(c.bus.read(0x1122), 0x5A);
    }

    #[test]
    fn op_ld_indreg16_dec_reg() {
        let mut c = cpu(&[0x32]); // LD (HL-),A
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.regs.a = 0x5A;
        cpu_run(&mut c);
        assert_eq!((c.regs.h, c.regs.l), (0x11, 0x21));
        assert_eq!(c.bus.read(0x1122), 0x5A);
    }

    #[test]
    fn op_ld_indreg16_inc_reg() {
        let mut c = cpu(&[0x22]); // LD (HL+),A
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.regs.a = 0x5A;
        cpu_run(&mut c);
        assert_eq!((c.regs.h, c.regs.l), (0x11, 0x23));
        assert_eq!(c.bus.read(0x1122), 0x5A);
    }

    #[test]
    fn op_ld_indimm8_reg() {
        let c = run_reg(&[0xE0, 0x5A], Register::A, 0x12);
        assert_eq!(c.bus.read(0xFF5A), 0x12);
    }

    #[test]
    fn op_ld_indimm16_reg() {
        let c = run_reg(&[0xEA, 0x55, 0xAA], Register::A, 0x12);
        assert_eq!(c.bus.read(0xAA55), 0x12);
    }

    #[test]
    fn op_ld_reg_reg() {
        let mut c = cpu(&[0x78]); // LD A,B
        c.regs.b = 0x55;
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x55);
    }

    #[test]
    fn op_ld_reg_indreg16() {
        let mut c = cpu(&[0x1A]); // LD A,(DE)
        c.regs.write(Register::DE, 0x1122).unwrap();
        c.bus.write(0x1122, 0x5A);
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x5A);
    }

    #[test]
    fn op_ld_reg_indimm8() {
        let mut c = cpu(&[0xF0, 0x22]); // LD A,(imm8)
        c.bus.write(0xFF22, 0x5A);
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x5A);
    }

    #[test]
    fn op_set_reg() {
        let c = run(&[0xCB, 0xC7]); // SET 0,A
        assert_eq!(c.regs.a, 0x01);

        let c = run(&[0xCB, 0xFB]); // SET 7,E
        assert_eq!(c.regs.e, 0x80);

        let mut c = cpu(&[0xCB, 0xE2]); // SET 4,D
        c.regs.d = 0x0F;
        cpu_run(&mut c);
        assert_eq!(c.regs.d, 0x1F);
    }

    #[test]
    fn op_set_indreg() {
        let c = run_reg(&[0xCB, 0xC6], Register::HL, 0x1122); // SET 0,(HL)
        assert_eq!(c.bus.read(0x1122), 0x01);
    }

    #[test]
    fn op_res_reg() {
        let mut c = cpu(&[0xCB, 0x80]); // RES 0,B
        c.regs.b = 0xFF;
        cpu_run(&mut c);
        assert_eq!(c.regs.b, 0xFE);

        let c = run(&[0xCB, 0x81]); // RES 0,C
        assert_eq!(c.regs.c, 0x00);
    }

    #[test]
    fn op_res_indreg() {
        let mut c = cpu(&[0xCB, 0x86]); // RES 0,(HL)
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.bus.write(0x1122, 0xFF);
        cpu_run(&mut c);
        assert_eq!(c.bus.read(0x1122), 0xFE);
    }

    #[test]
    fn op_bit_reg() {
        let mut c = cpu(&[0xCB, 0x47]); // BIT 0,A
        c.regs.a = 0x01;
        cpu_run(&mut c);
        assert!(
            !c.regs.test_flag(Flag::Z) && c.regs.test_flag(Flag::H) && !c.regs.test_flag(Flag::N)
        );

        let c = run(&[0xCB, 0x61]); // BIT 4,C
        assert!(
            c.regs.test_flag(Flag::Z) && c.regs.test_flag(Flag::H) && !c.regs.test_flag(Flag::N)
        );
    }

    #[test]
    fn op_bit_indreg() {
        let mut c = cpu(&[0xCB, 0x46]); // BIT 0,(HL)
        c.regs.write(Register::HL, 0x1122).unwrap();
        c.bus.write(0x1122, 0x01);
        cpu_run(&mut c);
        assert!(
            !c.regs.test_flag(Flag::Z) && c.regs.test_flag(Flag::H) && !c.regs.test_flag(Flag::N)
        );

        let c = run_reg(&[0xCB, 0x66], Register::HL, 0x1122); // BIT 4,(HL)
        assert!(
            c.regs.test_flag(Flag::Z) && c.regs.test_flag(Flag::H) && !c.regs.test_flag(Flag::N)
        );
    }

    #[test]
    fn op_jr() {
        let c = run(&[0x18, (-10_i8 - 2) as u8]); // JR -10
        assert_eq!(c.regs.pc, -10_i16 as u16);

        let c = run(&[0x18, 10 - 2]); // JR 10
        assert_eq!(c.regs.pc, 10);
    }

    #[test]
    fn op_jr_nz() {
        let c = run(&[0x20, 10 - 2]); // JR NZ 10
        assert_eq!(c.regs.pc, 10);
        assert_eq!(c.cycles, 12);

        let c = run_flags(
            &[0x20, 10 - 2], // JR NZ 10
            &[Flag::Z],
        );
        assert_ne!(c.regs.pc, 10);
        assert_eq!(c.cycles, 8);
    }

    #[test]
    fn op_jr_z() {
        let c = run(&[0x28, 10 - 2]); // JR Z 10
        assert_ne!(c.regs.pc, 10);
        assert_eq!(c.cycles, 8);

        let c = run_flags(
            &[0x28, 10 - 2], // JR Z 10
            &[Flag::Z],
        );
        assert_eq!(c.regs.pc, 10);
        assert_eq!(c.cycles, 12);
    }

    #[test]
    fn op_jr_nc() {
        let c = run(&[0x30, 10 - 2]); // JR NC 10
        assert_eq!(c.regs.pc, 10);
        assert_eq!(c.cycles, 12);

        let c = run_flags(
            &[0x30, 10 - 2], // JR NC 10
            &[Flag::C],
        );
        assert_ne!(c.regs.pc, 10);
        assert_eq!(c.cycles, 8);
    }

    #[test]
    fn op_jr_c() {
        let c = run(&[0x38, 10 - 2]); // JR C 10
        assert_ne!(c.regs.pc, 10);
        assert_eq!(c.cycles, 8);

        let c = run_flags(
            &[0x38, 10 - 2], // JR C 10
            &[Flag::C],
        );
        assert_eq!(c.regs.pc, 10);
        assert_eq!(c.cycles, 12);
    }

    #[test]
    fn op_inc_reg() {
        let c = run_reg(&[0x3C], Register::A, 0x00);
        assert_eq!(c.regs.a, 1);
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::N));

        let c = run_reg(&[0x3C], Register::A, 0x0F);
        assert_eq!(c.regs.a, 0x10);
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::N));

        let c = run_reg(&[0x3C], Register::A, 0xFF);
        assert_eq!(c.regs.a, 0);
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_call() {
        let mut c = cpu(&[]);
        c.bus.write_slice(&[0xCD, 0x34, 0x12], 0x8000);
        c.regs.pc = 0x8000;
        c.regs.sp = 0xFFFE;
        cpu_run(&mut c);
        assert_eq!(c.regs.pc, 0x1234);
        assert_eq!(c.regs.sp, 0xFFFC);
        assert_eq!(c.bus.read16(0xFFFC), 0x8003);

        // Wrapping past 0
        let c = run(&[0xCD, 0x34, 0x12]);
        assert_eq!(c.regs.pc, 0x1234);
        assert_eq!(c.regs.sp, 0xFFFE);
        assert_eq!(c.bus.read16(0xFFFE), 0x0003);
    }

    #[test]
    fn op_call_c() {
        let c = run(&[0xDC, 0x34, 0x12]);
        assert_ne!(c.regs.pc, 0x1234);
        assert_eq!(c.cycles, 12);

        let c = run_flags(&[0xDC, 0x34, 0x12], &[Flag::C]);
        assert_eq!(c.regs.pc, 0x1234);
        assert_eq!(c.cycles, 24);
    }

    #[test]
    fn op_call_nc() {
        let c = run(&[0xD4, 0x34, 0x12]);
        assert_eq!(c.regs.pc, 0x1234);
        assert_eq!(c.cycles, 24);

        let c = run_flags(&[0xD4, 0x34, 0x12], &[Flag::C]);
        assert_ne!(c.regs.pc, 0x1234);
        assert_eq!(c.cycles, 12);
    }

    #[test]
    fn op_call_z() {
        let c = run(&[0xCC, 0x34, 0x12]);
        assert_ne!(c.regs.pc, 0x1234);
        assert_eq!(c.cycles, 12);

        let c = run_flags(&[0xCC, 0x34, 0x12], &[Flag::Z]);
        assert_eq!(c.regs.pc, 0x1234);
        assert_eq!(c.cycles, 24);
    }

    #[test]
    fn op_call_nz() {
        let c = run(&[0xC4, 0x34, 0x12]);
        assert_eq!(c.regs.pc, 0x1234);
        assert_eq!(c.cycles, 24);

        let c = run_flags(&[0xC4, 0x34, 0x12], &[Flag::Z]);
        assert_ne!(c.regs.pc, 0x1234);
        assert_eq!(c.cycles, 12);
    }

    #[test]
    fn op_push() {
        let c = run_reg(&[0xC5], Register::BC, 0xABCD);
        assert_ne!(c.regs.sp, 0);
        assert_eq!(c.bus.read16(c.regs.sp), 0xABCD);
    }

    #[test]
    fn op_pop() {
        let mut c = cpu(&[0xC1]);
        c.stack_push(0xABCD);
        assert_ne!(c.regs.sp, 0);
        cpu_run(&mut c);
        assert_eq!(c.regs.sp, 0);
        assert_eq!(c.regs.read16(Register::BC).unwrap(), 0xABCD);
    }

    #[test]
    fn op_rl_reg() {
        let c = run_reg(&[0xCB, 0x10], Register::B, 0x80); // RL B
        assert_eq!(c.regs.b, 0x00);
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x10], Register::B, 0x40); // RL B
        assert_eq!(c.regs.b, 0x80);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x10], Register::B, 0x00); // RL B
        assert_eq!(c.regs.b, 0x00);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_rr_reg() {
        let c = run_reg(&[0xCB, 0x18], Register::B, 0x01); // RR B
        assert_eq!(c.regs.b, 0x00);
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x18], Register::B, 0x40); // RR B
        assert_eq!(c.regs.b, 0x20);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x18], Register::B, 0x00); // RR B
        assert_eq!(c.regs.b, 0x00);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_rl_indreg() {
        let mut c = cpu(&[0xCB, 0x16]); // RL (HL)
        c.regs.write(Register::HL, 0x1122).unwrap();
        c.bus.write(0x1122, 0x40);
        cpu_run(&mut c);
        assert_eq!(c.bus.read(0x1122), 0x80);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_rr_indreg() {
        let mut c = cpu(&[0xCB, 0x1E]); // RR (HL)
        c.regs.write(Register::HL, 0x1122).unwrap();
        c.bus.write(0x1122, 0x40);
        cpu_run(&mut c);
        assert_eq!(c.bus.read(0x1122), 0x20);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_rla() {
        let c = run_reg(&[0x17], Register::A, 0x80);
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0x17], Register::A, 0x40);
        assert_eq!(c.regs.a, 0x80);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0x17], Register::A, 0x00);
        assert_eq!(c.regs.a, 0x00);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg_flags(&[0x17], Register::A, 0x95, &[Flag::C]);
        assert_eq!(c.regs.a, 0x2B);
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_rlc_reg() {
        let c = run_reg(&[0xCB, 0x00], Register::B, 0x80); // RLC B
        assert_eq!(c.regs.b, 0x01);
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x00], Register::B, 0x40); // RLC B
        assert_eq!(c.regs.b, 0x80);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x00], Register::B, 0x00); // RLC B
        assert_eq!(c.regs.b, 0x00);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_rlc_indreg() {
        let mut c = cpu(&[0xCB, 0x06]); // RLC (HL)
        c.regs.write(Register::HL, 0x1122).unwrap();
        c.bus.write(0x1122, 0x80);
        cpu_run(&mut c);
        assert_eq!(c.bus.read(0x1122), 0x01);
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_rrc_reg() {
        let c = run_reg(&[0xCB, 0x08], Register::B, 0x01); // RRC B
        assert_eq!(c.regs.b, 0x80);
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x08], Register::B, 0x80); // RRC B
        assert_eq!(c.regs.b, 0x40);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x08], Register::B, 0x00); // RRC B
        assert_eq!(c.regs.b, 0x00);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_rrc_indreg() {
        let mut c = cpu(&[0xCB, 0x0E]); // RRC (HL)
        c.regs.write(Register::HL, 0x1122).unwrap();
        c.bus.write(0x1122, 0x01);
        cpu_run(&mut c);
        assert_eq!(c.bus.read(0x1122), 0x80);
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_dec_8b() {
        let c = run_reg(&[0x3D], Register::A, 0x00); // DEC A
        assert_eq!(c.regs.a, 0xFF);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0x3D], Register::A, 0x01); // DEC A
        assert_eq!(c.regs.a, 0x00);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_dec_8b_ind() {
        let mut c = cpu(&[0x35]); // DEC (HL)
        c.regs.write(Register::HL, 0x1122).unwrap();
        cpu_run(&mut c);
        assert_eq!(c.bus.read(0x1122), 0xFF);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let mut c = cpu(&[0x35]); // DEC (HL)
        c.regs.write(Register::HL, 0x1122).unwrap();
        c.bus.write(0x1122, 1);
        cpu_run(&mut c);
        assert_eq!(c.bus.read(0x1122), 0);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_inc_16b() {
        let c = run_reg(&[0x23], Register::HL, 0x00);
        assert_eq!(c.regs.l, 0x01);
        assert_eq!(c.regs.h, 0x00);

        let c = run_reg(&[0x23], Register::HL, 0xFFFF);
        assert_eq!(c.regs.l, 0x00);
        assert_eq!(c.regs.h, 0x00);
    }

    #[test]
    fn op_ret() {
        let mut c = cpu(&[0xC9]);
        c.stack_push(0xABCD);
        cpu_run(&mut c);
        assert_eq!(c.regs.pc, 0xABCD);
    }

    #[test]
    fn op_reti() {
        let mut c = cpu(&[0xD9]);
        c.stack_push(0xABCD);
        cpu_run(&mut c);
        assert_eq!(c.regs.pc, 0xABCD);
        assert!(c.ime);
    }

    #[test]
    fn op_ret_z() {
        let mut c = cpu(&[0xC8]);
        c.regs.write_flags(&[(Flag::Z, true)]);
        c.stack_push(0xABCD);
        cpu_run(&mut c);
        assert_eq!(c.regs.pc, 0xABCD);
        assert_eq!(c.cycles, 20);

        let mut c = cpu(&[0xC8]);
        c.regs.write_flags(&[(Flag::Z, false)]);
        c.stack_push(0xABCD);
        cpu_run(&mut c);
        assert_ne!(c.regs.pc, 0xABCD);
        assert_eq!(c.cycles, 8);
    }

    #[test]
    fn op_ret_nz() {
        let mut c = cpu(&[0xC0]);
        c.regs.write_flags(&[(Flag::Z, true)]);
        c.stack_push(0xABCD);
        cpu_run(&mut c);
        assert_ne!(c.regs.pc, 0xABCD);
        assert_eq!(c.cycles, 8);

        let mut c = cpu(&[0xC0]);
        c.regs.write_flags(&[(Flag::Z, false)]);
        c.stack_push(0xABCD);
        cpu_run(&mut c);
        assert_eq!(c.regs.pc, 0xABCD);
        assert_eq!(c.cycles, 20);
    }

    #[test]
    fn op_ret_c() {
        let mut c = cpu(&[0xD8]);
        c.regs.write_flags(&[(Flag::C, true)]);
        c.stack_push(0xABCD);
        cpu_run(&mut c);
        assert_eq!(c.regs.pc, 0xABCD);
        assert_eq!(c.cycles, 20);

        let mut c = cpu(&[0xD8]);
        c.regs.write_flags(&[(Flag::C, false)]);
        c.stack_push(0xABCD);
        cpu_run(&mut c);
        assert_ne!(c.regs.pc, 0xABCD);
        assert_eq!(c.cycles, 8);
    }

    #[test]
    fn op_ret_nc() {
        let mut c = cpu(&[0xD0]);
        c.regs.write_flags(&[(Flag::C, true)]);
        c.stack_push(0xABCD);
        cpu_run(&mut c);
        assert_ne!(c.regs.pc, 0xABCD);
        assert_eq!(c.cycles, 8);

        let mut c = cpu(&[0xD0]);
        c.regs.write_flags(&[(Flag::C, false)]);
        c.stack_push(0xABCD);
        cpu_run(&mut c);
        assert_eq!(c.regs.pc, 0xABCD);
        assert_eq!(c.cycles, 20);
    }

    #[test]
    fn op_cp_imm8() {
        let c = run_reg(&[0xFE, 0x2F], Register::A, 0x3C);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));

        let c = run_reg(&[0xFE, 0x3C], Register::A, 0x3C);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));

        let c = run_reg(&[0xFE, 0x40], Register::A, 0x3C);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_cp_reg8() {
        let c = run_reg(&[0xB8], Register::B, 0x3C);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));

        let c = run_reg(&[0xB8], Register::B, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_cp_indreg16() {
        let mut c = cpu(&[0xBE]);
        (c.regs.h, c.regs.l) = (0x08, 0x00);
        c.bus.write(0x0800, 0x3C);
        cpu_run(&mut c);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));

        let c = run_reg(&[0xBE], Register::HL, 0x0800);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_sub_reg8() {
        let mut c = cpu(&[0x90]); // SUB B
        c.regs.write8(Register::A, 0x3E).unwrap();
        c.regs.write8(Register::B, 0x3E).unwrap();
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));

        let mut c = cpu(&[0x90]); // SUB B
        c.regs.write8(Register::A, 0x3E).unwrap();
        c.regs.write8(Register::B, 0x0F).unwrap();
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x2F);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));

        let mut c = cpu(&[0x90]); // SUB B
        c.regs.write8(Register::A, 0x3E).unwrap();
        c.regs.write8(Register::B, 0x40).unwrap();
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0xFE);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_sub_imm8() {
        let mut c = cpu(&[0xD6, 0x3E]); // SUB $3E
        c.regs.write8(Register::A, 0x3E).unwrap();
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_sub_indreg() {
        let mut c = cpu(&[0x96]); // SUB (HL)
        c.regs.write8(Register::A, 0x3E).unwrap();
        c.bus.write(0x1122, 0x3E);
        c.regs.write(Register::HL, 0x1122).unwrap();
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_add_reg() {
        let mut c = cpu(&[0x80]); // ADD A,B
        c.regs.write8(Register::A, 0x3A).unwrap();
        c.regs.write8(Register::B, 0xC6).unwrap();
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::N));

        let mut c = cpu(&[0x80]); // ADD A,B
        c.regs.write8(Register::A, 0x3C).unwrap();
        c.regs.write8(Register::B, 0xFF).unwrap();
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x3B);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_add_indreg() {
        let mut c = cpu(&[0x86]); // ADD A,(HL)
        c.regs.write8(Register::A, 0x3A).unwrap();
        c.regs.write(Register::HL, 0x55AA).unwrap();
        c.bus.write(0x55AA, 0xC6);
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::N));

        let mut c = cpu(&[0x86]); // ADD A,(HL)
        c.regs.write8(Register::A, 0x3C).unwrap();
        c.regs.write(Register::HL, 0x55AA).unwrap();
        c.bus.write(0x55AA, 0xFF);
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x3B);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_add_imm8() {
        let c = run_reg(&[0xC6, 0xC6], Register::A, 0x3A); // ADD A, 0xC6
        assert_eq!(c.regs.a, 0);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::N));

        let c = run_reg(&[0xC6, 0xFF], Register::A, 0x3C); // ADD A, 0xFF
        assert_eq!(c.regs.a, 0x3B);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_halt() {
        let mut c = run(&[0x76]);
        assert!(c.halted);
        let pc = c.regs.pc;
        c.step().unwrap();
        assert_eq!(pc, c.regs.pc);
    }

    #[test]
    fn op_nop() {
        run(&[0x00]);
    }

    #[test]
    fn op_jp_imm16() {
        let c = run(&[0xC3, 0xBB, 0xAA]);
        assert_eq!(c.regs.pc, 0xAABB);
    }

    #[test]
    fn op_jp_indreg16() {
        let c = run_reg(&[0xE9], Register::HL, 0xAABB); // JP (HL)
        assert_eq!(c.regs.pc, 0xAABB);
    }

    #[test]
    fn op_jp_c_imm16() {
        let c = run(&[0xDA, 0xBB, 0xAA]);
        assert_ne!(c.regs.pc, 0xAABB);
        assert_eq!(c.cycles, 12);

        let c = run_flags(&[0xDA, 0xBB, 0xAA], &[Flag::C]);
        assert_eq!(c.regs.pc, 0xAABB);
        assert_eq!(c.cycles, 16);
    }

    #[test]
    fn op_jp_nc_imm16() {
        let c = run(&[0xD2, 0xBB, 0xAA]);
        assert_eq!(c.regs.pc, 0xAABB);
        assert_eq!(c.cycles, 16);

        let c = run_flags(&[0xD2, 0xBB, 0xAA], &[Flag::C]);
        assert_ne!(c.regs.pc, 0xAABB);
        assert_eq!(c.cycles, 12);
    }

    #[test]
    fn op_jp_z_imm16() {
        let c = run(&[0xCA, 0xBB, 0xAA]);
        assert_ne!(c.regs.pc, 0xAABB);
        assert_eq!(c.cycles, 12);

        let c = run_flags(&[0xCA, 0xBB, 0xAA], &[Flag::Z]);
        assert_eq!(c.regs.pc, 0xAABB);
        assert_eq!(c.cycles, 16);
    }

    #[test]
    fn op_jp_nz_imm16() {
        let c = run(&[0xC2, 0xBB, 0xAA]);
        assert_eq!(c.regs.pc, 0xAABB);
        assert_eq!(c.cycles, 16);

        let c = run_flags(&[0xC2, 0xBB, 0xAA], &[Flag::Z]);
        assert_ne!(c.regs.pc, 0xAABB);
        assert_eq!(c.cycles, 12);
    }

    #[test]
    fn op_ei() {
        let c = run(&[0xFB]);
        assert!(c.ime);
    }

    #[test]
    fn op_di() {
        let mut c = cpu(&[0xF3]);
        c.ime = true;
        cpu_run(&mut c);
        assert!(!c.ime);
    }

    #[test]
    fn op_ld_reg_indreg16_inc() {
        let mut c = cpu(&[0x2A]); // LD A,(HL+)
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.bus.write(0x1122, 0x5A);
        cpu_run(&mut c);
        assert_eq!((c.regs.h, c.regs.l), (0x11, 0x23));
        assert_eq!(c.regs.a, 0x5A);
    }

    #[test]
    fn op_ld_reg_indreg16_dec() {
        let mut c = cpu(&[0x3A]); // LD A,(HL-)
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.bus.write(0x1122, 0x5A);
        cpu_run(&mut c);
        assert_eq!((c.regs.h, c.regs.l), (0x11, 0x21));
        assert_eq!(c.regs.a, 0x5A);
    }

    #[test]
    fn op_xor_indreg() {
        let mut c = cpu(&[0xAE]); // XOR (HL)
        c.regs.a = 0x55;
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.bus.write(0x1122, 0xAA);
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0xFF);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let mut c = cpu(&[0xAE]); // XOR (HL)
        c.regs.a = 0xAA;
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.bus.write(0x1122, 0xAA);
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_xor_imm8() {
        let c = run_reg(&[0xEE, 0xAA], Register::A, 0x55); // XOR 0xAA
        assert_eq!(c.regs.a, 0xFF);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let c = run_reg(&[0xEE, 0xAA], Register::A, 0xAA); // XOR 0xAA
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_or_reg() {
        let mut c = cpu(&[0xB0]); // OR B
        c.regs.a = 0x55;
        c.regs.b = 0xAA;
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0xFF);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let mut c = cpu(&[0xB0]); // OR B
        c.regs.a = 0xAA;
        c.regs.b = 0xAA;
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0xAA);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let c = run(&[0xB0]); // OR B
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_or_imm8() {
        let c = run_reg(&[0xF6, 0xAA], Register::A, 0x55); // OR 0xAA
        assert_eq!(c.regs.a, 0xFF);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let c = run_reg(&[0xF6, 0xAA], Register::A, 0xAA); // OR 0xAA
        assert_eq!(c.regs.a, 0xAA);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let c = run(&[0xF6, 0x00]); // OR 0x00
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_or_indreg() {
        let mut c = cpu(&[0xB6]); // OR (HL)
        c.regs.a = 0x55;
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.bus.write(0x1122, 0xAA);
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0xFF);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let mut c = cpu(&[0xB6]); // OR (HL)
        c.regs.a = 0xAA;
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.bus.write(0x1122, 0xAA);
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0xAA);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_ld_reg_indimm16() {
        let mut c = cpu(&[0xFA, 0x22, 0x11]); // LD A,(imm16)
        c.bus.write(0x1122, 0x5A);
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x5A);
    }

    #[test]
    fn op_and_reg() {
        let mut c = cpu(&[0xA0]); // AND B
        c.regs.a = 0x55;
        c.regs.b = 0xAA;
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let mut c = cpu(&[0xA0]); // AND B
        c.regs.a = 0xAA;
        c.regs.b = 0xAA;
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0xAA);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let c = run_reg(&[0xA0], Register::A, 0xFF); // AND B
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_and_imm8() {
        let c = run_reg(&[0xE6, 0xAA], Register::A, 0x55); // AND 0xAA
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let c = run_reg(&[0xE6, 0xAA], Register::A, 0xAA); // AND 0xAA
        assert_eq!(c.regs.a, 0xAA);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let c = run_reg(&[0xE6, 0x00], Register::A, 0xAA); // AND 0x00
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_and_indreg() {
        let mut c = cpu(&[0xA6]); // AND (HL)
        c.regs.a = 0x55;
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.bus.write(0x1122, 0xAA);
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));

        let mut c = cpu(&[0xA6]); // OR (HL)
        c.regs.a = 0xAA;
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.bus.write(0x1122, 0xAA);
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0xAA);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_dec_16b() {
        let c = run_reg(&[0x2B], Register::HL, 0x00); // DEC HL
        assert_eq!(c.regs.l, 0xFF);
        assert_eq!(c.regs.h, 0xFF);

        let c = run_reg(&[0x2B], Register::HL, 0xFFFF); // DEC HL
        assert_eq!(c.regs.l, 0xFE);
        assert_eq!(c.regs.h, 0xFF);
    }

    #[test]
    fn op_cpl() {
        let c = run_reg(&[0x2F], Register::A, 0x35);
        assert_eq!(c.regs.a, 0xCA);
        assert!(c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::H));
    }

    #[test]
    fn op_swap_reg() {
        let c = run_reg(&[0xCB, 0x30], Register::B, 0xAB); // SWAP B
        assert_eq!(c.regs.b, 0xBA);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));

        let c = run(&[0xCB, 0x30]); // SWAP B
        assert_eq!(c.regs.b, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
    }

    #[test]
    fn op_swap_indreg() {
        let mut c = cpu(&[0xCB, 0x36]); // SWAP (HL)
        (c.regs.h, c.regs.l) = (0x11, 0x22);
        c.bus.write(0x1122, 0xAB);
        cpu_run(&mut c);
        assert_eq!(c.bus.read(0x1122), 0xBA);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
    }

    #[test]
    fn op_rst() {
        let c = run(&[0xC7]); // RST 00H
        assert_eq!(c.regs.pc, 0x0000);
        assert_ne!(c.regs.sp, 0x0000);
        let c = run(&[0xD7]); // RST 10H
        assert_eq!(c.regs.pc, 0x0010);
        assert_ne!(c.regs.sp, 0x0000);
        let c = run(&[0xE7]); // RST 20H
        assert_eq!(c.regs.pc, 0x0020);
        assert_ne!(c.regs.sp, 0x0000);
        let c = run(&[0xF7]); // RST 30H
        assert_eq!(c.regs.pc, 0x0030);
        assert_ne!(c.regs.sp, 0x0000);

        let c = run(&[0xCF]); // RST 08H
        assert_eq!(c.regs.pc, 0x0008);
        assert_ne!(c.regs.sp, 0x0000);
        let c = run(&[0xDF]); // RST 18H
        assert_eq!(c.regs.pc, 0x0018);
        assert_ne!(c.regs.sp, 0x0000);
        let c = run(&[0xEF]); // RST 28H
        assert_eq!(c.regs.pc, 0x0028);
        assert_ne!(c.regs.sp, 0x0000);
        let c = run(&[0xFF]); // RST 38H
        assert_eq!(c.regs.pc, 0x0038);
        assert_ne!(c.regs.sp, 0x0000);
    }

    #[test]
    fn op_add_16b() {
        let mut c = cpu(&[0x09]); // ADD HL,BC
        c.regs.write(Register::HL, 0x8A23).unwrap();
        c.regs.write(Register::BC, 0x0605).unwrap();
        cpu_run(&mut c);
        assert_eq!(c.regs.read16(Register::HL).unwrap(), 0x9028);
        assert!(!c.regs.test_flag(Flag::Z));
        assert!(c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::N));
    }

    #[test]
    fn op_scf() {
        let c = run(&[0x37]);
        assert!(c.regs.test_flag(Flag::C));
    }

    #[test]
    fn op_ccf() {
        let c = run(&[0x3F]);
        assert!(c.regs.test_flag(Flag::C));

        let c = run_flags(&[0x3F], &[Flag::C]);
        assert!(!c.regs.test_flag(Flag::C));
    }

    #[test]
    fn op_sla_reg() {
        let c = run_reg(&[0xCB, 0x20], Register::B, 0x84); // SLA B
        assert!(c.regs.test_flag(Flag::C));
        assert_eq!(c.regs.b, 0x08);
    }

    #[test]
    fn op_sla_indreg() {
        let mut c = cpu(&[0xCB, 0x26]); // SLA (HL)
        c.regs.write(Register::HL, 0x1122).unwrap();
        c.bus.write(0x1122, 0x82);
        cpu_run(&mut c);
        assert!(c.regs.test_flag(Flag::C));
        assert_eq!(c.bus.read(0x1122), 0x04);
    }

    #[test]
    fn op_sra_reg() {
        let c = run_reg(&[0xCB, 0x28], Register::B, 0x81); // SRA B
        assert!(c.regs.test_flag(Flag::C));
        assert_eq!(c.regs.b, 0xC0);
    }

    #[test]
    fn op_sra_indreg() {
        let mut c = cpu(&[0xCB, 0x2E]); // SRA (HL)
        c.regs.write(Register::HL, 0x1122).unwrap();
        c.bus.write(0x1122, 0x82);
        cpu_run(&mut c);
        assert!(!c.regs.test_flag(Flag::C));
        assert_eq!(c.bus.read(0x1122), 0xC1);
    }

    #[test]
    fn op_srl_reg() {
        let c = run_reg(&[0xCB, 0x38], Register::B, 0x81); // SRL B
        assert!(c.regs.test_flag(Flag::C));
        assert_eq!(c.regs.b, 0x40);
    }

    #[test]
    fn op_srl_indreg() {
        let mut c = cpu(&[0xCB, 0x3E]); // SRL (HL)
        c.regs.write(Register::HL, 0x1122).unwrap();
        c.bus.write(0x1122, 0x82);
        cpu_run(&mut c);
        assert!(!c.regs.test_flag(Flag::C));
        assert_eq!(c.bus.read(0x1122), 0x41);
    }

    #[test]
    fn interrupt_vblank() {
        let mut c = cpu(&[0x00]); // NOP
        c.ime = true;
        c.bus.write(0xFFFF, 1); // IE
        c.bus.write(0xFF0F, 1); // IF
        cpu_run(&mut c);
        assert_eq!(c.regs.pc, 0x41);
        assert!(!c.ime);
        assert_eq!(c.bus.read(0xFF0F), 0);
    }
}
