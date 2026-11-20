use std::fs::{self, File, TryLockError};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::state::Counts;

const LONGEST: u64 = 128;

pub fn directory(state_home: Option<PathBuf>, home: Option<PathBuf>) -> Option<PathBuf> {
    state_home
        .filter(|dir| dir.is_absolute())
        .or_else(|| home.map(|home| home.join(".local/state")))
        .map(|dir| dir.join("harbour/status"))
}

pub fn total(dir: &Path) -> Counts {
    let Ok(entries) = fs::read_dir(dir) else {
        return Counts::default();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "counts"))
        .filter(|path| running(&path.with_extension("lock")))
        .filter_map(|path| read(&path))
        .fold(Counts::default(), Counts::plus)
}

fn running(lock: &Path) -> bool {
    regular(lock)
        && File::open(lock)
            .is_ok_and(|file| matches!(file.try_lock_shared(), Err(TryLockError::WouldBlock)))
}

fn read(path: &Path) -> Option<Counts> {
    if !regular(path) {
        return None;
    }
    let mut text = String::new();
    File::open(path)
        .ok()?
        .take(LONGEST)
        .read_to_string(&mut text)
        .ok()?;
    let numbers: Vec<usize> = text
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    let [blocked, working, idle, unknown] = numbers[..] else {
        return None;
    };
    Some(Counts {
        blocked,
        working,
        idle,
        unknown,
    })
}

fn regular(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.is_file())
}
