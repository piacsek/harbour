use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::Shutdown;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, sleep};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use harbour::handoff::{Listener, Reply, Verb};

#[test]
fn a_wait_for_a_socket_never_bound_fails_naming_its_folder() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    fs::create_dir(&dir).unwrap();

    let failure = failure(|| {
        bound_again(&dir);
    });

    assert!(failure.contains("a socket in"), "{failure}");
    assert!(failure.contains(&dir.display().to_string()), "{failure}");
}

#[test]
fn the_listener_makes_a_private_folder_and_socket() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);

    assert!(listener.incoming().is_none());

    assert_eq!(fs::metadata(&dir).unwrap().mode() & 0o777, 0o700);
    assert_eq!(socket_files(&dir).len(), 1);
    assert_eq!(
        fs::symlink_metadata(&socket_files(&dir)[0]).unwrap().mode() & 0o777,
        0o600
    );
}

#[test]
fn a_folder_left_open_to_others_is_made_private_again() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();

    Listener::new(&dir).incoming();

    assert_eq!(fs::metadata(&dir).unwrap().mode() & 0o777, 0o700);
}

#[test]
fn a_request_is_read_with_its_verb_and_file() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    listener.incoming();
    let file = existing(scratch.path(), "acme/main.rs");
    let mut stream = sent(&socket_files(&dir)[0], "open", later(), &bytes(&file));

    let request = arrived(&mut listener);

    assert_eq!(request.verb, Verb::Open);
    assert_eq!(request.file, file);
    request.answer(Reply::Opened);
    assert_eq!(reply(&mut stream), "opened\n");
}

#[test]
fn an_expired_request_is_dropped_unanswered() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    listener.incoming();
    let mut stream = sent(&socket_files(&dir)[0], "open", earlier(), b"/acme/main.rs");
    sleep(Duration::from_millis(50));

    assert!(listener.incoming().is_none());
    assert_eq!(reply(&mut stream), "");
}

#[test]
fn a_relative_path_is_dropped_unanswered() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    listener.incoming();
    let mut stream = sent(&socket_files(&dir)[0], "open", later(), b"acme/main.rs");
    sleep(Duration::from_millis(50));

    assert!(listener.incoming().is_none());
    assert_eq!(reply(&mut stream), "");
}

#[test]
fn an_unknown_verb_is_dropped_unanswered() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    listener.incoming();
    let mut stream = sent(&socket_files(&dir)[0], "run", later(), b"/acme/main.rs");
    sleep(Duration::from_millis(50));

    assert!(listener.incoming().is_none());
    assert_eq!(reply(&mut stream), "");
}

#[test]
fn a_refusal_spanning_lines_is_answered_on_one() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    listener.incoming();
    let file = existing(scratch.path(), "acme/main.rs");
    let mut stream = sent(&socket_files(&dir)[0], "open", later(), &bytes(&file));

    arrived(&mut listener).answer(Reply::Refused("acme\nrefusal".to_string()));

    assert_eq!(reply(&mut stream), "refused acme refusal\n");
}

#[test]
fn a_dead_harbours_socket_is_swept() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    let dead = dir.join("99999-0.sock");
    drop(UnixListener::bind(&dead).unwrap());

    Listener::new(&dir).incoming();

    assert!(fs::symlink_metadata(&dead).is_err());
}

#[test]
fn a_dead_socket_of_another_naming_is_left_alone() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    let other = dir.join("acme.sock");
    drop(UnixListener::bind(&other).unwrap());

    Listener::new(&dir).incoming();

    assert!(fs::symlink_metadata(&other).is_ok());
}

#[test]
fn a_regular_file_of_harbours_naming_is_left_alone() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    let planted = dir.join("99999-0.sock");
    fs::write(&planted, "acme").unwrap();

    Listener::new(&dir).incoming();

    assert_eq!(fs::read_to_string(&planted).unwrap(), "acme");
}

#[test]
fn a_live_harbours_socket_is_left_alone() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut first = Listener::new(&dir);
    first.incoming();

    let mut second = Listener::new(&dir);
    second.incoming();

    assert_eq!(socket_files(&dir).len(), 2);
    drop(second);
    drop(first);
}

#[test]
fn a_removed_socket_is_bound_again_at_the_next_check() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::retrying_every(&dir, Duration::ZERO);
    listener.incoming();
    fs::remove_file(&socket_files(&dir)[0]).unwrap();

    assert_eq!(bound_again(&dir).len(), 1);
}

#[test]
fn dropping_the_listener_removes_its_socket() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    listener.incoming();

    drop(listener);

    assert_eq!(socket_files(&dir).len(), 0);
}

#[test]
fn a_folder_that_is_a_symlink_is_never_listened_in() {
    let scratch = tempfile::tempdir().unwrap();
    let elsewhere = private_folder(scratch.path());
    let dir = scratch.path().join("edit");
    std::os::unix::fs::symlink(&elsewhere, &dir).unwrap();

    let mut listener = Listener::new(&dir);
    listener.incoming();

    assert_eq!(socket_files(&elsewhere).len(), 0);
}

#[test]
fn a_request_arrives_while_nobody_asks_for_it() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    let file = existing(scratch.path(), "acme/main.rs");
    let _stream = sent(&socket_files(&dir)[0], "ask", later(), &bytes(&file));

    let request = arrived(&mut listener);

    assert_eq!(request.file, file);
}

#[test]
fn a_path_through_dot_dot_arrives_resolved() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    let file = existing(scratch.path(), "acme/main.rs");
    let climbing = file.parent().unwrap().join("../acme/main.rs");
    let _stream = sent(&socket_files(&dir)[0], "ask", later(), &bytes(&climbing));

    let request = arrived(&mut listener);

    assert_eq!(request.file, file);
}

#[test]
fn a_symlink_arrives_as_the_file_it_points_to() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    let file = existing(scratch.path(), "acme/main.rs");
    let link = scratch.path().canonicalize().unwrap().join("acme-link.rs");
    std::os::unix::fs::symlink(&file, &link).unwrap();
    let _stream = sent(&socket_files(&dir)[0], "ask", later(), &bytes(&link));

    let request = arrived(&mut listener);

    assert_eq!(request.file, file);
}

#[test]
fn a_dangling_symlink_arrives_as_the_path_it_points_to() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    let away = existing(scratch.path(), "acme-away/kept.rs").with_file_name("new.rs");
    let link = existing(scratch.path(), "acme/kept.rs").with_file_name("new.rs");
    std::os::unix::fs::symlink(&away, &link).unwrap();
    let _stream = sent(&socket_files(&dir)[0], "ask", later(), &bytes(&link));

    let request = arrived(&mut listener);

    assert_eq!(request.file, away);
}

#[test]
fn a_missing_file_arrives_in_its_resolved_folder() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    let missing = existing(scratch.path(), "acme/kept.rs").with_file_name("new.rs");
    let _stream = sent(&socket_files(&dir)[0], "ask", later(), &bytes(&missing));

    let request = arrived(&mut listener);

    assert_eq!(request.file, missing);
}

#[test]
fn an_ask_for_a_path_in_a_missing_folder_is_answered_none_by_the_listener() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    let missing = scratch.path().join("acme-missing/main.rs");
    let mut stream = sent(&socket_files(&dir)[0], "ask", later(), &bytes(&missing));

    assert_eq!(reply(&mut stream), "none\n");
    assert!(listener.incoming().is_none());
}

#[test]
fn an_open_for_a_directory_is_refused_by_the_listener() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    let folder = scratch.path().canonicalize().unwrap();
    let mut stream = sent(&socket_files(&dir)[0], "open", later(), &bytes(&folder));

    assert_eq!(
        reply(&mut stream),
        format!("refused {} is a directory\n", folder.display())
    );
    assert!(listener.incoming().is_none());
}

#[test]
fn a_connection_over_the_limit_in_flight_is_closed_at_once() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let mut listener = Listener::new(&dir);
    let file = existing(scratch.path(), "acme/main.rs");
    let socket = socket_files(&dir)[0].clone();
    let _held: Vec<UnixStream> = (0..16)
        .map(|_| sent(&socket, "ask", later(), &bytes(&file)))
        .collect();
    let mut extra = sent(&socket, "ask", later(), &bytes(&file));
    let started = Instant::now();

    assert_eq!(unanswered(&mut extra), "");
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(arrived(&mut listener).verb, Verb::Ask);
}

#[test]
fn dropping_an_idle_listener_returns_at_once() {
    let scratch = tempfile::tempdir().unwrap();
    let mut listener = Listener::new(&scratch.path().join("edit"));
    listener.incoming();
    sleep(Duration::from_millis(50));
    let started = Instant::now();

    drop(listener);

    assert!(
        started.elapsed() < Duration::from_millis(500),
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn dropping_a_listener_during_a_flood_of_connections_returns_at_once() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("edit");
    let listener = Listener::new(&dir);
    let socket = socket_files(&dir)[0].clone();
    let flooding = Arc::new(AtomicBool::new(true));
    let flood = {
        let flooding = Arc::clone(&flooding);
        thread::spawn(move || {
            let mut connected = 0;
            while flooding.load(Ordering::Relaxed) {
                if UnixStream::connect(&socket).is_ok() {
                    connected += 1;
                }
            }
            connected
        })
    };
    sleep(Duration::from_millis(200));
    let started = Instant::now();

    drop(listener);

    let elapsed = started.elapsed();
    flooding.store(false, Ordering::Relaxed);
    assert!(flood.join().unwrap() > 0);
    assert!(elapsed < Duration::from_millis(500), "{elapsed:?}");
}

fn arrived(listener: &mut Listener) -> harbour::handoff::Request {
    for _ in 0..200 {
        if let Some(request) = listener.incoming() {
            return request;
        }
        sleep(Duration::from_millis(5));
    }
    panic!("no request arrived");
}

fn sent(socket: &Path, verb: &str, expires: u128, file: &[u8]) -> UnixStream {
    let mut stream = UnixStream::connect(socket).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut request = format!("{verb} {expires}\0").into_bytes();
    request.extend_from_slice(file);
    stream.write_all(&request).unwrap();
    stream.shutdown(Shutdown::Write).unwrap();
    stream
}

fn reply(stream: &mut UnixStream) -> String {
    let mut text = String::new();
    stream.read_to_string(&mut text).unwrap();
    text
}

fn unanswered(stream: &mut UnixStream) -> String {
    let mut text = String::new();
    match stream.read_to_string(&mut text) {
        Err(error) if error.kind() == ErrorKind::ConnectionReset => text,
        read => {
            read.unwrap();
            text
        }
    }
}

fn later() -> u128 {
    millis(SystemTime::now() + Duration::from_secs(5))
}

fn earlier() -> u128 {
    millis(SystemTime::now() - Duration::from_secs(5))
}

fn millis(at: SystemTime) -> u128 {
    at.duration_since(UNIX_EPOCH).unwrap().as_millis()
}

fn bytes(path: &Path) -> Vec<u8> {
    path.as_os_str().as_bytes().to_vec()
}

fn existing(scratch: &Path, name: &str) -> PathBuf {
    let file = scratch.join(name);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "").unwrap();
    file.canonicalize().unwrap()
}

fn bound_again(dir: &Path) -> Vec<PathBuf> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let found = socket_files(dir);
        if !found.is_empty() {
            return found;
        }
        assert!(
            Instant::now() < deadline,
            "waited 2 s for a socket in {}",
            dir.display()
        );
        sleep(Duration::from_millis(5));
    }
}

fn private_folder(scratch: &Path) -> PathBuf {
    let dir = scratch.join("elsewhere");
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    dir
}

fn socket_files(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "sock")
        })
        .collect();
    found.sort();
    found
}

fn failure(wait: impl FnOnce()) -> String {
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(wait))
        .expect_err("the wait returned without failing");
    unwound
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_default()
}
