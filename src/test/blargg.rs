use super::test_serial;

#[test]
fn cpu_instrs_01() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/01-special.gb"),
        "Passed",
        30000,
    );
}

#[test]
fn cpu_instrs_02() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/02-interrupts.gb"),
        "Passed",
        30000,
    );
}

#[test]
fn cpu_instrs_03() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/03-op sp,hl.gb"),
        "Passed",
        30000,
    );
}

#[test]
fn cpu_instrs_04() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/04-op r,imm.gb"),
        "Passed",
        30000,
    );
}

#[test]
fn cpu_instrs_05() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/05-op rp.gb"),
        "Passed",
        30000,
    );
}

#[test]
fn cpu_instrs_06() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/06-ld r,r.gb"),
        "Passed",
        30000,
    );
}

#[test]
fn cpu_instrs_07() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb"),
        "Passed",
        30000,
    );
}

#[test]
fn cpu_instrs_08() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/08-misc instrs.gb"),
        "Passed",
        30000,
    );
}

#[test]
fn cpu_instrs_09() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/09-op r,r.gb"),
        "Passed",
        30000,
    );
}

#[test]
fn cpu_instrs_10() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/10-bit ops.gb"),
        "Passed",
        30000,
    );
}

#[test]
fn cpu_instrs_11() {
    test_serial(
        include_bytes!("../../tests/blargg/cpu_instrs/individual/11-op a,(hl).gb"),
        "Passed",
        30000,
    );
}
