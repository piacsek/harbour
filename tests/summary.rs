use std::fs::{self, File};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use harbour::state::Counts;
use harbour::summary::Summary;

#[test]
fn publishing_writes_the_counts_blocked_working_idle_unknown() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::new(&dir);

    summary.publish(counts(1, 2, 3, 4));

    assert_eq!(
        fs::read_to_string(only(&dir, "counts")).unwrap(),
        "1 2 3 4\n"
    );
}

#[test]
fn harbour_status_reads_what_a_live_summary_published() {
    let scratch = tempfile::tempdir().unwrap();
    let mut summary = Summary::new(&scratch.path().join("harbour/status"));

    summary.publish(counts(1, 1, 3, 0));

    assert_eq!(
        status(scratch.path()),
        "#[fg=red]◉ 1#[default] #[fg=yellow]● 1#[default] #[dim]○ 3#[default]\n"
    );
}

#[test]
fn a_new_count_replaces_the_last_one() {
    let scratch = tempfile::tempdir().unwrap();
    let mut summary = Summary::new(&scratch.path().join("harbour/status"));
    summary.publish(counts(1, 0, 0, 0));

    summary.publish(counts(0, 1, 0, 0));

    assert_eq!(status(scratch.path()), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn the_same_counts_are_not_written_again() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::new(&dir);
    summary.publish(counts(1, 0, 0, 0));
    fs::remove_file(only(&dir, "counts")).unwrap();

    summary.publish(counts(1, 0, 0, 0));

    assert_eq!(named(&dir, "counts"), Vec::<PathBuf>::new());
}

#[test]
fn the_folder_is_private_to_the_user() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::new(&dir);

    summary.publish(counts(1, 0, 0, 0));

    assert_eq!(mode(&dir), 0o700);
}

#[test]
fn an_existing_folder_is_made_private() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    fs::create_dir_all(&dir).unwrap();
    set_mode(&dir, 0o755);
    let mut summary = Summary::new(&dir);

    summary.publish(counts(1, 0, 0, 0));

    assert_eq!(mode(&dir), 0o700);
}

#[test]
fn the_counts_are_private_to_the_user() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::new(&dir);

    summary.publish(counts(1, 0, 0, 0));

    assert_eq!(mode(&only(&dir, "counts")), 0o600);
}

#[test]
fn the_lock_is_private_to_the_user() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::new(&dir);

    summary.publish(counts(1, 0, 0, 0));

    assert_eq!(mode(&only(&dir, "lock")), 0o600);
}

#[test]
fn dropping_the_summary_removes_its_files() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::new(&dir);
    summary.publish(counts(1, 0, 0, 0));

    drop(summary);

    assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
}

#[test]
fn once_dropped_harbour_status_prints_nothing_for_it() {
    let scratch = tempfile::tempdir().unwrap();
    let mut summary = Summary::new(&scratch.path().join("harbour/status"));
    summary.publish(counts(1, 0, 0, 0));

    drop(summary);

    assert_eq!(status(scratch.path()), "");
}

#[test]
fn two_summaries_in_one_folder_never_touch_each_others_files() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut first = Summary::new(&dir);
    let mut second = Summary::new(&dir);
    first.publish(counts(1, 0, 0, 0));
    second.publish(counts(0, 1, 0, 0));

    drop(first);

    assert_eq!(status(scratch.path()), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn a_stale_pair_with_this_pid_does_not_stop_publishing() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    fs::create_dir_all(&dir).unwrap();
    let pid = std::process::id();
    fs::write(dir.join(format!("{pid}.counts")), "5 5 5 5\n").unwrap();
    let reader = File::create(dir.join(format!("{pid}.lock"))).unwrap();
    reader.lock_shared().unwrap();
    let mut summary = Summary::new(&dir);

    summary.publish(counts(0, 1, 0, 0));

    assert_eq!(status(scratch.path()), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn a_folder_that_cannot_be_created_publishes_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("harbour"), "acme").unwrap();
    let mut summary = Summary::new(&scratch.path().join("harbour/status"));

    summary.publish(counts(1, 0, 0, 0));

    assert_eq!(status(scratch.path()), "");
}

#[test]
fn a_symlink_in_place_of_the_folder_publishes_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let elsewhere = scratch.path().join("acme");
    fs::create_dir(&elsewhere).unwrap();
    fs::create_dir(scratch.path().join("harbour")).unwrap();
    symlink(&elsewhere, scratch.path().join("harbour/status")).unwrap();
    let mut summary = Summary::new(&scratch.path().join("harbour/status"));

    summary.publish(counts(1, 0, 0, 0));

    assert_eq!(fs::read_dir(&elsewhere).unwrap().count(), 0);
}

#[test]
fn a_failed_write_stops_the_old_counts_being_counted() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::new(&dir);
    summary.publish(counts(0, 1, 0, 0));
    set_mode(&dir, 0o500);

    summary.publish(counts(1, 0, 0, 0));

    let line = status(scratch.path());
    set_mode(&dir, 0o700);
    assert_eq!(line, "");
}

#[test]
fn once_writing_works_again_the_counts_are_counted_again() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::new(&dir).retrying_every(Duration::ZERO);
    summary.publish(counts(0, 1, 0, 0));
    set_mode(&dir, 0o500);
    summary.publish(counts(1, 0, 0, 0));
    set_mode(&dir, 0o700);

    summary.publish(counts(1, 0, 0, 0));

    assert_eq!(status(scratch.path()), "#[fg=red]◉ 1#[default]\n");
}

#[test]
fn a_failed_folder_is_not_tried_again_before_the_interval() {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("harbour"), "acme").unwrap();
    let mut summary = Summary::new(&scratch.path().join("harbour/status"));
    summary.publish(counts(1, 0, 0, 0));
    fs::remove_file(scratch.path().join("harbour")).unwrap();

    summary.publish(counts(1, 0, 0, 0));
    summary.publish(counts(0, 1, 0, 0));

    assert_eq!(status(scratch.path()), "");
}

#[test]
fn a_failed_folder_is_tried_again_once_the_interval_passed() {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("harbour"), "acme").unwrap();
    let mut summary =
        Summary::new(&scratch.path().join("harbour/status")).retrying_every(Duration::ZERO);
    summary.publish(counts(1, 0, 0, 0));
    fs::remove_file(scratch.path().join("harbour")).unwrap();

    summary.publish(counts(1, 0, 0, 0));

    assert_eq!(status(scratch.path()), "#[fg=red]◉ 1#[default]\n");
}

#[test]
fn a_folder_removed_and_made_again_is_counted_again() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::new(&dir).retrying_every(Duration::ZERO);
    summary.publish(counts(1, 0, 0, 0));
    fs::remove_dir_all(&dir).unwrap();
    fs::create_dir_all(&dir).unwrap();

    summary.publish(counts(1, 0, 0, 0));
    summary.publish(counts(1, 0, 0, 0));

    assert_eq!(status(scratch.path()), "#[fg=red]◉ 1#[default]\n");
}

#[test]
fn a_folder_made_again_leaves_no_counts_without_a_lock() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("harbour/status");
    let mut summary = Summary::new(&dir);
    summary.publish(counts(1, 0, 0, 0));
    fs::remove_dir_all(&dir).unwrap();
    fs::create_dir_all(&dir).unwrap();

    summary.publish(counts(0, 1, 0, 0));

    assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
}

fn counts(blocked: usize, working: usize, idle: usize, unknown: usize) -> Counts {
    Counts {
        blocked,
        working,
        idle,
        unknown,
    }
}

fn named(dir: &Path, extension: &str) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == extension))
        .collect()
}

fn only(dir: &Path, extension: &str) -> PathBuf {
    let found = named(dir, extension);
    assert_eq!(found.len(), 1, "{found:?}");
    found[0].clone()
}

fn set_mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
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
