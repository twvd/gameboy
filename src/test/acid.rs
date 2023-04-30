use super::test_display;

use hex_literal::hex;

#[test]
fn dmg_acid2() {
    test_display(
        include_bytes!("../../tests/dmg-acid2/dmg-acid2.gb"),
        &hex!("3b4e785d4de53352e90bf6f51eea85600a0b57c410b9f48d1f2ded84cfacf513"),
        20000,
    );
}
