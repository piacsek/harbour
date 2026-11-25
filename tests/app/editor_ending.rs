use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use ratatui::style::Modifier;

use crate::{
    Screen, below, deadline, hangups, reading_agent, recording_hangups, region, script,
    staying_after_hangups,
};

#[test]
fn yes_to_ending_a_session_ends_its_editor_too() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path(), reading_agent());
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('x');

    screen.press('y');

    assert!(gone(&pid(scratch.path())));
}

#[test]
fn the_editor_of_an_exited_session_is_not_dimmed() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = with_exited_agent(scratch.path());
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor ready");
    let (x, y) = position_of(&shown, "acme editor ready");
    assert!(
        !screen.cell(x, y).modifier.contains(Modifier::DIM),
        "{shown}"
    );
}

#[test]
fn an_exited_session_shows_its_exit_beside_its_editor() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = with_exited_agent(scratch.path());
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor ready");
    assert_eq!(below(&shown, "acme-home 1"), "exited");
}

#[test]
fn x_on_an_exited_session_whose_editor_runs_asks_first() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = with_exited_agent(scratch.path());

    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains("End? y/n"), "{shown}");
    assert!(shown.contains("acme-home 1"), "{shown}");
    assert!(shown.contains("exited"), "{shown}");
}

#[test]
fn yes_on_an_exited_session_ends_its_editor_and_removes_the_row() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = with_exited_agent(scratch.path());
    screen.press('x');

    screen.press('y');

    let shown = screen.draw();
    assert!(!shown.contains("acme-home 1"), "{shown}");
    assert!(gone(&pid(scratch.path())));
}

#[test]
fn x_says_an_editor_harbour_cannot_ask_may_have_unsaved_changes() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());

    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains("May have unsaved"), "{shown}");
}

#[test]
fn the_whole_question_shows_in_the_narrowest_sidebar_beside_the_pane() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());
    screen.resize(60, 20);

    screen.press('x');

    let shown = screen.draw();
    let sidebar = words_in(&shown, 20);
    assert!(
        sidebar.contains("May have unsaved changes. End? y/n"),
        "{shown}"
    );
}

#[test]
fn the_whole_question_shows_in_the_sidebar_of_an_80_column_terminal() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());
    screen.resize(80, 24);

    screen.prefixed('q');

    let shown = screen.draw();
    let sidebar = words_in(&shown, 26);
    assert!(
        sidebar.contains("May have unsaved changes. Quit? y/n"),
        "{shown}"
    );
}

#[test]
fn ctrl_c_to_the_quit_question_when_an_editor_may_have_unsaved_changes_keeps_harbour_running() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = in_the_sidebar(scratch.path());
    screen.prefixed('q');
    let asked = screen.draw();
    assert!(asked.contains("May have unsaved changes"), "{asked}");

    screen.key(KeyCode::Char('c'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(!shown.contains("y/n"), "{shown}");
    assert!(screen.app.running());
}

#[test]
fn q_with_only_an_editor_running_asks_before_quitting() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = with_exited_agent(scratch.path());

    screen.prefixed('q');

    let shown = screen.draw();
    assert!(shown.contains("Quit? y/n"), "{shown}");
    assert!(screen.app.running());
}

#[test]
fn quitting_hangs_up_on_editors_and_sessions_before_it_kills_any() {
    let scratch = tempfile::tempdir().unwrap();
    let editor = Program::new(
        "acme-edit",
        hanging_up(scratch.path(), "editor", "acme editor ready"),
    );
    let agent = hanging_up(scratch.path(), "agent", "acme agent ready");
    let mut screen = editing_with(
        App::new(agent)
            .with_editor(editor)
            .with_hangup_grace(5 * deadline()),
    );
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.prefixed('q');
    screen.press('y');
    assert!(!screen.app.running());

    drop(screen);

    let hangups = hangups(scratch.path());
    assert_eq!(hangups.len(), 2, "{hangups:?}");
    assert_eq!(hangups[0], "agent saw 2 alive", "{hangups:?}");
    assert_eq!(hangups[1], "editor saw 2 alive", "{hangups:?}");
}

fn editing(scratch: &Path, agent: Vec<String>) -> Screen {
    editing_with(App::new(agent).with_editor(recording_editor(scratch)))
}

fn editing_with(app: App) -> Screen {
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");
    screen
}

fn in_the_sidebar(scratch: &Path) -> Screen {
    let mut screen = editing(scratch, reading_agent());
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen
}

fn with_exited_agent(scratch: &Path) -> Screen {
    let agent = script("echo 'acme agent ready'; read -r line; exit 0");
    let mut screen = editing(scratch, agent);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_until("the session exited", |shown| {
        below(shown, "acme-home 1") == "exited"
    });
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen
}

fn recording_editor(scratch: &Path) -> Program {
    Program::new(
        "acme-edit",
        script(&format!(
            "echo $$ > '{}/pid'; echo 'acme editor ready'; exec sleep 600",
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

fn hanging_up(scratch: &Path, name: &str, ready: &str) -> Vec<String> {
    script(&format!(
        "{}echo '{ready}'; {}",
        recording_hangups(scratch, name),
        staying_after_hangups()
    ))
}

pub(crate) fn gone(pid: &str) -> bool {
    let until = Instant::now() + 5 * deadline();
    loop {
        let alive = Command::new("kill")
            .args(["-0", pid])
            .status()
            .unwrap()
            .success();
        if !alive {
            return true;
        }
        if Instant::now() > until {
            return false;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn words_in(shown: &str, width: usize) -> String {
    region(shown, 0, width)
        .split(|c: char| c.is_whitespace() || crate::drawn(c))
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn position_of(shown: &str, text: &str) -> (u16, u16) {
    let (row, line) = shown
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains(text))
        .unwrap();
    let column = line[..line.find(text).unwrap()].chars().count();
    (u16::try_from(column).unwrap(), u16::try_from(row).unwrap())
}
