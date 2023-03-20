use anyhow::{bail, Result};

use super::super::bus::bus::{Bus, BusIterator};
use super::alu;
use super::instruction::{Instruction, Operand};
use super::regs::{Flag, Register, RegisterFile, RegisterWidth};

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

    /// Interrupts enabled
    interrupts: bool,
}

impl CPU {
    /// IE register address on address bus
    const BUS_IE: u16 = 0xFFFF;

    pub fn new(bus: Box<dyn Bus>) -> Self {
        Self {
            bus,
            regs: RegisterFile::new(),
            cycles: 0,
            interrupts: false,
        }
    }

    pub fn peek_next_instr(&self) -> Result<Instruction> {
        let mut busiter = BusIterator::new_from(&self.bus, self.regs.pc);
        Instruction::decode(&mut busiter)
    }

    pub fn step(&mut self) -> Result<()> {
        let instr = self.peek_next_instr()?;
        let result = (instr.def.func)(self, &instr)?;
        self.regs.pc = result.pc;
        self.cycles += result.cycles;
        Ok(())
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

    pub fn op_srl(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_swap(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_sla(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_sra(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
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
            _ => todo!(),
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
            _ => todo!(),
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
            _ => todo!(),
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

    pub fn op_rr(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_rra(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_rrc(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_rrca(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    /// EI - Enable Interrupts
    pub fn op_ei(&mut self, instr: &Instruction) -> CPUOpResult {
        self.interrupts = true;
        Ok(OpOk::ok(self, instr))
    }

    /// DI - Disable Interrupts
    pub fn op_di(&mut self, instr: &Instruction) -> CPUOpResult {
        self.interrupts = false;
        Ok(OpOk::ok(self, instr))
    }

    pub fn op_rst(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    /// NOP - No Operation
    pub fn op_nop(&mut self, instr: &Instruction) -> CPUOpResult {
        Ok(OpOk::ok(self, instr))
    }

    pub fn op_stop(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_halt(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
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

    pub fn op_scf(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_ccf(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
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

    pub fn op_cpl(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    pub fn op_or(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
    }

    /// XOR - Bitwise XOR
    pub fn op_xor(&mut self, instr: &Instruction) -> CPUOpResult {
        let a = self.regs.read8(Register::A)?;
        let val = match instr.def.operands[0] {
            // XOR reg
            Operand::Register(r) => self.regs.read8(r)?,
            _ => todo!(),
        };
        let result = a ^ val;
        self.regs.write(Register::A, result.into())?;
        self.regs.write_flags(&[(Flag::Z, result == 0)]);

        Ok(OpOk::ok(self, instr))
    }

    pub fn op_and(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
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

    /// SUB - Subtract (8-bit)
    pub fn op_sub(&mut self, instr: &Instruction) -> CPUOpResult {
        let val: u8 = match instr.def.operands[0] {
            Operand::Register(reg) => {
                assert_eq!(reg.width(), RegisterWidth::EightBit);
                self.regs.read8(reg)?
            }
            _ => todo!(),
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
            _ => todo!(),
        }

        Ok(OpOk::ok(self, instr))
    }

    pub fn op_dec_16b(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
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
            _ => todo!(),
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

    pub fn op_reti(&mut self, _instr: &Instruction) -> CPUOpResult {
        todo!();
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

        let mut c = cpu(&[0xA8]); // XOR B
        c.regs.a = 0xAA;
        c.regs.b = 0xAA;
        cpu_run(&mut c);
        assert_eq!(c.regs.a, 0x00);
        assert!(c.regs.test_flag(Flag::Z));
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
    fn op_res_reg() {
        let mut c = cpu(&[0xCB, 0x80]); // RES 0,B
        c.regs.b = 0xFF;
        cpu_run(&mut c);
        assert_eq!(c.regs.b, 0xFE);

        let c = run(&[0xCB, 0x81]); // RES 0,C
        assert_eq!(c.regs.c, 0x00);
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
        let c = run_reg(&[0xCB, 0x10], Register::B, 0x80);
        assert_eq!(c.regs.b, 0x00);
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x10], Register::B, 0x40);
        assert_eq!(c.regs.b, 0x80);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x10], Register::B, 0x00);
        assert_eq!(c.regs.b, 0x00);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::Z));
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
        let c = run_reg(&[0xCB, 0x00], Register::B, 0x80);
        assert_eq!(c.regs.b, 0x01);
        assert!(c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x00], Register::B, 0x40);
        assert_eq!(c.regs.b, 0x80);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0xCB, 0x00], Register::B, 0x00);
        assert_eq!(c.regs.b, 0x00);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(!c.regs.test_flag(Flag::H));
        assert!(!c.regs.test_flag(Flag::N));
        assert!(c.regs.test_flag(Flag::Z));
    }

    #[test]
    fn op_dec_8b() {
        let c = run_reg(&[0x3D], Register::A, 0x00);
        assert_eq!(c.regs.a, 0xFF);
        assert!(!c.regs.test_flag(Flag::C));
        assert!(c.regs.test_flag(Flag::H));
        assert!(c.regs.test_flag(Flag::N));
        assert!(!c.regs.test_flag(Flag::Z));

        let c = run_reg(&[0x3D], Register::A, 0x01);
        assert_eq!(c.regs.a, 0x00);
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
    fn op_nop() {
        run(&[0x00]);
    }

    #[test]
    fn op_jp_imm16() {
        let c = run(&[0xC3, 0xBB, 0xAA]);
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
        assert!(c.interrupts);
    }

    #[test]
    fn op_di() {
        let mut c = cpu(&[0xF3]);
        c.interrupts = true;
        cpu_run(&mut c);
        assert!(!c.interrupts);
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
}
