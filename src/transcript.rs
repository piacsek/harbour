use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use serde_json::{Map, Value};

const TAIL: u64 = 1024 * 1024;
const SEARCH_EVERY: Duration = Duration::from_secs(5);
const META: u64 = 64 * 1024;
const CLOCK_SLACK: Duration = Duration::from_secs(2);
const SHOWN: usize = 4096;
const TASK_FILES: usize = 256;
const WRITING: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Facts {
    pub title: Option<String>,
    pub prompt: Option<String>,
    pub branch: Option<String>,
    pub directory: Option<PathBuf>,
    pub pull_request: Option<u64>,
    pub subagents: usize,
    pub tasks: Option<Tasks>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tasks {
    pub done: usize,
    pub total: usize,
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
    since: SystemTime,
    done: HashSet<OsString>,
    list: Option<List>,
    writing: Duration,
}

#[derive(Default)]
struct List {
    seen: BTreeMap<u64, bool>,
    done: BTreeSet<u64>,
    largest: Option<u64>,
}

impl List {
    fn replaced_by(&self, present: &BTreeMap<u64, bool>) -> bool {
        let Some(&first) = present.keys().next() else {
            return false;
        };
        self.largest.is_none_or(|largest| first > largest)
    }
}

enum Listing {
    Missing,
    Partial,
    Whole(BTreeMap<u64, bool>, Option<u64>),
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
            since: SystemTime::UNIX_EPOCH,
            done: HashSet::new(),
            list: None,
            writing: WRITING,
        }
    }

    pub fn since(mut self, started: SystemTime) -> Transcript {
        self.since = started;
        self
    }

    pub fn with_writing(mut self, window: Duration) -> Transcript {
        self.writing = window;
        self
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn existing(projects: &Path) -> Option<HashSet<String>> {
        let folders = fs::read_dir(projects).ok()?;
        let ids = folders
            .flatten()
            .filter_map(|folder| fs::read_dir(folder.path()).ok())
            .flat_map(|files| files.flatten())
            .filter_map(|file| {
                let name = file.file_name();
                let id = name.to_str()?.strip_suffix(".jsonl")?;
                (plain(id) && file.path().is_file()).then(|| id.to_string())
            })
            .collect();
        Some(ids)
    }

    pub fn located(projects: &Path, session_id: &str) -> Option<PathBuf> {
        Transcript::new(projects, session_id).found()
    }

    pub fn exists(projects: &Path, session_id: &str) -> bool {
        Transcript::new(projects, session_id).found().is_some()
    }

    pub fn is_found(&self) -> bool {
        self.path.is_some()
    }

    pub fn facts(&self) -> &Facts {
        &self.facts
    }

    pub fn read_now(&mut self) {
        self.searched_at = None;
        self.read();
    }

    pub fn read(&mut self) {
        self.read_tasks();
        let Some(path) = self.found() else {
            return;
        };
        self.facts.subagents = self.running(&path.with_extension("").join("subagents"));
        let Ok(mut file) = File::open(&path) else {
            self.lost();
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

    fn lost(&mut self) {
        self.path = None;
        self.searched_at = None;
        self.offset = 0;
        self.partial.clear();
        self.skipping = false;
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
        if !plain(&self.session_id) {
            return None;
        }
        let name = format!("{}.jsonl", self.session_id);
        self.path = fs::read_dir(&self.projects)
            .ok()?
            .flatten()
            .map(|folder| folder.path().join(&name))
            .find(|path| path.is_file());
        self.path.clone()
    }

    fn read_tasks(&mut self) {
        let listing = match self.projects.parent() {
            Some(config) if plain(&self.session_id) => {
                listing(&config.join("tasks").join(&self.session_id), self.writing)
            }
            _ => Listing::Missing,
        };
        let (present, largest) = match listing {
            Listing::Missing => {
                self.list = None;
                self.facts.tasks = None;
                return;
            }
            Listing::Partial => return,
            Listing::Whole(present, largest) => (present, largest),
        };
        let list = self.list.get_or_insert_with(List::default);
        if list.replaced_by(&present) {
            *list = List::default();
        }
        list.largest = list.largest.max(largest);
        let emptied = present.is_empty();
        for (id, completed) in &list.seen {
            if !present.contains_key(id) && (*completed || emptied) {
                list.done.insert(*id);
            }
        }
        list.seen = present;
        let completed = list.seen.values().filter(|completed| **completed).count();
        let total = list.seen.len() + list.done.len();
        self.facts.tasks = (total > 0).then_some(Tasks {
            done: completed + list.done.len(),
            total,
        });
    }

    fn running(&mut self, folder: &Path) -> usize {
        let Ok(entries) = fs::read_dir(folder) else {
            return 0;
        };
        let mut count = 0;
        for entry in entries.flatten() {
            let name = entry.file_name();
            if self.done.contains(&name) || !name.to_string_lossy().ends_with(".meta.json") {
                continue;
            }
            match meta(&entry.path(), self.since) {
                Meta::Running => count += 1,
                Meta::Done => {
                    self.done.insert(name);
                }
                Meta::Unread => {}
            }
        }
        count
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
            if whole.len() as u64 > TAIL {
                continue;
            }
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
        let text = |key: &str| {
            fields
                .get(key)
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
        };
        if let Some(branch) = text("gitBranch") {
            self.facts.branch = Some(shown(branch));
        }
        if let Some(directory) = text("cwd") {
            self.facts.directory = Some(PathBuf::from(directory));
        }
        match fields.get("type").and_then(Value::as_str) {
            Some("ai-title") => {
                if let Some(title) = text("aiTitle") {
                    self.facts.title = Some(shown(title));
                }
            }
            Some("last-prompt") => {
                if let Some(prompt) = text("lastPrompt") {
                    self.facts.prompt = Some(shown(prompt));
                }
            }
            Some("pr-link") => {
                if let Some(number) = fields.get("prNumber").and_then(Value::as_u64) {
                    self.facts.pull_request = Some(number);
                }
            }
            _ => {}
        }
    }
}

pub fn new_session_id() -> io::Result<String> {
    let mut bytes = [0u8; 16];
    File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    bytes[6] = bytes[6] & 0x0f | 0x40;
    bytes[8] = bytes[8] & 0x3f | 0x80;
    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    ))
}

fn plain(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn shown(text: &str) -> String {
    let mut end = text.len().min(SHOWN);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_string()
}

fn listing(folder: &Path, writing: Duration) -> Listing {
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Listing::Missing,
        Err(_) => return Listing::Partial,
    };
    let mut files = BTreeMap::new();
    let mut largest = None;
    for entry in entries {
        let Ok(entry) = entry else {
            return Listing::Partial;
        };
        let Some(id) = task_id(&entry.file_name()) else {
            continue;
        };
        match entry.file_type() {
            Ok(kind) if kind.is_file() => {
                largest = largest.max(Some(id));
                files.insert(id, entry.path());
                if files.len() > TASK_FILES {
                    files.pop_last();
                }
            }
            Ok(_) => {}
            Err(_) => return Listing::Partial,
        }
    }
    let mut present = BTreeMap::new();
    for (id, path) in files {
        let completed = match task_completed(&path) {
            Err(_) => return Listing::Partial,
            Ok(Some(completed)) => completed,
            Ok(None) if being_written(&path, writing) => return Listing::Partial,
            Ok(None) => false,
        };
        present.insert(id, completed);
    }
    Listing::Whole(present, largest)
}

fn task_id(name: &OsStr) -> Option<u64> {
    let digits = name.to_str()?.strip_suffix(".json")?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

fn being_written(path: &Path, writing: Duration) -> bool {
    let Ok(modified) = fs::metadata(path).and_then(|meta| meta.modified()) else {
        return true;
    };
    let now = SystemTime::now();
    let age = now
        .duration_since(modified)
        .or_else(|_| modified.duration_since(now))
        .unwrap_or_default();
    age < writing
}

fn task_completed(path: &Path) -> io::Result<Option<bool>> {
    let mut bytes = Vec::new();
    File::open(path)?.take(META).read_to_end(&mut bytes)?;
    Ok(match serde_json::from_slice(&bytes) {
        Ok(Value::Object(fields)) => {
            Some(fields.get("status").and_then(Value::as_str) == Some("completed"))
        }
        _ => None,
    })
}

enum Meta {
    Running,
    Done,
    Unread,
}

fn meta(path: &Path, since: SystemTime) -> Meta {
    let Ok(file) = File::open(path) else {
        return Meta::Unread;
    };
    let modified = file.metadata().and_then(|meta| meta.modified());
    if modified.is_ok_and(|modified| modified + CLOCK_SLACK < since) {
        return Meta::Done;
    }
    let mut bytes = Vec::new();
    if file.take(META).read_to_end(&mut bytes).is_err() {
        return Meta::Unread;
    }
    match serde_json::from_slice(&bytes) {
        Ok(Value::Object(fields)) if fields.get("finished") == Some(&Value::Bool(true)) => {
            Meta::Done
        }
        Ok(Value::Object(_)) => Meta::Running,
        _ => Meta::Unread,
    }
}
