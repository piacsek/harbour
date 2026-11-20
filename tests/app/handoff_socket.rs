use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::Shutdown;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::symlink;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::thread::{self, sleep};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use harbour::editor::Program;
use harbour::handoff::{Listener, Reply, Verb};

use crate::handing::answering;
use crate::handoff::{app_with, file, listening, socket_in, soon, start_in};
use crate::{Screen, script};

#[test]
fn a_flood_of_silent_connections_leaves_keys_prompt() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = listening(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let socket = socket_in(&scratch.path().join("edit"));
    let _silent: Vec<UnixStream> = (0..100)
        .map(|_| UnixStream::connect(&socket).unwrap())
        .collect();
    let started = Instant::now();

    screen.type_text("acme typed");

    let shown = screen.wait_for("acme typed");
    assert!(shown.contains("acme typed"), "{shown}");
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn a_path_climbing_out_of_the_root_through_dot_dot_is_held_by_none() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = listening(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let climbing = root(scratch.path(), "acme-api").join("../acme-web/main.rs");

    let reply = asked_about(&mut screen, scratch.path(), "ask", &climbing);

    assert_eq!(reply, "none\n");
}

#[test]
fn a_path_through_dot_dot_that_stays_in_the_root_is_held() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = listening(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let staying = root(scratch.path(), "acme-api").join("src/../src/main.rs");

    let reply = asked_about(&mut screen, scratch.path(), "ask", &staying);

    assert!(reply.starts_with("holds "), "{reply}");
}

#[test]
fn a_symlink_in_the_root_to_a_file_outside_it_is_held_by_none() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = listening(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let outside = root(scratch.path(), "acme-web").join("main.rs");
    fs::write(&outside, "").unwrap();
    let link = root(scratch.path(), "acme-api").join("src/linked.rs");
    symlink(&outside, &link).unwrap();

    let reply = asked_about(&mut screen, scratch.path(), "ask", &link);

    assert_eq!(reply, "none\n");
}

#[test]
fn a_dangling_symlink_in_the_root_to_a_path_outside_it_is_held_by_none() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = listening(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let outside = root(scratch.path(), "acme-web").join("new.rs");
    let link = root(scratch.path(), "acme-api").join("src/linked.rs");
    symlink(&outside, &link).unwrap();

    let reply = asked_about(&mut screen, scratch.path(), "ask", &link);

    assert_eq!(reply, "none\n");
}

#[test]
fn a_folder_whose_name_starts_with_the_root_is_held_by_none() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = listening(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    fs::create_dir_all(scratch.path().join("acme-api-other")).unwrap();
    let alike = root(scratch.path(), "acme-api-other").join("main.rs");

    let reply = asked_about(&mut screen, scratch.path(), "ask", &alike);

    assert_eq!(reply, "none\n");
}

#[test]
fn a_request_that_expires_while_queued_opens_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = listening(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let socket = socket_in(&scratch.path().join("edit"));
    let mut stream = requested(
        &socket,
        "open",
        Duration::from_millis(200),
        &file(scratch.path(), "acme-api"),
    );
    sleep(Duration::from_millis(400));

    screen.draw();

    assert_eq!(answered(&mut stream), "");
    assert!(!scratch.path().join("args").exists());
}

#[test]
fn an_editor_that_exited_before_the_frame_is_replaced_by_the_open() {
    let scratch = tempfile::tempdir().unwrap();
    let gone = scratch.path().join("gone");
    let editor = Program::neovim(
        "nvim",
        script(&format!(
            "{{ pwd -P; printf '%s\\n' \"$0\" \"$@\"; }} > '{}/args'; \
             stty raw -echo; echo 'acme nvim ready'; dd bs=1 count=1 >/dev/null 2>&1; \
             touch '{}'",
            scratch.path().display(),
            gone.display()
        )),
    );
    let _asked = answering(scratch.path());
    let listener = Listener::new(&scratch.path().join("edit"));
    let mut screen = Screen::of(app_with(scratch.path(), editor).with_handoff(listener));
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let file = file(scratch.path(), "acme-api");
    assert_eq!(screen.app.hand(Verb::Open, &file, soon()), Reply::Opened);
    let shown = screen.wait_for("acme nvim ready");
    assert!(shown.contains("acme nvim ready"), "{shown}");
    screen
        .app
        .key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
    assert!(appeared(&gone));
    sleep(Duration::from_millis(200));

    let reply = asked_about(&mut screen, scratch.path(), "open", &file);

    assert_eq!(reply, "opened\n");
}

#[test]
fn one_frame_answers_at_most_four_requests() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = listening(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    let socket = socket_in(&scratch.path().join("edit"));
    let file = file(scratch.path(), "acme-api");
    let asking = {
        let (socket, file) = (socket.clone(), file.clone());
        let started = scratch.path().join("args");
        thread::spawn(move || {
            assert!(appeared(&started));
            (0..5)
                .map(|_| requested(&socket, "ask", Duration::from_secs(5), &file))
                .collect::<Vec<_>>()
        })
    };

    let mut opening = requested(&socket, "open", Duration::from_secs(5), &file);
    let opened = first_frame_answering(&mut screen, &mut opening);

    assert!(
        opened.starts_with("refused Neovim did not answer"),
        "{opened}"
    );
    let mut streams = asking.join().unwrap();
    let answered: Vec<Option<String>> = streams.iter_mut().map(pending).collect();
    assert_eq!(answered.iter().flatten().count(), 3);
    screen.draw();
    let mut waiting: Vec<UnixStream> = streams
        .into_iter()
        .zip(&answered)
        .filter(|(_, answered)| answered.is_none())
        .map(|(stream, _)| stream)
        .collect();
    assert!(pending(&mut waiting[0]).unwrap().starts_with("holds "));
}

fn root(scratch: &Path, folder: &str) -> PathBuf {
    scratch.canonicalize().unwrap().join(folder)
}

fn asked_about(screen: &mut Screen, scratch: &Path, verb: &str, file: &Path) -> String {
    let socket = socket_in(&scratch.join("edit"));
    let mut stream = requested(&socket, verb, Duration::from_secs(5), file);
    stream.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut reply = Vec::new();
    let mut buffer = [0; 256];
    while Instant::now() < deadline {
        screen.draw();
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => reply.extend_from_slice(&buffer[..read]),
            Err(error) if error.kind() == ErrorKind::WouldBlock => sleep(Duration::from_millis(10)),
            Err(error) => panic!("{error}"),
        }
    }
    String::from_utf8(reply).unwrap()
}

fn requested(socket: &Path, verb: &str, within: Duration, file: &Path) -> UnixStream {
    let mut stream = UnixStream::connect(socket).unwrap();
    let expires = (SystemTime::now() + within)
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let mut request = format!("{verb} {expires}\0").into_bytes();
    request.extend_from_slice(file.as_os_str().as_bytes());
    stream.write_all(&request).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    stream
}

fn answered(stream: &mut UnixStream) -> String {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut text = String::new();
    match stream.read_to_string(&mut text) {
        Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
            panic!("no answer and still open")
        }
        _ => text,
    }
}

fn appeared(path: &Path) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() && Instant::now() < deadline {
        sleep(Duration::from_millis(10));
    }
    path.exists()
}

fn pending(stream: &mut UnixStream) -> Option<String> {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(300)));
    let mut text = String::new();
    match stream.read_to_string(&mut text) {
        Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => None,
        result => {
            result.unwrap();
            Some(text)
        }
    }
}

fn first_frame_answering(screen: &mut Screen, stream: &mut UnixStream) -> String {
    stream.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut buffer = [0; 256];
    while Instant::now() < deadline {
        screen.draw();
        match stream.read(&mut buffer) {
            Ok(read) => return String::from_utf8_lossy(&buffer[..read]).into_owned(),
            Err(error) if error.kind() == ErrorKind::WouldBlock => sleep(Duration::from_millis(5)),
            Err(error) => panic!("{error}"),
        }
    }
    panic!("no frame answered");
}
