use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use super::{gates, hold_by_hand, locking_repository, pretender};
use crate::helpers::{calls, stderr};

#[test]
fn gates_waiting_for_the_lock_check_it_again_after_the_poll() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let real = recording_sleep(dir.path());
    let sleeps = dir.path().join("sleeps");
    let other = pretender();
    hold_by_hand(&work, other.id());

    let output = gates(
        dir.path(),
        &work,
        &[
            ("HARBOUR_GATES_WAIT", "1"),
            ("HARBOUR_GATES_POLL", "0.2"),
            ("REAL_SLEEP", &real),
            ("SLEEPS_LOG", &sleeps.display().to_string()),
        ],
    );

    assert!(!output.status.success());
    let slept = fs::read_to_string(&sleeps).unwrap();
    assert!(slept.starts_with("sleep 0.2\n"), "{slept}");
    assert!(!slept.contains("sleep 1\n"), "{slept}");
}

#[test]
fn gates_refuse_a_poll_that_is_not_a_number_of_seconds() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_POLL", "soon")]);

    assert!(!output.status.success());
    assert!(
        stderr(&output)
            .contains("gates: HARBOUR_GATES_POLL must be a positive number of seconds, not soon\n"),
        "{}",
        stderr(&output)
    );
    assert!(calls(dir.path()).is_empty());
}

#[test]
fn gates_refuse_a_poll_of_no_time() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_POLL", "0.0")]);

    assert!(!output.status.success());
    assert!(
        stderr(&output)
            .contains("gates: HARBOUR_GATES_POLL must be a positive number of seconds, not 0.0\n"),
        "{}",
        stderr(&output)
    );
    assert!(calls(dir.path()).is_empty());
}

fn recording_sleep(root: &Path) -> String {
    let link = root.join("bin/sleep");
    let real = fs::read_link(&link).unwrap();
    fs::remove_file(&link).unwrap();
    symlink(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/recording-sleep.sh"),
        &link,
    )
    .unwrap();
    real.display().to_string()
}
