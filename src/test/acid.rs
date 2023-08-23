use super::test_display;

use hex_literal::hex;

#[test]
fn dmg_acid2() {
    test_display(
        include_bytes!("../../tests/dmg-acid2/dmg-acid2.gb"),
        &hex!("d6b6323524d570d90f34793530f51a026cdfeaf1103b674d0c88be87f44ab92e"),
        20000,
    );
}
