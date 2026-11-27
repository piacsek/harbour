use std::cell::RefCell;
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use std::thread;
use std::time::Duration;

thread_local! {
    static HELD: RefCell<Option<File>> = const { RefCell::new(None) };
}

pub fn hold_in(folder: &Path, count: usize) {
    if HELD.with(|held| held.borrow().is_some()) {
        return;
    }
    fs::create_dir_all(folder).unwrap();
    let slot = loop {
        match free_slot(folder, count) {
            Some(slot) => break slot,
            None => thread::sleep(Duration::from_millis(20)),
        }
    };
    HELD.with(|held| *held.borrow_mut() = Some(slot));
}

fn free_slot(folder: &Path, count: usize) -> Option<File> {
    (0..count)
        .map(|n| open_slot(folder, n))
        .find(|slot| slot.try_lock().is_ok())
}

fn open_slot(folder: &Path, n: usize) -> File {
    OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(folder.join(format!("{n}.lock")))
        .unwrap()
}
