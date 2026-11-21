use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{registry, summary};

const QUEUE: usize = 4096;
const KEPT: Duration = Duration::from_secs(7 * 24 * 60 * 60);

#[derive(Clone, Default)]
pub struct Log {
    sender: Option<SyncSender<String>>,
    written: Option<Arc<Mutex<Receiver<()>>>>,
}

impl Log {
    pub fn open(dir: &Path) -> Log {
        let (sender, lines) = mpsc::sync_channel(QUEUE);
        let (done, written) = mpsc::sync_channel(0);
        let dir = dir.to_path_buf();
        let spawned = std::thread::Builder::new()
            .name("log".to_string())
            .spawn(move || {
                write_all(&dir, &lines);
                drop(done);
            });
        match spawned {
            Ok(_) => Log {
                sender: Some(sender),
                written: Some(Arc::new(Mutex::new(written))),
            },
            Err(_) => Log::default(),
        }
    }

    pub fn sending(sender: SyncSender<String>) -> Log {
        Log {
            sender: Some(sender),
            written: None,
        }
    }

    pub fn finish(self, within: Duration) {
        drop(self.sender);
        if let Some(written) = self.written
            && let Ok(written) = written.lock()
        {
            let _ = written.recv_timeout(within);
        }
    }

    pub fn write(&self, line: impl FnOnce() -> String) {
        if let Some(sender) = &self.sender {
            let _ = sender.try_send(format!("{} {}\n", stamp(SystemTime::now()), line()));
        }
    }
}

pub fn directory_from_env() -> Option<PathBuf> {
    summary::directory_from_env().map(|dir| dir.with_file_name("log"))
}

pub fn newest(dir: &Path) -> Option<PathBuf> {
    logs(dir)
        .filter_map(|(path, metadata)| Some((metadata.modified().ok()?, path)))
        .max()
        .map(|(_, path)| path)
}

pub fn stamp(time: SystemTime) -> String {
    let since = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seconds = since.as_secs();
    let (year, month, day) = civil(seconds / 86_400);
    let of_day = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
        of_day / 3600,
        of_day / 60 % 60,
        of_day % 60,
        since.subsec_millis()
    )
}

fn civil(days: u64) -> (u64, u64, u64) {
    let shifted = days + 719_468;
    let era = shifted / 146_097;
    let of_era = shifted % 146_097;
    let year_of_era = (of_era - of_era / 1460 + of_era / 36_524 - of_era / 146_096) / 365;
    let of_year = of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * of_year + 2) / 153;
    let day = of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    (year, month, day)
}

fn write_all(dir: &Path, lines: &Receiver<String>) {
    let Ok(mut file) = prepared(dir) else {
        return;
    };
    for line in lines {
        if file.write_all(line.as_bytes()).is_err() {
            return;
        }
    }
}

fn prepared(dir: &Path) -> io::Result<File> {
    DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
    if !fs::symlink_metadata(dir)?.is_dir() {
        return Err(io::Error::from(io::ErrorKind::NotADirectory));
    }
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    sweep(dir);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .custom_flags(nix::libc::O_NOFOLLOW)
        .open(dir.join(format!("{}.log", std::process::id())))?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(file)
}

fn sweep(dir: &Path) {
    let now = SystemTime::now();
    for (path, metadata) in logs(dir).collect::<Vec<_>>() {
        let age = metadata
            .modified()
            .ok()
            .and_then(|modified| now.duration_since(modified).ok());
        if age.is_some_and(|age| age > KEPT) && !running(&path) {
            let _ = fs::remove_file(path);
        }
    }
}

fn running(path: &Path) -> bool {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.parse::<i32>().ok())
        .is_some_and(|pid| pid > 0 && registry::alive(pid))
}

fn logs(dir: &Path) -> impl Iterator<Item = (PathBuf, fs::Metadata)> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| named_like_a_log(path))
        .filter_map(|path| {
            let metadata = fs::symlink_metadata(&path).ok()?;
            metadata.is_file().then_some((path, metadata))
        })
}

fn named_like_a_log(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".log"))
        .is_some_and(|stem| !stem.is_empty() && stem.bytes().all(|byte| byte.is_ascii_digit()))
}
