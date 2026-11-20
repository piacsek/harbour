use std::fs;
use std::os::unix::fs::symlink;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use harbour::memory::Memory;

use crate::{ID, OTHER_ID, age, folder, mode, names, remembered};

#[test]
fn a_reserved_id_cannot_be_reserved_again() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let first = Memory::new(&dir).reserve(ID).unwrap();

    let second = Memory::new(&dir).reserve(ID).unwrap();

    assert!(first.is_some());
    assert!(second.is_none());
}

#[test]
fn a_released_id_can_be_reserved_again() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    drop(Memory::new(&dir).reserve(ID).unwrap());

    let again = Memory::new(&dir).reserve(ID).unwrap();

    assert!(again.is_some());
}

#[test]
fn another_id_is_reserved_on_its_own() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let _first = Memory::new(&dir).reserve(ID).unwrap();

    let other = Memory::new(&dir).reserve(OTHER_ID).unwrap();

    assert!(other.is_some());
}

#[test]
fn reserved_ids_are_listed() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let _held = Memory::new(&dir).reserve(ID).unwrap();

    let reserved = Memory::new(&dir).reserved();

    assert!(reserved.contains(ID));
    assert!(!reserved.contains(OTHER_ID));
}

#[test]
fn a_dead_reservation_is_swept_and_not_listed() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(&dir).unwrap();
    let dead = dir.join(format!("{ID}.1-2-3.lock"));
    fs::write(&dead, "").unwrap();

    let reserved = Memory::new(&dir).reserve(OTHER_ID).unwrap();

    assert!(reserved.is_some());
    assert!(!Memory::new(&dir).reserved().contains(ID));
    assert!(!dead.exists());
}

#[test]
fn a_reservation_taken_while_another_harbour_sweeps_stays_listed() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(&dir).unwrap();
    let done = Arc::new(AtomicBool::new(false));
    let sweeper = {
        let dir = dir.clone();
        let done = Arc::clone(&done);
        thread::spawn(move || {
            while !done.load(Ordering::Relaxed) {
                Memory::new(&dir).reserved();
            }
        })
    };

    let lost = (0..2000)
        .filter(|_| {
            Memory::new(&dir)
                .reserve(ID)
                .ok()
                .flatten()
                .is_some_and(|_held| !Memory::new(&dir).reserved().contains(ID))
        })
        .count();

    done.store(true, Ordering::Relaxed);
    sweeper.join().unwrap();
    assert_eq!(lost, 0);
}

#[test]
fn two_reservations_of_one_id_at_once_never_both_hold() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(&dir).unwrap();

    let both = (0..100)
        .filter(|_| {
            let start = Arc::new(Barrier::new(2));
            let other = {
                let dir = dir.clone();
                let start = Arc::clone(&start);
                thread::spawn(move || {
                    start.wait();
                    Memory::new(&dir).reserve(ID).ok().flatten()
                })
            };
            start.wait();
            let mine = Memory::new(&dir).reserve(ID).ok().flatten();
            let theirs = other.join().unwrap();
            mine.is_some() && theirs.is_some()
        })
        .count();

    assert_eq!(both, 0);
}

#[test]
fn a_reservation_is_private() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let _held = Memory::new(&dir).reserve(ID).unwrap();

    let lock = names(&dir)
        .into_iter()
        .find(|name| name.ends_with(".lock"))
        .unwrap();

    assert_eq!(mode(&dir.join(lock)), 0o600);
}

#[test]
fn a_reservation_is_not_recalled() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    let _held = memory.reserve(OTHER_ID).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn a_staged_file_left_over_a_minute_is_swept() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(&dir).unwrap();
    let left = dir.join(format!(".{ID}.1-2-3.tmp"));
    fs::write(&left, "{").unwrap();
    age(&left, Duration::from_secs(120));

    Memory::new(&dir).reserved();

    assert!(!left.exists());
}

#[test]
fn a_staged_file_being_written_is_not_swept() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(&dir).unwrap();
    let fresh = dir.join(format!(".{ID}.1-2-3.tmp"));
    fs::write(&fresh, "{").unwrap();

    Memory::new(&dir).reserved();

    assert!(fresh.exists());
}

#[test]
fn only_harbours_own_old_staged_files_are_swept() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(dir.join(format!(".{ID}.1-2-4.tmp"))).unwrap();
    let target = scratch.path().join("acme-target");
    fs::write(&target, "kept").unwrap();
    age(&target, Duration::from_secs(120));
    symlink(&target, dir.join(format!(".{ID}.1-2-5.tmp"))).unwrap();
    let foreign = dir.join(".acme-notes.tmp");
    fs::write(&foreign, "kept").unwrap();
    age(&foreign, Duration::from_secs(120));

    Memory::new(&dir).reserved();

    assert!(dir.join(format!(".{ID}.1-2-4.tmp")).is_dir());
    assert!(fs::symlink_metadata(dir.join(format!(".{ID}.1-2-5.tmp"))).is_ok());
    assert!(target.exists());
    assert!(foreign.exists());
}
