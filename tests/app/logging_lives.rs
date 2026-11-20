use std::fs;
use std::sync::mpsc::Receiver;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;

use crate::logging::{count, focused, logged, only, read};
use crate::remembering::{ID, OTHER_ID, Scratch, TITLE};
use crate::{Screen, home, reading_agent, script};

#[test]
fn a_started_session_logs_its_program_and_directory() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);

    screen.press('n');

    let started = only(&read(&lines), " started ");
    assert!(started.contains(" session=1 "), "{started}");
    assert!(started.contains(r#" program="/bin/sh" "#), "{started}");
    assert!(
        started.ends_with(&format!(" directory={:?}\n", home())),
        "{started}"
    );
}

#[test]
fn a_session_given_an_id_logs_it_when_it_starts() {
    let scratch = tempfile::tempdir().unwrap();
    fs::create_dir(scratch.path().join("projects")).unwrap();
    let agent = script("echo \"acme agent $0 $1\"; exec sleep 600");
    let (app, lines) = logged(
        App::new(agent)
            .with_transcripts(scratch.path().join("projects"))
            .naming_sessions(),
    );
    let mut screen = Screen::of(app);

    screen.press('n');

    let shown = screen.wait_for("acme agent --session-id ");
    let id = shown
        .split("acme agent --session-id ")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .to_string();
    let started = only(&read(&lines), " started ");
    assert!(started.contains(&format!(" id={id:?} ")), "{started}");
}

#[test]
fn a_session_without_an_id_logs_none() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);

    screen.press('n');

    assert!(!only(&read(&lines), " started ").contains(" id="));
}

#[test]
fn a_session_that_exits_logs_its_code_once() {
    let (app, lines) = logged(App::new(script("exit 3")));
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("exited 3");

    screen.draw();

    let exited = only(&read(&lines), " exited ");
    assert!(exited.ends_with(" exited session=1 code=3\n"), "{exited}");
}

#[test]
fn a_session_killed_by_a_signal_logs_the_signal() {
    let (app, lines) = logged(App::new(script("kill -9 $$")));
    let mut screen = Screen::of(app);
    screen.press('n');

    screen.wait_for("Killed");

    let exited = only(&read(&lines), " exited ");
    assert!(exited.contains(r#" session=1 signal="Killed"#), "{exited}");
}

#[test]
fn a_resumed_session_logs_its_id_and_directory() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let agent = script("echo \"acme agent $0 $1\"; exec sleep 600");
    let (app, lines) = logged(App::new(agent).with_memory(scratch.memory()));
    let mut screen = Screen::of(app);

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.wait_for("acme agent --resume");
    let lines = read(&lines);
    let resumed = only(&lines, " resumed ");
    assert!(resumed.contains(" session=1 "), "{resumed}");
    assert!(resumed.contains(&format!(" id={ID:?} ")), "{resumed}");
    assert!(
        resumed.ends_with(&format!(" directory={:?}\n", scratch.checkout())),
        "{resumed}"
    );
    assert_eq!(count(&lines, " started "), 0);
}

#[test]
fn a_move_between_stopped_cards_logs_the_conversation_selected() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.remember(OTHER_ID, "Bump the acme client", 1);
    let (app, lines) = logged(App::new(reading_agent()).with_memory(scratch.memory()));
    let mut screen = Screen::of(app);
    read(&lines);

    screen.press('j');

    let selected = only(&read(&lines), " selected ");
    assert!(
        selected.contains(&format!(" id={OTHER_ID:?} ")),
        "{selected}"
    );
}

#[test]
fn an_editor_that_starts_logs_its_program_and_directory() {
    let (app, lines) = logged(App::new(reading_agent()).with_editor(editor("exec sleep 600")));
    let mut screen = focused(app);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    screen.wait_for("acme editor ready");
    let started = only(&read(&lines), " editor started ");
    assert!(started.contains(" session=1 editor=2 "), "{started}");
    assert!(started.contains(r#" program="acme-edit" "#), "{started}");
    assert!(started.contains(" directory="), "{started}");
}

#[test]
fn an_editor_that_exits_logs_its_end_once() {
    let (app, lines) = logged(App::new(reading_agent()).with_editor(editor("exit 0")));
    let mut screen = focused(app);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let lines = until_logged(&mut screen, &lines, " editor ended ");
    let ended = only(&lines, " editor ended ");
    assert!(
        ended.ends_with(" editor ended session=1 editor=2\n"),
        "{ended}"
    );
}

fn editor(then: &str) -> Program {
    Program::new(
        "acme-edit",
        script(&format!("echo 'acme editor ready'; sleep 0.2; {then}")),
    )
}

fn until_logged(screen: &mut Screen, lines: &Receiver<String>, event: &str) -> Vec<String> {
    let mut seen = Vec::new();
    screen.wait_until(&format!("{event:?} in the log"), |_| {
        seen.extend(lines.try_iter());
        count(&seen, event) > 0
    });
    screen.draw();
    seen.extend(lines.try_iter());
    seen
}
