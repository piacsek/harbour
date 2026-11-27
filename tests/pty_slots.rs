#[path = "support/pty_slots.rs"]
mod pty_slots;
#[path = "support/timing.rs"]
mod timing;

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Instant;

use timing::deadline;

#[test]
fn holding_a_slot_locks_one_slot_file() {
    let folder = tempfile::tempdir().unwrap();
    pty_slots::hold_in(folder.path(), 3, 5 * deadline());
    assert_eq!(locked(folder.path(), 3), 1);
}

#[test]
fn two_threads_hold_two_different_slots() {
    let folder = tempfile::tempdir().unwrap();
    let _first = Holder::start(folder.path(), 3);
    let _second = Holder::start(folder.path(), 3);
    assert_eq!(locked(folder.path(), 3), 2);
}

#[test]
fn a_thread_waits_for_a_slot_while_all_are_taken() {
    let folder = tempfile::tempdir().unwrap();
    let first = Holder::start(folder.path(), 2);
    let _second = Holder::start(folder.path(), 2);
    let (_third, held) = Holder::spawn(folder.path().to_path_buf(), 2);
    assert!(held.recv_timeout(deadline() / 4).is_err());
    drop(first);
    held.recv_timeout(5 * deadline())
        .expect("the waiting thread never held the freed slot");
    assert_eq!(locked(folder.path(), 2), 2);
}

#[test]
fn a_thread_that_holds_twice_keeps_one_slot() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().to_path_buf();
    let (_holder, held) = Holder::doing(move || {
        pty_slots::hold_in(&path, 1, 5 * deadline());
        pty_slots::hold_in(&path, 1, 5 * deadline());
    });
    held.recv_timeout(5 * deadline())
        .expect("the thread waited on its own slot");
    assert_eq!(locked(folder.path(), 1), 1);
}

#[test]
fn a_slot_is_freed_when_its_thread_ends() {
    let folder = tempfile::tempdir().unwrap();
    drop(Holder::start(folder.path(), 1));
    assert_eq!(locked(folder.path(), 1), 0);
}

#[test]
fn a_slot_is_freed_when_its_thread_panics() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().to_path_buf();
    let (holder, _held) = Holder::doing(move || {
        pty_slots::hold_in(&path, 1, 5 * deadline());
        panic!("the test holding the slot failed");
    });
    drop(holder);
    assert_eq!(locked(folder.path(), 1), 0);
}

#[test]
fn a_wait_for_a_slot_gives_up_naming_the_folder() {
    let folder = tempfile::tempdir().unwrap();
    let _holder = Holder::start(folder.path(), 1);
    let path = folder.path().to_path_buf();
    let started = Instant::now();
    let refusal = thread::spawn(move || pty_slots::hold_in(&path, 1, deadline()))
        .join()
        .expect_err("the wait never gave up");
    let elapsed = started.elapsed();
    let message = refusal
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_default();
    assert!(
        message.contains(&folder.path().display().to_string()),
        "{message}"
    );
    assert!(elapsed >= deadline());
    assert!(elapsed < 2 * deadline());
}

fn locked(folder: &Path, count: usize) -> usize {
    (0..count)
        .filter(|n| match File::open(folder.join(format!("{n}.lock"))) {
            Ok(file) => file.try_lock().is_err(),
            Err(_) => false,
        })
        .count()
}

struct Holder {
    release: Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl Holder {
    fn start(folder: &Path, count: usize) -> Holder {
        let (holder, held) = Holder::spawn(folder.to_path_buf(), count);
        held.recv_timeout(5 * deadline())
            .expect("the thread never held a slot");
        holder
    }

    fn spawn(folder: PathBuf, count: usize) -> (Holder, Receiver<()>) {
        Holder::doing(move || pty_slots::hold_in(&folder, count, 5 * deadline()))
    }

    fn doing(holds: impl FnOnce() + Send + 'static) -> (Holder, Receiver<()>) {
        let (held_tx, held) = mpsc::channel();
        let (release, release_rx) = mpsc::channel::<()>();
        let thread = thread::spawn(move || {
            holds();
            let _ = held_tx.send(());
            let _ = release_rx.recv();
        });
        let holder = Holder {
            release,
            thread: Some(thread),
        };
        (holder, held)
    }
}

impl Drop for Holder {
    fn drop(&mut self) {
        let _ = self.release.send(());
        if thread::panicking() {
            return;
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
