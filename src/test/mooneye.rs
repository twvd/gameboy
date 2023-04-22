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
