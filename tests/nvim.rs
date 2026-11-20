use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use harbour::nvim;

#[test]
fn a_socket_that_never_accepts_gives_no_answer_by_the_deadline() {
    let scratch = tempfile::tempdir().unwrap();
    let socket = scratch.path().join("socket");
    let listener = UnixListener::bind(&socket).unwrap();
    let filling = socket.clone();
    thread::spawn(move || {
        let mut queued = Vec::new();
        while let Ok(stream) = UnixStream::connect(&filling) {
            queued.push(stream);
        }
        thread::sleep(Duration::from_secs(600));
    });
    thread::sleep(Duration::from_millis(500));
    let (answered, answer) = mpsc::channel();

    thread::spawn(move || {
        let _ = answered.send(nvim::mode(
            &socket,
            Instant::now() + Duration::from_millis(100),
        ));
    });

    let answer = answer.recv_timeout(Duration::from_secs(2));
    assert!(matches!(answer, Ok(None)));
    drop(listener);
}
