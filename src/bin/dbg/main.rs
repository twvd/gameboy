use std::fs;
use std::io::{stdin, Read};

use anyhow::Result;
use clap::Parser;

use gbrust::gameboy::bus::bus::Bus;
use gbrust::gameboy::bus::testbus::Testbus;
use gbrust::gameboy::cpu::cpu::CPU;

#[derive(Parser)]
#[command(
    about = "A crude ROM debugger for development purposes",
    long_about = None)]
struct Args {
    /// ROM filename to load.
    filename: String,

    /// Wait for keystroke after each CPU step.
    #[arg(short, long)]
    pause: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let f = fs::read(args.filename)?;
    let mut bus = Testbus::new();
    bus.write_slice(&f, 0);
    let mut cpu = CPU::new(Box::from(bus));

    loop {
        println!("Cycle: {}", cpu.get_cycles());
        println!("{}", cpu.regs);
        println!(" --> {}", cpu.peek_next_instr()?);

        if args.pause {
            let _ = stdin().read(&mut [0u8]).unwrap();
        }

        cpu.step()?;
    }
}
