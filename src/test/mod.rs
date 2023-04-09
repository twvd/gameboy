mod acid;
mod blargg;

use crate::display::display::NullDisplay;
use crate::display::test::TestDisplay;
use crate::gameboy::bus::gbbus::Gameboybus;
use crate::gameboy::cartridge::cartridge;
use crate::gameboy::cpu::cpu::CPU;
use crate::gameboy::lcd::{LCDController, LCD_H, LCD_W};
use crate::input::input::NullInput;
use crate::tickable::Tickable;

use itertools::Itertools;

use std::sync::mpsc;
use std::time::Instant;

fn test_serial(rom: &[u8], pass_text: &str, time_limit: u128) {
    let cart = cartridge::load(rom);
    let display = Box::new(NullDisplay::new());
    let input = Box::new(NullInput::new());
    let lcd = LCDController::new(display);

    let mut bus = Box::new(Gameboybus::new(cart, None, lcd, input));
    let (tx, rx) = mpsc::channel::<u8>();
    bus.enable_serial_output();
    bus.enable_serial_channel(tx);

    let mut cpu = CPU::new(bus);

    let start = Instant::now();
    let mut output = String::new();
    loop {
        if start.elapsed().as_millis() > time_limit {
            panic!("Timeout");
        }
        cpu.tick(1).unwrap();

        if let Ok(c) = rx.try_recv() {
            output.push(c as char);
            if output.ends_with(pass_text) {
                return;
            }
        }
    }
}

fn test_display(rom: &[u8], pass_hash: &[u8], time_limit: u128) {
    let cart = cartridge::load(rom);
    let (display, dispstatus) = TestDisplay::new(LCD_W, LCD_H);
    let input = Box::new(NullInput::new());
    let lcd = LCDController::new(display);

    let bus = Box::new(Gameboybus::new(cart, None, lcd, input));
    let mut cpu = CPU::new(bus);

    let start = Instant::now();
    loop {
        if start.elapsed().as_millis() > time_limit {
            dbg!(dispstatus.get());
            panic!("Timeout");
        }
        cpu.tick(1).unwrap();

        let newstatus = dispstatus.get();
        if newstatus.stable_frames >= 100 {
            if newstatus.hash != pass_hash {
                panic!(
                    "Expected hash {:02x} but saw {:02x} (for {} frames)",
                    pass_hash.iter().format(""),
                    newstatus.hash.iter().format(""),
                    newstatus.stable_frames
                );
            } else {
                return;
            }
        }
    }
}
