use super::cpu::cpu::CPU_CLOCK_HZ;
use crate::gameboy::bus::bus::BusMember;
use crate::tickable::Tickable;

use anyhow::Result;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

const TAC_ENABLE: u8 = 1 << 2;
const TAC_DIV_MASK: u8 = 0x03;
const TAC_MASK: u8 = 0x07;

#[derive(FromPrimitive)]
enum TimerInput {
    CPUDiv1024 = 0,
    CPUDiv16 = 1,
    CPUDiv64 = 2,
    CPUDiv256 = 3,
}

impl TimerInput {
    #[allow(dead_code)]
    pub fn get_hz(&self) -> usize {
        CPU_CLOCK_HZ / self.get_div()
    }

    pub fn get_div(&self) -> usize {
        match self {
            TimerInput::CPUDiv1024 => 1024,
            TimerInput::CPUDiv16 => 16,
            TimerInput::CPUDiv64 => 64,
            TimerInput::CPUDiv256 => 256,
        }
    }

    pub fn get_mask(&self) -> usize {
        self.get_div() >> 1
    }
}

pub struct Timer {
    cycles: usize,
    tima: u8,
    tma: u8,
    tac: u8,
    intreq: bool,
}

impl Timer {
    pub fn from_div(div: u8) -> Self {
        Self {
            cycles: (div as usize) << 8,
            tima: 0,
            tma: 0,
            tac: 0,
            intreq: false,
        }
    }

    pub fn new() -> Self {
        Self::from_div(0)
    }

    pub fn get_clr_intreq(&mut self) -> bool {
        let val = self.intreq;
        self.intreq = false;
        val
    }

    fn update_timer(&mut self, prev_bit: usize, new_bit: usize) {
        if self.tac & TAC_ENABLE == TAC_ENABLE {
            if prev_bit != 0 && new_bit == 0 {
                self.tima = self.tima.wrapping_add(1);
                if self.tima == 0 {
                    self.tima = self.tma;
                    self.intreq = true;
                }
            }
        }
    }

    fn update_cycles(&mut self, cycles: usize) {
        let mask = TimerInput::from_u8(self.tac & TAC_DIV_MASK)
            .unwrap()
            .get_mask();
        self.update_timer(self.cycles & mask, cycles & mask);
        self.cycles = cycles;
    }

    fn update_tac(&mut self, new_tac: u8) {
        // Timer glitch - switching input dividers may increment the timer
        let old_mask = TimerInput::from_u8(self.tac & TAC_DIV_MASK)
            .unwrap()
            .get_mask();
        self.update_timer(self.cycles & old_mask, 0);

        self.tac = new_tac;
    }
}

impl BusMember for Timer {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // DIV - Divider
            0xFF04 => ((self.cycles >> 8) & 0xFF) as u8,

            // TIMA - Timer counter
            0xFF05 => self.tima,

            // TMA - Timer counter reload register
            0xFF06 => self.tma,

            // TAC - Timer control
            0xFF07 => self.tac | !TAC_MASK,

            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            // DIV - Divider
            0xFF04 => self.update_cycles(0),

            // TIMA - Timer counter
            0xFF05 => self.tima = val,

            // TMA - Timer counter reload register
            0xFF06 => self.tma = val,

            // TAC - Timer control
            0xFF07 => self.update_tac(val & TAC_MASK),

            _ => unreachable!(),
        }
    }
}

impl Tickable for Timer {
    fn tick(&mut self, ticks: usize) -> Result<usize> {
        for _ in 0..ticks {
            self.update_cycles((self.cycles + 1) % CPU_CLOCK_HZ);
        }

        Ok(ticks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn div() {
        let mut t = Timer::new();
        assert_eq!(t.read(0xFF04), 0);
        t.tick(256).unwrap();
        assert_eq!(t.read(0xFF04), 1);
        t.tick(256 * 254).unwrap();
        assert_eq!(t.read(0xFF04), 255);
        t.tick(256).unwrap();
        assert_eq!(t.read(0xFF04), 0);
    }

    #[test]
    fn div_reset() {
        let mut t = Timer::new();
        assert_eq!(t.read(0xFF04), 0);
        t.tick(256).unwrap();
        assert_eq!(t.read(0xFF04), 1);
        t.write(0xFF04, 123);
        assert_eq!(t.read(0xFF04), 0);
    }

    #[test]
    fn interrupt() {
        let mut t = Timer::new();
        t.write(0xFF07, 0x07);
        assert!(!t.get_clr_intreq());
        t.tick(256 * 256).unwrap();
        assert!(t.get_clr_intreq());
        assert!(!t.get_clr_intreq());
    }

    #[test]
    fn counter() {
        let mut t = Timer::new();
        t.write(0xFF07, 0x07);
        assert_eq!(t.read(0xFF05), 0);
        t.tick(256).unwrap();
        assert_eq!(t.read(0xFF05), 1);
    }

    #[test]
    fn reload() {
        let mut t = Timer::new();
        t.write(0xFF07, 0x07);
        t.write(0xFF06, 0xAA);
        assert_eq!(t.read(0xFF05), 0);
        t.tick(256 * 256).unwrap();
        assert_eq!(t.read(0xFF05), 0xAA);
    }
}
