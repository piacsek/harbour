use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;
use harbour::editor::Program;
use harbour::picker::Picker;

use crate::editor::{editor_open, reading_editor};
use crate::zoom::raw_reader;
use crate::{Screen, home, raw_agent, reading_agent, script};

#[test]
fn a_paste_reaches_the_shown_editor_and_not_claude_code() {
    let mut screen = editor_open();

    screen.app.paste("pasted\r");

    let shown = screen.wait_for("acme editor got: pasted");
    assert!(shown.contains("acme editor got: pasted"), "{shown}");
    assert!(!shown.contains("acme agent got"), "{shown}");
}

#[test]
fn a_click_reaches_the_shown_editor_that_asked_for_the_mouse() {
    let mut screen = opened(
        reading_agent(),
        raw_editor("\\033[?1000h\\033[?1006h", 9),
        "acme editor ready",
    );

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 45, 3);

    let shown = screen.wait_for("acme editor read:[<0;5;3M");
    assert!(shown.contains("acme editor read:[<0;5;3M"), "{shown}");
}

#[test]
fn a_click_while_the_editor_is_shown_never_reaches_claude_code() {
    let mut screen = opened(
        raw_agent("\\033[?1000h\\033[?1006h", 9),
        reading_editor(),
        "acme editor in",
    );
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 45, 3);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 45, 3);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 46, 4);

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read:[<0;6;4M"), "{shown}");
}

#[test]
fn a_release_after_the_pane_switched_to_claude_code_does_not_reach_it() {
    let mut screen = opened(
        raw_agent("\\033[?1000h\\033[?1006h", 9),
        reading_editor(),
        "acme editor in",
    );
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 45, 3);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 45, 3);

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 46, 4);

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read:[<0;6;4M"), "{shown}");
}

#[test]
fn a_new_session_shows_its_claude_code_while_the_other_keeps_its_editor() {
    let mut screen = editor_open();
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 1, 1);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 1, 1);

    screen.press('n');

    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
    assert!(!shown.contains("acme editor in"), "{shown}");
}

#[test]
fn selecting_a_session_again_shows_its_editor_still_running() {
    let mut screen = editor_open();
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 1, 1);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 1, 1);
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.press('k');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("still\r");

    let shown = screen.wait_for("acme editor got: still");
    assert!(shown.contains("acme editor got: still"), "{shown}");
}

#[test]
fn keys_typed_while_the_popup_is_open_reach_the_picker_and_not_the_editor() {
    let mut screen = picking_over_the_editor();

    screen.type_text("zz\r");

    let shown = screen.wait_for("Not a directory");
    assert!(shown.contains("Not a directory"), "{shown}");
    assert!(!shown.contains("acme editor got"), "{shown}");
}

#[test]
fn a_paste_while_the_popup_is_open_reaches_the_picker_and_not_the_editor() {
    let mut screen = picking_over_the_editor();

    screen.app.paste("zz\r");

    let shown = screen.wait_for("Not a directory");
    assert!(shown.contains("Not a directory"), "{shown}");
    assert!(!shown.contains("acme editor got"), "{shown}");
}

#[test]
fn shift_enter_reaches_the_editor_as_enter() {
    let mut screen = opened(reading_agent(), hex_editor(), "acme editor ready");

    screen.key(KeyCode::Enter, KeyModifiers::SHIFT);

    let shown = screen.wait_for("acme editor read: 0d");
    assert!(shown.contains("acme editor read: 0d"), "{shown}");
}

#[test]
fn ctrl_enter_reaches_the_editor_as_enter() {
    let mut screen = opened(reading_agent(), hex_editor(), "acme editor ready");

    screen.key(KeyCode::Enter, KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor read: 0d");
    assert!(shown.contains("acme editor read: 0d"), "{shown}");
}

#[test]
fn shift_enter_reaches_the_picker_as_enter() {
    let mut screen = picking_hex();

    screen.key(KeyCode::Enter, KeyModifiers::SHIFT);

    let shown = screen.wait_for("acme picker read: 0d");
    assert!(shown.contains("acme picker read: 0d"), "{shown}");
}

#[test]
fn ctrl_enter_reaches_the_picker_as_enter() {
    let mut screen = picking_hex();

    screen.key(KeyCode::Enter, KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme picker read: 0d");
    assert!(shown.contains("acme picker read: 0d"), "{shown}");
}

#[test]
fn ctrl_z_never_reaches_the_editor() {
    let mut screen = opened(reading_agent(), hex_editor(), "acme editor ready");

    screen.key(KeyCode::Char('z'), KeyModifiers::CONTROL);
    screen.press('a');

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read: 61"), "{shown}");
}

#[test]
fn ctrl_z_never_reaches_claude_code() {
    let mut screen = Screen::of(App::new(raw_reader(1)));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('z'), KeyModifiers::CONTROL);
    screen.press('a');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 61"), "{shown}");
}

fn opened(agent: Vec<String>, editor: Program, ready: &str) -> Screen {
    let mut screen = Screen::editing(agent, editor);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for(ready);
    screen
}

fn picking_over_the_editor() -> Screen {
    let picker = script("echo 'acme picker' >&2; read -r line; printf '%s\\n' \"$line\"");
    let mut screen = Screen::wide(
        App::new(reading_agent())
            .with_editor(reading_editor())
            .with_picker(Picker::Command(picker)),
    );
    screen.press('n');
    screen.type_text(&format!("{}\r", home().display()));
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor in");
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 1, 1);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 1, 1);
    screen.press('n');
    screen.wait_for("acme picker");
    screen
}

pub(crate) fn raw_editor(request: &str, length: usize) -> Program {
    Program::new(
        "acme-edit",
        script(&format!(
            "stty raw -echo; printf '{request}'; echo 'acme editor ready'; \
             r=$(dd bs=1 count={length} 2>/dev/null); \
             printf 'acme editor read:%s' \"$r\" | tr -d '\\033'; exec sleep 600"
        )),
    )
}

fn hex_reader(name: &str) -> String {
    format!(
        "stty raw -echo; echo 'acme {name} ready' >&2; \
         dd bs=1 count=1 2>/dev/null | od -An -tx1 | tr -s ' \\n' '  ' | \
         sed 's/^/acme {name} read:/' >&2; exec sleep 600"
    )
}

fn hex_editor() -> Program {
    Program::new("acme-edit", script(&hex_reader("editor")))
}

fn picking_hex() -> Screen {
    let picker = script(&hex_reader("picker"));
    let mut screen = Screen::wide(App::new(reading_agent()).with_picker(Picker::Command(picker)));
    screen.press('n');
    screen.wait_for("acme picker ready");
    screen
}
