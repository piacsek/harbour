use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::side::{editing, side};
use crate::{Screen, needs_a_terminal, pane_corner, raw_reader, region};

#[test]
fn the_prefix_then_closing_brace_puts_claude_code_on_the_right_and_its_editor_on_the_left() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = split(scratch.path());

    right(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert!(
        region(&shown, 0, 60).contains("acme editor ready"),
        "{shown}"
    );
    assert!(
        region(&shown, 60, 60).contains("acme agent 1 got: hi"),
        "{shown}"
    );
}

#[test]
fn the_prefix_then_opening_brace_brings_claude_code_back_to_the_left() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = split(scratch.path());
    right(&mut screen);

    left(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert!(
        region(&shown, 0, 60).contains("acme agent 1 got: hi"),
        "{shown}"
    );
    assert!(
        region(&shown, 60, 60).contains("acme editor ready"),
        "{shown}"
    );
}

#[test]
fn the_prefix_then_opening_brace_with_claude_code_on_the_left_leaves_it_there() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = split(scratch.path());

    left(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert!(
        region(&shown, 0, 60).contains("acme agent 1 got: hi"),
        "{shown}"
    );
}

#[test]
fn the_prefix_then_shifted_closing_brace_swaps_the_halves_too() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = split(scratch.path());

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('}'), KeyModifiers::SHIFT);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert!(
        region(&shown, 60, 60).contains("acme agent 1 got: hi"),
        "{shown}"
    );
}

#[test]
fn the_prefix_then_shifted_opening_brace_brings_claude_code_back_to_the_left_too() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = split(scratch.path());
    right(&mut screen);

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('{'), KeyModifiers::SHIFT);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert!(
        region(&shown, 0, 60).contains("acme agent 1 got: hi"),
        "{shown}"
    );
}

#[test]
fn each_program_is_sized_to_the_half_it_moves_to() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path());
    screen.resize(121, 20);
    side(&mut screen);
    screen.wait_for_all(&["acme editor ready 18 58", "acme agent 1 size: 18 59"]);

    right(&mut screen);

    screen.wait_for_all(&["acme agent 1 size: 18 58", "acme editor size: 18 59"]);
}

#[test]
fn a_swap_key_after_an_escape_reaches_claude_code() {
    let mut screen = Screen::new(raw_reader(2));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen
        .app
        .key(KeyEvent::new(KeyCode::Char('}'), KeyModifiers::ALT));

    let shown = screen.wait_for("acme agent read: 1b 7d");
    assert!(shown.contains("acme agent read: 1b 7d"), "{shown}");
}

#[test]
fn the_order_set_is_kept_when_the_mode_is_left_and_entered_again() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = split(scratch.path());
    right(&mut screen);
    side(&mut screen);
    screen.wait_until("the sidebar back", |shown| pane_corner(shown) == 40);

    side(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert!(
        region(&shown, 60, 60).contains("acme agent 1 got: hi"),
        "{shown}"
    );
    assert!(
        region(&shown, 0, 60).contains("acme editor ready"),
        "{shown}"
    );
}

#[test]
fn swapped_ctrl_h_from_claude_code_focuses_the_editor_on_the_left() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = split(scratch.path());
    right(&mut screen);

    ctrl(&mut screen, 'h');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme editor got: hi");
    assert!(
        region(&shown, 0, 60).contains("acme editor got: hi"),
        "{shown}"
    );
}

#[test]
fn swapped_ctrl_l_from_the_editor_focuses_claude_code_on_the_right() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = split(scratch.path());
    right(&mut screen);
    ctrl(&mut screen, 'h');

    ctrl(&mut screen, 'l');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert!(
        region(&shown, 60, 60).contains("acme agent 1 got: hi"),
        "{shown}"
    );
}

#[test]
fn swapped_ctrl_l_from_claude_code_reaches_claude_code() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = split(scratch.path());
    right(&mut screen);

    ctrl(&mut screen, 'l');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got:");
    assert!(region(&shown, 60, 60).contains("hi"), "{shown}");
    assert!(!shown.contains("acme editor got"), "{shown}");
}

#[test]
fn swapped_ctrl_h_from_the_editor_brings_the_sidebar_back() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = split(scratch.path());
    right(&mut screen);
    ctrl(&mut screen, 'h');

    ctrl(&mut screen, 'h');

    let shown = screen.draw();
    assert_eq!(pane_corner(&shown), 40, "{shown}");
}

#[test]
fn swapped_ctrl_2_shows_the_second_session_in_the_order_set() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path());
    ctrl(&mut screen, 'h');
    screen.press('n');
    screen.wait_for("acme agent 2");
    ctrl(&mut screen, '1');
    side(&mut screen);
    screen.wait_for("acme editor ready");
    right(&mut screen);

    ctrl(&mut screen, '2');

    let shown = screen.wait_until("the second editor", |shown| {
        region(shown, 60, 60).contains("acme agent 2")
            && region(shown, 0, 60).contains("acme editor ready")
    });
    assert_eq!(pane_corner(&shown), 60, "{shown}");
}

fn split(scratch: &std::path::Path) -> Screen {
    let mut screen = editing(scratch);
    side(&mut screen);
    screen.wait_for("acme editor ready");
    screen
}

fn right(screen: &mut Screen) {
    screen.prefixed('}');
}

fn left(screen: &mut Screen) {
    screen.prefixed('{');
}

fn ctrl(screen: &mut Screen, c: char) {
    screen.key(KeyCode::Char(c), KeyModifiers::CONTROL);
}
