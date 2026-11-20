use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::follow::Mover;
use harbour::picker::Picker;

use crate::directory::{cwd, end_agent, folder, recording_agent, repository, write_transcript};
use crate::{Screen, home, reading_agent, script};

#[test]
fn with_no_session_harbour_works_where_it_was_opened() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, moves) = opened(scratch.path(), reading_agent(), None);

    let at = wait_until_at(&mut screen, &moves, &home());

    assert_eq!(at, Some(home()));
}

#[test]
fn a_session_inside_a_repository_moves_harbour_to_its_root() {
    let scratch = tempfile::tempdir().unwrap();
    let api = repository(scratch.path(), "acme-api");
    let (mut screen, moves) = opened(scratch.path(), reading_agent(), None);

    start(
        &mut screen,
        scratch.path(),
        &folder(&api, "src/deep"),
        "deep 1",
    );

    let root = api.canonicalize().unwrap();
    assert_eq!(wait_until_at(&mut screen, &moves, &root), Some(root));
}

#[test]
fn a_session_outside_any_repository_moves_harbour_to_its_own_directory() {
    let scratch = tempfile::tempdir().unwrap();
    let notes = folder(scratch.path(), "acme-plain/notes");
    let (mut screen, moves) = opened(scratch.path(), reading_agent(), None);

    start(&mut screen, scratch.path(), &notes, "notes 1");

    let notes = notes.canonicalize().unwrap();
    assert_eq!(wait_until_at(&mut screen, &moves, &notes), Some(notes));
}

#[test]
fn moving_the_selection_moves_harbour_to_the_selected_sessions_root() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let (mut screen, moves) = opened(scratch.path(), reading_agent(), None);
    start(&mut screen, scratch.path(), &folder(&one, "src"), "src 1");
    start(&mut screen, scratch.path(), &two, "acme-two 2");
    let two = two.canonicalize().unwrap();
    assert_eq!(wait_until_at(&mut screen, &moves, &two), Some(two.clone()));

    screen.press('k');

    let one = one.canonicalize().unwrap();
    assert_eq!(wait_until_at(&mut screen, &moves, &one), Some(one));
    screen.press('j');
    assert_eq!(wait_until_at(&mut screen, &moves, &two), Some(two));
}

#[test]
fn claude_code_moving_into_a_worktree_moves_harbour_to_the_worktree_root() {
    let scratch = tempfile::tempdir().unwrap();
    let worktree = worktree(scratch.path());
    let (mut screen, moves) = transcribed(scratch.path());

    write_transcript(scratch.path(), &[&cwd(&worktree.join("lib"))]);

    assert_eq!(
        wait_until_at(&mut screen, &moves, &worktree),
        Some(worktree)
    );
}

#[test]
fn an_exited_session_keeps_harbour_where_it_was_last_seen_working() {
    let scratch = tempfile::tempdir().unwrap();
    let worktree = worktree(scratch.path());
    let (mut screen, moves) = transcribed(scratch.path());
    write_transcript(scratch.path(), &[&cwd(&worktree.join("lib"))]);
    wait_until_at(&mut screen, &moves, &worktree);

    end_agent(scratch.path());
    let shown = screen.wait_for("exited");
    assert!(shown.contains("exited"), "{shown}");

    assert_eq!(settled(&mut screen, &moves), Some(worktree));
}

#[test]
fn a_directory_that_is_gone_moves_harbour_back_where_it_was_opened() {
    let scratch = tempfile::tempdir().unwrap();
    let gone = folder(scratch.path(), "acme-gone");
    let (mut screen, moves) = opened(scratch.path(), reading_agent(), None);
    start(&mut screen, scratch.path(), &gone, "acme-gone 1");
    let gone = gone.canonicalize().unwrap();
    assert_eq!(
        wait_until_at(&mut screen, &moves, &gone),
        Some(gone.clone())
    );

    fs::remove_dir_all(&gone).unwrap();

    assert_eq!(wait_until_at(&mut screen, &moves, &home()), Some(home()));
}

#[test]
fn a_directory_harbour_cannot_enter_moves_harbour_back_where_it_was_opened() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let locked = repository(scratch.path(), "acme-locked");
    let refused = locked.canonicalize().unwrap();
    let (mut screen, moves) = opened(scratch.path(), reading_agent(), Some(refused));
    start(&mut screen, scratch.path(), &one, "acme-one 1");
    let one = one.canonicalize().unwrap();
    assert_eq!(wait_until_at(&mut screen, &moves, &one), Some(one));

    start(&mut screen, scratch.path(), &locked, "acme-locked 2");

    assert_eq!(wait_until_at(&mut screen, &moves, &home()), Some(home()));
}

#[test]
fn removing_the_last_session_moves_harbour_back_where_it_was_opened() {
    let scratch = tempfile::tempdir().unwrap();
    let api = repository(scratch.path(), "acme-api");
    let quitting = script("echo 'acme agent ready'; exit 0");
    let (mut screen, moves) = opened(scratch.path(), quitting, None);
    start(&mut screen, scratch.path(), &api, "acme-api 1");
    let api = api.canonicalize().unwrap();
    assert_eq!(wait_until_at(&mut screen, &moves, &api), Some(api));
    let shown = screen.wait_for("exited");
    assert!(shown.contains("exited"), "{shown}");

    screen.press('x');

    assert_eq!(wait_until_at(&mut screen, &moves, &home()), Some(home()));
}

#[test]
fn an_open_picker_leaves_harbour_where_the_selected_session_works() {
    let scratch = tempfile::tempdir().unwrap();
    let api = repository(scratch.path(), "acme-api");
    let (mut screen, moves) = opened(scratch.path(), reading_agent(), None);
    start(&mut screen, scratch.path(), &api, "acme-api 1");
    let api = api.canonicalize().unwrap();
    assert_eq!(wait_until_at(&mut screen, &moves, &api), Some(api.clone()));

    screen.press('n');
    let shown = screen.wait_for("acme picker waiting");
    assert!(shown.contains("acme picker waiting"), "{shown}");

    assert_eq!(settled(&mut screen, &moves), Some(api));
}

#[test]
fn an_app_given_no_mover_never_moves_the_process() {
    let scratch = tempfile::tempdir().unwrap();
    let api = repository(scratch.path(), "acme-api");
    let before = std::env::current_dir().unwrap();
    let mut screen = Screen::of(App::new(reading_agent()).with_picker(next_picked(scratch.path())));

    start(&mut screen, scratch.path(), &api, "acme-api 1");
    settled(&mut screen, &Moves::default());

    assert_eq!(std::env::current_dir().unwrap(), before);
}

type Moves = Arc<Mutex<Vec<PathBuf>>>;

fn opened(scratch: &Path, agent: Vec<String>, refused: Option<PathBuf>) -> (Screen, Moves) {
    let (mover, moves) = recording(refused);
    let app = App::new(agent)
        .with_picker(next_picked(scratch))
        .with_mover(mover);
    (Screen::of(app), moves)
}

fn transcribed(scratch: &Path) -> (Screen, Moves) {
    let start = repository(scratch, "acme-start");
    let (mover, moves) = recording(None);
    let app = App::new(recording_agent(scratch))
        .with_picker(next_picked(scratch))
        .with_transcripts(scratch.join("projects"))
        .naming_sessions()
        .with_mover(mover);
    let mut screen = Screen::of(app);
    self::start(&mut screen, scratch, &start, "acme-start 1");
    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
    let start = start.canonicalize().unwrap();
    assert_eq!(wait_until_at(&mut screen, &moves, &start), Some(start));
    (screen, moves)
}

fn start(screen: &mut Screen, scratch: &Path, directory: &Path, row: &str) {
    fs::write(scratch.join("next"), directory.display().to_string()).unwrap();
    screen.press('n');
    let shown = screen.wait_for(row);
    assert!(shown.contains(row), "{shown}");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
}

fn next_picked(scratch: &Path) -> Picker {
    Picker::Command(script(&format!(
        "next='{}/next'; echo 'acme picker waiting' >&2; \
         while [ ! -s \"$next\" ]; do sleep 0.05; done; cat \"$next\"; rm \"$next\"",
        scratch.display()
    )))
}

fn worktree(scratch: &Path) -> PathBuf {
    let worktree = folder(scratch, "acme-wt");
    folder(&worktree, "lib");
    let git_dir = folder(scratch, "acme/.git/worktrees/acme-wt");
    fs::write(
        worktree.join(".git"),
        format!("gitdir: {}\n", git_dir.display()),
    )
    .unwrap();
    worktree.canonicalize().unwrap()
}

fn recording(refused: Option<PathBuf>) -> (Mover, Moves) {
    let moves = Moves::default();
    let record = Arc::clone(&moves);
    let mover: Mover = Box::new(move |path: &Path| {
        if refused.as_deref() == Some(path) {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
        record.lock().unwrap().push(path.to_path_buf());
        Ok(())
    });
    (mover, moves)
}

fn wait_until_at(screen: &mut Screen, moves: &Moves, expected: &Path) -> Option<PathBuf> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        screen.draw();
        let at = moves.lock().unwrap().last().cloned();
        if at.as_deref() == Some(expected) || Instant::now() > deadline {
            return at;
        }
        sleep(Duration::from_millis(20));
    }
}

fn settled(screen: &mut Screen, moves: &Moves) -> Option<PathBuf> {
    let deadline = Instant::now() + Duration::from_millis(600);
    while Instant::now() < deadline {
        screen.draw();
        sleep(Duration::from_millis(20));
    }
    moves.lock().unwrap().last().cloned()
}
