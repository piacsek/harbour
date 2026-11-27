use std::time::{Duration, Instant};

use harbour::app::{FOCUSED_BORDER, UNFOCUSED_BORDER};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::BorderType;

use crate::real_neovim::{held, reached};
use crate::{
    Screen, deadline, drawn_only_by, ends_with_a_pane_corner, failure, needs_a_terminal,
    pane_corner, pty_slots, reading_agent, script, settled,
};

#[test]
fn a_screen_of_a_test_that_needs_a_terminal_holds_a_pty_slot() {
    needs_a_terminal();
    let _screen = Screen::new(reading_agent());

    assert!(pty_slots::holding());
}

#[test]
fn a_wait_for_text_that_never_shows_fails_naming_it_and_showing_the_screen() {
    let mut screen = Screen::new(reading_agent());
    let started = Instant::now();

    let failure = failure(|| {
        screen.wait_for_within("acme never shown", deadline());
    });

    let elapsed = started.elapsed();
    assert!(elapsed >= deadline(), "{elapsed:?}");
    assert!(elapsed < 2 * deadline(), "{elapsed:?}");
    assert!(failure.contains(r#"for ["acme never shown"]"#), "{failure}");
    assert!(failure.contains("No session yet"), "{failure}");
}

#[test]
fn a_wait_for_a_screen_that_never_stops_changing_fails_showing_the_last_screen() {
    let mut screen = Screen::new(script(
        "i=0; while :; do i=$((i+1)); echo \"acme tick $i\"; sleep 0.05; done",
    ));
    screen.press('n');
    screen.wait_for("acme tick");
    let started = Instant::now();

    let failure = failure(|| {
        screen.wait_for_stable_within(deadline());
    });

    let elapsed = started.elapsed();
    assert!(elapsed >= deadline(), "{elapsed:?}");
    assert!(elapsed < 2 * deadline(), "{elapsed:?}");

    assert!(
        failure.contains("for the screen to stop changing"),
        "{failure}"
    );
    assert!(failure.contains("acme tick"), "{failure}");
}

#[test]
fn a_screen_that_changes_slower_than_the_500_ms_tick_is_not_yet_stable() {
    let mut screen = Screen::new(script(
        "i=0; while :; do i=$((i+1)); echo \"acme tick $i\"; sleep 0.6; done",
    ));
    screen.press('n');
    screen.wait_for("acme tick");
    let started = Instant::now();

    let failure = failure(|| {
        screen.wait_for_stable_within(deadline());
    });

    let elapsed = started.elapsed();
    assert!(elapsed >= deadline(), "{elapsed:?}");
    assert!(elapsed < 2 * deadline(), "{elapsed:?}");

    assert!(
        failure.contains("for the screen to stop changing"),
        "{failure}"
    );
}

#[test]
fn a_wait_for_nothing_to_happen_returns_on_the_first_draw_that_shows_it() {
    let mut screen = Screen::new(script("exec sleep 600"));
    let mut draws = 0;

    settled(&mut screen, |_| {
        draws += 1;
        draws == 2
    });

    assert_eq!(draws, 2);
}

#[test]
fn a_wait_for_nothing_to_happen_keeps_drawing_for_its_whole_window() {
    let mut screen = Screen::new(script("exec sleep 600"));
    let started = Instant::now();

    settled(&mut screen, |_| false);

    assert!(started.elapsed() >= deadline() * 3 / 4);
}

#[test]
fn a_wait_for_a_mode_never_reached_fails_naming_it_and_the_last_mode() {
    let scratch = tempfile::tempdir().unwrap();

    let failure = failure(|| {
        reached(
            &scratch.path().join("socket"),
            "mode \"acme\"",
            Duration::from_millis(200),
            |mode| mode.mode == "acme",
        );
    });

    assert!(failure.contains(r#"for mode "acme""#), "{failure}");
    assert!(failure.contains("last mode: none"), "{failure}");
}

#[test]
fn a_wait_for_an_expression_never_true_fails_naming_it_and_the_last_value() {
    let scratch = tempfile::tempdir().unwrap();

    let failure = failure(|| {
        held(
            &scratch.path().join("socket"),
            "acme == 1",
            Duration::from_millis(200),
        );
    });

    assert!(failure.contains("for acme == 1"), "{failure}");
    assert!(failure.contains("last value: None"), "{failure}");
}

#[test]
fn a_wait_for_focus_never_reached_fails_naming_the_pane_and_showing_the_screen() {
    let mut screen = Screen::new(reading_agent());

    let failure = failure(|| {
        screen.wait_for_focused("acme never focused");
    });

    assert!(
        failure.contains("for focus on acme never focused"),
        "{failure}"
    );
    assert!(failure.contains("No session yet"), "{failure}");
}

#[test]
fn a_wait_for_a_pane_without_focus_never_reached_fails_naming_it_and_showing_the_screen() {
    let mut screen = Screen::new(reading_agent());

    let failure = failure(|| {
        screen.wait_for_unfocused("acme never shown");
    });

    assert!(
        failure.contains("for acme never shown without focus"),
        "{failure}"
    );
    assert!(failure.contains("No session yet"), "{failure}");
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
