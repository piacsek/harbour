use std::fs;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::handoff::{Reply, Verb};
use harbour::picker::Picker;

use crate::finding::FINDER;
use crate::handing::{Edit, answering, neovim_at};
use crate::handoff::{app_running, file, harbour, recorded, soon, start_in};
use crate::jumping::{assert_sidebar_focused, hold};
use crate::remembering::{ID, Scratch, TITLE};
use crate::side::{agent, editing, editor, side};
use crate::zoom::corners;
use crate::{Screen, ends_with_a_pane_corner, needs_a_terminal, pane_corner, region};

#[test]
fn a_jump_from_the_claude_code_half_to_a_session_last_in_claude_code_focuses_its_claude_code() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_side_by_side(scratch.path(), Half::Claude, Half::Claude);

    ctrl(&mut screen, '2');

    assert_claude_code_focused(&mut screen);
}

#[test]
fn a_jump_from_the_editor_half_to_a_session_last_in_claude_code_focuses_its_claude_code() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_side_by_side(scratch.path(), Half::Editor, Half::Claude);

    ctrl(&mut screen, '2');

    assert_claude_code_focused(&mut screen);
}

#[test]
fn a_jump_from_the_claude_code_half_to_a_session_last_in_its_editor_focuses_its_editor() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_side_by_side(scratch.path(), Half::Claude, Half::Editor);

    ctrl(&mut screen, '2');

    assert_editor_focused(&mut screen);
}

#[test]
fn a_jump_from_the_editor_half_to_a_session_last_in_its_editor_focuses_its_editor() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_side_by_side(scratch.path(), Half::Editor, Half::Editor);

    ctrl(&mut screen, '2');

    assert_editor_focused(&mut screen);
}

#[test]
fn a_pick_from_the_session_finder_to_a_session_last_in_its_editor_focuses_its_editor() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(agent(scratch.path()))
        .with_editor(Program::new("acme-edit", editor()))
        .with_picker(Picker::Command(vec![FINDER.into()]));
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent 1");
    second_session(&mut screen, Half::Editor);
    split_first(&mut screen);

    screen.prefixed('s');
    screen.wait_for("acme fzf ready");
    screen.type_text("home 2\r");

    screen.wait_until("the finder closed", |shown| !shown.contains("acme fzf"));
    assert_editor_focused(&mut screen);
}

#[test]
fn a_jump_to_a_session_whose_editor_cannot_start_zooms_its_claude_code_with_the_message() {
    let scratch = tempfile::tempdir().unwrap();
    let editor = Program::new("acme-edit", editor());
    let mut screen = Screen::of(app_running(scratch.path(), agent(scratch.path()), editor));
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    start_in(&mut screen, scratch.path(), "acme-web", 2);
    split_first(&mut screen);
    fs::remove_dir_all(scratch.path().join("acme-web")).unwrap();

    ctrl(&mut screen, '2');

    let shown = screen.wait_for("Not a directory");
    assert_eq!(corners(&shown), 0, "{shown}");
    assert!(!ends_with_a_pane_corner(&shown), "{shown}");
    assert!(shown.contains("acme agent 2"), "{shown}");
    assert!(!shown.contains("acme editor ready"), "{shown}");
    assert!(screen.cursor().is_some(), "{shown}");
}

#[test]
fn a_jump_under_120_columns_keeps_the_mode_and_splits_again_once_wider() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_side_by_side(scratch.path(), Half::Claude, Half::Claude);
    screen.resize(100, 20);

    ctrl(&mut screen, '2');

    let narrow = screen.wait_for("acme agent 2");
    assert_eq!(corners(&narrow), 1, "{narrow}");
    assert!(!narrow.contains("acme editor ready"), "{narrow}");
    assert!(screen.cursor().is_some(), "{narrow}");
    screen.resize(120, 20);
    assert_claude_code_focused(&mut screen);
}

#[test]
fn a_jump_to_a_stopped_card_resumes_it_side_by_side() {
    needs_a_terminal();
    let scratch = Scratch::new();
    let mut screen = live_beside_stopped(&scratch);

    ctrl(&mut screen, '1');

    assert_claude_code_focused(&mut screen);
}

#[test]
fn a_refused_jump_to_a_stopped_card_brings_the_sidebar_back_focused() {
    let scratch = Scratch::new();
    let mut screen = live_beside_stopped(&scratch);
    hold(&scratch);

    ctrl(&mut screen, '1');

    let shown = screen.wait_for("is open elsewhere");
    assert!(screen.cursor().is_none(), "{shown}");
    assert_sidebar_focused(&mut screen, &shown);
}

#[test]
fn harbour_edit_into_another_session_shows_it_side_by_side_with_the_file_on_the_right() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_neovims(scratch.path());
    let _asked = answering(scratch.path());

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-web"), soon());

    assert_eq!(reply, Reply::Opened);
    assert_neovim_focused(&mut screen, "acme agent 2");
}

#[test]
fn harbour_edit_into_the_shown_session_keeps_it_side_by_side_with_the_editor_focused() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_neovims(scratch.path());
    let socket = PathBuf::from(&recorded(scratch.path())[2]);
    let _asked = neovim_at(&socket, false, Edit::Done);

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Opened);
    assert_neovim_focused(&mut screen, "acme agent 1");
}

#[derive(Clone, Copy, PartialEq)]
enum Half {
    Claude,
    Editor,
}

fn two_side_by_side(scratch: &Path, from: Half, target: Half) -> Screen {
    let mut screen = editing(scratch);
    second_session(&mut screen, target);
    split_first(&mut screen);
    if from == Half::Editor {
        screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
        assert!(screen.cursor().unwrap().x > 60);
    }
    screen
}

fn second_session(screen: &mut Screen, last: Half) {
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('n');
    screen.wait_for("acme agent 2");
    if last == Half::Editor {
        screen.key(KeyCode::Enter, KeyModifiers::NONE);
        screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
        screen.wait_for("acme editor ready");
    }
}

fn split_first(screen: &mut Screen) {
    ctrl(screen, '1');
    side(screen);
    screen.wait_until("the first session side by side", |shown| {
        region(shown, 0, 60).contains("acme agent 1")
            && region(shown, 60, 60).contains("acme editor ready")
    });
}

fn two_neovims(scratch: &Path) -> Screen {
    let mut screen = harbour(scratch);
    fs::create_dir_all(scratch.join("acme-web/src")).unwrap();
    start_in(&mut screen, scratch, "acme-api", 1);
    start_in(&mut screen, scratch, "acme-web", 2);
    ctrl(&mut screen, '1');
    side(&mut screen);
    screen.wait_until("the first session side by side", |shown| {
        region(shown, 0, 60).contains("acme agent 1")
            && region(shown, 60, 60).contains("acme nvim ready")
    });
    screen
}

fn live_beside_stopped(scratch: &Scratch) -> Screen {
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let app = scratch
        .app(agent(scratch.dir.path()))
        .with_editor(Program::new("acme-edit", editor()));
    let mut screen = Screen::of(app);
    screen.wait_for(TITLE);
    screen.press('n');
    screen.wait_for("acme agent 1");
    side(&mut screen);
    screen.wait_until("the live session side by side", |shown| {
        region(shown, 0, 60).contains("acme agent 1")
            && region(shown, 60, 60).contains("acme editor ready")
    });
    screen
}

fn ctrl(screen: &mut Screen, digit: char) {
    screen.prefixed(digit);
}

fn assert_claude_code_focused(screen: &mut Screen) {
    let shown = screen.wait_until("the second session side by side", |shown| {
        region(shown, 0, 60).contains("acme agent 2")
            && region(shown, 60, 60).contains("acme editor ready")
    });
    assert_eq!(pane_corner(&shown), 60, "{shown}");
    assert!(screen.cursor().unwrap().x < 60, "{shown}");
    screen.type_text("hi\r");
    let typed = screen.wait_for("got: hi");
    assert!(
        region(&typed, 0, 60).contains("acme agent 2 got: hi"),
        "{typed}"
    );
}

fn assert_editor_focused(screen: &mut Screen) {
    let shown = screen.wait_until("the second session side by side", |shown| {
        region(shown, 0, 60).contains("acme agent 2")
            && region(shown, 60, 60).contains("acme editor ready")
    });
    assert_eq!(pane_corner(&shown), 60, "{shown}");
    assert!(screen.cursor().unwrap().x > 60, "{shown}");
    screen.type_text("hi\r");
    let typed = screen.wait_for("got: hi");
    assert!(
        region(&typed, 60, 60).contains("acme editor got: hi"),
        "{typed}"
    );
}

fn assert_neovim_focused(screen: &mut Screen, agent: &str) {
    let shown = screen.wait_until("the holder side by side", |shown| {
        region(shown, 0, 60).contains(agent) && region(shown, 60, 60).contains("acme nvim ready")
    });
    assert_eq!(pane_corner(&shown), 60, "{shown}");
    assert!(screen.cursor().unwrap().x > 60, "{shown}");
    screen.press('x');
    let typed = screen.wait_for("acme nvim read:78");
    assert!(
        region(&typed, 60, 60).contains("acme nvim read:78"),
        "{typed}"
    );
}
