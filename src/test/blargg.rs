use super::test_serial;

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
        30000,
    );
}

#[test]
fn cpu_instrs_10() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/10-bit ops.gb"),
        b"Passed",
        b"Failed",
        60000,
    );
}

#[test]
fn cpu_instrs_11() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/11-op a,(hl).gb"),
        b"Passed",
        b"Failed",
        60000,
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
