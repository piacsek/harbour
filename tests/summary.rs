use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use harbour::state::Counts;
use harbour::summary::Summary;

#[test]
fn publishing_writes_the_counts_blocked_working_idle_unknown() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::create(&dir, 101).unwrap();

    summary.publish(counts(1, 2, 3, 4));

    assert_eq!(read(&dir, "101.counts"), "1 2 3 4\n");
}

#[test]
fn harbour_status_reads_what_a_live_summary_published() {
    let scratch = tempfile::tempdir().unwrap();
    let mut summary = Summary::create(&scratch.path().join("harbour/status"), 101).unwrap();

    summary.publish(counts(1, 1, 3, 0));

    assert_eq!(
        status(scratch.path()),
        "#[fg=red]◉ 1#[default] #[fg=yellow]● 1#[default] #[dim]○ 3#[default]\n"
    );
}

#[test]
fn a_new_count_replaces_the_last_one() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::create(&dir, 101).unwrap();
    summary.publish(counts(1, 0, 0, 0));

    summary.publish(counts(0, 1, 0, 0));

    assert_eq!(read(&dir, "101.counts"), "0 1 0 0\n");
}

#[test]
fn the_same_counts_are_not_written_again() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::create(&dir, 101).unwrap();
    summary.publish(counts(1, 0, 0, 0));
    fs::remove_file(dir.join("101.counts")).unwrap();

    summary.publish(counts(1, 0, 0, 0));

    assert!(!dir.join("101.counts").exists());
}

#[test]
fn the_folder_is_private_to_the_user() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let _summary = Summary::create(&dir, 101).unwrap();

    assert_eq!(mode(&dir), 0o700);
}

#[test]
fn the_counts_are_private_to_the_user() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::create(&dir, 101).unwrap();

    summary.publish(counts(1, 0, 0, 0));

    assert_eq!(mode(&dir.join("101.counts")), 0o600);
}

#[test]
fn the_lock_is_private_to_the_user() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let _summary = Summary::create(&dir, 101).unwrap();

    assert_eq!(mode(&dir.join("101.lock")), 0o600);
}

#[test]
fn dropping_the_summary_removes_its_counts() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::create(&dir, 101).unwrap();
    summary.publish(counts(1, 0, 0, 0));

    drop(summary);

    assert!(!dir.join("101.counts").exists());
}

#[test]
fn dropping_the_summary_removes_its_lock() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let summary = Summary::create(&dir, 101).unwrap();

    drop(summary);

    assert!(!dir.join("101.lock").exists());
}

#[test]
fn once_dropped_harbour_status_prints_nothing_for_it() {
    let scratch = tempfile::tempdir().unwrap();
    let mut summary = Summary::create(&scratch.path().join("harbour/status"), 101).unwrap();
    summary.publish(counts(1, 0, 0, 0));

    drop(summary);

    assert_eq!(status(scratch.path()), "");
}

#[test]
fn counts_left_by_an_earlier_process_with_the_same_pid_are_cleared() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("101.counts"), "5 5 5 5\n").unwrap();
    fs::write(dir.join("101.lock"), "").unwrap();

    let _summary = Summary::create(&dir, 101).unwrap();

    assert_eq!(status(scratch.path()), "");
}

#[test]
fn a_folder_that_cannot_be_created_is_an_error() {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("harbour"), "acme").unwrap();

    let summary = Summary::create(&scratch.path().join("harbour/status"), 101);

    assert!(summary.is_err());
}

fn counts(blocked: usize, working: usize, idle: usize, unknown: usize) -> Counts {
    Counts {
        blocked,
        working,
        idle,
        unknown,
    }
}

fn read(dir: &Path, name: &str) -> String {
    fs::read_to_string(dir.join(name)).unwrap()
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

fn status(state: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("status")
        .env("XDG_STATE_HOME", state)
        .env("HOME", state)
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).into_owned()
}
