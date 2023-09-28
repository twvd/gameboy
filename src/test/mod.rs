mod acid;
mod blargg;
mod mooneye;
mod sm83;

use crate::display::display::NullDisplay;
use crate::display::test::TestDisplay;
use crate::gameboy::bus::gbbus::Gameboybus;
use crate::gameboy::cartridge::cartridge;
use crate::gameboy::cpu::cpu::CPU;
use crate::gameboy::lcd::{LCDController, LCD_H, LCD_W};
use crate::gameboy::serial::Serial;
use crate::input::input::NullInput;
use crate::misc::WritableSender;

use itertools::Itertools;

use std::sync::mpsc;
use std::time::Instant;

fn test_serial(rom: &[u8], pass_text: &[u8], fail_text: &[u8], time_limit: u128) {
    let cart = cartridge::load(rom);
    let display = Box::new(NullDisplay::new());
    let input = Box::new(NullInput::new());
    let lcd = LCDController::new(display, false);

    let (tx, rx) = mpsc::channel::<u8>();
    let bus = Box::new(Gameboybus::new_with_serial(
        cart,
        None,
        lcd,
        input,
        false,
        Serial::new_out(Box::new(WritableSender::new(tx))),
    ));

    let mut cpu = CPU::new(bus, false);

    let start = Instant::now();
    let mut output: Vec<u8> = vec![];
    loop {
        if start.elapsed().as_millis() > time_limit {
            panic!("Timeout");
        }
        cpu.step().unwrap();

        if let Ok(c) = rx.try_recv() {
            output.push(c);
            if output.ends_with(&pass_text) {
                return;
            }
            if output.ends_with(&fail_text) {
                panic!("Test failed");
            }
        }
    }
}

fn test_display(rom: &[u8], pass_hash: &[u8], time_limit: u128, cgb: bool) {
    let cart = cartridge::load(rom);
    let (display, dispstatus) = TestDisplay::new(LCD_W, LCD_H);
    let input = Box::new(NullInput::new());
    let lcd = LCDController::new(display, cgb);

    let bus = Box::new(Gameboybus::new(cart, None, lcd, input, cgb));
    let mut cpu = CPU::new(bus, cgb);

    let start = Instant::now();
    loop {
        if start.elapsed().as_millis() > time_limit {
            dbg!(dispstatus.get());
            panic!("Timeout");
        }
        cpu.step().unwrap();

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
