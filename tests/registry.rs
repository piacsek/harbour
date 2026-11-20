use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use harbour::registry::{Record, Status, read, sessions_dir};

const SAMPLE: &str = r#"{"pid":27756,"sessionId":"0f1e2d3c-4b5a-6978-8a9b-0c1d2e3f4a5b","cwd":"/home/acme/api","startedAt":1788798199062,"procStart":"Mon Sep  7 16:23:16 2026","version":"2.1.328","peerProtocol":1,"peerFeatures":["notify_idle","reply_across_default_dirs","artifact_yield"],"kind":"interactive","entrypoint":"cli","pidDomain":"darwin","messagingSocketPath":"/tmp/cc-socks/27756.sock","name":"api-d8","nameSource":"derived","nameSince":1788798199063,"status":"busy","updatedAt":1788804089018,"statusUpdatedAt":1788804089018}"#;

#[test]
fn a_session_file_is_read_by_pid_ignoring_unknown_fields() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("27756.json"), SAMPLE).unwrap();

    let record = read(dir.path(), 27756, UNIX_EPOCH);

    assert_eq!(
        record,
        Some(Record {
            status: Status::Busy,
            waiting_for: None,
        })
    );
}

#[test]
fn every_status_claude_code_writes_is_read() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("1.json"), r#"{"status":"busy"}"#).unwrap();
    fs::write(dir.path().join("2.json"), r#"{"status":"shell"}"#).unwrap();
    fs::write(dir.path().join("3.json"), r#"{"status":"idle"}"#).unwrap();
    fs::write(dir.path().join("4.json"), r#"{"status":"waiting"}"#).unwrap();

    assert_eq!(status(&dir, 1), Some(Status::Busy));
    assert_eq!(status(&dir, 2), Some(Status::Shell));
    assert_eq!(status(&dir, 3), Some(Status::Idle));
    assert_eq!(status(&dir, 4), Some(Status::Waiting));
}

#[test]
fn waiting_for_is_read_when_present() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("1.json"),
        r#"{"status":"waiting","waitingFor":"permission prompt"}"#,
    )
    .unwrap();

    let record = read(dir.path(), 1, UNIX_EPOCH).unwrap();

    assert_eq!(record.waiting_for.as_deref(), Some("permission prompt"));
}

#[test]
fn an_unknown_or_missing_status_reads_as_unknown() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("1.json"), r#"{"status":"meditating"}"#).unwrap();
    fs::write(dir.path().join("2.json"), r#"{"pid":2}"#).unwrap();

    assert_eq!(status(&dir, 1), Some(Status::Unknown));
    assert_eq!(status(&dir, 2), Some(Status::Unknown));
}

#[test]
fn a_waiting_for_that_is_not_text_keeps_the_status_and_drops_the_reason() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("1.json"),
        r#"{"status":"waiting","waitingFor":{"kind":"permission"}}"#,
    )
    .unwrap();

    let record = read(dir.path(), 1, UNIX_EPOCH);

    assert_eq!(
        record,
        Some(Record {
            status: Status::Waiting,
            waiting_for: None,
        })
    );
}

#[test]
fn a_status_that_is_not_text_reads_as_unknown() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("1.json"), r#"{"status":3}"#).unwrap();

    assert_eq!(status(&dir, 1), Some(Status::Unknown));
}

#[test]
fn a_missing_or_malformed_file_reads_as_nothing() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("1.json"), "{not json").unwrap();
    fs::write(dir.path().join("2.json.tmp"), SAMPLE).unwrap();

    assert_eq!(read(dir.path(), 1, UNIX_EPOCH), None);
    assert_eq!(read(dir.path(), 2, UNIX_EPOCH), None);
    assert_eq!(read(dir.path(), 3, UNIX_EPOCH), None);
}

#[test]
fn a_file_last_written_well_before_the_session_started_is_left_from_another_process() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("1.json"), r#"{"status":"busy"}"#).unwrap();
    let started = SystemTime::now();
    fs::File::options()
        .write(true)
        .open(dir.path().join("1.json"))
        .unwrap()
        .set_modified(started - Duration::from_secs(3600))
        .unwrap();

    assert_eq!(read(dir.path(), 1, started), None);
}

#[test]
fn a_file_written_around_the_start_of_the_session_is_read() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("1.json"), r#"{"status":"busy"}"#).unwrap();
    let started = SystemTime::now() + Duration::from_secs(1);

    assert_eq!(
        read(dir.path(), 1, started).map(|record| record.status),
        Some(Status::Busy)
    );
}

#[test]
fn the_sessions_dir_honours_the_config_dir_and_defaults_to_home() {
    let home = Some(PathBuf::from("/home/acme"));

    assert_eq!(
        sessions_dir(Some(PathBuf::from("/cfg")), home.clone()),
        Some(PathBuf::from("/cfg/sessions"))
    );
    assert_eq!(
        sessions_dir(None, home.clone()),
        Some(PathBuf::from("/home/acme/.claude/sessions"))
    );
    assert_eq!(
        sessions_dir(Some(PathBuf::new()), home),
        Some(PathBuf::from("/home/acme/.claude/sessions"))
    );
    assert_eq!(sessions_dir(None, None), None);
}

fn status(dir: &tempfile::TempDir, pid: u32) -> Option<Status> {
    read(dir.path(), pid, UNIX_EPOCH).map(|record| record.status)
}
