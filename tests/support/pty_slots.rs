use std::cell::RefCell;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

thread_local! {
    static HELD: RefCell<Option<File>> = const { RefCell::new(None) };
}

pub const SLOTS: usize = 16;

pub fn folder() -> PathBuf {
    PathBuf::from(format!("/tmp/harbour-pty-slots-{}", nix::unistd::getuid()))
}

pub fn hold(patience: Duration) {
    hold_in(&folder(), SLOTS, patience);
}

pub fn hold_in(folder: &Path, count: usize, patience: Duration) {
    if holding() {
        return;
    }
    fs::create_dir_all(folder).unwrap();
    let started = Instant::now();
    let slot = loop {
        match free_slot(folder, count) {
            Some(slot) => break slot,
            None if started.elapsed() >= patience => panic!(
                "no PTY slot of {} freed within {patience:?}",
                folder.display()
            ),
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

pub fn holding() -> bool {
    HELD.with(|held| held.borrow().is_some())
}
