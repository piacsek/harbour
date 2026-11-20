use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::handoff::{Claim, Reply, Verb};

use crate::directory::repository;
use crate::handing::answering;
use crate::handoff::{
    app_running, file, harbour, last_use, opened_beside, recording_neovim, soon, start_in,
};
use crate::{Screen, raw_agent};

#[test]
fn a_session_deeper_in_the_tree_takes_the_file_over_the_selected_home_session() {
    let scratch = tempfile::tempdir().unwrap();
    let _asked = answering(scratch.path());
    let mut screen = harbour(scratch.path());
    repository(scratch.path(), "acme-api");
    start_at(&mut screen, scratch.path(), "", 1);
    start_at(&mut screen, scratch.path(), "acme-api", 2);
    screen.press('k');

    screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(opened_beside(&mut screen), "acme agent 2");
}

#[test]
fn a_worktree_session_takes_its_files_over_the_selected_repository_session() {
    let scratch = tempfile::tempdir().unwrap();
    let _asked = answering(scratch.path());
    let mut screen = harbour(scratch.path());
    repository(scratch.path(), "acme-api");
    repository(scratch.path(), WORKTREE);
    fs::create_dir_all(scratch.path().join(WORKTREE).join("src")).unwrap();
    start_at(&mut screen, scratch.path(), "acme-api", 1);
    start_at(&mut screen, scratch.path(), WORKTREE, 2);
    screen.press('k');

    screen
        .app
        .hand(Verb::Open, &file(scratch.path(), WORKTREE), soon());

    assert_eq!(opened_beside(&mut screen), "acme agent 2");
}

#[test]
fn an_ask_answers_with_how_deep_the_holders_root_lies() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    repository(scratch.path(), "acme-api");
    start_at(&mut screen, scratch.path(), "acme-api", 1);
    let root = scratch.path().canonicalize().unwrap().join("acme-api");

    let reply = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-api"), soon());

    let Reply::Holds(Claim {
        depth, selected, ..
    }) = reply
    else {
        panic!("not held: {reply:?}");
    };
    assert_eq!(depth, root.components().count());
    assert!(selected);
}

#[test]
fn a_key_typed_into_a_session_moves_its_last_use_forward() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let earlier = last_use(&mut screen, scratch.path());
    sleep(Duration::from_millis(5));

    screen.press('a');

    assert!(last_use(&mut screen, scratch.path()) > earlier);
}

#[test]
fn a_paste_into_a_session_moves_its_last_use_forward() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let earlier = last_use(&mut screen, scratch.path());
    sleep(Duration::from_millis(5));

    screen.app.paste("acme");

    assert!(last_use(&mut screen, scratch.path()) > earlier);
}

#[test]
fn a_click_sent_to_a_session_moves_its_last_use_forward() {
    let scratch = tempfile::tempdir().unwrap();
    let agent = raw_agent("\\033[?1000h\\033[?1006h", 9);
    let mut screen = Screen::of(app_running(
        scratch.path(),
        agent,
        recording_neovim(scratch.path()),
    ));
    fs::write(
        scratch.path().join("next"),
        scratch.path().join("acme-api").display().to_string(),
    )
    .unwrap();
    screen.press('n');
    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
    let earlier = last_use(&mut screen, scratch.path());
    sleep(Duration::from_millis(5));

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 45, 3);

    assert!(last_use(&mut screen, scratch.path()) > earlier);
}

#[test]
fn opening_the_editor_beside_a_session_moves_its_last_use_forward() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let earlier = last_use(&mut screen, scratch.path());
    sleep(Duration::from_millis(5));

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    assert!(last_use(&mut screen, scratch.path()) > earlier);
}

#[test]
fn moving_over_a_session_leaves_its_last_use_alone() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    start_in(&mut screen, scratch.path(), "acme-web", 2);
    let earlier = last_use(&mut screen, scratch.path());
    sleep(Duration::from_millis(5));

    screen.press('k');
    screen.press('j');

    assert_eq!(last_use(&mut screen, scratch.path()), earlier);
}

const WORKTREE: &str = "acme-api/.claude/worktrees/feat";

fn start_at(screen: &mut Screen, scratch: &Path, folder: &str, number: usize) {
    fs::write(
        scratch.join("next"),
        scratch.join(folder).display().to_string(),
    )
    .unwrap();
    screen.press('n');
    let agent = format!("acme agent {number}");
    let shown = screen.wait_for(&agent);
    assert!(shown.contains(&agent), "{shown}");
    screen.key(
        crossterm::event::KeyCode::Char('h'),
        crossterm::event::KeyModifiers::CONTROL,
    );
}
