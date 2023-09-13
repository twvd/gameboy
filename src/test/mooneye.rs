use super::test_serial;

macro_rules! mooneye {
    ( $( $x:expr ),* $(,)? ) => {
        {
            $(
				println!("Testing {}...", $x);
				test_mooneye(include_bytes!($x));
			)*
        }
    };
}

fn test_mooneye(rom: &[u8]) {
    test_serial(
        rom,
        &[3, 5, 8, 13, 21, 34],
        &[0x42, 0x42, 0x42, 0x42, 0x42, 0x42],
        120000,
    )
}

#[test]
fn boot() {
    mooneye!(
        "../../tests/mooneye/acceptance/boot_regs-dmgABC.gb",
        //"../../tests/mooneye/acceptance/boot_div-dmgABCmgb.gb",
        //"../../tests/mooneye/acceptance/boot_hwio-dmgABCmgb.gb",
    );
}

#[test]
fn instr_daa() {
    mooneye!("../../tests/mooneye/acceptance/instr/daa.gb",);
}

#[test]
fn mbc1_ram() {
    mooneye!(
        "../../tests/mooneye/emulator-only/mbc1/bits_bank1.gb",
        "../../tests/mooneye/emulator-only/mbc1/bits_bank2.gb",
        "../../tests/mooneye/emulator-only/mbc1/bits_mode.gb",
        "../../tests/mooneye/emulator-only/mbc1/bits_ramg.gb",
        "../../tests/mooneye/emulator-only/mbc1/ram_256kb.gb",
        "../../tests/mooneye/emulator-only/mbc1/ram_64kb.gb",
    );
}

#[test]
fn mbc1_rom() {
    mooneye!(
        //"../../tests/mooneye/emulator-only/mbc1/multicart_rom_8Mb.gb",
        "../../tests/mooneye/emulator-only/mbc1/rom_16Mb.gb",
        "../../tests/mooneye/emulator-only/mbc1/rom_1Mb.gb",
        "../../tests/mooneye/emulator-only/mbc1/rom_2Mb.gb",
        "../../tests/mooneye/emulator-only/mbc1/rom_4Mb.gb",
        "../../tests/mooneye/emulator-only/mbc1/rom_512kb.gb",
        "../../tests/mooneye/emulator-only/mbc1/rom_8Mb.gb"
    );
}

#[test]
fn mbc5() {
    mooneye!(
        "../../tests/mooneye/emulator-only/mbc5/rom_16Mb.gb",
        "../../tests/mooneye/emulator-only/mbc5/rom_1Mb.gb",
        "../../tests/mooneye/emulator-only/mbc5/rom_2Mb.gb",
        "../../tests/mooneye/emulator-only/mbc5/rom_32Mb.gb",
        "../../tests/mooneye/emulator-only/mbc5/rom_4Mb.gb",
        "../../tests/mooneye/emulator-only/mbc5/rom_512kb.gb",
        "../../tests/mooneye/emulator-only/mbc5/rom_64Mb.gb",
        "../../tests/mooneye/emulator-only/mbc5/rom_8Mb.gb"
    );
}

#[test]
fn ei_sequence() {
    mooneye!("../../tests/mooneye/acceptance/ei_sequence.gb",);
}

#[test]
fn ei_timing() {
    mooneye!("../../tests/mooneye/acceptance/ei_timing.gb",);
}

#[test]
fn rapid_ei_di() {
    mooneye!("../../tests/mooneye/acceptance/rapid_di_ei.gb",);
}

#[test]
fn oam_dma_timing() {
    mooneye!(
        "../../tests/mooneye/acceptance/oam_dma_timing.gb",
        "../../tests/mooneye/acceptance/oam_dma_start.gb",
        "../../tests/mooneye/acceptance/oam_dma_restart.gb",
    );
}

#[test]
fn timer() {
    mooneye!(
        "../../tests/mooneye/acceptance/timer/div_write.gb",
        //"../../tests/mooneye/acceptance/timer/rapid_toggle.gb",
        "../../tests/mooneye/acceptance/timer/tim00.gb",
        "../../tests/mooneye/acceptance/timer/tim00_div_trigger.gb",
        "../../tests/mooneye/acceptance/timer/tim01.gb",
        "../../tests/mooneye/acceptance/timer/tim01_div_trigger.gb",
        "../../tests/mooneye/acceptance/timer/tim10.gb",
        "../../tests/mooneye/acceptance/timer/tim10_div_trigger.gb",
        "../../tests/mooneye/acceptance/timer/tim11.gb",
        "../../tests/mooneye/acceptance/timer/tim11_div_trigger.gb",
        //"../../tests/mooneye/acceptance/timer/tima_reload.gb",
        //"../../tests/mooneye/acceptance/timer/tima_write_reloading.gb",
        //"../../tests/mooneye/acceptance/timer/tma_write_reloading.gb",
    );
}
