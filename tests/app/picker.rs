use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};

use harbour::picker::Picker;

use crate::{Screen, script};

#[test]
fn a_directory_the_picker_prints_starts_a_session_there_named_after_it() {
    let scratch = tempfile::tempdir().unwrap();
    let web = directory_in(scratch.path(), "acme-web");
    let mut screen = Screen::picking(placed_agent(), Picker::Command(typed_picker()));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text(&format!("{}\r", web.display()));

    let shown = screen.wait_for_all(&["acme-web 1", "acme agent in acme-web"]);
    assert!(shown.contains("acme-web 1"), "{shown}");
    assert!(shown.contains("acme agent in acme-web"), "{shown}");
}

#[test]
fn esc_in_the_picker_starts_nothing() {
    let mut screen = Screen::picking(placed_agent(), Picker::Command(escapable_picker()));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    let shown = closed(&mut screen);
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

#[test]
fn a_picker_that_prints_nothing_starts_nothing() {
    let picker = script("echo 'acme picker' >&2; read -r line");
    let mut screen = Screen::picking(placed_agent(), Picker::Command(picker));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text("\r");

    let shown = closed(&mut screen);
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

#[test]
fn a_picker_that_exits_with_an_error_starts_nothing() {
    let picker = script("echo 'acme picker' >&2; read -r line; pwd; exit 1");
    let mut screen = Screen::picking(placed_agent(), Picker::Command(picker));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text("\r");

    let shown = closed(&mut screen);
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

#[test]
fn a_path_that_is_not_a_directory_is_named_and_starts_nothing() {
    let picker = script("echo 'acme picker' >&2; read -r line; echo /acme/gone");
    let mut screen = Screen::picking(placed_agent(), Picker::Command(picker));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text("\r");

    let shown = screen.wait_for_all(&["Not a directory: /acme/gone", "No session yet"]);
    assert!(shown.contains("Not a directory: /acme/gone"), "{shown}");
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

#[test]
fn a_relative_path_is_read_from_the_directory_of_harbour() {
    let picker = script("echo 'acme picker' >&2; read -r line; echo src");
    let mut screen = Screen::picking(placed_agent(), Picker::Command(picker));
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text("\r");

    let shown = screen.wait_for_all(&["src 1", "acme agent in src"]);
    assert!(shown.contains("src 1"), "{shown}");
    assert!(shown.contains("acme agent in src"), "{shown}");
}

#[test]
fn a_picker_that_cannot_start_is_named_and_starts_nothing() {
    let picker = vec!["acme-nopicker".to_string()];
    let mut screen = Screen::picking(placed_agent(), Picker::Command(picker));

    screen.press('n');

    let shown = screen.wait_for_all(&["Could not start acme-nopicker", "No session yet"]);
    assert!(shown.contains("Could not start acme-nopicker"), "{shown}");
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(!shown.contains(" directory "), "{shown}");
}

fn placed_agent() -> Vec<String> {
    script("echo \"acme agent in ${PWD##*/}\"; exec sleep 600")
}

fn typed_picker() -> Vec<String> {
    script("echo 'acme picker' >&2; read -r line; printf '%s\\n' \"$line\"")
}

fn escapable_picker() -> Vec<String> {
    script("stty raw -echo; echo 'acme picker' >&2; dd bs=1 count=1 >/dev/null 2>&1; exit 130")
}

fn closed(screen: &mut Screen) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let shown = screen.draw();
        if !shown.contains(" directory ") || Instant::now() > deadline {
            return screen.wait_for("No session yet");
        }
        sleep(Duration::from_millis(20));
    }
}

fn directory_in(scratch: &Path, name: &str) -> PathBuf {
    let directory = scratch.join(name);
    std::fs::create_dir(&directory).unwrap();
    directory
}
