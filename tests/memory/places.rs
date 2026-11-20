use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use harbour::memory::{Memory, Remembered};

use crate::{ID, folder, remembered};

#[test]
fn a_directory_that_is_not_utf8_comes_back_byte_for_byte() {
    let scratch = tempfile::tempdir().unwrap();
    let memory = Memory::new(&folder(scratch.path()));
    let kept = Remembered {
        directory: PathBuf::from(OsStr::from_bytes(b"/home/acme/caf\xe9")),
        ..remembered(ID, 0)
    };

    memory.keep(&kept).unwrap();

    assert_eq!(memory.recall(), vec![kept]);
}

#[test]
fn a_utf8_directory_is_written_as_text() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());

    Memory::new(&dir).keep(&remembered(ID, 0)).unwrap();

    let text = fs::read_to_string(dir.join(format!("{ID}.json"))).unwrap();
    assert!(text.contains(r#""directory":"/home/acme/api""#), "{text}");
}

#[test]
fn each_claude_code_configuration_remembers_its_own_conversations() {
    let scratch = tempfile::tempdir().unwrap();
    let root = folder(scratch.path());
    let work = Memory::of(&root, Path::new("/home/acme/.claude-work"));
    let personal = Memory::of(&root, Path::new("/home/acme/.claude"));

    work.keep(&remembered(ID, 0)).unwrap();

    assert_eq!(work.recall(), vec![remembered(ID, 0)]);
    assert_eq!(personal.recall(), vec![]);
}

#[test]
fn one_id_in_two_configurations_is_two_conversations() {
    let scratch = tempfile::tempdir().unwrap();
    let root = folder(scratch.path());
    let work = Memory::of(&root, Path::new("/home/acme/.claude-work"));
    let personal = Memory::of(&root, Path::new("/home/acme/.claude"));
    let other = Remembered {
        title: "Plan the acme-docs move".to_string(),
        ..remembered(ID, 0)
    };
    work.keep(&remembered(ID, 0)).unwrap();
    personal.keep(&other).unwrap();

    personal.forget(ID);

    assert_eq!(work.recall(), vec![remembered(ID, 0)]);
    assert_eq!(personal.recall(), vec![]);
}

#[test]
fn a_configuration_named_the_same_way_twice_is_one_folder() {
    let scratch = tempfile::tempdir().unwrap();
    let root = folder(scratch.path());
    Memory::of(&root, Path::new("/home/acme/.claude"))
        .keep(&remembered(ID, 0))
        .unwrap();

    let again = Memory::of(&root, Path::new("/home/acme/.claude"));

    assert_eq!(again.recall(), vec![remembered(ID, 0)]);
}
