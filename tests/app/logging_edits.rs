use std::path::Path;
use std::sync::mpsc::Receiver;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::handoff::{Listener, Reply, Verb};

use crate::Screen;
use crate::handing::{Edit, answering, neovim_at};
use crate::handoff::{app_with, file, recorded, recording_neovim, socket_in, soon, start_in};
use crate::handoff_socket::{asked_about, requested};
use crate::logging::{count, logged, only, read};

#[test]
fn an_ask_served_logs_its_file_reply_and_time() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, lines) = listening(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let file = file(scratch.path(), "acme-api");

    asked_about(&mut screen, scratch.path(), "ask", &file);

    let served = only(&read(&lines), " edit verb=ask ");
    assert!(served.contains(&format!(" file={file:?} ")), "{served}");
    assert!(served.contains(" reply=holds "), "{served}");
    assert!(served.contains(" took="), "{served}");
}

#[test]
fn a_refused_open_logs_the_message_it_answered() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, lines) = listening(scratch.path());
    let file = file(scratch.path(), "acme-api");

    asked_about(&mut screen, scratch.path(), "open", &file);

    let served = only(&read(&lines), " edit verb=open ");
    let message = format!("no harbour session holds {}", file.display());
    assert!(
        served.contains(&format!(" reply=refused message={message:?} ")),
        "{served}"
    );
}

#[test]
fn a_request_that_expired_while_queued_logs_that_it_expired() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, lines) = listening(scratch.path());
    let file = file(scratch.path(), "acme-api");
    let socket = socket_in(&scratch.path().join("edit"));
    let _stream = requested(&socket, "open", Duration::from_millis(200), &file);
    sleep(Duration::from_millis(400));

    screen.draw();

    let served = only(&read(&lines), " edit verb=open ");
    assert!(served.ends_with(" expired\n"), "{served}");
}

#[test]
fn an_open_into_neovim_logs_each_call_with_its_time() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, lines) = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let _asked = answering(scratch.path());

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Opened);
    let lines = read(&lines);
    let mode = only(&lines, " neovim call=mode ");
    assert!(mode.contains(" editor=2 "), "{mode}");
    assert!(mode.contains(" took="), "{mode}");
    assert!(mode.contains(" answered=yes"), "{mode}");
    let edit = only(&lines, " neovim call=edit ");
    assert!(edit.contains(" answered=yes"), "{edit}");
}

#[test]
fn a_neovim_that_never_answers_logs_one_unanswered_call_with_its_tries() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, lines) = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);

    screen.app.hand(
        Verb::Open,
        &file(scratch.path(), "acme-api"),
        Instant::now() + Duration::from_millis(600),
    );

    let mode = only(&read(&lines), " neovim call=mode ");
    assert!(mode.contains(" answered=no"), "{mode}");
    assert!(mode.contains(" tries="), "{mode}");
}

#[test]
fn x_over_neovim_logs_the_question_it_asked() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, lines) = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme nvim ready");
    let _asked = neovim_at(Path::new(&recorded(scratch.path())[2]), false, Edit::Done);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    read(&lines);

    screen.press('x');

    let eval = only(&read(&lines), " neovim call=eval ");
    assert!(eval.contains(" editor=2 "), "{eval}");
}

#[test]
fn a_ctrl_h_kept_by_neovim_is_logged_as_input_to_the_editor() {
    let scratch = tempfile::tempdir().unwrap();
    let (mut screen, lines) = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme nvim ready");
    let _asked = neovim_at(Path::new(&recorded(scratch.path())[2]), true, Edit::Done);
    read(&lines);

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let lines = read(&lines);
    let input = only(&lines, " input ");
    assert!(
        input.ends_with(" input to=editor session=1 editor=2 bytes=1\n"),
        "{input}"
    );
    assert_eq!(count(&lines, " key "), 0);
}

fn harbour(scratch: &Path) -> (Screen, Receiver<String>) {
    let (app, lines) = logged(app_with(scratch, recording_neovim(scratch)));
    (Screen::of(app), lines)
}

fn listening(scratch: &Path) -> (Screen, Receiver<String>) {
    let app = app_with(scratch, recording_neovim(scratch))
        .with_handoff(Listener::new(&scratch.join("edit")));
    let (app, lines) = logged(app);
    (Screen::of(app), lines)
}
