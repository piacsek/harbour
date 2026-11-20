use crossterm::event::{KeyCode, KeyModifiers};
use harbour::editor::Program;

use crate::{Screen, reading_agent, script};

#[test]
fn an_editor_that_exits_with_zero_shows_claude_code_again_without_a_message() {
    let mut screen = with_editor(exiting_editor("exit 0"));

    screen.type_text("done\r");

    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
    assert!(!shown.contains("acme-edit exited"), "{shown}");
}

#[test]
fn keys_reach_claude_code_once_the_editor_exited() {
    let mut screen = with_editor(exiting_editor("exit 0"));
    screen.type_text("done\r");
    screen.wait_for("acme agent ready");

    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent got: hi");
    assert!(shown.contains("acme agent got: hi"), "{shown}");
}

#[test]
fn an_editor_that_exits_with_a_code_is_named_with_it() {
    let mut screen = with_editor(exiting_editor("exit 3"));

    screen.type_text("done\r");

    let shown = screen.wait_for_all(&["acme agent ready", "acme-edit exited 3"]);
    assert!(shown.contains("acme agent ready"), "{shown}");
    assert!(shown.contains("acme-edit exited 3"), "{shown}");
}

#[test]
fn an_editor_killed_by_a_signal_is_named_with_it() {
    let mut screen = with_editor(exiting_editor("kill -9 $$"));

    screen.type_text("done\r");

    let shown = screen.wait_for("acme-edit exited (Killed");
    assert!(shown.contains("acme-edit exited (Killed"), "{shown}");
}

#[test]
fn ctrl_l_after_the_editor_exited_starts_a_fresh_one_and_clears_the_message() {
    let mut screen = with_editor(exiting_editor("exit 3"));
    screen.type_text("done\r");
    screen.wait_for("acme-edit exited 3");

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor up");
    assert!(shown.contains("acme editor up"), "{shown}");
    assert!(!shown.contains("acme-edit exited"), "{shown}");
}

#[test]
fn a_hidden_editor_that_exits_is_named_and_the_next_ctrl_l_starts_a_fresh_one() {
    let mut screen = with_editor(Program::new(
        "acme-edit",
        script("echo 'acme editor up'; sleep 1; exit 4"),
    ));
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.wait_for("acme-edit exited 4");

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor up");
    assert!(shown.contains("acme editor up"), "{shown}");
    assert!(!shown.contains("acme agent ready"), "{shown}");
}

#[test]
fn an_editor_that_cannot_start_is_named_and_claude_code_stays_shown() {
    let missing = Program::new("acme-edit", vec!["acme-editor-missing".to_string()]);
    let mut screen = Screen::editing(reading_agent(), missing);
    screen.resize(200, 20);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("Could not start acme-edit");
    assert!(
        shown.contains("Could not start acme-edit: is it installed and on PATH?"),
        "{shown}"
    );
    assert!(shown.contains("acme agent ready"), "{shown}");
}

#[test]
fn claude_code_keeps_the_keys_when_the_editor_cannot_start() {
    let missing = Program::new("acme-edit", vec!["acme-editor-missing".to_string()]);
    let mut screen = Screen::editing(reading_agent(), missing);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent got: hi");
    assert!(shown.contains("acme agent got: hi"), "{shown}");
}

fn with_editor(editor: Program) -> Screen {
    let mut screen = Screen::editing(reading_agent(), editor);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor up");
    screen
}

fn exiting_editor(end: &str) -> Program {
    Program::new(
        "acme-edit",
        script(&format!("echo 'acme editor up'; read -r line; {end}")),
    )
}
