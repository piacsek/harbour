use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde::Deserialize;

const READ_AT_MOST: u64 = 4 * 1024 * 1024;

pub struct Transcript {
    projects: PathBuf,
    id: String,
    path: Option<PathBuf>,
}

#[derive(Deserialize)]
struct Line {
    cwd: Option<PathBuf>,
}

impl Transcript {
    pub fn new(projects: &Path) -> io::Result<Transcript> {
        Ok(Transcript {
            projects: projects.to_path_buf(),
            id: session_id()?,
            path: None,
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn directory(&mut self) -> Option<PathBuf> {
        if self.path.is_none() {
            self.path = self.find();
        }
        last_directory(self.path.as_ref()?)
    }

    fn find(&self) -> Option<PathBuf> {
        let name = format!("{}.jsonl", self.id);
        fs::read_dir(&self.projects)
            .ok()?
            .flatten()
            .map(|project| project.path().join(&name))
            .find(|path| path.is_file())
    }
}

fn last_directory(path: &Path) -> Option<PathBuf> {
    let mut file = File::open(path).ok()?;
    let start = file.metadata().ok()?.len().saturating_sub(READ_AT_MOST);
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::new();
    file.take(READ_AT_MOST).read_to_end(&mut bytes).ok()?;
    let whole_lines = usize::from(start > 0);
    let lines: Vec<&[u8]> = bytes
        .split(|byte| *byte == b'\n')
        .skip(whole_lines)
        .collect();
    lines
        .into_iter()
        .rev()
        .find_map(|line| serde_json::from_slice::<Line>(line).ok()?.cwd)
}

fn session_id() -> io::Result<String> {
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
