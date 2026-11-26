use std::fs;
use std::path::{Path, PathBuf};

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
fn at_most_256_task_files_are_counted() {
    let config = tempfile::tempdir().unwrap();
    let projects = config.path().join("projects");
    let file = transcript_file(&projects, "-home-acme-api");
    append(&file, r#"{"type":"ai-title","aiTitle":"Many steps"}"#);
    for number in 1..=257 {
        task(
            config.path(),
            &format!("{number}.json"),
            r#"{"status":"pending"}"#,
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
fn a_list_emptied_while_two_tasks_were_open_has_no_task_fact() {
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

    assert_eq!(transcript.facts().tasks, None);
}

#[test]
fn a_list_read_halfway_through_its_clearing_shows_its_largest_total_done() {
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

    for gone in &files[4..] {
        fs::remove_file(gone).unwrap();
    }
    transcript.read();

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
fn a_listed_file_that_vanishes_mid_read_keeps_the_previous_count() {
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

    let folder = config.path().join("tasks").join(ID);
    fs::remove_file(folder.join("2.json")).unwrap();
    std::os::unix::fs::symlink(folder.join("gone.json"), folder.join("2.json")).unwrap();
    transcript.read();

    assert_eq!(transcript.facts().tasks, Some(Tasks { done: 1, total: 2 }));
}

fn task(config: &Path, name: &str, text: &str) -> PathBuf {
    let folder = config.join("tasks").join(ID);
    fs::create_dir_all(&folder).unwrap();
    let file = folder.join(name);
    fs::write(&file, text).unwrap();
    file
}
