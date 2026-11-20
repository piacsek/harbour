use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::editor::Program;

use crate::{Screen, directory, reading_agent, script};

#[test]
fn yes_to_ending_a_session_ends_its_editor_too() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path(), reading_agent(), "");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('x');

    screen.press('y');

    assert!(gone(&pid(scratch.path())));
}

#[test]
fn x_on_an_exited_session_whose_editor_runs_asks_first() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = with_exited_agent(scratch.path());

    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains(" End? y/n "), "{shown}");
    assert!(
        shown.contains(&format!("{} 1 exited", directory())),
        "{shown}"
    );
}

#[test]
fn yes_on_an_exited_session_ends_its_editor_and_removes_the_row() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = with_exited_agent(scratch.path());
    screen.press('x');

    screen.press('y');

    let shown = screen.draw();
    assert!(!shown.contains(&format!("{} 1", directory())), "{shown}");
    assert!(gone(&pid(scratch.path())));
}

#[test]
fn q_with_only_an_editor_running_asks_before_quitting() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = with_exited_agent(scratch.path());

    screen.press('q');

    let shown = screen.draw();
    assert!(shown.contains(" Quit? y/n "), "{shown}");
    assert!(screen.app.running());
}

#[test]
fn quitting_ends_editors_and_sessions_within_one_grace_period() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path(), reading_agent(), "trap '' HUP; ");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('q');
    screen.press('y');
    let started = Instant::now();

    drop(screen);

    assert!(started.elapsed() < Duration::from_millis(350));
    assert!(gone(&pid(scratch.path())));
}

fn editing(scratch: &Path, agent: Vec<String>, before: &str) -> Screen {
    let mut screen = Screen::editing(agent, recording_editor(scratch, before));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");
    screen
}

fn with_exited_agent(scratch: &Path) -> Screen {
    let agent = script("echo 'acme agent ready'; read -r line; exit 0");
    let mut screen = editing(scratch, agent, "");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for(&format!("{} 1 exited", directory()));
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen
}

fn recording_editor(scratch: &Path, before: &str) -> Program {
    Program::new(
        "acme-edit",
        script(&format!(
            "{before}echo $$ > '{}/pid'; echo 'acme editor ready'; exec sleep 600",
            scratch.display()
        )),
    )
}

fn pid(scratch: &Path) -> String {
    std::fs::read_to_string(scratch.join("pid"))
        .unwrap()
        .trim()
        .to_string()
}

fn gone(pid: &str) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let alive = Command::new("kill")
            .args(["-0", pid])
            .status()
            .unwrap()
            .success();
        if !alive {
            return true;
        }
        if Instant::now() > deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
