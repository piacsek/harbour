#[path = "support/pty_slots.rs"]
mod pty_slots;

use std::fs::File;
use std::path::Path;

#[test]
fn holding_a_slot_locks_one_slot_file() {
    let folder = tempfile::tempdir().unwrap();
    pty_slots::hold_in(folder.path(), 3);
    assert_eq!(locked(folder.path(), 3), 1);
}

fn locked(folder: &Path, count: usize) -> usize {
    (0..count)
        .filter(|n| match File::open(folder.join(format!("{n}.lock"))) {
            Ok(file) => file.try_lock().is_err(),
            Err(_) => false,
        })
        .count()
}
