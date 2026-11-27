use harbour::app::{FOCUSED_BORDER, UNFOCUSED_BORDER};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::BorderType;

use crate::{
    Screen, drawn_only_by, ends_with_a_pane_corner, needs_a_terminal, pane_corner, pty_slots,
    reading_agent,
};

#[test]
fn a_screen_of_a_test_that_needs_a_terminal_holds_a_pty_slot() {
    needs_a_terminal();
    let _screen = Screen::new(reading_agent());

    assert!(pty_slots::holding());
}

#[test]
fn a_glyph_both_styles_draw_tells_neither_apart() {
    let rounded = BorderType::Rounded.to_border_set();
    assert!(!drawn_only_by(
        rounded.vertical_left,
        BorderType::Rounded,
        BorderType::Plain
    ));
    assert!(drawn_only_by(
        rounded.top_left,
        BorderType::Rounded,
        BorderType::Plain
    ));
}

#[test]
fn a_wide_title_before_the_pane_corner_finds_the_corners_cell() {
    let corner = FOCUSED_BORDER.to_border_set().top_left;
    let mut buffer = Buffer::empty(Rect::new(0, 0, 6, 1));
    buffer.set_string(0, 0, format!("{corner}修{corner}"), Style::default());
    let shown: String = (0..6).map(|x| buffer[(x, 0)].symbol()).collect();
    assert_eq!(pane_corner(&shown), 3);
}

#[test]
fn a_top_row_ending_in_a_block_glyph_ends_with_no_pane_corner() {
    let block = BorderType::QuadrantOutside.to_border_set().top_right;
    assert!(!ends_with_a_pane_corner(&format!("acme agent 1 {block}")));
}

#[test]
fn a_top_row_ending_in_either_panes_corner_ends_with_a_pane_corner() {
    let focused = format!("acme {}", FOCUSED_BORDER.to_border_set().top_right);
    let unfocused = format!("acme {}", UNFOCUSED_BORDER.to_border_set().top_right);
    assert!(ends_with_a_pane_corner(&focused));
    assert!(ends_with_a_pane_corner(&unfocused));
}
