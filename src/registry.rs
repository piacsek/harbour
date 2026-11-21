use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use nix::errno::Errno;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use serde::Deserialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Busy,
    Shell,
    Idle,
    Waiting,
    #[default]
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub status: Status,
    pub waiting_for: Option<String>,
    pub status_since: Option<SystemTime>,
    pub session_id: Option<String>,
    pub cwd: Option<PathBuf>,
}

const CLOCK_SLACK: Duration = Duration::from_secs(2);
const LONGEST: u64 = 64 * 1024;

pub fn read(dir: &Path, pid: u32, since: SystemTime, last: Option<Record>) -> Option<Record> {
    let Some(fields) = fields(dir, pid, since)? else {
        return last;
    };
    let status_since = match fields.get("statusUpdatedAt") {
        None => None,
        Some(value) => value
            .as_u64()
            .map(|millis| UNIX_EPOCH + Duration::from_millis(millis))
            .or(last.as_ref().and_then(|last| last.status_since)),
    };
    let session_id = match fields.get("sessionId") {
        None => None,
        Some(value) => value
            .as_str()
            .map(str::to_string)
            .or(last.and_then(|last| last.session_id)),
    };
    Some(Record {
        status_since,
        session_id,
        cwd: fields.get("cwd").and_then(Value::as_str).map(PathBuf::from),
        status: fields
            .get("status")
            .and_then(|status| Status::deserialize(status).ok())
            .unwrap_or_default(),
        waiting_for: fields
            .get("waitingFor")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn fields(dir: &Path, pid: u32, since: SystemTime) -> Option<Option<Map<String, Value>>> {
    let mut file = File::open(dir.join(format!("{pid}.json"))).ok()?;
    let modified = file.metadata().and_then(|meta| meta.modified()).ok();
    if modified.is_some_and(|modified| modified + CLOCK_SLACK < since) {
        return None;
    }
    let mut text = String::new();
    if file.read_to_string(&mut text).is_err() {
        return Some(None);
    }
    Some(serde_json::from_str(&text).ok())
}

pub fn sessions_dir(config_dir: Option<PathBuf>, home: Option<PathBuf>) -> Option<PathBuf> {
    claude_dir(config_dir, home).map(|dir| dir.join("sessions"))
}

pub fn projects_dir(config_dir: Option<PathBuf>, home: Option<PathBuf>) -> Option<PathBuf> {
    claude_dir(config_dir, home).map(|dir| dir.join("projects"))
}

fn claude_dir(config_dir: Option<PathBuf>, home: Option<PathBuf>) -> Option<PathBuf> {
    config_dir
        .filter(|dir| !dir.as_os_str().is_empty())
        .or_else(|| home.map(|home| home.join(".claude")))
}

pub fn open_ids(dir: &Path) -> HashSet<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return HashSet::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name();
            let pid: i32 = name.to_str()?.strip_suffix(".json")?.parse().ok()?;
            if pid <= 0 || !alive(pid) || !entry.file_type().ok()?.is_file() {
                return None;
            }
            let mut text = String::new();
            File::open(entry.path())
                .ok()?
                .take(LONGEST)
                .read_to_string(&mut text)
                .ok()?;
            let fields: Map<String, Value> = serde_json::from_str(&text).ok()?;
            fields.get("sessionId")?.as_str().map(str::to_string)
        })
        .collect()
}

pub(crate) fn alive(pid: i32) -> bool {
    matches!(kill(Pid::from_raw(pid), None), Ok(()) | Err(Errno::EPERM))
}
