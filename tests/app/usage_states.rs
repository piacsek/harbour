use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::picker::Picker;

use crate::{Screen, reading_agent, script};

#[test]
fn the_popup_says_it_is_loading_until_the_report_prints() {
    let mut screen = Screen::of(App::new(reading_agent()).with_usage(script("exec sleep 600")));

    screen.press('u');

    let shown = screen.draw();
    assert!(shown.contains(" usage "), "{shown}");
    assert!(shown.contains("loading…"), "{shown}");
}

#[test]
fn a_report_that_prints_nothing_says_so() {
    let mut screen = Screen::of(App::new(reading_agent()).with_usage(script("exit 0")));

    screen.press('u');

    let shown = screen.wait_for("printed nothing");
    assert!(!shown.contains("loading…"), "{shown}");
}

#[test]
fn a_report_that_fails_keeps_what_it_printed_and_names_its_exit_code() {
    let failing = script("echo 'acme usage: not logged in'; exit 3");
    let mut screen = Screen::of(App::new(reading_agent()).with_usage(failing));

    screen.press('u');

    let shown = screen.wait_for(" exit 3 ");
    assert!(shown.contains("acme usage: not logged in"), "{shown}");
}

#[test]
fn a_report_that_succeeds_names_no_exit_code() {
    let mut screen = Screen::of(App::new(reading_agent()).with_usage(placed_report()));
    screen.press('u');
    screen.wait_for("acme usage in");

    std::thread::sleep(Duration::from_millis(300));

    let shown = screen.draw();
    assert!(!shown.contains(" exit "), "{shown}");
}

#[test]
fn the_report_runs_where_harbour_was_opened_without_a_session() {
    let mut screen = Screen::of(App::new(reading_agent()).with_usage(placed_report()));

    screen.press('u');

    let shown = screen.wait_for("acme usage in");
    assert!(shown.contains("acme usage in acme-home"), "{shown}");
}

#[test]
fn the_report_runs_in_the_selected_sessions_directory() {
    let scratch = tempfile::tempdir().unwrap();
    let web = directory_in(scratch.path(), "acme-web");
    let mut screen = placed(&web);

    screen.press('u');

    let shown = screen.wait_for("acme usage in");
    assert!(shown.contains("acme usage in acme-web"), "{shown}");
}

#[test]
fn the_report_runs_where_harbour_was_opened_when_the_sessions_directory_is_gone() {
    let scratch = tempfile::tempdir().unwrap();
    let web = directory_in(scratch.path(), "acme-web");
    let mut screen = placed(&web);
    std::fs::remove_dir(&web).unwrap();

    screen.press('u');

    let shown = screen.wait_for("acme usage in");
    assert!(shown.contains("acme usage in acme-home"), "{shown}");
}

#[test]
fn a_report_longer_than_the_popup_shows_its_start() {
    let long = script("for n in $(seq 1 40); do echo \"acme line $n.\"; done");
    let mut screen = Screen::of(App::new(reading_agent()).with_usage(long));

    screen.press('u');

    let shown = screen.wait_for("acme line 1.");
    assert!(!shown.contains("acme line 40."), "{shown}");
}

#[test]
fn the_popup_takes_a_terminal_smaller_than_it() {
    let mut screen = Screen::of(App::new(reading_agent()).with_usage(placed_report()));
    screen.resize(40, 12);

    screen.press('u');

    let shown = screen.wait_for("acme usage in");
    let first = shown.lines().next().unwrap();
    assert!(first.starts_with("┏ usage "), "{shown}");
    assert!(first.ends_with('┓'), "{shown}");
    let last = shown.lines().last().unwrap();
    assert!(last.starts_with('┗'), "{shown}");
}

#[test]
fn the_popup_is_80_by_30_on_a_larger_terminal() {
    let mut screen = Screen::of(App::new(reading_agent()).with_usage(placed_report()));
    screen.resize(120, 40);

    screen.press('u');

    let shown = screen.wait_for("acme usage in");
    let top = shown
        .lines()
        .position(|line| line.contains("┏ usage "))
        .unwrap();
    let bottom = shown.lines().position(|line| line.contains('┗')).unwrap();
    let line = shown.lines().nth(top).unwrap();
    let left = line.chars().position(|c| c == '┏').unwrap();
    let right = line.chars().position(|c| c == '┓').unwrap();
    assert_eq!(bottom - top + 1, 30);
    assert_eq!(right - left + 1, 80);
}

fn placed(directory: &Path) -> Screen {
    let picker = Picker::Command(script(&format!("printf '%s\\n' '{}'", directory.display())));
    let app = App::new(reading_agent())
        .with_picker(picker)
        .with_usage(placed_report());
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen
}

fn placed_report() -> Vec<String> {
    script("echo \"acme usage in ${PWD##*/}\"")
}

fn directory_in(scratch: &Path, name: &str) -> PathBuf {
    let directory = scratch.join(name);
    std::fs::create_dir(&directory).unwrap();
    directory
}
