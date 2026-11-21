use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use harbour::editor::Program;
use harbour::input::ESCAPED;

use crate::remembering::{ID, Scratch, TITLE};
use crate::zoom::{corners, editing_session, raw_reader};
use crate::{Screen, pane_corner, region, script};

#[test]
fn alt_backslash_from_the_sidebar_shows_claude_code_left_and_its_editor_right() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path());

    side(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for_all(&["acme editor ready", "acme agent 1 got: hi"]);
    assert_eq!(pane_corner(&shown), 60, "{shown}");
    assert!(
        region(&shown, 0, 60).contains("acme agent 1 got: hi"),
        "{shown}"
    );
    assert!(
        region(&shown, 60, 60).contains("acme editor ready"),
        "{shown}"
    );
}

#[test]
fn side_by_side_sizes_claude_code_and_its_editor_to_their_halves() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path());

    screen
        .app
        .key(KeyEvent::new(KeyCode::Char('\\'), KeyModifiers::ALT));
    sleep(Duration::from_millis(300));

    let shown = screen.wait_for_all(&["acme editor ready", "acme agent 1 size:"]);
    assert!(shown.contains("acme editor ready 18 58"), "{shown}");
    assert!(shown.contains("acme agent 1 size: 18 58"), "{shown}");
}

#[test]
fn alt_backslash_on_a_stopped_card_does_nothing() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(script("exec sleep 600"));
    let before = screen.wait_for(TITLE);

    side(&mut screen);

    assert_eq!(screen.draw(), before);
}

#[test]
fn alt_backslash_on_an_exited_session_does_nothing() {
    let editor = Program::new("acme-edit", editor());
    let mut screen = Screen::editing(script("echo 'acme agent done'"), editor);
    screen.press('n');
    let before = screen.wait_for("exited");

    side(&mut screen);
    sleep(Duration::from_millis(300));

    assert_eq!(screen.draw(), before);
}

#[test]
fn alt_backslash_under_120_columns_does_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path());
    screen.resize(119, 20);
    let before = screen.wait_for("acme agent 1 size: 18 77");

    side(&mut screen);
    sleep(Duration::from_millis(300));

    assert_eq!(screen.draw(), before);
}

#[test]
fn alt_backslash_while_the_editor_has_focus_reaches_the_editor() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing_session(scratch.path());

    side(&mut screen);

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read: 1b"), "{shown}");
    assert_eq!(pane_corner(&shown), 40, "{shown}");
}

#[test]
fn alt_backslash_does_not_split_when_the_editor_cannot_start() {
    let missing = Program::new("acme-edit", vec!["acme-editor-missing".to_string()]);
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::editing(agent(scratch.path()), missing);
    screen.press('n');
    screen.wait_for("acme agent 1");

    side(&mut screen);

    let shown = screen.wait_for("Could not start acme-edit");
    assert_eq!(pane_corner(&shown), 40, "{shown}");
}

#[test]
fn alt_backslash_again_brings_the_sidebar_back_with_claude_code_focused() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path());
    side(&mut screen);
    screen.wait_for("acme editor ready");

    side(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert_eq!(pane_corner(&shown), 40, "{shown}");
    assert!(!shown.contains("acme editor ready"), "{shown}");
}

#[test]
fn alt_z_from_side_by_side_zooms_claude_code() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path());
    side(&mut screen);
    screen.wait_for("acme editor ready");

    zoom(&mut screen);

    let shown = screen.draw();
    assert_eq!(corners(&shown), 1, "{shown}");
    assert!(shown.contains("acme agent 1"), "{shown}");
    assert!(!shown.contains("acme editor ready"), "{shown}");
}

#[test]
fn alt_backslash_from_a_zoom_shows_side_by_side_and_again_the_sidebar() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path());
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    zoom(&mut screen);

    side(&mut screen);
    screen.wait_for("acme editor ready");
    side(&mut screen);

    let shown = screen.draw();
    assert_eq!(pane_corner(&shown), 40, "{shown}");
}

#[test]
fn alt_backslash_with_no_session_does_nothing() {
    let editor = Program::new("acme-edit", editor());
    let mut screen = Screen::editing(script("exec sleep 600"), editor);
    let before = screen.draw();

    side(&mut screen);

    assert_eq!(screen.draw(), before);
}

#[test]
fn alt_backslash_read_after_an_escape_reaches_claude_code_and_splits_nothing() {
    let editor = Program::new("acme-edit", editor());
    let mut screen = Screen::editing(raw_reader(2), editor);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.app.key(KeyEvent::new_with_kind_and_state(
        KeyCode::Char('\\'),
        KeyModifiers::ALT,
        KeyEventKind::Press,
        ESCAPED,
    ));

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 1b 5c"), "{shown}");
    assert_eq!(pane_corner(&shown), 40, "{shown}");
}

#[test]
fn the_width_keys_do_nothing_side_by_side() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing(scratch.path());
    side(&mut screen);
    screen.wait_for("acme editor ready");

    screen.key(KeyCode::Char('='), KeyModifiers::ALT);
    side(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert_eq!(pane_corner(&shown), 40, "{shown}");
    assert!(!shown.contains("got: ="), "{shown}");
}

fn zoom(screen: &mut Screen) {
    screen.key(KeyCode::Char('z'), KeyModifiers::ALT);
}

fn side(screen: &mut Screen) {
    screen.key(KeyCode::Char('\\'), KeyModifiers::ALT);
}

fn editing(scratch: &Path) -> Screen {
    let editor = Program::new("acme-edit", editor());
    let mut screen = Screen::editing(agent(scratch), editor);
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen
}

fn agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         trap 'echo \"acme agent $n size: $(stty size)\"' WINCH; \
         echo \"acme agent $n\"; \
         while :; do read -r line && echo \"acme agent $n got: $line\"; done",
        count = count.display()
    ))
}

fn editor() -> Vec<String> {
    script(
        "trap 'echo \"acme editor size: $(stty size)\"' WINCH; \
         echo \"acme editor ready $(stty size)\"; \
         while :; do read -r line && echo \"acme editor got: $line\"; done",
    )
}
