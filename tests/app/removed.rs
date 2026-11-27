use std::thread::sleep;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;

use crate::finding;
use crate::zoom::corners;
use crate::{Screen, needs_a_terminal, raw_agent, reading_agent, script, shows_titled};

#[test]
fn alt_z_reaches_claude_code_and_zooms_nothing() {
    let mut screen = Screen::new(raw_agent("", 2));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('z'), KeyModifiers::ALT);

    let shown = screen.wait_for("acme agent read:z");
    assert!(corners(&shown) > 0, "{shown}");
}

#[test]
fn alt_s_in_the_sidebar_opens_no_finder() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = finding::three_sessions(scratch.path());

    screen.key(KeyCode::Char('s'), KeyModifiers::ALT);
    sleep(Duration::from_millis(300));

    let shown = screen.draw();
    assert!(!shown.contains("acme fzf ready"), "{shown}");
}

#[test]
fn ctrl_two_from_the_sidebar_jumps_nowhere() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = finding::three_sessions(scratch.path());
    screen.key(KeyCode::Char('k'), KeyModifiers::NONE);

    screen.key(KeyCode::Char('2'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('k'), KeyModifiers::NONE);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("hi\r");

    let shown = screen.wait_for("got: hi");
    assert!(shown.contains("acme agent 1 got: hi"), "{shown}");
}

#[test]
fn question_mark_in_the_sidebar_shows_no_help() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(!shows_titled(&shown, "help"), "{shown}");
}

#[test]
fn q_in_the_sidebar_asks_nothing() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    screen.press('q');

    let shown = screen.draw();
    assert!(!shown.contains("Quit? y/n"), "{shown}");
}

#[test]
fn c_in_the_sidebar_shows_no_changelog() {
    let mut screen = Screen::of(App::new(reading_agent()).with_changelog("# Changelog\n"));

    screen.press('c');

    let shown = screen.draw();
    assert!(!shown.contains(" changelog "), "{shown}");
}

#[test]
fn u_in_the_sidebar_opens_no_usage_popup() {
    let mut screen =
        Screen::of(crate::usage_states::app().with_usage(script("echo 'acme session: 12% spent'")));

    screen.press('u');

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
}
