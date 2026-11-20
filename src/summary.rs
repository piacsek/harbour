use std::fs::{self, DirBuilder, File, OpenOptions, Permissions, TryLockError};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime};

use crate::state::Counts;

const LONGEST: u64 = 128;
const RETRY: Duration = Duration::from_secs(5);

static CREATED: AtomicU64 = AtomicU64::new(0);

pub fn directory_from_env() -> Option<PathBuf> {
    directory(
        std::env::var_os("XDG_STATE_HOME").map(Into::into),
        std::env::var_os("HOME").map(Into::into),
    )
}

fn directory(state_home: Option<PathBuf>, home: Option<PathBuf>) -> Option<PathBuf> {
    state_home
        .filter(|dir| dir.is_absolute())
        .or_else(|| {
            home.filter(|home| home.is_absolute())
                .map(|home| home.join(".local/state"))
        })
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
        .filter(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(published_name)
        })
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
    fs::symlink_metadata(path).is_ok_and(|meta| meta.is_file())
}

pub struct Summary {
    dir: PathBuf,
    retry: Duration,
    due: Instant,
    publication: Option<Publication>,
    last: Option<Counts>,
}

impl Summary {
    pub fn new(dir: &Path) -> Summary {
        Summary {
            dir: dir.to_path_buf(),
            retry: RETRY,
            due: Instant::now(),
            publication: None,
            last: None,
        }
    }

    pub fn retrying_every(mut self, retry: Duration) -> Summary {
        self.retry = retry;
        self
    }

    pub fn publish(&mut self, counts: Counts) {
        let now = Instant::now();
        if self.publication.is_none() {
            if now < self.due {
                return;
            }
            self.due = now + self.retry;
            self.last = None;
            self.publication = Publication::create(&self.dir).ok();
        }
        let Some(publication) = &self.publication else {
            return;
        };
        let result = if self.last != Some(counts) {
            publication.write(counts)
        } else if now >= self.due {
            self.due = now + self.retry;
            publication.check()
        } else {
            Ok(())
        };
        match result {
            Ok(()) => self.last = Some(counts),
            Err(_) => {
                self.publication = None;
                self.last = None;
                self.due = now + self.retry;
            }
        }
    }
}

struct Publication {
    dir: PathBuf,
    name: String,
    lock: File,
}

impl Publication {
    fn create(dir: &Path) -> io::Result<Publication> {
        DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
        if !fs::symlink_metadata(dir)?.is_dir() {
            return Err(io::ErrorKind::NotADirectory.into());
        }
        fs::set_permissions(dir, Permissions::from_mode(0o700))?;
        let name = unique_name();
        let lock = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(dir.join(format!("{name}.lock")))?;
        let publication = Publication {
            dir: dir.to_path_buf(),
            name,
            lock,
        };
        publication.lock.try_lock()?;
        sweep(dir, &publication.name);
        Ok(publication)
    }

    fn write(&self, counts: Counts) -> io::Result<()> {
        let staged = self.path("tmp");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&staged)?;
        writeln!(
            file,
            "{} {} {} {}",
            counts.blocked, counts.working, counts.idle, counts.unknown
        )?;
        fs::rename(staged, self.path("counts"))?;
        self.check()
    }

    fn check(&self) -> io::Result<()> {
        fs::metadata(self.path("lock"))?;
        fs::metadata(self.path("counts"))?;
        Ok(())
    }

    fn path(&self, extension: &str) -> PathBuf {
        self.dir.join(format!("{}.{extension}", self.name))
    }
}

impl Drop for Publication {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.path("counts"));
        let _ = fs::remove_file(self.path("tmp"));
        let _ = fs::remove_file(self.path("lock"));
    }
}

pub fn sweep(dir: &Path, own: &str) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".lock"))
            .filter(|name| published_name(name) && *name != own)
        else {
            continue;
        };
        if !entry.file_type().is_ok_and(|kind| kind.is_file()) {
            continue;
        }
        let Ok(lock) = OpenOptions::new().write(true).open(&path) else {
            continue;
        };
        if lock.try_lock().is_err() {
            continue;
        }
        let _ = fs::remove_file(dir.join(format!("{name}.counts")));
        let _ = fs::remove_file(dir.join(format!("{name}.tmp")));
        let _ = fs::remove_file(&path);
    }
}

fn published_name(name: &str) -> bool {
    let parts: Vec<&str> = name.split('-').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn unique_name() -> String {
    let started = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    let created = CREATED.fetch_add(1, Ordering::Relaxed);
    format!("{}-{started}-{created}", std::process::id())
}
