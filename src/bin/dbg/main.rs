use std::fs;
use std::io::{stdin, Read};

use anyhow::Result;
use clap::Parser;

use gbrust::gameboy::bus::bus::Bus;
use gbrust::gameboy::bus::gbbus::Gameboybus;
use gbrust::gameboy::bus::testbus::Testbus;
use gbrust::gameboy::cpu::cpu::CPU;

#[derive(Parser)]
#[command(
    about = "A crude ROM debugger for development purposes",
    long_about = None)]
struct Args {
    /// ROM filename to load.
    filename: String,

    /// Boot ROM to optionally load
    #[arg(short, long)]
    bootrom: Option<String>,

    /// Wait for keystroke after each CPU step.
    #[arg(short, long)]
    pause: bool,

    /// Use testing address bus
    #[arg(short, long)]
    testbus: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let rom = fs::read(args.filename)?;

    let mut bus: Box<dyn Bus> = if args.testbus {
        Box::new(Testbus::new())
    } else {
        if let Some(brfile) = args.bootrom {
            let bootrom = fs::read(brfile)?;
            Box::new(Gameboybus::new(&rom, Some(bootrom.as_slice())))
        } else {
            Box::new(Gameboybus::new(&rom, None))
        }
    };

    if args.testbus {
        bus.write_slice(&rom, 0);

        // Indicate start of VBlank for testing purposes
        bus.write(0xFF44, 0x90);
    }

    let mut cpu = CPU::new(bus);

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
