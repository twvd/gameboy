use std::fs;
use std::io::{stdin, Read};

use anyhow::Result;
use clap::Parser;

const DISPLAY_W: usize = 160;
const DISPLAY_H: usize = 144;

use gbrust::display::curses::CursesDisplay;
use gbrust::display::display::{Display, NullDisplay};
use gbrust::gameboy::bus::bus::Bus;
use gbrust::gameboy::bus::gbbus::Gameboybus;
use gbrust::gameboy::bus::testbus::Testbus;
use gbrust::gameboy::cartridge::cartridge;
use gbrust::gameboy::cpu::cpu::CPU;
use gbrust::gameboy::cpu::regs::Register;
use gbrust::gameboy::lcd::LCDController;
use gbrust::tickable::Tickable;

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

    /// Print CPU state after each instruction
    #[arg(short, long)]
    verbose: bool,

    /// Enable display
    #[arg(short, long)]
    display: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let rom = fs::read(args.filename)?;

    let display: Box<dyn Display> = if args.display {
        Box::new(CursesDisplay::new(DISPLAY_W, DISPLAY_H))
    } else {
        Box::new(NullDisplay::new())
    };

    let lcd = LCDController::new(display);
    let mut bus: Box<dyn Bus> = if args.testbus {
        Box::new(Testbus::new())
    } else {
        let cartridge = cartridge::load(&rom);
        println!("Cartridge loaded");
        println!("{}", cartridge);
        if let Some(ref brfile) = args.bootrom {
            let bootrom = fs::read(brfile)?;
            Box::new(Gameboybus::new(cartridge, Some(bootrom.as_slice()), lcd))
        } else {
            Box::new(Gameboybus::new(cartridge, None, lcd))
        }
    };

    if args.testbus {
        bus.write_slice(&rom, 0);

        // Indicate start of VBlank for testing purposes
        bus.write(0xFF44, 0x90);
    }

    let mut cpu = CPU::new(bus);

    if args.bootrom.is_none() {
        // Initialize registers to post-boot
        cpu.regs.write(Register::A, 0x01)?;
        cpu.regs.write(Register::F, 0xB0)?;
        cpu.regs.write(Register::B, 0x00)?;
        cpu.regs.write(Register::C, 0x13)?;
        cpu.regs.write(Register::D, 0x00)?;
        cpu.regs.write(Register::E, 0xD8)?;
        cpu.regs.write(Register::H, 0x01)?;
        cpu.regs.write(Register::H, 0x01)?;
        cpu.regs.write(Register::L, 0x4D)?;
        cpu.regs.write(Register::SP, 0xFFFE)?;
        cpu.regs.write(Register::PC, 0x0100)?;
    }

    loop {
        if args.verbose && cpu.bus.read(0xFF50) == 1 {
            let state = format!(
                "Cycle: {}\n{}\n --> {}\n",
                cpu.get_cycles(),
                cpu.regs,
                cpu.peek_next_instr()?
            );

            println!("{}", state);
        }

        if args.pause {
            let _ = stdin().read(&mut [0u8]).unwrap();
        }

        cpu.tick(1)?;
    }
}
