use super::{test_display, test_serial};

use hex_literal::hex;

#[test]
fn cpu_instrs_01() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/01-special.gb"),
        b"Passed",
        b"Failed",
        30000,
    );
}

#[test]
fn cpu_instrs_02() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/02-interrupts.gb"),
        b"Passed",
        b"Failed",
        30000,
    );
}

#[test]
fn cpu_instrs_03() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/03-op sp,hl.gb"),
        b"Passed",
        b"Failed",
        30000,
    );
}

#[test]
fn cpu_instrs_04() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/04-op r,imm.gb"),
        b"Passed",
        b"Failed",
        30000,
    );
}

#[test]
fn cpu_instrs_05() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/05-op rp.gb"),
        b"Passed",
        b"Failed",
        30000,
    );
}

#[test]
fn cpu_instrs_06() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/06-ld r,r.gb"),
        b"Passed",
        b"Failed",
        30000,
    );
}

#[test]
fn cpu_instrs_07() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb"),
        b"Passed",
        b"Failed",
        30000,
    );
}

#[test]
fn cpu_instrs_08() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/08-misc instrs.gb"),
        b"Passed",
        b"Failed",
        30000,
    );
}

#[test]
fn cpu_instrs_09() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/09-op r,r.gb"),
        b"Passed",
        b"Failed",
        60000,
    );
}

#[test]
fn cpu_instrs_10() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/10-bit ops.gb"),
        b"Passed",
        b"Failed",
        90000,
    );
}

#[test]
fn cpu_instrs_11() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/11-op a,(hl).gb"),
        b"Passed",
        b"Failed",
        180000,
    );
}

#[test]
fn instr_timing() {
    test_serial(
        include_bytes!("../../tests/blargg/instr_timing/instr_timing.gb"),
        b"Passed",
        b"Failed",
        30000,
    );
}

#[test]
fn mem_timing() {
    test_serial(
        include_bytes!("../../tests/blargg/mem_timing/mem_timing.gb"),
        b"Passed",
        b"Failed",
        30000,
    );
}

#[test]
fn mem_timing_2() {
    test_display(
        include_bytes!("../../tests/blargg/mem_timing-2/mem_timing.gb"),
        &hex!("180edbacf7255addb9537cc7c95b1f5352ee7061b973ecab4e8054b0502eba4e"),
        60000,
    );
}

#[test]
fn oam_bug_lcd_sync() {
    test_display(
        include_bytes!("../../tests/blargg/oam_bug/rom_singles/1-lcd_sync.gb"),
        &hex!("35081c557a9cb2717998045663132658cdba0fd454765a2145b14546f83587aa"),
        60000,
    );
}

#[test]
fn oam_bug_non_causes() {
    test_display(
        include_bytes!("../../tests/blargg/oam_bug/rom_singles/3-non_causes.gb"),
        &hex!("f417f087dc9aefd1a853719415c01f68142ab9d9e30b66c73e1ff429e5152a92"),
        60000,
    );
}

#[test]
fn oam_bug_timing_no_bug() {
    test_display(
        include_bytes!("../../tests/blargg/oam_bug/rom_singles/6-timing_no_bug.gb"),
        &hex!("b4cc0155826c546939b7df321b8653d7fdc5235e38f907172ecfa3f1a7947c4f"),
        60000,
    );
}
