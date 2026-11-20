use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::Server;

#[test]
#[ignore]
fn with_harbour_log_set_harbour_logs_its_start() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("log-on", 120, 20);

    server.run(home.path(), &[("HARBOUR_LOG", "1")], &[]);

    server.wait_for_screen("No session yet");
    let text = wait_for_log(home.path(), " start ");
    assert!(text.contains(" size=120x20"), "{text}");
}

#[test]
#[ignore]
fn with_harbour_log_empty_harbour_makes_no_log_folder() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("log-off", 120, 20);
    server.run(home.path(), &[("HARBOUR_LOG", "")], &[]);
    server.wait_for_screen("No session yet");

    server.send_keys(&["q"]);

    server.wait_for_screen("harbour exited 0");
    assert!(!home.path().join(".local/state/harbour/log").exists());
}

pub(crate) fn wait_for_log(home: &Path, text: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let last = newest_log(home);
        if last.contains(text) {
            return last;
        }
        assert!(
            Instant::now() < deadline,
            "waited 10 s for {text:?} in harbour's log, which held:\n{last}"
        );
        sleep(Duration::from_millis(50));
    }
}

pub(crate) fn newest_log(home: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("log")
        .env("HOME", home)
        .env_remove("XDG_STATE_HOME")
        .output()
        .unwrap();
    let path = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    if path.is_empty() {
        return String::new();
    }
    std::fs::read_to_string(path).unwrap_or_default()
}
