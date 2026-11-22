use std::fs;
use std::path::Path;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::picker::Picker;

use crate::remembering::{ID, OTHER_ID, Scratch, TITLE};
use crate::{Screen, script};

#[test]
fn the_prefix_then_two_from_the_sidebar_selects_the_second_card_and_focuses_its_pane() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    ctrl(&mut screen, '2');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 2 got:");
    assert!(shown.contains("acme agent 2 got: hi"), "{shown}");
}

#[test]
fn the_prefix_then_one_from_inside_a_pane_jumps_and_the_pane_gets_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    ctrl(&mut screen, '1');
    screen.type_text("hi\r");
    screen.wait_for("acme agent 1 got:");
    ctrl(&mut screen, '3');
    screen.type_text("yo\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: yo"), "{shown}");
}

#[test]
fn the_prefix_then_nine_jumps_to_the_last_card() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.press('k');
    screen.press('k');

    ctrl(&mut screen, '9');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
}

#[test]
fn ctrl_on_the_selected_card_focuses_its_pane() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    ctrl(&mut screen, '3');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
}

#[test]
fn ctrl_five_with_three_cards_does_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.press('k');
    screen.press('k');

    ctrl(&mut screen, '5');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got:");
    assert!(shown.contains("acme agent 1 got: hi"), "{shown}");
}

#[test]
fn ctrl_five_with_three_cards_from_inside_a_pane_reaches_no_session() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    ctrl(&mut screen, '5');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
    assert!(!shown.contains("^]"), "{shown}");
}

#[test]
fn the_prefix_then_nine_with_no_card_does_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(lined_agent(scratch.path()));

    let before = screen.draw();

    ctrl(&mut screen, '9');

    assert_eq!(screen.draw(), before);
}

#[test]
fn the_narrow_layout_shows_the_pane_of_the_card_jumped_to() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.resize(50, 20);

    ctrl(&mut screen, '2');

    let shown = screen.wait_for("acme agent 2");
    assert!(shown.contains("acme agent 2"), "{shown}");
    assert!(!shown.contains("acme agent 3"), "{shown}");
}

#[test]
fn the_help_view_takes_the_key_and_nothing_jumps() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.prefixed('?');

    ctrl(&mut screen, '1');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
}

#[test]
fn a_question_takes_the_key_and_nothing_jumps() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.press('x');

    ctrl(&mut screen, '1');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
}

#[test]
fn the_picker_gets_the_prefix_and_nothing_jumps() {
    let picker = script(
        "stty raw -echo; echo 'acme picker ready' >&2; \
         r=$(dd bs=1 count=2 2>/dev/null); echo \"acme picker read:$r\" >&2; exec sleep 600",
    );
    let mut screen =
        Screen::wide(App::new(script("exec sleep 600")).with_picker(Picker::Command(picker)));
    screen.press('n');
    screen.wait_for("acme picker ready");

    ctrl(&mut screen, '1');

    let shown = screen.wait_for("acme picker read:");
    assert!(shown.contains("acme picker read:1"), "{shown}");
}

#[test]
fn an_editor_that_asked_for_extended_keys_gets_nothing_from_a_jump() {
    let scratch = tempfile::tempdir().unwrap();
    let editor = Program::new("acme-edit", asking_editor());
    let mut screen = Screen::editing(lined_agent(scratch.path()), editor);
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");

    ctrl(&mut screen, '1');
    screen.type_text("hi\r");
    screen.wait_for("acme agent 1 got:");
    ctrl(&mut screen, '2');
    screen.press('z');

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read: 7a"), "{shown}");
}

#[test]
fn the_prefix_then_two_resumes_the_second_stopped_card() {
    let scratch = Scratch::new();
    scratch.remember(ID, "The first one", 0);
    scratch.remember(OTHER_ID, TITLE, 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(resuming_agent());
    screen.wait_for(TITLE);

    ctrl(&mut screen, '2');
    screen.wait_for(&format!("acme agent --resume {OTHER_ID}"));
    screen.type_text("hello\r");

    let shown = screen.wait_for("acme agent got:");
    assert!(shown.contains("acme agent got: hello"), "{shown}");
}

#[test]
fn a_stopped_card_whose_directory_is_gone_says_why_as_enter_would() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    fs::remove_dir_all(scratch.checkout()).unwrap();
    let mut screen = scratch.watching(resuming_agent());
    screen.wait_for(TITLE);

    ctrl(&mut screen, '1');

    let shown = screen.wait_for("Not a directory");
    assert!(shown.contains("Not a directory"), "{shown}");
    assert!(!shown.contains("acme agent --resume"), "{shown}");
}

#[test]
fn the_keys_a_terminal_without_extended_keys_sends_for_ctrl_one_to_ctrl_nine_reach_the_session() {
    let mut screen = Screen::new(raw_reader(8));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.press('1');
    screen.key(KeyCode::Char(' '), KeyModifiers::CONTROL);
    screen.key(KeyCode::Esc, KeyModifiers::NONE);
    screen.key(KeyCode::Char('\\'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char(']'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('^'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Backspace, KeyModifiers::NONE);
    screen.press('9');

    let shown = screen.wait_for("acme agent read:");
    assert!(
        shown.contains("acme agent read: 31 00 1b 1c 1d 1e 7f 39"),
        "{shown}"
    );
}

#[test]
fn a_jump_from_a_pane_to_a_card_whose_directory_is_gone_leaves_the_sidebar_focused() {
    let (scratch, mut screen) = live_beside_stopped(120);
    fs::remove_dir_all(scratch.checkout()).unwrap();

    ctrl(&mut screen, '1');

    let shown = screen.wait_for("Not a directory");
    assert_sidebar_focused(&mut screen, &shown);
}

#[test]
fn a_jump_from_a_pane_to_a_card_whose_directory_is_gone_leaves_the_sidebar_focused_when_narrow() {
    let (scratch, mut screen) = live_beside_stopped(59);
    fs::remove_dir_all(scratch.checkout()).unwrap();

    ctrl(&mut screen, '1');

    let shown = screen.wait_for("Not a directory");
    assert_sidebar_focused(&mut screen, &shown);
}

#[test]
fn a_jump_from_a_pane_to_a_card_open_elsewhere_leaves_the_sidebar_focused() {
    let (scratch, mut screen) = live_beside_stopped(120);
    hold(&scratch);

    ctrl(&mut screen, '1');

    let shown = screen.wait_for("is open elsewhere");
    assert_sidebar_focused(&mut screen, &shown);
}

#[test]
fn a_jump_from_a_pane_to_a_card_open_elsewhere_leaves_the_sidebar_focused_when_narrow() {
    let (scratch, mut screen) = live_beside_stopped(59);
    hold(&scratch);

    ctrl(&mut screen, '1');

    let shown = screen.wait_for("is open elsewhere");
    assert_sidebar_focused(&mut screen, &shown);
}

#[test]
fn a_jump_from_a_pane_to_a_card_whose_transcript_is_gone_leaves_the_sidebar_focused() {
    let (scratch, mut screen) = live_beside_stopped(120);
    fs::remove_file(scratch.transcript_file(ID)).unwrap();

    ctrl(&mut screen, '1');

    let shown = screen.wait_for("This conversation is gone");
    assert_sidebar_focused(&mut screen, &shown);
}

#[test]
fn a_jump_from_a_pane_to_a_card_whose_transcript_is_gone_leaves_the_sidebar_focused_when_narrow() {
    let (scratch, mut screen) = live_beside_stopped(59);
    fs::remove_file(scratch.transcript_file(ID)).unwrap();

    ctrl(&mut screen, '1');

    let shown = screen.wait_for("This conversation is gone");
    assert_sidebar_focused(&mut screen, &shown);
}

#[test]
fn a_jump_in_the_narrow_layout_clears_the_message_drawn_over_the_pane() {
    let scratch = tempfile::tempdir().unwrap();
    let editor = Program::new("acme-edit", script("exit 3"));
    let mut screen = Screen::of(App::new(lined_agent(scratch.path())).with_editor(editor));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");
    screen.resize(59, 20);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme-edit exited 3");

    ctrl(&mut screen, '1');

    let shown = screen.wait_for("acme agent 1");
    assert!(!shown.contains("acme-edit exited"), "{shown}");
}

fn ctrl(screen: &mut Screen, digit: char) {
    screen.prefixed(digit);
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

fn resuming_agent() -> Vec<String> {
    script(
        "echo \"acme agent $0 $1 in $(basename \"$PWD\")\"; \
         while :; do read -r line || continue; echo \"acme agent got: $line\"; done",
    )
}

fn raw_reader(length: usize) -> Vec<String> {
    script(&format!(
        "stty raw -echo; echo 'acme agent ready' >&2; \
         dd bs=1 count={length} 2>/dev/null | od -An -tx1 | tr -s ' \\n' '  ' | \
         sed 's/^/acme agent read:/' >&2; exec sleep 600"
    ))
}

const LIVE: &str = "acme agent live";

fn live_beside_stopped(width: u16) -> (Scratch, Screen) {
    let scratch = Scratch::new();
    scratch.remember(ID, "The first one", 0);
    scratch.remember(OTHER_ID, TITLE, 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(script(&format!(
        "[ \"$1\" ] || echo '{LIVE}'; exec sleep 600"
    )));
    screen.resize(width, 20);
    screen.wait_for(TITLE);
    screen.press('n');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for(LIVE);
    (scratch, screen)
}

pub(crate) fn hold(scratch: &Scratch) {
    fs::write(
        scratch
            .sessions()
            .join(format!("{}.json", std::process::id())),
        format!(r#"{{"status":"idle","sessionId":"{ID}"}}"#),
    )
    .unwrap();
}

pub(crate) fn assert_sidebar_focused(screen: &mut Screen, shown: &str) {
    assert!(
        shown.lines().next().unwrap().starts_with("┏ sessions"),
        "{shown}"
    );
    let before = selected_line(shown);
    screen.press('j');
    let moved = screen.draw();
    assert_ne!(selected_line(&moved), before, "{moved}");
}

fn selected_line(shown: &str) -> String {
    shown
        .lines()
        .find(|line| line.contains('▌'))
        .unwrap_or_default()
        .to_string()
}
