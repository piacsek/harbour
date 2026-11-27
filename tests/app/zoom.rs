use std::fs;
use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;
use harbour::editor::Program;
use harbour::handoff::Verb;

use crate::handing::answering;
use crate::handoff::{file, harbour, soon, start_in};
use crate::remembering::{ID, Scratch, TITLE};
use crate::{Screen, numbered_agent, pane_corner, raw_agent, raw_reader, script, settled};

#[test]
fn the_prefix_then_z_from_the_sidebar_shows_the_selected_pane_alone_and_focuses_it() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    zoom(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
    assert_eq!(corners(&shown), 0, "{shown}");
}

#[test]
fn the_prefix_then_z_from_a_zoomed_pane_brings_the_sidebar_back_and_keeps_the_pane_focused() {
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
fn the_prefix_then_z_from_the_claude_code_pane_zooms_it() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    zoom(&mut screen);

    assert_eq!(corners(&screen.draw()), 0);
}

#[test]
fn the_session_never_receives_the_prefix_then_z() {
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

    screen
        .app
        .key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::ALT));

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 1b"), "{shown}");
    assert_eq!(corners(&shown), 2, "{shown}");
}

#[test]
fn the_prefix_then_z_with_no_session_does_nothing() {
    let mut screen = Screen::new(script("exec sleep 600"));
    let before = screen.draw();

    zoom(&mut screen);

    assert_eq!(screen.draw(), before);
}

#[test]
fn the_prefix_then_z_on_a_stopped_card_does_nothing() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(script("exec sleep 600"));
    screen.wait_for(TITLE);
    let before = screen.wait_for_stable();

    zoom(&mut screen);

    let after = settled(&mut screen, |shown| shown != before);
    assert_eq!(after, before);
}

#[test]
fn the_prefix_then_z_on_an_exited_session_does_nothing() {
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

    screen.prefixed('2');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 2 got:");
    assert!(shown.contains("acme agent 2 got: hi"), "{shown}");
    assert_eq!(corners(&shown), 0, "{shown}");
}

#[test]
fn capital_x_answered_yes_while_zoomed_brings_the_sidebar_back() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    zoom(&mut screen);
    screen.prefixed('X');
    screen.wait_for("End and remove? y/n");

    screen.press('y');

    let shown = screen.wait_until("the sidebar", |shown| corners(shown) == 2);
    assert_eq!(corners(&shown), 2, "{shown}");
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
fn the_prefix_then_z_in_the_narrow_layout_shows_the_pane() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.resize(50, 20);

    zoom(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
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
    assert_eq!(corners(&shown), 0, "{shown}");
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

    screen.prefixed('1');

    let shown = screen.wait_for("Not a directory");
    assert_eq!(corners(&shown), 2, "{shown}");
}

#[test]
fn the_prefix_then_z_with_caps_lock_on_zooms() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    screen.prefixed('Z');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
    assert_eq!(corners(&shown), 0, "{shown}");
}

#[test]
fn the_prefix_then_z_from_the_sidebar_on_a_card_showing_its_editor_zooms_claude_code() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing_session(scratch.path());
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 2, 2);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 2, 2);

    zoom(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got:");
    assert!(shown.contains("acme agent 1 got: hi"), "{shown}");
    assert_eq!(corners(&shown), 0, "{shown}");
}

#[test]
fn leaving_zoom_keeps_the_sidebar_scrolled_where_it_was() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.resize(120, 26);
    for started in 1..=12 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {started}"));
    }
    for _ in 0..9 {
        screen.press('k');
    }
    screen.press('j');
    screen.press('j');
    let before = screen.draw();
    assert!(card_at_top(&before).contains("acme-home 3 "), "{before}");

    zoom(&mut screen);
    zoom(&mut screen);

    let shown = screen.draw();
    assert!(card_at_top(&shown).contains("acme-home 3 "), "{shown}");
}

#[test]
fn the_prefix_then_z_from_the_sidebar_clears_the_failure_shown_there() {
    let scratch = tempfile::tempdir().unwrap();
    let editor = Program::new("acme-edit", script("exit 1"));
    let mut screen = Screen::editing(lined_agent(scratch.path()), editor);
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme-edit exited");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    zoom(&mut screen);

    let shown = screen.draw();
    assert!(!shown.contains("acme-edit exited"), "{shown}");
    assert_eq!(corners(&shown), 0, "{shown}");
}

#[test]
fn a_zoomed_session_gets_no_click_on_the_top_row_and_its_origin_from_the_row_below() {
    let mut screen = Screen::new(raw_agent("\\033[?1000h\\033[?1006h", 9));
    screen.press('n');
    screen.wait_for("acme agent ready");
    zoom(&mut screen);
    screen.draw();

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 5, 0);
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 0, 1);

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read:[<0;1;1M"), "{shown}");
}

fn zoom(screen: &mut Screen) {
    screen.prefixed('z');
}

pub(crate) fn corners(shown: &str) -> usize {
    shown
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|c| crate::top_left(*c))
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

fn asking_editor() -> Vec<String> {
    script(
        "stty raw -echo; printf '\\033[>4;2m' >&2; echo 'acme editor ready' >&2; \
         dd bs=1 count=1 2>/dev/null | od -An -tx1 | tr -s ' \\n' '  ' | \
         sed 's/^/acme editor read:/' >&2; exec sleep 600",
    )
}

pub(crate) fn editing_session(scratch: &Path) -> Screen {
    let editor = Program::new("acme-edit", asking_editor());
    let mut screen = Screen::editing(lined_agent(scratch), editor);
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");
    screen
}

fn card_at_top(shown: &str) -> String {
    shown.lines().nth(1).unwrap_or_default().to_string()
}
