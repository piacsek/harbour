use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::{Duration, Instant, SystemTime};

use harbour::editor::Program;
use harbour::handoff::{Reply, Verb};

use crate::directory::{cwd, following_with, repository, write_transcript};
use crate::handoff::{Edit, called, file, harbour, last_use, neovim_at, recorded, soon, start_in};
use crate::script;

#[test]
fn a_second_open_right_after_the_first_waits_for_the_new_neovim() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let file = file(scratch.path(), "acme-api");
    assert_eq!(screen.app.hand(Verb::Open, &file, soon()), Reply::Opened);
    let shown = screen.wait_for("acme nvim ready");
    assert!(shown.contains("acme nvim ready"), "{shown}");
    let socket = PathBuf::from(&recorded(scratch.path())[2]);
    let (bound, listening) = mpsc::channel();
    thread::spawn(move || {
        sleep(Duration::from_millis(300));
        let _ = bound.send(neovim_at(&socket, false, Edit::Done));
    });

    let reply = screen.app.hand(Verb::Open, &file, soon());

    assert_eq!(reply, Reply::Opened);
    let asked = listening.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(called(&asked).0, "nvim_get_mode");
}

#[test]
fn an_open_past_its_deadline_starts_no_editor() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);

    let reply = screen.app.hand(
        Verb::Open,
        &file(scratch.path(), "acme-api"),
        Instant::now(),
    );

    assert_eq!(reply, Reply::Refused("harbour took too long".to_string()));
    sleep(Duration::from_millis(200));
    assert!(!scratch.path().join("args").exists());
}

#[test]
fn open_goes_to_the_directory_the_transcript_names_now() {
    let scratch = tempfile::tempdir().unwrap();
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following_with(scratch.path(), &[], locating_neovim());
    write_transcript(scratch.path(), &[&cwd(&two)]);
    let file = two.canonicalize().unwrap().join("main.rs");

    let reply = screen.app.hand(Verb::Open, &file, soon());

    assert_eq!(reply, Reply::Opened);
    let shown = screen.wait_for("acme nvim at");
    assert!(shown.contains("acme nvim at acme-two"), "{shown}");
}

#[test]
fn ask_while_a_question_is_asked_is_answered_none() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.press('x');

    let reply = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::None);
}

#[test]
fn ask_while_the_picker_is_open_is_answered_none() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.press('n');

    let reply = screen
        .app
        .hand(Verb::Ask, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::None);
}

#[test]
fn a_key_that_selects_another_session_counts_as_a_use_of_that_session() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    start_in(&mut screen, scratch.path(), "acme-web", 2);
    let before = SystemTime::now();
    sleep(Duration::from_millis(5));

    screen.press('k');

    assert!(last_use(&mut screen, scratch.path()) >= before);
    assert!(used(&mut screen, &file(scratch.path(), "acme-web")) < before);
}

fn locating_neovim() -> Program {
    Program::neovim(
        "nvim",
        script("echo \"acme nvim at $(basename \"$(pwd -P)\")\"; exec sleep 600"),
    )
}

fn used(screen: &mut crate::Screen, file: &Path) -> SystemTime {
    let reply = screen.app.hand(Verb::Ask, file, soon());
    let Reply::Holds(used) = reply else {
        panic!("not held: {reply:?}");
    };
    used
}
