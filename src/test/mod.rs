mod blargg;

use crate::display::display::NullDisplay;
use crate::gameboy::bus::gbbus::Gameboybus;
use crate::gameboy::cartridge::cartridge;
use crate::gameboy::cpu::cpu::CPU;
use crate::gameboy::lcd::LCDController;
use crate::input::input::NullInput;
use crate::tickable::Tickable;
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
