use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::editor::EditorDir;
use harbour::handoff::{Claim, Reply, Verb};

use crate::directory::{cwd, folder, following_from, following_in, repository, write_transcript};
use crate::handing::answering;
use crate::handoff::{
    app_running, app_with, file, harbour, last_use, opened_beside, recording_neovim, soon, start_in,
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
fn with_the_exact_directory_an_ask_answers_with_how_deep_the_sessions_own_folder_lies() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = exactly(scratch.path());
    repository(scratch.path(), "acme-api");
    start_at(&mut screen, scratch.path(), "acme-api/src", 1);
    let src = scratch.path().canonicalize().unwrap().join("acme-api/src");

    let reply = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-api/src"), soon());

    let Reply::Holds(Claim { depth, .. }) = reply else {
        panic!("not held: {reply:?}");
    };
    assert_eq!(depth, src.components().count());
}

#[test]
fn with_the_exact_directory_a_file_beside_the_sessions_folder_is_held_by_none() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = exactly(scratch.path());
    repository(scratch.path(), "acme-api");
    start_at(&mut screen, scratch.path(), "acme-api/src", 1);

    let reply = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-api/lib"), soon());

    assert_eq!(reply, Reply::None);
}

#[test]
fn an_ask_judges_by_the_repository_claude_code_moved_to_since_the_last_read() {
    let scratch = tempfile::tempdir().unwrap();
    let api = repository(scratch.path(), "acme-api");
    let web = repository(scratch.path(), "acme-web");
    let mut screen = following_in(
        scratch.path(),
        &[&cwd(&api)],
        recording_neovim(scratch.path()),
        EditorDir::Root,
    );
    let shown = screen.wait_for("acme-api");
    assert!(shown.contains("acme-api"), "{shown}");
    write_transcript(scratch.path(), &[&cwd(&web)]);

    let in_api = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-api"), soon());
    let in_web = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-web"), soon());

    assert_eq!(in_api, Reply::None);
    assert!(matches!(in_web, Reply::Holds(_)), "{in_web:?}");
}

#[test]
fn with_the_exact_directory_an_ask_holds_the_start_folder_after_claude_code_moves() {
    let scratch = tempfile::tempdir().unwrap();
    repository(scratch.path(), "acme-api");
    let rust = folder(scratch.path(), "acme-api/rust");
    let tests = folder(scratch.path(), "acme-api/rust/tests");
    let mut screen = following_from(
        scratch.path(),
        &rust,
        &[&cwd(&tests)],
        recording_neovim(scratch.path()),
        EditorDir::Exact,
    );

    let reply = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-api/rust"), soon());

    assert!(matches!(reply, Reply::Holds(_)), "{reply:?}");
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

fn exactly(scratch: &Path) -> Screen {
    Screen::of(app_with(scratch, recording_neovim(scratch)).with_editor_dir(EditorDir::Exact))
}
