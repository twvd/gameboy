#[cfg(test)]
mod tests {
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

    #[test]
    fn cpu_instrs_01() {
        test_serial(
            include_bytes!("../../gb-test-roms/cpu_instrs/individual/01-special.gb"),
            "Passed",
            30000,
        );
    }

    #[test]
    fn cpu_instrs_02() {
        test_serial(
            include_bytes!("../../gb-test-roms/cpu_instrs/individual/02-interrupts.gb"),
            "Passed",
            30000,
        );
    }

    #[test]
    fn cpu_instrs_03() {
        test_serial(
            include_bytes!("../../gb-test-roms/cpu_instrs/individual/03-op sp,hl.gb"),
            "Passed",
            30000,
        );
    }

    #[test]
    fn cpu_instrs_04() {
        test_serial(
            include_bytes!("../../gb-test-roms/cpu_instrs/individual/04-op r,imm.gb"),
            "Passed",
            30000,
        );
    }

    #[test]
    fn cpu_instrs_05() {
        test_serial(
            include_bytes!("../../gb-test-roms/cpu_instrs/individual/05-op rp.gb"),
            "Passed",
            30000,
        );
    }

    #[test]
    fn cpu_instrs_06() {
        test_serial(
            include_bytes!("../../gb-test-roms/cpu_instrs/individual/06-ld r,r.gb"),
            "Passed",
            30000,
        );
    }

    #[test]
    fn cpu_instrs_07() {
        test_serial(
            include_bytes!("../../gb-test-roms/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb"),
            "Passed",
            30000,
        );
    }

    #[test]
    fn cpu_instrs_08() {
        test_serial(
            include_bytes!("../../gb-test-roms/cpu_instrs/individual/08-misc instrs.gb"),
            "Passed",
            30000,
        );
    }

    #[test]
    fn cpu_instrs_09() {
        test_serial(
            include_bytes!("../../gb-test-roms/cpu_instrs/individual/09-op r,r.gb"),
            "Passed",
            30000,
        );
    }

    #[test]
    fn cpu_instrs_10() {
        test_serial(
            include_bytes!("../../gb-test-roms/cpu_instrs/individual/10-bit ops.gb"),
            "Passed",
            30000,
        );
    }

    #[test]
    fn cpu_instrs_11() {
        test_serial(
            include_bytes!("../../gb-test-roms/cpu_instrs/individual/11-op a,(hl).gb"),
            "Passed",
            30000,
        );
    }
}
