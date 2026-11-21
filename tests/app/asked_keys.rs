use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::picker::Picker;

use crate::{Screen, reading_agent, script};

const XTERM: &str = "\\033[>4;2m";
const KITTY: &str = "\\033[>1u";

#[test]
fn ctrl_escape_reaches_claude_code_that_asked_for_modify_other_keys_as_ctrl_escape() {
    let mut screen = focused(XTERM, 10);

    screen.key(KeyCode::Esc, KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme agent read:");
    assert!(
        shown.contains("acme agent read: 1b 5b 32 37 3b 35 3b 32 37 7e"),
        "{shown}"
    );
}

#[test]
fn ctrl_escape_reaches_claude_code_that_asked_for_the_kitty_protocol_as_ctrl_escape() {
    let mut screen = focused(KITTY, 7);

    screen.key(KeyCode::Esc, KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme agent read:");
    assert!(
        shown.contains("acme agent read: 1b 5b 32 37 3b 35 75"),
        "{shown}"
    );
}

#[test]
fn ctrl_escape_reaches_claude_code_that_asked_for_nothing_as_escape() {
    let mut screen = focused("", 2);

    screen.key(KeyCode::Esc, KeyModifiers::CONTROL);
    screen.press('z');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 1b 7a"), "{shown}");
}

#[test]
fn shift_enter_reaches_claude_code_that_asked_for_modify_other_keys_as_a_new_line() {
    let mut screen = focused(XTERM, 2);

    screen.key(KeyCode::Enter, KeyModifiers::SHIFT);

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 1b 0d"), "{shown}");
}

#[test]
fn shift_enter_reaches_claude_code_that_asked_for_the_kitty_protocol_as_a_new_line() {
    let mut screen = focused(KITTY, 2);

    screen.key(KeyCode::Enter, KeyModifiers::SHIFT);

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 1b 0d"), "{shown}");
}

#[test]
fn ctrl_enter_reaches_claude_code_that_asked_for_the_kitty_protocol_as_a_new_line() {
    let mut screen = focused(KITTY, 2);

    screen.key(KeyCode::Enter, KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 1b 0d"), "{shown}");
}

#[test]
fn ctrl_h_in_claude_code_that_asked_for_modify_other_keys_gives_the_keys_back_to_the_sidebar() {
    let mut screen = focused(XTERM, 1);

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.press('z');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 7a"), "{shown}");
}

#[test]
fn ctrl_escape_reaches_an_editor_that_asked_for_modify_other_keys_as_ctrl_escape() {
    let mut screen = editing(XTERM, 10);

    screen.key(KeyCode::Esc, KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor read:");
    assert!(
        shown.contains("acme editor read: 1b 5b 32 37 3b 35 3b 32 37 7e"),
        "{shown}"
    );
}

#[test]
fn shift_enter_reaches_an_editor_that_asked_for_modify_other_keys_as_shift_enter() {
    let mut screen = editing(XTERM, 10);

    screen.key(KeyCode::Enter, KeyModifiers::SHIFT);

    let shown = screen.wait_for("acme editor read:");
    assert!(
        shown.contains("acme editor read: 1b 5b 32 37 3b 32 3b 31 33 7e"),
        "{shown}"
    );
}

#[test]
fn ctrl_enter_reaches_an_editor_that_asked_for_the_kitty_protocol_as_ctrl_enter() {
    let mut screen = editing(KITTY, 7);

    screen.key(KeyCode::Enter, KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor read:");
    assert!(
        shown.contains("acme editor read: 1b 5b 31 33 3b 35 75"),
        "{shown}"
    );
}

#[test]
fn ctrl_tab_reaches_an_editor_that_asked_for_the_kitty_protocol_as_ctrl_tab() {
    let mut screen = editing(KITTY, 6);

    screen.key(KeyCode::Tab, KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor read:");
    assert!(
        shown.contains("acme editor read: 1b 5b 39 3b 35 75"),
        "{shown}"
    );
}

#[test]
fn shift_enter_reaches_a_picker_that_asked_for_modify_other_keys_as_shift_enter() {
    let mut screen = picking(XTERM, 10);

    screen.key(KeyCode::Enter, KeyModifiers::SHIFT);

    let shown = screen.wait_for("acme picker read:");
    assert!(
        shown.contains("acme picker read: 1b 5b 32 37 3b 32 3b 31 33 7e"),
        "{shown}"
    );
}

#[test]
fn shift_enter_reaches_a_picker_that_asked_for_nothing_as_enter() {
    let mut screen = picking("", 1);

    screen.key(KeyCode::Enter, KeyModifiers::SHIFT);

    let shown = screen.wait_for("acme picker read:");
    assert!(shown.contains("acme picker read: 0d"), "{shown}");
}

fn asking(name: &str, request: &str, length: usize) -> Vec<String> {
    script(&format!(
        "stty raw -echo; printf '{request}' >&2; echo 'acme {name} ready' >&2; \
         dd bs=1 count={length} 2>/dev/null | od -An -tx1 | tr -s ' \\n' '  ' | \
         sed 's/^/acme {name} read:/' >&2; exec sleep 600"
    ))
}

fn focused(request: &str, length: usize) -> Screen {
    let mut screen = Screen::new(asking("agent", request, length));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen
}

fn editing(request: &str, length: usize) -> Screen {
    let editor = Program::new("acme-edit", asking("editor", request, length));
    let mut screen = Screen::editing(reading_agent(), editor);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");
    screen
}

fn picking(request: &str, length: usize) -> Screen {
    let picker = asking("picker", request, length);
    let mut screen = Screen::wide(App::new(reading_agent()).with_picker(Picker::Command(picker)));
    screen.press('n');
    screen.wait_for("acme picker ready");
    screen
}
