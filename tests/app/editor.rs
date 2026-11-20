use crossterm::event::{KeyCode, KeyModifiers};
use harbour::editor::Program;

use crate::{Screen, directory, reading_agent, script};

#[test]
fn ctrl_l_in_claude_code_starts_the_editor_in_the_session_directory_and_shows_it() {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let expected = format!("acme editor in {}", directory());
    let shown = screen.wait_for(&expected);
    assert!(shown.contains(&expected), "{shown}");
    assert!(!shown.contains("acme agent ready"), "{shown}");
}

#[test]
fn keys_typed_in_the_shown_editor_reach_it_and_not_claude_code() {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor in");

    screen.type_text("hello\r");

    let shown = screen.wait_for("acme editor got: hello");
    assert!(shown.contains("acme editor got: hello"), "{shown}");
    assert!(!shown.contains("acme agent got"), "{shown}");
}

fn reading_editor() -> Program {
    Program::new(
        "acme-edit",
        script(
            "echo \"acme editor in ${PWD##*/}\"; \
             while :; do read -r line || continue; echo \"acme editor got: $line\"; done",
        ),
    )
}
