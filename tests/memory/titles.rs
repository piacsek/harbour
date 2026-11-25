use std::fs;
use std::time::Duration;

use harbour::memory::{Memory, Remembered};

use crate::{ID, age, folder, json, newer, remembered};

#[test]
fn a_given_title_is_kept_and_recalled() {
    let scratch = tempfile::tempdir().unwrap();
    let memory = Memory::new(&folder(scratch.path()));

    memory.keep(&given(ID, "acme login fix")).unwrap();

    assert_eq!(memory.recall(), vec![given(ID, "acme login fix")]);
}

#[test]
fn a_file_written_before_given_titles_reads_without_one() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{ID}.json")), json(ID, 0)).unwrap();

    assert_eq!(Memory::new(&dir).recall(), vec![remembered(ID, 0)]);
}

#[test]
fn a_file_without_a_given_title_is_written_as_before() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());

    Memory::new(&dir).keep(&remembered(ID, 0)).unwrap();

    assert_eq!(
        fs::read_to_string(dir.join(format!("{ID}.json"))).unwrap(),
        json(ID, 0)
    );
}

#[test]
fn placing_keeps_the_given_title() {
    let scratch = tempfile::tempdir().unwrap();
    let memory = Memory::new(&folder(scratch.path()));
    memory.keep(&given(ID, "acme login fix")).unwrap();

    memory.place(ID, 3).unwrap();

    let mut placed = given(ID, "acme login fix");
    placed.position = 3;
    assert_eq!(memory.recall(), vec![placed]);
}

#[test]
fn titling_changes_only_the_given_title() {
    let scratch = tempfile::tempdir().unwrap();
    let memory = Memory::new(&folder(scratch.path()));
    memory.keep(&remembered(ID, 2)).unwrap();

    memory.title(ID, Some("acme login fix")).unwrap();

    assert_eq!(memory.recall(), vec![given(ID, "acme login fix")]);
}

#[test]
fn titling_with_none_takes_the_given_title_away() {
    let scratch = tempfile::tempdir().unwrap();
    let memory = Memory::new(&folder(scratch.path()));
    memory.keep(&given(ID, "acme login fix")).unwrap();

    memory.title(ID, None).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 2)]);
}

#[test]
fn titling_with_the_same_title_writes_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&given(ID, "acme login fix")).unwrap();
    let file = dir.join(format!("{ID}.json"));
    age(&file, Duration::from_secs(60 * 60));
    let before = fs::metadata(&file).unwrap().modified().unwrap();

    memory.title(ID, Some("acme login fix")).unwrap();

    assert_eq!(fs::metadata(&file).unwrap().modified().unwrap(), before);
}

#[test]
fn titling_a_missing_file_writes_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let memory = Memory::new(&folder(scratch.path()));

    assert!(memory.title(ID, Some("acme login fix")).is_err());
    assert!(!memory.holds(ID));
}

#[test]
fn titling_a_file_from_a_newer_harbour_leaves_it_as_it_was() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{ID}.json")), newer(ID)).unwrap();

    assert!(Memory::new(&dir).title(ID, Some("acme login fix")).is_err());
    assert_eq!(
        fs::read_to_string(dir.join(format!("{ID}.json"))).unwrap(),
        newer(ID)
    );
}

fn given(id: &str, title: &str) -> Remembered {
    Remembered {
        given_title: Some(title.to_string()),
        ..remembered(id, 2)
    }
}
