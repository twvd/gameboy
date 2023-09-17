use super::test_display;

use hex_literal::hex;

#[test]
fn dmg_acid2() {
    test_display(
        include_bytes!("../../tests/dmg-acid2/dmg-acid2.gb"),
        &hex!("d6b6323524d570d90f34793530f51a026cdfeaf1103b674d0c88be87f44ab92e"),
        20000,
        false,
    );
}

#[test]
fn cgb_acid2() {
    test_display(
        include_bytes!("../../tests/cgb-acid2/cgb-acid2.gbc"),
        &hex!("c587a0e67f4a9e7ceccfc3b1c1991510a6476bd6b4a8b2f109f83e94f97116cb"),
        20000,
        true,
    );
}
