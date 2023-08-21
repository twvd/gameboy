use super::test_display;

use hex_literal::hex;

#[test]
fn dmg_acid2() {
    test_display(
        include_bytes!("../../tests/dmg-acid2/dmg-acid2.gb"),
        &hex!("117528fd9c5efb1feaec71c57637db9e7370fae07bf5e419b44e5f044aa86774"),
        20000,
    );
}
