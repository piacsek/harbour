use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs::{self, DirBuilder, File, OpenOptions, Permissions, TryLockError};
use std::io::{self, Read, Write};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const VERSION: u64 = 1;
const LONGEST: u64 = 64 * 1024;
const STALE: Duration = Duration::from_secs(30 * 24 * 60 * 60);
const ATTEMPTS: usize = 5;
const LEFT_OVER: Duration = Duration::from_secs(60);

static STAGED: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remembered {
    pub session_id: String,
    pub directory: PathBuf,
    pub title: String,
    pub position: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Saved {
    version: u64,
    session_id: String,
    directory: Directory,
    title: String,
    position: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Directory {
    Text(String),
    Bytes(Vec<u8>),
}

impl Directory {
    fn of(path: &Path) -> Directory {
        match path.to_str() {
            Some(text) => Directory::Text(text.to_string()),
            None => Directory::Bytes(path.as_os_str().as_bytes().to_vec()),
        }
    }

    fn path(self) -> PathBuf {
        match self {
            Directory::Text(text) => PathBuf::from(text),
            Directory::Bytes(bytes) => PathBuf::from(OsString::from_vec(bytes)),
        }
    }
}

#[derive(Deserialize)]
struct Versioned {
    version: u64,
}

pub struct Reservation {
    path: PathBuf,
    lock: File,
}

impl Drop for Reservation {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
        let _ = self.lock.unlock();
    }
}

pub struct Memory {
    dir: PathBuf,
}

impl Memory {
    pub fn new(dir: &Path) -> Memory {
        Memory {
            dir: dir.to_path_buf(),
        }
    }

    pub fn of(root: &Path, config: &Path) -> Memory {
        let config = resolved(config);
        Memory::new(&root.join(format!(
            "{:016x}",
            fingerprint(config.as_os_str().as_bytes())
        )))
    }

    pub fn recall(&self) -> Vec<Remembered> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut recalled: Vec<Remembered> = entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name();
                let id = name.to_str()?.strip_suffix(".json")?;
                is_session_id(id).then_some(())?;
                remembered(&entry.path(), id)
            })
            .collect();
        recalled.sort_by(|a, b| (a.position, &a.session_id).cmp(&(b.position, &b.session_id)));
        recalled
    }

    pub fn keep(&self, remembered: &Remembered) -> io::Result<()> {
        let id = &remembered.session_id;
        if !is_session_id(id) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        self.private_dir()?;
        let file = self.file(id);
        if self.newer(&file) {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        let text = serde_json::to_vec(&Saved {
            version: VERSION,
            session_id: id.clone(),
            directory: Directory::of(&remembered.directory),
            title: remembered.title.clone(),
            position: remembered.position,
        })?;
        let staged = self.dir.join(format!(".{id}.{}.tmp", unique()));
        let written = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&staged)
            .and_then(|mut out| out.write_all(&text))
            .and_then(|()| fs::rename(&staged, &file));
        if written.is_err() {
            let _ = fs::remove_file(&staged);
        }
        written
    }

    pub fn place(&self, id: &str, position: usize) -> io::Result<()> {
        if !is_session_id(id) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let mut placed = remembered(&self.file(id), id).ok_or(io::ErrorKind::NotFound)?;
        placed.position = position;
        self.keep(&placed)
    }

    pub fn forget(&self, session_id: &str) {
        if !is_session_id(session_id) {
            return;
        }
        let file = self.file(session_id);
        if !self.newer(&file) {
            let _ = fs::remove_file(file);
        }
    }

    pub fn replace(&self, old: &str, remembered: &Remembered) -> io::Result<()> {
        self.keep(remembered)?;
        self.forget(old);
        Ok(())
    }

    pub fn holds(&self, id: &str) -> bool {
        is_session_id(id)
            && fs::symlink_metadata(self.file(id)).is_ok_and(|meta| meta.file_type().is_file())
    }

    pub fn forget_stale(&self, id: &str) {
        let stale = is_session_id(id)
            && fs::symlink_metadata(self.file(id))
                .and_then(|meta| meta.modified())
                .is_ok_and(|modified| {
                    SystemTime::now()
                        .duration_since(modified)
                        .is_ok_and(|age| age > STALE)
                });
        if stale {
            self.forget(id);
        }
    }

    pub fn reserve(&self, id: &str) -> io::Result<Option<Reservation>> {
        if !is_session_id(id) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        self.private_dir()?;
        let mut held_before = false;
        for _ in 0..ATTEMPTS {
            let Some(reservation) = self.lock(id)? else {
                continue;
            };
            let own = reservation.path.file_name().map(ToOwned::to_owned);
            if !self.sweep(own.as_deref()).contains(id) {
                return Ok(Some(reservation));
            }
            drop(reservation);
            if held_before {
                return Ok(None);
            }
            held_before = true;
            thread::sleep(jitter());
        }
        Err(io::ErrorKind::WouldBlock.into())
    }

    fn lock(&self, id: &str) -> io::Result<Option<Reservation>> {
        let path = self.dir.join(format!("{id}.{}.lock", unique()));
        let lock = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        let reservation = Reservation { path, lock };
        match reservation.lock.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => return Ok(None),
            Err(TryLockError::Error(error)) => return Err(error),
        }
        let locked = reservation.lock.metadata()?;
        let still_there = fs::symlink_metadata(&reservation.path)
            .is_ok_and(|there| there.dev() == locked.dev() && there.ino() == locked.ino());
        Ok(still_there.then_some(reservation))
    }

    pub fn reserved(&self) -> HashSet<String> {
        self.sweep(None)
    }

    fn sweep(&self, own: Option<&OsStr>) -> HashSet<String> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return HashSet::new();
        };
        let mut held = HashSet::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            if Some(name.as_os_str()) == own {
                continue;
            }
            if is_staged(&name) {
                remove_left_over(&entry);
                continue;
            }
            let Some(id) = name
                .to_str()
                .and_then(|name| name.strip_suffix(".lock"))
                .and_then(|name| name.split_once('.'))
                .map(|(id, _)| id)
                .filter(|id| is_session_id(id))
            else {
                continue;
            };
            if !entry.file_type().is_ok_and(|kind| kind.is_file()) {
                continue;
            }
            let Ok(lock) = OpenOptions::new().write(true).open(entry.path()) else {
                continue;
            };
            match lock.try_lock() {
                Ok(()) => {
                    let _ = fs::remove_file(entry.path());
                }
                Err(_) => {
                    held.insert(id.to_string());
                }
            }
        }
        held
    }

    fn private_dir(&self) -> io::Result<()> {
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&self.dir)?;
        if !fs::symlink_metadata(&self.dir)?.is_dir() {
            return Err(io::ErrorKind::NotADirectory.into());
        }
        fs::set_permissions(&self.dir, Permissions::from_mode(0o700))
    }

    fn file(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }

    fn newer(&self, file: &Path) -> bool {
        read(file)
            .and_then(|bytes| serde_json::from_slice::<Versioned>(&bytes).ok())
            .is_some_and(|saved| saved.version > VERSION)
    }
}

fn remembered(path: &Path, id: &str) -> Option<Remembered> {
    let saved: Saved = serde_json::from_slice(&read(path)?).ok()?;
    let directory = saved.directory.path();
    (saved.version == VERSION && saved.session_id == id && directory.is_absolute()).then_some(
        Remembered {
            session_id: saved.session_id,
            directory,
            title: saved.title,
            position: saved.position,
        },
    )
}

fn read(path: &Path) -> Option<Vec<u8>> {
    if !fs::symlink_metadata(path).ok()?.is_file() {
        return None;
    }
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW)
        .open(path)
        .ok()?;
    let mut bytes = Vec::new();
    File::take(file, LONGEST + 1).read_to_end(&mut bytes).ok()?;
    (bytes.len() as u64 <= LONGEST).then_some(bytes)
}

fn is_session_id(id: &str) -> bool {
    let groups: Vec<&str> = id.split('-').collect();
    groups.len() == 5
        && groups.iter().zip([8, 4, 4, 4, 12]).all(|(group, length)| {
            group.len() == length
                && group
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
}

fn is_staged(name: &OsStr) -> bool {
    name.to_str()
        .and_then(|name| name.strip_prefix('.'))
        .and_then(|name| name.strip_suffix(".tmp"))
        .and_then(|name| name.split_once('.'))
        .is_some_and(|(id, _)| is_session_id(id))
}

fn remove_left_over(entry: &fs::DirEntry) {
    let left_over = fs::symlink_metadata(entry.path()).is_ok_and(|meta| {
        meta.is_file()
            && meta
                .modified()
                .ok()
                .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                .is_some_and(|age| age > LEFT_OVER)
    });
    if left_over {
        let _ = fs::remove_file(entry.path());
    }
}

fn resolved(path: &Path) -> PathBuf {
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    path.ancestors()
        .find_map(|ancestor| {
            let mut real = fs::canonicalize(ancestor).ok()?;
            real.extend(path.strip_prefix(ancestor).ok()?);
            Some(real)
        })
        .unwrap_or(path)
}

fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn jitter() -> Duration {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.subsec_nanos());
    Duration::from_millis(20 + u64::from(nanos % 60))
}

fn unique() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    let staged = STAGED.fetch_add(1, Ordering::Relaxed);
    format!("{}-{now}-{staged}", std::process::id())
}
