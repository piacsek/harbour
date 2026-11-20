use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;

use crate::{Screen, numbered_agent, pane_corner, reading_agent};

#[test]
fn enter_focuses_the_pane_and_keys_typed_there_reach_the_session() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("hello\r");

    let shown = screen.wait_for("acme agent got: hello");
    assert!(shown.contains("acme agent got: hello"), "{shown}");
}

#[test]
fn keys_typed_while_the_sidebar_has_focus_do_not_reach_the_session() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.type_text("zz\r\r");

    let shown = screen.wait_for("acme agent got:");
    assert!(shown.contains("acme agent got:"), "{shown}");
    assert!(!shown.contains("got: zz"), "{shown}");
}

#[test]
fn ctrl_l_focuses_the_pane() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.type_text("hello\r");

    let shown = screen.wait_for("acme agent got: hello");
    assert!(shown.contains("acme agent got: hello"), "{shown}");
}

#[test]
fn a_focused_pane_has_the_thick_border() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.draw();
    let top = shown.lines().next().unwrap();
    assert!(top.starts_with('┌'), "{shown}");
    assert!(top.contains('┏'), "{shown}");
}

#[test]
fn a_focused_sidebar_has_the_thick_border() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');

    let shown = screen.wait_for("acme agent 1");

    let top = shown.lines().next().unwrap();
    assert!(top.starts_with('┏'), "{shown}");
    assert!(top.contains('┌'), "{shown}");
}

#[test]
fn ctrl_h_in_the_pane_gives_the_keys_back_to_the_sidebar() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('n');

    let shown = screen.wait_for("acme agent 2");
    assert!(shown.contains("acme agent 2"), "{shown}");
}

#[test]
fn a_focused_pane_has_the_cyan_border() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let pane = pane_corner(&screen.draw());
    assert_eq!(screen.cell(pane, 0).fg, Color::Cyan);
    assert_eq!(screen.cell(0, 0).fg, Color::Reset);
}

#[test]
fn a_focused_sidebar_has_the_cyan_border() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');

    let shown = screen.wait_for("acme agent 1");

    let pane = pane_corner(&shown);
    assert_eq!(screen.cell(0, 0).fg, Color::Cyan);
    assert_eq!(screen.cell(pane, 0).fg, Color::Reset);
}

#[test]
fn the_bar_stays_on_the_selected_session_while_the_pane_has_focus() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");

    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.draw();

    assert_eq!(screen.cell(1, 1).symbol(), "▌");
    assert_eq!(screen.cell(1, 1).fg, Color::Cyan);
}

#[test]
fn a_narrow_terminal_draws_the_focused_pane_with_the_cyan_border() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.resize(50, 16);

    assert_eq!(screen.cell(0, 0).symbol(), "┏");
    assert_eq!(screen.cell(0, 0).fg, Color::Cyan);
}
