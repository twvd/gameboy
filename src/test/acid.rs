use super::test_display;

use hex_literal::hex;

#[test]
fn dmg_acid2() {
    test_display(
        include_bytes!("../../tests/dmg-acid2/dmg-acid2.gb"),
        &hex!("f844ea760a6f1fe137f7f992c7ab1c72d34c7fcd3a807b4174a78eb04a32a458"),
        20000,
    );
}
