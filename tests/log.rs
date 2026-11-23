use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use harbour::log::{self, Log};

#[path = "support/timing.rs"]
mod timing;

use timing::deadline;

#[test]
fn a_log_writes_its_lines_to_the_file_named_by_its_pid() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("log");

    Log::open(&dir).write(|| "start acme=1".to_string());

    let text = wait_for_line(&own_file(&dir), "start acme=1");
    assert!(text.contains(" start acme=1\n"), "{text}");
}

#[test]
fn a_line_starts_with_the_time_in_utc_with_milliseconds() {
    let then = UNIX_EPOCH + Duration::from_millis(1_795_738_642_007);

    assert_eq!(log::stamp(then), "2026-11-27T00:17:22.007Z");
}

#[test]
fn the_time_of_a_leap_day_reads_as_its_date() {
    let then = UNIX_EPOCH + Duration::from_millis(1_709_164_800_000);

    assert_eq!(log::stamp(then), "2024-02-29T00:00:00.000Z");
}

#[test]
fn a_written_line_starts_with_the_time_it_was_written() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("log");
    let year = log::stamp(SystemTime::now())[..4].to_string();

    Log::open(&dir).write(|| "start acme=1".to_string());

    let text = wait_for_line(&own_file(&dir), "start acme=1");
    assert!(text.starts_with(&year), "{text}");
}

#[test]
fn the_folder_a_log_makes_is_private() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("state/harbour/log");

    Log::open(&dir).write(|| "start".to_string());

    wait_for_line(&own_file(&dir), "start");
    assert_eq!(mode(&dir), 0o700);
}

#[test]
fn a_looser_folder_is_made_private_again() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("log");
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();

    Log::open(&dir).write(|| "start".to_string());

    wait_for_line(&own_file(&dir), "start");
    assert_eq!(mode(&dir), 0o700);
}

#[test]
fn the_file_of_a_log_is_private() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = scratch.path().join("log");

    Log::open(&dir).write(|| "start".to_string());

    wait_for_line(&own_file(&dir), "start");
    assert_eq!(mode(&own_file(&dir)), 0o600);
}

#[test]
fn a_looser_file_of_a_reused_pid_is_made_private_again() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    fs::write(own_file(&dir), "").unwrap();
    fs::set_permissions(own_file(&dir), fs::Permissions::from_mode(0o644)).unwrap();

    Log::open(&dir).write(|| "start".to_string());

    wait_for_line(&own_file(&dir), "start");
    assert_eq!(mode(&own_file(&dir)), 0o600);
}

#[test]
fn a_reused_pid_replaces_the_old_file() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    fs::write(own_file(&dir), "acme old run\n".repeat(20)).unwrap();

    Log::open(&dir).write(|| "start".to_string());

    let text = wait_for_line(&own_file(&dir), "start");
    assert!(!text.contains("acme old run"), "{text}");
}

#[test]
fn a_log_older_than_seven_days_is_removed_at_the_start() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    write_aged(&dir.join("999999999.log"), days(7) + 60);

    Log::open(&dir).write(|| "start".to_string());

    wait_for_line(&own_file(&dir), "start");
    assert!(!dir.join("999999999.log").exists());
}

#[test]
fn an_old_log_named_for_pid_zero_is_removed() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    write_aged(&dir.join("0.log"), days(7) + 60);

    Log::open(&dir).write(|| "start".to_string());

    wait_for_line(&own_file(&dir), "start");
    assert!(!dir.join("0.log").exists());
}

#[test]
fn a_log_younger_than_seven_days_is_kept() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    write_aged(&dir.join("4100.log"), days(7) - 60);

    Log::open(&dir).write(|| "start".to_string());

    wait_for_line(&own_file(&dir), "start");
    assert!(dir.join("4100.log").exists());
}

#[test]
fn an_old_file_not_named_like_a_log_is_kept() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    write_aged(&dir.join("notes.log"), days(30));

    Log::open(&dir).write(|| "start".to_string());

    wait_for_line(&own_file(&dir), "start");
    assert!(dir.join("notes.log").exists());
}

#[test]
fn an_old_folder_named_like_a_log_is_kept() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    fs::create_dir(dir.join("4100.log")).unwrap();
    let file = fs::File::open(dir.join("4100.log")).unwrap();
    file.set_modified(SystemTime::now() - Duration::from_secs(days(30)))
        .unwrap();

    Log::open(&dir).write(|| "start".to_string());

    wait_for_line(&own_file(&dir), "start");
    assert!(dir.join("4100.log").is_dir());
}

#[test]
fn a_finished_log_holds_every_line_written_before_it_finished() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    let log = Log::open(&dir);
    for line in 0..2000 {
        log.write(|| format!("input to=picker bytes={line}"));
    }

    log.finish(5 * deadline());

    let text = fs::read_to_string(own_file(&dir)).unwrap();
    assert!(text.ends_with(" input to=picker bytes=1999\n"), "{text}");
}

#[test]
fn finishing_a_log_whose_disk_stalls_waits_no_longer_than_it_was_given() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    let made = std::process::Command::new("mkfifo")
        .arg(own_file(&dir))
        .status()
        .unwrap();
    assert!(made.success());
    let log = Log::open(&dir);
    log.write(|| "quit sessions=0".to_string());

    let started = Instant::now();
    log.finish(deadline());

    let elapsed = started.elapsed();
    assert!(elapsed >= deadline(), "{elapsed:?}");
    assert!(elapsed < 2 * deadline(), "{elapsed:?}");
}

#[test]
fn an_old_log_of_a_harbour_still_running_is_kept() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    let mut running = std::process::Command::new("sleep")
        .arg("600")
        .spawn()
        .unwrap();
    let kept = dir.join(format!("{}.log", running.id()));
    write_aged(&kept, days(8));

    Log::open(&dir).write(|| "start".to_string());

    wait_for_line(&own_file(&dir), "start");
    let exists = kept.exists();
    running.kill().unwrap();
    running.wait().unwrap();
    assert!(exists);
}

#[test]
fn a_symlink_at_the_log_file_is_never_followed() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    let target = scratch.path().join("acme-target");
    fs::write(&target, "acme kept\n").unwrap();
    std::os::unix::fs::symlink(&target, own_file(&dir)).unwrap();
    let log = Log::open(&dir);
    log.write(|| "start".to_string());

    log.finish(5 * deadline());

    assert_eq!(fs::read_to_string(&target).unwrap(), "acme kept\n");
}

#[test]
fn a_log_folder_that_is_a_symlink_is_never_used() {
    let scratch = tempfile::tempdir().unwrap();
    let target = scratch.path().join("acme-shared");
    fs::create_dir(&target).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o755)).unwrap();
    let dir = scratch.path().join("log");
    std::os::unix::fs::symlink(&target, &dir).unwrap();
    let log = Log::open(&dir);
    log.write(|| "start".to_string());

    log.finish(5 * deadline());

    assert_eq!(mode(&target), 0o755);
    assert!(!own_file(&target).exists());
}

#[test]
fn a_log_whose_folder_cannot_be_made_drops_its_lines() {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("state"), "").unwrap();
    let dir = scratch.path().join("state/harbour/log");
    let log = Log::open(&dir);

    let started = Instant::now();
    for _ in 0..10_000 {
        log.write(|| "input to=1 bytes=1".to_string());
    }

    let elapsed = started.elapsed();
    assert!(elapsed < deadline(), "{elapsed:?}");
}

#[test]
fn a_log_on_a_path_that_stalls_never_delays_a_write() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = private_folder(scratch.path());
    let made = std::process::Command::new("mkfifo")
        .arg(own_file(&dir))
        .status()
        .unwrap();
    assert!(made.success());
    let log = Log::open(&dir);

    let started = Instant::now();
    for _ in 0..10_000 {
        log.write(|| "input to=1 bytes=1".to_string());
    }

    let elapsed = started.elapsed();
    assert!(elapsed < deadline(), "{elapsed:?}");
}

#[test]
fn a_log_that_is_off_never_builds_its_line() {
    Log::default().write(|| panic!("an off log built its line"));
}

#[test]
fn a_sending_log_hands_each_line_to_its_channel() {
    let (sender, lines) = mpsc::sync_channel(4);

    Log::sending(sender).write(|| "quit".to_string());

    assert!(lines.try_recv().unwrap().ends_with(" quit\n"));
}

#[test]
fn a_full_channel_drops_the_line_instead_of_waiting() {
    let (sender, lines) = mpsc::sync_channel(1);
    let log = Log::sending(sender);

    log.write(|| "first".to_string());
    log.write(|| "second".to_string());

    assert!(lines.try_recv().unwrap().ends_with(" first\n"));
    assert!(lines.try_recv().is_err());
}

#[test]
fn a_wait_for_a_line_never_written_fails_naming_it_and_the_file() {
    let scratch = tempfile::tempdir().unwrap();
    let path = scratch.path().join("4100.log");
    fs::write(&path, "acme written\n").unwrap();

    let failure = failure(|| {
        wait_for_line(&path, "acme never written");
    });

    assert!(failure.contains("\"acme never written\""), "{failure}");
    assert!(failure.contains("acme written"), "{failure}");
}

fn own_file(dir: &Path) -> PathBuf {
    dir.join(format!("{}.log", std::process::id()))
}

fn private_folder(scratch: &Path) -> PathBuf {
    let dir = scratch.join("log");
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    dir
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

fn days(count: u64) -> u64 {
    count * 24 * 60 * 60
}

fn write_aged(path: &Path, seconds: u64) {
    let file = fs::File::create(path).unwrap();
    file.set_modified(SystemTime::now() - Duration::from_secs(seconds))
        .unwrap();
}

fn wait_for_line(path: &Path, text: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut last = String::new();
    while Instant::now() < deadline {
        last = fs::read_to_string(path).unwrap_or_default();
        if last.contains(text) {
            return last;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!(
        "waited 10 s for {text:?} in {}, which held:\n{last}",
        path.display()
    );
}

fn failure(wait: impl FnOnce()) -> String {
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(wait))
        .expect_err("the wait returned without failing");
    unwound
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_default()
}
