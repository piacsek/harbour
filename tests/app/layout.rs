use crossterm::event::{KeyCode, KeyModifiers};

use crate::{Screen, directory, numbered_agent, sized_agent};

#[test]
fn a_resize_reaches_a_session_that_is_not_shown() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(sized_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");

    screen.resize(90, 16);
    screen.press('k');

    let shown = screen.wait_for("size:14 58");
    assert!(shown.contains("acme agent 1"), "{shown}");
    assert!(shown.contains("size:14 58"), "{shown}");
}

#[test]
fn a_narrow_terminal_shows_the_sidebar_alone_while_it_has_focus() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");

    screen.resize(50, 16);

    let shown = screen.draw();
    let row = format!("● {} 1", directory());
    assert!(shown.contains(&row), "{shown}");
    assert!(!shown.contains("acme agent 1"), "{shown}");
}

#[test]
fn a_narrow_terminal_shows_the_pane_alone_while_it_has_focus() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.resize(50, 16);

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("acme agent 1");
    assert!(shown.contains("acme agent 1"), "{shown}");
    assert!(!shown.contains("sessions"), "{shown}");
}

#[test]
fn a_narrow_terminal_gives_a_session_the_whole_width() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(sized_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.resize(50, 16);

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("size:14 48");
    assert!(shown.contains("size:14 48"), "{shown}");
}
