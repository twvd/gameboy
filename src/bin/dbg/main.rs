use std::env;
use std::fs;
use std::io::{stdin, Read};

use anyhow::{bail, Result};

use gbrust::gameboy::bus::bus::Bus;
use gbrust::gameboy::bus::testbus::Testbus;
use gbrust::gameboy::cpu::cpu::CPU;
use gbrust::gameboy::cpu::instruction::Instruction;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        bail!("Syntax: {} <filename>", args[0]);
    }

    let f = fs::read(&args[1])?;
    let mut bus = Testbus::new();
    bus.write_slice(&f, 0);
    let mut cpu = CPU::new(Box::from(bus));

    loop {
        println!("{}", cpu.regs);

        // TODO add accessor for bus on CPU,
        // add iterator for Bus,
        // pass iterator to Instruction::decode
        println!(" --> {}", Instruction::decode(&f[cpu.regs.pc as usize..])?);

        let _ = stdin().read(&mut [0u8]).unwrap();

        cpu.step();
    }
}
