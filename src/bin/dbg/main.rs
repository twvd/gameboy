use std::env;
use std::fs;
use std::io::{stdin, Read};

use anyhow::{bail, Result};

use gbrust::gameboy::bus::bus::Bus;
use gbrust::gameboy::bus::testbus::Testbus;
use gbrust::gameboy::cpu::cpu::CPU;

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

        println!(" --> {}", cpu.peek_next_instr()?);

        let _ = stdin().read(&mut [0u8]).unwrap();

        cpu.step()?;
    }
}
