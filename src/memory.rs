use std::fs::{self, DirBuilder, File, OpenOptions, Permissions};
use std::io::{self, Read, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

const VERSION: u64 = 1;
const LONGEST: u64 = 64 * 1024;
const LONGEST_ID: usize = 64;

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
    directory: String,
    title: String,
    position: usize,
}

#[derive(Deserialize)]
struct Versioned {
    version: u64,
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

    pub fn recall(&self) -> Vec<Remembered> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut recalled: Vec<Remembered> = entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name();
                let id = name.to_str()?.strip_suffix(".json")?;
                plain_id(id).then_some(())?;
                let saved: Saved = serde_json::from_slice(&read(&entry.path())?).ok()?;
                let directory = PathBuf::from(saved.directory);
                (saved.version == VERSION && saved.session_id == id && directory.is_absolute())
                    .then_some(Remembered {
                        session_id: saved.session_id,
                        directory,
                        title: saved.title,
                        position: saved.position,
                    })
            })
            .collect();
        recalled.sort_by(|a, b| (a.position, &a.session_id).cmp(&(b.position, &b.session_id)));
        recalled
    }

    pub fn keep(&self, remembered: &Remembered) -> io::Result<()> {
        let id = &remembered.session_id;
        if !plain_id(id) {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let directory = remembered
            .directory
            .to_str()
            .ok_or(io::ErrorKind::InvalidData)?;
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&self.dir)?;
        if !fs::symlink_metadata(&self.dir)?.is_dir() {
            return Err(io::ErrorKind::NotADirectory.into());
        }
        fs::set_permissions(&self.dir, Permissions::from_mode(0o700))?;
        let file = self.file(id);
        if self.newer(&file) {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        let text = serde_json::to_vec(&Saved {
            version: VERSION,
            session_id: id.clone(),
            directory: directory.to_string(),
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

    pub fn forget(&self, session_id: &str) {
        if !plain_id(session_id) {
            return;
        }
        let file = self.file(session_id);
        if !self.newer(&file) {
            let _ = fs::remove_file(file);
        }
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

fn plain_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= LONGEST_ID
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn unique() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    let staged = STAGED.fetch_add(1, Ordering::Relaxed);
    format!("{}-{now}-{staged}", std::process::id())
}
