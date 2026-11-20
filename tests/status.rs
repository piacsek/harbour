use std::path::Path;
use std::process::{Command, Output};

#[test]
fn with_no_harbour_ever_run_status_prints_nothing() {
    let state = tempfile::tempdir().unwrap();

    let output = status(state.path());

    assert!(output.stdout.is_empty(), "{}", stdout(&output));
}

#[test]
fn with_no_harbour_ever_run_status_exits_zero() {
    let state = tempfile::tempdir().unwrap();

    assert!(status(state.path()).status.success());
}

#[test]
fn status_writes_nothing_on_stderr() {
    let state = tempfile::tempdir().unwrap();

    assert!(status(state.path()).stderr.is_empty());
}

fn status(state: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("status")
        .env("XDG_STATE_HOME", state)
        .env("HOME", state)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}
