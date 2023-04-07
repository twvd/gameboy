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
use gbrust::input::input::{Input, NullInput};
use gbrust::tickable::Tickable;

#[derive(Parser)]
#[command(
    about = "Gameboy Emulator",
    author = "Thomas <thomas@thomasw.dev>",
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

    /// Disable display
    #[arg(long)]
    no_display: bool,

    /// Framerate limit
    #[arg(long, default_value = "80")]
    fps: u64,

    /// Output serial output to terminal
    #[arg(short, long)]
    serial_out: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let rom = fs::read(args.filename)?;

    let display: Box<dyn Display> = if !args.no_display {
        Box::new(CursesDisplay::new(DISPLAY_W, DISPLAY_H, args.fps))
    } else {
        Box::new(NullDisplay::new())
    };

    let lcd = LCDController::new(display);
    let input: Box<dyn Input> = Box::new(NullInput::new());
    let mut bus: Box<dyn Bus> = if args.testbus {
        Box::new(Testbus::new())
    } else {
        let cartridge = cartridge::load(&rom);
        println!("Cartridge loaded");
        println!("{}", cartridge);
        let mut b = if let Some(ref brfile) = args.bootrom {
            let bootrom = fs::read(brfile)?;
            Box::new(Gameboybus::new(
                cartridge,
                Some(bootrom.as_slice()),
                lcd,
                input,
            ))
        } else {
            Box::new(Gameboybus::new(cartridge, None, lcd, input))
        };
        if args.serial_out {
            b.enable_serial_output();
        }

        b
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
                "Cycle: {}\n{}\nIME: {} IE: {} IF:{}\n --> {}\n",
                cpu.get_cycles(),
                cpu.regs,
                if cpu.ime { "ON" } else { "off" },
                cpu.bus.read(0xFFFF),
                cpu.bus.read(0xFF0F),
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
