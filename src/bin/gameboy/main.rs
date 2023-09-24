use std::fs;
use std::fs::File;
use std::io;
use std::io::{stdin, Read, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use terminal::{stdout, Action, Clear, Event, KeyCode, Retrieved, Value};

const DISPLAY_W: usize = 160;
const DISPLAY_H: usize = 144;

#[cfg(not(feature = "sixel"))]
use gbrust::display::terminal::TerminalDisplay;

#[cfg(feature = "sixel")]
use gbrust::display::sixel::SixelDisplay;

use gbrust::display::display::{Display, NullDisplay};
use gbrust::gameboy::bus::bus::Bus;
use gbrust::gameboy::bus::gbbus::Gameboybus;
use gbrust::gameboy::bus::testbus::Testbus;
use gbrust::gameboy::cartridge::cartridge;
use gbrust::gameboy::cpu::cpu::CPU;
use gbrust::gameboy::lcd::LCDController;
use gbrust::input::input::{Input, NullInput};
use gbrust::tickable::Tickable;

/// Emulation mode/Gameboy model to emulate
#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
enum EmulationMode {
    Auto,
    DMG,
    Color,
}

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

    /// Gameboy model to emulate
    #[arg(
        long,
        require_equals = true,
        value_name = "MODE",
        num_args = 0..=1,
        default_value_t = EmulationMode::Auto,
        default_missing_value = "auto",
        value_enum
    )]
    mode: EmulationMode,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut savefn = PathBuf::from(&args.filename);
    savefn.set_extension("sav");

    let rom = fs::read(&args.filename)?;
    let sav = fs::read(&savefn).unwrap_or(vec![]);

    let display: Box<dyn Display>;
    let input: Box<dyn Input>;

    let cartridge = cartridge::load_with_save(&rom, &sav);
    println!("Cartridge: {}", cartridge.borrow());

    let cgb = match args.mode {
        EmulationMode::Auto => cartridge.borrow().is_cgb(),
        EmulationMode::DMG => false,
        EmulationMode::Color => true,
    };

    if cgb {
        println!("Mode: Gameboy Color (CGB)");
    } else {
        println!("Mode: Gameboy (DMG)");
    }

    let terminal = stdout();
    terminal.act(Action::EnableRawMode).unwrap();
    let (key_tx, key_rx) = mpsc::channel();

    if !args.no_display {
        #[cfg(not(feature = "sixel"))]
        {
            let cdisplay = Box::new(TerminalDisplay::new(DISPLAY_W, DISPLAY_H, args.fps));
            input = Box::new(cdisplay.create_input(key_rx));
            display = cdisplay as Box<dyn Display>;
        }

        #[cfg(feature = "sixel")]
        {
            let sdisplay = Box::new(SixelDisplay::new(DISPLAY_W, DISPLAY_H, args.fps));
            input = Box::new(NullInput::new());
            display = sdisplay as Box<dyn Display>;
        }
    } else {
        display = Box::new(NullDisplay::new());
        input = Box::new(NullInput::new());
    }

    let lcd = LCDController::new(display, cgb);
    let mut bus: Box<dyn Bus> = if args.testbus {
        Box::new(Testbus::new())
    } else {
        let serial_out: Box<dyn Write> = if args.serial_out {
            Box::new(stdout())
        } else {
            Box::new(io::sink())
        };

        let b = if let Some(ref brfile) = args.bootrom {
            let bootrom = fs::read(brfile)?;
            Box::new(Gameboybus::new_with_serial(
                Rc::clone(&cartridge),
                Some(bootrom.as_slice()),
                lcd,
                input,
                cgb,
                Box::new(io::empty()),
                serial_out,
            ))
        } else {
            Box::new(Gameboybus::new_with_serial(
                Rc::clone(&cartridge),
                None,
                lcd,
                input,
                cgb,
                Box::new(io::empty()),
                serial_out,
            ))
        };
        b
    };

    if args.testbus {
        bus.write_slice(&rom, 0);

        // Indicate start of VBlank for testing purposes
        bus.write(0xFF44, 0x90);
    }

    let mut cpu = CPU::new(bus, cgb);

    'mainloop: loop {
        if let Retrieved::Event(Some(Event::Key(keyevent))) = terminal
            .get(Value::Event(Some(Duration::from_millis(0))))
            .unwrap()
        {
            match keyevent.code {
                KeyCode::Esc => {
                    terminal.act(Action::DisableRawMode).unwrap();
                    terminal.act(Action::ShowCursor).unwrap();
                    terminal.act(Action::ResetColor).unwrap();
                    terminal.act(Action::MoveCursorTo(0, 0)).unwrap();
                    terminal.act(Action::ClearTerminal(Clear::All)).unwrap();

                    break 'mainloop;
                }
                _ => key_tx.send(keyevent)?,
            }
        }

        if args.verbose {
            eprintln!("{}", cpu.dump_state());
        }

        if args.pause {
            let _ = stdin().read(&mut [0u8]).unwrap();
        }

        cpu.tick(1)?;
    }

    let mut save = File::create(savefn)?;
    save.write_all(&cartridge.borrow().get_save())?;

    Ok(())
}
