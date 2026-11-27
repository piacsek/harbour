use std::cell::RefCell;
use std::fs::{self, File, OpenOptions};
use std::path::Path;

thread_local! {
    static HELD: RefCell<Option<File>> = const { RefCell::new(None) };
}

pub fn hold_in(folder: &Path, _count: usize) {
    fs::create_dir_all(folder).unwrap();
    let slot = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(folder.join("0.lock"))
        .unwrap();
    slot.lock().unwrap();
    HELD.with(|held| *held.borrow_mut() = Some(slot));
}
