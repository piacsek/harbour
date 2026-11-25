use std::fs;

use harbour::memory::{Memory, Remembered};

use crate::{ID, folder, json, remembered};

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

fn given(id: &str, title: &str) -> Remembered {
    Remembered {
        given_title: Some(title.to_string()),
        ..remembered(id, 2)
    }
}
