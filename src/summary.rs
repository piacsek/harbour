use std::fs::{self, DirBuilder, File, OpenOptions, TryLockError};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use crate::state::Counts;

const LONGEST: u64 = 128;

pub fn directory_from_env() -> Option<PathBuf> {
    directory(
        std::env::var_os("XDG_STATE_HOME").map(Into::into),
        std::env::var_os("HOME").map(Into::into),
    )
}

fn directory(state_home: Option<PathBuf>, home: Option<PathBuf>) -> Option<PathBuf> {
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

pub struct Summary {
    dir: PathBuf,
    pid: u32,
    _lock: File,
    last: Option<Counts>,
}

impl Summary {
    pub fn create(dir: &Path, pid: u32) -> io::Result<Summary> {
        DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
        let lock = private(&dir.join(format!("{pid}.lock")))?;
        lock.lock()?;
        let summary = Summary {
            dir: dir.to_path_buf(),
            pid,
            _lock: lock,
            last: None,
        };
        let _ = fs::remove_file(summary.path("counts"));
        Ok(summary)
    }

    pub fn publish(&mut self, counts: Counts) {
        if self.last == Some(counts) {
            return;
        }
        if self.write(counts).is_ok() {
            self.last = Some(counts);
        }
    }

    fn write(&self, counts: Counts) -> io::Result<()> {
        let staged = self.path("tmp");
        let mut file = private(&staged)?;
        writeln!(
            file,
            "{} {} {} {}",
            counts.blocked, counts.working, counts.idle, counts.unknown
        )?;
        fs::rename(staged, self.path("counts"))
    }

    fn path(&self, extension: &str) -> PathBuf {
        self.dir.join(format!("{}.{extension}", self.pid))
    }
}

impl Drop for Summary {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path("counts"));
        let _ = fs::remove_file(self.path("tmp"));
        let _ = fs::remove_file(self.path("lock"));
    }
}

fn private(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
}
