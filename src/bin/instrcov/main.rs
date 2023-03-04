use std::panic;

use gbrust::gameboy::bus::testbus::Testbus;
use gbrust::gameboy::cpu::cpu::CPU;
use gbrust::gameboy::cpu::instructions::{INSTRUCTIONS, INSTRUCTIONS_CB};

fn main() {
    println!(" --- Instruction set coverage --- ");

    let mut ok = 0;
    let mut fail = 0;

    for (opcode, i) in INSTRUCTIONS
        .iter()
        .enumerate()
        .filter(|&(opcode, i)| i.mnemonic != "INVALID" && opcode != 0xCB)
        .map(|(opcode, i)| ([opcode as u8, 0_u8], i))
        .chain(
            INSTRUCTIONS_CB
                .iter()
                .enumerate()
                .map(|(opcode, i)| ([0xCB_u8, opcode as u8], i)),
        )
    {
        panic::set_hook(Box::new(|_| {}));
        let result = panic::catch_unwind(|| {
            let bus = Testbus::from(&opcode);
            let mut cpu = CPU::new(Box::new(bus));
            cpu.step().unwrap();
        });
        let _ = panic::take_hook();

        println!(
            "{:02X?} {:<20} ... {}",
            opcode,
            i.mnemonic,
            if result.is_ok() { "OK" } else { "FAILED" }
        );
        if result.is_ok() {
            ok += 1;
        } else {
            fail += 1;
        }
    }

    println!(
        "{:>3} ok ({}%), {:>3} failed, {} total",
        ok,
        (ok * 100) / (ok + fail),
        fail,
        ok + fail
    );
}
