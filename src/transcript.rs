use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{Map, Value};

const TAIL: u64 = 1024 * 1024;
const SEARCH_EVERY: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Facts {
    pub title: Option<String>,
    pub activity: Option<String>,
    pub prompt: Option<String>,
    pub branch: Option<String>,
    pub directory: Option<PathBuf>,
    pub pull_request: Option<u64>,
}

pub struct Transcript {
    projects: PathBuf,
    session_id: String,
    path: Option<PathBuf>,
    searched_at: Option<Instant>,
    offset: u64,
    partial: Vec<u8>,
    skipping: bool,
    facts: Facts,
}

impl Transcript {
    pub fn new(projects: &Path, session_id: &str) -> Transcript {
        Transcript {
            projects: projects.to_path_buf(),
            session_id: session_id.to_string(),
            path: None,
            searched_at: None,
            offset: 0,
            partial: Vec::new(),
            skipping: false,
            facts: Facts::default(),
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn facts(&self) -> &Facts {
        &self.facts
    }

    pub fn read(&mut self) {
        let Some(path) = self.found() else {
            return;
        };
        let Ok(mut file) = File::open(&path) else {
            return;
        };
        let Ok(length) = file.metadata().map(|meta| meta.len()) else {
            return;
        };
        if length < self.offset {
            self.offset = 0;
            self.partial.clear();
            self.skipping = false;
        }
        let fresh = self.offset == 0;
        let start = if length - self.offset > TAIL {
            length - TAIL
        } else {
            self.offset
        };
        if start != self.offset || (fresh && start > 0) {
            self.partial.clear();
            self.skipping = start > 0;
        }
        let mut bytes = Vec::new();
        if file.seek(SeekFrom::Start(start)).is_err()
            || file.take(length - start).read_to_end(&mut bytes).is_err()
        {
            return;
        }
        self.offset = start + bytes.len() as u64;
        self.take_lines(&bytes);
    }

    fn found(&mut self) -> Option<PathBuf> {
        if let Some(path) = &self.path {
            return Some(path.clone());
        }
        if self
            .searched_at
            .is_some_and(|at| at.elapsed() < SEARCH_EVERY)
        {
            return None;
        }
        self.searched_at = Some(Instant::now());
        let name = format!("{}.jsonl", self.session_id);
        self.path = fs::read_dir(&self.projects)
            .ok()?
            .flatten()
            .map(|folder| folder.path().join(&name))
            .find(|path| path.is_file());
        self.path.clone()
    }

    fn take_lines(&mut self, bytes: &[u8]) {
        let mut rest = bytes;
        while let Some(end) = rest.iter().position(|byte| *byte == b'\n') {
            let (line, after) = rest.split_at(end);
            rest = &after[1..];
            if self.skipping {
                self.skipping = false;
                self.partial.clear();
                continue;
            }
            self.partial.extend_from_slice(line);
            let whole = std::mem::take(&mut self.partial);
            if let Ok(Value::Object(fields)) = serde_json::from_slice(&whole) {
                self.learn(&fields);
            }
        }
        if self.skipping {
            return;
        }
        self.partial.extend_from_slice(rest);
        if self.partial.len() as u64 > TAIL {
            self.partial.clear();
            self.skipping = true;
        }
    }

    fn learn(&mut self, fields: &Map<String, Value>) {
        let text = |key: &str| fields.get(key).and_then(Value::as_str).map(str::to_string);
        if let Some(branch) = text("gitBranch") {
            self.facts.branch = Some(branch);
        }
        if let Some(directory) = text("cwd") {
            self.facts.directory = Some(PathBuf::from(directory));
        }
        match fields.get("type").and_then(Value::as_str) {
            Some("ai-title") => {
                if let Some(title) = text("aiTitle") {
                    self.facts.title = Some(title);
                }
            }
            Some("last-prompt") => {
                if let Some(prompt) = text("lastPrompt") {
                    self.facts.prompt = Some(prompt);
                }
            }
            Some("pr-link") => {
                if let Some(number) = fields.get("prNumber").and_then(Value::as_u64) {
                    self.facts.pull_request = Some(number);
                }
            }
            Some("assistant") => {
                if let Some(activity) = last_tool_call(fields) {
                    self.facts.activity = Some(activity);
                }
            }
            _ => {}
        }
    }
}

fn last_tool_call(fields: &Map<String, Value>) -> Option<String> {
    fields
        .get("message")?
        .get("content")?
        .as_array()?
        .iter()
        .rev()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("tool_use"))
        .find_map(described)
}

fn described(call: &Value) -> Option<String> {
    let name = call.get("name").and_then(Value::as_str)?;
    let input = call.get("input").and_then(Value::as_object);
    ["description", "command", "file_path"]
        .iter()
        .find_map(|key| {
            input
                .and_then(|input| input.get(*key))
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
        })
        .or(Some(name))
        .map(str::to_string)
}
