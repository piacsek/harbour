use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use harbour::transcript::{Tasks, Transcript};

use crate::{ID, append, transcript_file};

#[test]
fn the_tasks_done_and_their_total_are_counted() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"completed"}"#,
    );
    task(
        config.path(),
        "2.json",
        r#"{"id":"2","status":"in_progress"}"#,
    );
    task(config.path(), "3.json", r#"{"id":"3","status":"pending"}"#);
    task(config.path(), ".lock", "");
    task(config.path(), ".highwatermark", "3");
    let mut transcript = Transcript::new(&projects, ID);

    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 1, total: 3 }));
}

#[test]
fn no_tasks_folder_is_no_task_fact() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    let mut transcript = Transcript::new(&projects, ID);

    transcript.read();

    assert_eq!(transcript.facts().tasks, None);
}

#[test]
fn a_tasks_folder_without_a_task_file_is_no_task_fact() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(config.path(), ".lock", "");
    let mut transcript = Transcript::new(&projects, ID);

    transcript.read();

    assert_eq!(transcript.facts().tasks, None);
}

#[test]
fn a_task_without_a_status_is_open() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"completed"}"#,
    );
    task(config.path(), "2.json", r#"{"id":"2"}"#);
    let mut transcript = Transcript::new(&projects, ID);

    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 1, total: 2 }));
}

#[test]
fn a_first_read_with_a_file_that_is_not_an_object_has_no_task_fact() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"completed"}"#,
    );
    task(config.path(), "2.json", "[1, 2]");
    let mut transcript = Transcript::new(&projects, ID);

    transcript.read();

    assert_eq!(transcript.facts().tasks, None);
}

#[test]
fn a_task_that_completes_or_vanishes_is_followed_at_the_next_read() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"in_progress"}"#,
    );
    let gone = task(config.path(), "2.json", r#"{"id":"2","status":"pending"}"#);
    let mut transcript = Transcript::new(&projects, ID);
    transcript.read();
    let before = transcript.facts().tasks;

    task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"completed"}"#,
    );
    fs::remove_file(gone).unwrap();
    transcript.read();

    assert_eq!(before, Some(Tasks { done: 0, total: 2 }));
    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 1, total: 1 }));
}

#[test]
fn a_tasks_folder_removed_while_the_session_runs_drops_the_task_fact() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"completed"}"#,
    );
    let mut transcript = Transcript::new(&projects, ID);
    transcript.read();

    fs::remove_dir_all(config.path().join("tasks")).unwrap();
    transcript.read();

    assert_eq!(transcript.facts().tasks, None);
}

#[test]
fn a_task_list_whose_files_are_all_cleared_shows_every_task_done() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(config.path(), ".lock", "");
    let first = task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"completed"}"#,
    );
    let second = task(
        config.path(),
        "2.json",
        r#"{"id":"2","status":"in_progress"}"#,
    );
    let mut transcript = Transcript::new(&projects, ID);
    transcript.read();

    fs::remove_file(first).unwrap();
    fs::remove_file(second).unwrap();
    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 2, total: 2 }));
}

#[test]
fn a_new_task_after_a_cleared_list_shows_the_live_count() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    let first = task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"completed"}"#,
    );
    let mut transcript = Transcript::new(&projects, ID);
    transcript.read();
    fs::remove_file(first).unwrap();
    transcript.read();

    task(config.path(), "2.json", r#"{"id":"2","status":"pending"}"#);
    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 0, total: 1 }));
}

#[test]
fn the_tasks_are_still_read_while_the_transcript_is_lost() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(config.path(), "1.json", r#"{"id":"1","status":"pending"}"#);
    let mut transcript = Transcript::new(&projects, ID);
    transcript.read();
    fs::remove_file(&file).unwrap();
    transcript.read();

    task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"completed"}"#,
    );
    task(config.path(), "2.json", r#"{"id":"2","status":"pending"}"#);
    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 1, total: 2 }));
}

#[test]
fn a_task_file_over_64_kib_keeps_the_previous_count() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(config.path(), "1.json", r#"{"id":"1","status":"pending"}"#);
    let mut transcript = Transcript::new(&projects, ID);
    transcript.read();

    let padded = format!(r#"{{"status":"completed"{}}}"#, " ".repeat(64 * 1024));
    task(config.path(), "2.json", &padded);
    task(config.path(), "3.json", r#"{"id":"3","status":"pending"}"#);
    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 0, total: 1 }));
}

#[test]
fn a_list_emptied_after_two_tasks_completed_between_reads_shows_every_task_done() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    let first = task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"completed"}"#,
    );
    let second = task(config.path(), "2.json", r#"{"id":"2","status":"pending"}"#);
    let third = task(config.path(), "3.json", r#"{"id":"3","status":"pending"}"#);
    let mut transcript = Transcript::new(&projects, ID);
    transcript.read();

    fs::remove_file(first).unwrap();
    fs::remove_file(second).unwrap();
    fs::remove_file(third).unwrap();
    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 3, total: 3 }));
}

#[test]
fn open_tasks_deleted_before_the_list_is_cleared_leave_its_total() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    let files: Vec<PathBuf> = (1..=7)
        .map(|number| {
            task(
                config.path(),
                &format!("{number}.json"),
                r#"{"status":"pending"}"#,
            )
        })
        .collect();
    let mut transcript = Transcript::new(&projects, ID);
    transcript.read();
    fs::remove_file(&files[5]).unwrap();
    fs::remove_file(&files[6]).unwrap();
    transcript.read();
    for done in &files[..5] {
        fs::write(done, r#"{"status":"completed"}"#).unwrap();
    }
    transcript.read();

    for gone in &files[..5] {
        fs::remove_file(gone).unwrap();
    }
    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 5, total: 5 }));
}

#[test]
fn a_list_read_halfway_through_its_clearing_shows_every_task_done() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    let files: Vec<PathBuf> = (1..=7)
        .map(|number| {
            task(
                config.path(),
                &format!("{number}.json"),
                r#"{"status":"completed"}"#,
            )
        })
        .collect();
    let mut transcript = Transcript::new(&projects, ID);
    transcript.read();
    for gone in &files[..4] {
        fs::remove_file(gone).unwrap();
    }
    transcript.read();
    let halfway = transcript.facts().tasks;

    for gone in &files[4..] {
        fs::remove_file(gone).unwrap();
    }
    transcript.read();

    assert_eq!(halfway, Some(Tasks { done: 7, total: 7 }));
    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 7, total: 7 }));
}

#[test]
fn a_task_file_caught_mid_rewrite_keeps_the_previous_count() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"completed"}"#,
    );
    task(config.path(), "2.json", r#"{"id":"2","status":"pending"}"#);
    let mut transcript = Transcript::new(&projects, ID);
    transcript.read();

    task(config.path(), "2.json", r#"{"id":"2","sta"#);
    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 1, total: 2 }));
}

#[test]
fn a_file_not_named_by_a_number_is_not_a_task() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(config.path(), "1.json", r#"{"id":"1","status":"pending"}"#);
    task(config.path(), "notes.json", r#"{"status":"completed"}"#);
    let mut transcript = Transcript::new(&projects, ID);

    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 0, total: 1 }));
}

#[test]
fn a_directory_named_like_a_task_is_not_a_task() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(config.path(), "2.json", r#"{"id":"2","status":"pending"}"#);
    fs::create_dir(config.path().join("tasks").join(ID).join("1.json")).unwrap();
    let mut transcript = Transcript::new(&projects, ID);

    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 0, total: 1 }));
}

#[test]
fn a_task_file_over_64_kib_for_two_seconds_counts_as_open() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(config.path(), "1.json", r#"{"id":"1","status":"pending"}"#);
    let mut transcript = Transcript::new(&projects, ID);
    transcript.read();

    let padded = format!(r#"{{"status":"completed"{}}}"#, " ".repeat(64 * 1024));
    aged(&task(config.path(), "2.json", &padded), 2);
    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 0, total: 2 }));
}

#[test]
fn a_task_file_that_is_not_an_object_for_two_seconds_counts_as_open() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    task(
        config.path(),
        "1.json",
        r#"{"id":"1","status":"completed"}"#,
    );
    aged(&task(config.path(), "2.json", "[1, 2]"), 2);
    let mut transcript = Transcript::new(&projects, ID);

    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 1, total: 2 }));
}

#[test]
fn a_broken_task_file_dated_in_the_future_counts_as_open() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    let broken = task(config.path(), "1.json", r#"{"id":"1","sta"#);
    fs::File::options()
        .write(true)
        .open(broken)
        .unwrap()
        .set_modified(SystemTime::now() + Duration::from_secs(60))
        .unwrap();
    let mut transcript = Transcript::new(&projects, ID);

    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 0, total: 1 }));
}

#[test]
fn over_256_task_files_the_lowest_256_ids_are_counted() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    for number in 1..=300 {
        let status = if number > 256 { "completed" } else { "pending" };
        task(
            config.path(),
            &format!("{number}.json"),
            &format!(r#"{{"status":"{status}"}}"#),
        );
    }
    let mut transcript = Transcript::new(&projects, ID);

    transcript.read();

    assert_eq!(
        transcript.facts().tasks,
        Some(Tasks {
            done: 0,
            total: 256
        })
    );
}

fn task(config: &Path, name: &str, text: &str) -> PathBuf {
    let folder = config.join("tasks").join(ID);
    fs::create_dir_all(&folder).unwrap();
    let file = folder.join(name);
    fs::write(&file, text).unwrap();
    file
}

fn aged(file: &Path, seconds: u64) {
    fs::File::options()
        .write(true)
        .open(file)
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(seconds))
        .unwrap();
}
