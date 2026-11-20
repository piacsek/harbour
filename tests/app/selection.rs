use ratatui::style::Color;

use crate::{Screen, directory, numbered_agent};

#[test]
fn k_selects_the_previous_session_and_the_pane_follows() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");

    screen.press('k');

    let shown = screen.wait_for("acme agent 1");
    assert!(shown.contains("acme agent 1"), "{shown}");
    assert!(!shown.contains("acme agent 2"), "{shown}");
}

#[test]
fn j_selects_the_next_session_and_the_pane_follows() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");
    screen.press('k');
    screen.wait_for("acme agent 1");

    screen.press('j');

    let shown = screen.wait_for("acme agent 2");
    assert!(shown.contains("acme agent 2"), "{shown}");
    assert!(!shown.contains("acme agent 1"), "{shown}");
}

#[test]
fn j_on_the_last_session_keeps_it_selected() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");

    screen.press('j');

    let shown = screen.wait_for("acme agent 1");
    assert!(shown.contains("acme agent 1"), "{shown}");
}

#[test]
fn the_sidebar_marks_the_selected_session() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::wide(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");

    screen.press('k');

    let shown = screen.wait_for("acme agent 1");
    let first = format!("▌ ○ {} 1", directory());
    let second = format!("▌ ○ {} 2", directory());
    assert!(shown.contains(&first), "{shown}");
    assert!(!shown.contains(&second), "{shown}");
}

#[test]
fn the_selected_session_has_a_cyan_bar_on_its_left_edge() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");

    screen.press('k');
    screen.wait_for("acme agent 1");

    assert_eq!(screen.cell(1, 1).symbol(), "▌");
    assert_eq!(screen.cell(1, 1).fg, Color::Cyan);
    assert_eq!(screen.cell(1, 2).symbol(), " ");
}
