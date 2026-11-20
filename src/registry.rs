use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

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
}

const CLOCK_SLACK: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub session_id: Option<String>,
    pub cwd: Option<PathBuf>,
}

pub fn read(dir: &Path, pid: u32, since: SystemTime, last: Option<Record>) -> Option<Record> {
    let Some(fields) = fields(dir, pid, since)? else {
        return last;
    };
    Some(Record {
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

pub fn place(dir: &Path, pid: u32, since: SystemTime) -> Option<Place> {
    let fields = fields(dir, pid, since)??;
    Some(Place {
        session_id: fields
            .get("sessionId")
            .and_then(Value::as_str)
            .map(str::to_string),
        cwd: fields.get("cwd").and_then(Value::as_str).map(PathBuf::from),
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
    config_dir
        .filter(|dir| !dir.as_os_str().is_empty())
        .or_else(|| home.map(|home| home.join(".claude")))
        .map(|dir| dir.join("sessions"))
}
