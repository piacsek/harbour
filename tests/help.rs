use harbour::help::{KEYS, rows};
use harbour::keymap::Keymap;
use unicode_width::UnicodeWidthStr;

#[test]
fn the_default_bindings_draw_the_rows_of_keys() {
    let rows = rows(&Keymap::default());

    assert_eq!(rows.len(), KEYS.len());
    assert_eq!(rows[0], owned(KEYS[0]));
    assert_eq!(rows[1], owned(KEYS[1]));
    assert_eq!(rows[2], owned(KEYS[2]));
    assert_eq!(rows[3], owned(KEYS[3]));
    assert_eq!(rows[4], owned(KEYS[4]));
    assert_eq!(rows[5], owned(KEYS[5]));
    assert_eq!(rows[6], owned(KEYS[6]));
    assert_eq!(rows[7], owned(KEYS[7]));
    assert_eq!(rows[8], owned(KEYS[8]));
    assert_eq!(rows[9], owned(KEYS[9]));
    assert_eq!(rows[10], owned(KEYS[10]));
    assert_eq!(rows[11], owned(KEYS[11]));
    assert_eq!(rows[12], owned(KEYS[12]));
    assert_eq!(rows[13], owned(KEYS[13]));
}

#[test]
fn the_longest_prefix_keeps_every_row_within_the_narrowest_help() {
    let rows = rows(&Keymap::default().with_prefix(' '));

    assert!(widest(&rows) <= 57, "{rows:?}");
}

#[test]
fn the_keys_hold_the_fourteen_rows_the_narrow_help_fits() {
    assert_eq!(KEYS.len(), 14);
}

fn owned((key, what): (&str, &str)) -> (String, String) {
    (key.to_string(), what.to_string())
}

fn widest(rows: &[(String, String)]) -> usize {
    let keys = rows
        .iter()
        .map(|(key, _)| key.width())
        .max()
        .unwrap_or_default()
        .max(10);
    rows.iter()
        .map(|(_, what)| keys + 1 + what.width())
        .max()
        .unwrap_or_default()
}
