//! Regression tests for issue #694: the colour picker's hex parser must never panic on
//! multi-byte input. `parse_hex` used to validate the byte length and then slice at fixed
//! byte offsets, so a 6-byte string like "€€" (two 3-byte characters) panicked on a
//! mid-character slice boundary inside the dialog's per-frame parse.

use photocraft_ui_egui::color_picker_ui::parse_hex;

#[test]
fn parse_hex_rejects_six_byte_multibyte_input() {
    // Six bytes, but only two characters: both slice boundaries fall inside a '€'.
    assert_eq!(parse_hex("€€"), None);
    assert_eq!(parse_hex("#€€"), None);
}

#[test]
fn parse_hex_rejects_other_malformed_input() {
    assert_eq!(parse_hex(""), None);
    assert_eq!(parse_hex("#12345"), None);
    assert_eq!(parse_hex("#1234567"), None);
    assert_eq!(parse_hex("zzzzzz"), None);
    assert_eq!(parse_hex("€"), None);
}

#[test]
fn parse_hex_still_accepts_valid_colours() {
    // Discriminating control: a fix that rejects everything is not sufficient.
    let expected = [
        0x12 as f32 / 255.0,
        0xab as f32 / 255.0,
        0xcd as f32 / 255.0,
    ];
    assert_eq!(parse_hex("#12abcd"), Some(expected));
    assert_eq!(parse_hex("12ABCD"), parse_hex("#12abcd"));
}
