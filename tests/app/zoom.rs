use std::fs;
use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::handoff::Verb;
use harbour::input::ESCAPED;

use crate::handing::answering;
use crate::handoff::{file, harbour, soon, start_in};
use crate::remembering::{ID, Scratch, TITLE};
use crate::{Screen, pane_corner, script};

#[test]
fn alt_z_from_the_sidebar_shows_the_selected_pane_alone_and_focuses_it() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    zoom(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
    assert_eq!(corners(&shown), 1, "{shown}");
}

#[test]
fn alt_z_from_a_zoomed_pane_brings_the_sidebar_back_and_keeps_the_pane_focused() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    zoom(&mut screen);

    zoom(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
    assert_eq!(corners(&shown), 2, "{shown}");
}

#[test]
fn alt_z_from_the_claude_code_pane_zooms_it() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    zoom(&mut screen);

    assert_eq!(corners(&screen.draw()), 1);
}

#[test]
fn the_session_never_receives_alt_z() {
    let mut screen = Screen::new(raw_reader(1));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    zoom(&mut screen);
    screen.press('a');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 61"), "{shown}");
}

#[test]
fn alt_z_read_after_an_escape_reaches_the_session_and_zooms_nothing() {
    let mut screen = Screen::new(raw_reader(1));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.app.key(KeyEvent::new_with_kind_and_state(
        KeyCode::Char('z'),
        KeyModifiers::ALT,
        KeyEventKind::Press,
        ESCAPED,
    ));

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 1b"), "{shown}");
    assert_eq!(corners(&shown), 2, "{shown}");
}

#[test]
fn alt_z_with_no_session_does_nothing() {
    let mut screen = Screen::new(script("exec sleep 600"));
    let before = screen.draw();

    zoom(&mut screen);

    assert_eq!(screen.draw(), before);
}

#[test]
fn alt_z_on_a_stopped_card_does_nothing() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(script("exec sleep 600"));
    let before = screen.wait_for(TITLE);

    zoom(&mut screen);

    assert_eq!(screen.draw(), before);
}

#[test]
fn alt_z_on_an_exited_session_does_nothing() {
    let mut screen = Screen::new(script("echo 'acme agent done'"));
    screen.press('n');
    let before = screen.wait_for("exited");

    zoom(&mut screen);

    assert_eq!(corners(&screen.draw()), 2, "{before}");
}

#[test]
fn ctrl_h_from_a_zoomed_claude_code_pane_brings_the_sidebar_back_focused() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    zoom(&mut screen);

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('k');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 2 got:");
    assert!(shown.contains("acme agent 2 got: hi"), "{shown}");
    assert_eq!(corners(&shown), 2, "{shown}");
}

#[test]
fn the_zoomed_session_exiting_brings_the_sidebar_back() {
    let mut screen = Screen::new(script("echo 'acme agent ready'; read -r line"));
    screen.press('n');
    screen.wait_for("acme agent ready");
    zoom(&mut screen);

    screen.type_text("bye\r");

    let shown = screen.wait_until("the sidebar", |shown| corners(shown) == 2);
    assert_eq!(corners(&shown), 2, "{shown}");
}

#[test]
fn a_jump_while_zoomed_shows_the_jumped_to_session_still_zoomed() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    zoom(&mut screen);

    screen.key(KeyCode::Char('2'), KeyModifiers::CONTROL);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 2 got:");
    assert!(shown.contains("acme agent 2 got: hi"), "{shown}");
    assert_eq!(corners(&shown), 1, "{shown}");
}

#[test]
fn leaving_zoom_brings_back_the_configured_sidebar_width() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::of(App::new(lined_agent(scratch.path())).with_sidebar_width(30));
    screen.press('n');
    screen.wait_for("acme agent 1");
    zoom(&mut screen);

    zoom(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 30);
}

#[test]
fn alt_z_in_the_narrow_layout_shows_the_pane() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.resize(50, 20);

    zoom(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
}

#[test]
fn alt_z_while_the_editor_has_focus_reaches_the_editor() {
    let scratch = tempfile::tempdir().unwrap();
    let editor = Program::new("acme-edit", asking_editor());
    let mut screen = Screen::editing(lined_agent(scratch.path()), editor);
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");

    zoom(&mut screen);

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read: 1b"), "{shown}");
    assert_eq!(corners(&shown), 2, "{shown}");
}

#[test]
fn harbour_edit_opening_a_file_in_another_session_keeps_the_zoom() {
    let scratch = tempfile::tempdir().unwrap();
    let _asked = answering(scratch.path());
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    start_in(&mut screen, scratch.path(), "acme-web", 2);
    zoom(&mut screen);

    screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    let shown = screen.wait_for("acme nvim ready");
    assert!(shown.contains("acme nvim ready"), "{shown}");
    assert_eq!(corners(&shown), 1, "{shown}");
}

#[test]
fn a_refused_jump_while_zoomed_brings_the_sidebar_back() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(script(
        "[ \"$1\" ] || echo 'acme agent live'; exec sleep 600",
    ));
    screen.wait_for(TITLE);
    screen.press('n');
    screen.wait_for("acme agent live");
    zoom(&mut screen);
    fs::remove_dir_all(scratch.checkout()).unwrap();

    screen.key(KeyCode::Char('1'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("Not a directory");
    assert_eq!(corners(&shown), 2, "{shown}");
}

fn zoom(screen: &mut Screen) {
    screen.key(KeyCode::Char('z'), KeyModifiers::ALT);
}

fn corners(shown: &str) -> usize {
    shown
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|c| matches!(c, '┌' | '┏'))
        .count()
}

fn three_sessions(scratch: &Path) -> Screen {
    let mut screen = Screen::new(lined_agent(scratch));
    for n in 1..=3 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {n}"));
    }
    screen
}

fn lined_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         echo \"acme agent $n\"; \
         while read -r line; do echo \"acme agent $n got: $line\"; done; exec sleep 600",
        count = count.display()
    ))
}

fn raw_reader(length: usize) -> Vec<String> {
    script(&format!(
        "stty raw -echo; echo 'acme agent ready' >&2; \
         dd bs=1 count={length} 2>/dev/null | od -An -tx1 | tr -s ' \\n' '  ' | \
         sed 's/^/acme agent read:/' >&2; exec sleep 600"
    ))
}

fn asking_editor() -> Vec<String> {
    script(
        "stty raw -echo; printf '\\033[>4;2m' >&2; echo 'acme editor ready' >&2; \
         dd bs=1 count=1 2>/dev/null | od -An -tx1 | tr -s ' \\n' '  ' | \
         sed 's/^/acme editor read:/' >&2; exec sleep 600",
    )
}
