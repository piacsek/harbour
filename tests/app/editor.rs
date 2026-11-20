use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::editor::Program;
use ratatui::style::Modifier;

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

#[test]
fn ctrl_h_in_the_editor_shows_claude_code_again_and_keys_reach_it() {
    let mut screen = editor_open();

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent got: hi");
    assert!(shown.contains("acme agent got: hi"), "{shown}");
    assert!(!shown.contains("acme editor in"), "{shown}");
}

#[test]
fn ctrl_l_from_claude_code_shows_the_same_editor_again() {
    let mut screen = editor_open();
    screen.type_text("hello\r");
    screen.wait_for("acme editor got: hello");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.wait_for("acme agent ready");

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor got: hello");
    assert!(shown.contains("acme editor got: hello"), "{shown}");
    assert!(!shown.contains("acme agent ready"), "{shown}");
}

#[test]
fn enter_in_the_sidebar_goes_back_to_the_editor_the_pane_showed() {
    let mut screen = editor_open();
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 1, 1);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 1, 1);

    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("back\r");

    let shown = screen.wait_for("acme editor got: back");
    assert!(shown.contains("acme editor got: back"), "{shown}");
    assert!(!shown.contains("acme agent got"), "{shown}");
}

#[test]
fn ctrl_l_in_the_sidebar_goes_back_to_the_editor_the_pane_showed() {
    let mut screen = editor_open();
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 1, 1);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 1, 1);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.type_text("back\r");

    let shown = screen.wait_for("acme editor got: back");
    assert!(shown.contains("acme editor got: back"), "{shown}");
    assert!(!shown.contains("acme agent got"), "{shown}");
}

#[test]
fn the_pane_border_names_claude_code_and_the_editor_before_one_starts() {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');

    let shown = screen.wait_for("acme agent ready");

    let top = shown.lines().next().unwrap();
    assert!(top.contains(" claude | acme-edit "), "{top}");
}

#[test]
fn the_pane_border_names_the_directory_of_the_editor_once_it_started() {
    let mut screen = editor_open();
    screen.resize(wide_enough(), 20);

    let shown = screen.draw();

    let top = shown.lines().next().unwrap();
    let expected = format!(" claude | acme-edit: {} ", shown_directory());
    assert!(top.contains(&expected), "{top}");
}

#[test]
fn the_pane_border_marks_claude_code_while_it_is_shown() {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');
    let shown = screen.wait_for("acme agent ready");

    let (claude, editor) = tab_columns(&shown);

    assert!(screen.cell(claude, 0).modifier.contains(Modifier::BOLD));
    assert!(screen.cell(editor, 0).modifier.contains(Modifier::DIM));
}

#[test]
fn the_pane_border_marks_the_editor_while_it_is_shown() {
    let mut screen = editor_open();
    let shown = screen.draw();

    let (claude, editor) = tab_columns(&shown);

    assert!(screen.cell(claude, 0).modifier.contains(Modifier::DIM));
    assert!(screen.cell(editor, 0).modifier.contains(Modifier::BOLD));
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

fn editor_open() -> Screen {
    let mut screen = Screen::editing(reading_agent(), reading_editor());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor in");
    screen
}

fn shown_directory() -> String {
    let directory = std::env::current_dir().unwrap();
    let home = std::env::var_os("HOME").map(std::path::PathBuf::from);
    match home.and_then(|home| directory.strip_prefix(home).ok().map(ToOwned::to_owned)) {
        Some(below) => format!("~/{}", below.display()),
        None => directory.display().to_string(),
    }
}

fn wide_enough() -> u16 {
    u16::try_from(3 * (std::env::current_dir().unwrap().as_os_str().len() + 40)).unwrap()
}

fn tab_columns(shown: &str) -> (u16, u16) {
    let top: Vec<char> = shown.lines().next().unwrap().chars().collect();
    let text: String = top.iter().collect();
    let claude = text.find(" claude | ").unwrap();
    let claude = text[..claude].chars().count() + 1;
    let editor = claude + "claude | ".chars().count();
    (
        u16::try_from(claude).unwrap(),
        u16::try_from(editor).unwrap(),
    )
}
