use super::super::bus::bus::Bus;
use super::instruction::Instruction;
use super::regs::RegisterFile;

pub struct CPU {
    pub bus: Box<dyn Bus>,
    pub regs: RegisterFile,
}

impl CPU {
    pub fn new(bus: Box<dyn Bus>) -> Self {
        Self {
            bus,
            regs: RegisterFile::new(),
        }
    }

    pub fn op_set(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_res(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_srl(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_swap(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_sla(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_sra(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_bit(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_rl(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_rla(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_rlc(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_rlca(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_rr(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_rra(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_rrc(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_rrca(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_ei(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_di(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_rst(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_nop(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_stop(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_halt(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_ld(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_ldh(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_scf(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_ccf(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_cp(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_cpl(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_or(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_xor(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_and(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_push(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_pop(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_adc(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_daa(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_add(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_sub(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_dec(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_inc(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_jr(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_jr_nc(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_jr_nz(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_jr_z(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_jp(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_jp_nc(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_jp_nz(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_jp_z(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_call(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_call_nc(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_call_nz(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_call_z(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_ret(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_ret_nc(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_ret_nz(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_ret_z(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_reti(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_sbc(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_prefix_cb(&mut self, _instr: &Instruction) {
        todo!();
    }

    pub fn op_invalid(&mut self, _instr: &Instruction) {
        panic!("Invalid opcode");
    }
}
