use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    #[serde(default)]
    pub status: Status,
    pub waiting_for: Option<String>,
}

pub fn read(dir: &Path, pid: u32) -> Option<Record> {
    let text = fs::read_to_string(dir.join(format!("{pid}.json"))).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn sessions_dir(config_dir: Option<PathBuf>, home: Option<PathBuf>) -> Option<PathBuf> {
    config_dir
        .filter(|dir| !dir.as_os_str().is_empty())
        .or_else(|| home.map(|home| home.join(".claude")))
        .map(|dir| dir.join("sessions"))
}
