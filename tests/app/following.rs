use std::fs;
use std::path::Path;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::Screen;
use crate::directory::{
    cwd, end_agent, following, repository, write_named_transcript, write_registry, write_transcript,
};

#[test]
fn without_a_registry_a_transcript_with_no_directory_leaves_the_start_directory() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = following(scratch.path(), &["{\"type\":\"queue-operation\"}"]);

    let shown = editor_at(&mut screen);

    assert!(shown.contains("acme editor at acme-start"), "{shown}");
}

#[test]
fn a_registry_naming_harbours_own_id_follows_its_transcript() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    write_registry(scratch.path(), &own_id(scratch.path()), &two);

    let shown = editor_at(&mut screen);

    assert!(shown.contains("acme editor at acme-one"), "{shown}");
}

#[test]
fn a_registry_naming_harbours_own_id_gives_its_directory_while_the_transcript_has_none() {
    let scratch = tempfile::tempdir().unwrap();
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &["{\"type\":\"queue-operation\"}"]);
    write_registry(scratch.path(), &own_id(scratch.path()), &two);

    let shown = editor_at(&mut screen);

    assert!(shown.contains("acme editor at acme-two"), "{shown}");
}

#[test]
fn a_registry_naming_harbours_own_id_gives_its_directory_while_the_transcript_is_missing() {
    let scratch = tempfile::tempdir().unwrap();
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &[]);
    fs::remove_dir_all(scratch.path().join("projects")).unwrap();
    write_registry(scratch.path(), &own_id(scratch.path()), &two);

    let shown = editor_at(&mut screen);

    assert!(shown.contains("acme editor at acme-two"), "{shown}");
}

#[test]
fn another_id_whose_transcript_has_no_directory_keeps_the_directory_found_last() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    write_registry(scratch.path(), "acme-fresh", &two);
    write_named_transcript(
        scratch.path(),
        "acme-fresh",
        &["{\"type\":\"queue-operation\"}"],
    );

    let shown = editor_at(&mut screen);

    assert!(shown.contains("acme editor at acme-one"), "{shown}");
}

#[test]
fn a_registry_gone_or_half_written_keeps_following_the_id_it_named() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let start = repository(scratch.path(), "acme-start");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    write_registry(scratch.path(), "acme-resumed", &start);
    write_named_transcript(scratch.path(), "acme-resumed", &[&cwd(&two)]);
    let named = editor_at(&mut screen);
    assert!(named.contains("acme editor at acme-two"), "{named}");

    remove_registry(scratch.path());
    let gone = editor_at(&mut screen);
    half_write_registry(scratch.path());
    let half = editor_at(&mut screen);
    write_registry(scratch.path(), "acme-resumed", &start);
    let back = editor_at(&mut screen);

    assert!(!gone.contains("acme editor at acme-one"), "{gone}");
    assert!(gone.contains("acme editor at acme-two"), "{gone}");
    assert!(!half.contains("acme editor at acme-one"), "{half}");
    assert!(half.contains("acme editor at acme-two"), "{half}");
    assert!(!back.contains("acme editor at acme-one"), "{back}");
    assert!(back.contains("acme editor at acme-two"), "{back}");
}

#[test]
fn a_registry_gone_after_naming_an_id_still_follows_that_transcript() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let three = repository(scratch.path(), "acme-three");
    let start = repository(scratch.path(), "acme-start");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    write_registry(scratch.path(), "acme-resumed", &start);
    write_named_transcript(scratch.path(), "acme-resumed", &[&cwd(&two)]);
    editor_at(&mut screen);
    remove_registry(scratch.path());
    write_named_transcript(scratch.path(), "acme-resumed", &[&cwd(&three)]);

    let shown = editor_at(&mut screen);

    assert!(shown.contains("acme editor at acme-three"), "{shown}");
}

#[test]
fn a_registry_gone_keeps_its_last_directory_while_no_transcript_gave_one() {
    let scratch = tempfile::tempdir().unwrap();
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &["{\"type\":\"queue-operation\"}"]);
    write_registry(scratch.path(), "acme-fresh", &two);
    write_named_transcript(
        scratch.path(),
        "acme-fresh",
        &["{\"type\":\"queue-operation\"}"],
    );
    editor_at(&mut screen);
    remove_registry(scratch.path());

    let shown = editor_at(&mut screen);

    assert!(shown.contains("acme editor at acme-two"), "{shown}");
}

#[test]
fn a_registry_gone_after_naming_an_id_with_no_transcript_never_goes_back_to_harbours_own() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let start = repository(scratch.path(), "acme-start");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    write_registry(scratch.path(), "acme-fresh", &start);
    editor_at(&mut screen);
    remove_registry(scratch.path());
    write_transcript(scratch.path(), &[&cwd(&two)]);

    let shown = editor_at(&mut screen);

    assert!(shown.contains("acme editor at acme-one"), "{shown}");
}

#[test]
fn after_claude_code_exits_a_new_directory_in_its_transcript_is_not_read() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let mut screen = following(scratch.path(), &[&cwd(&one)]);
    editor_at(&mut screen);
    end_agent(scratch.path());
    screen.wait_for("sessions ○ 1");
    write_transcript(scratch.path(), &[&cwd(&two)]);

    let shown = editor_at(&mut screen);

    assert!(shown.contains("acme editor at acme-one"), "{shown}");
}

fn editor_at(screen: &mut Screen) -> String {
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let shown = screen.wait_for("acme editor at");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    shown
}

fn own_id(scratch: &Path) -> String {
    let args = fs::read_to_string(scratch.join("args")).unwrap();
    args.split_once(' ').unwrap().1.to_string()
}

fn registry_file(scratch: &Path) -> std::path::PathBuf {
    let pid = fs::read_to_string(scratch.join("pid")).unwrap();
    scratch.join(format!("sessions/{}.json", pid.trim()))
}

fn remove_registry(scratch: &Path) {
    fs::remove_file(registry_file(scratch)).unwrap();
}

fn half_write_registry(scratch: &Path) {
    fs::write(registry_file(scratch), "{\"pid\":").unwrap();
}
