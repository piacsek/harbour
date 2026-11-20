use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

const SMALL: u64 = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub repository: String,
    pub branch: Option<String>,
}

impl fmt::Display for Place {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self.branch {
            Some(branch) => write!(formatter, "{} · {branch}", self.repository),
            None => write!(formatter, "{}", self.repository),
        }
    }
}

pub fn place(directory: &Path) -> Place {
    directory
        .ancestors()
        .find_map(repository_at)
        .unwrap_or_else(|| Place {
            repository: name_of(directory),
            branch: None,
        })
}

fn repository_at(folder: &Path) -> Option<Place> {
    let dot_git = folder.join(".git");
    if dot_git.is_dir() {
        return Some(Place {
            repository: name_of(folder),
            branch: branch(&dot_git),
        });
    }
    if dot_git.is_file() {
        return linked(folder, &dot_git);
    }
    if folder.join("HEAD").is_file()
        && folder.join("objects").is_dir()
        && folder.join("refs").is_dir()
    {
        return Some(Place {
            repository: bare_name(folder),
            branch: branch(folder),
        });
    }
    None
}

fn linked(folder: &Path, dot_git: &Path) -> Option<Place> {
    let target = small(dot_git)?;
    let git_dir = folder.join(target.trim().strip_prefix("gitdir:")?.trim());
    if !git_dir.is_dir() {
        return None;
    }
    let repository = match small(&git_dir.join("commondir")) {
        Some(common) => {
            let common = normalized(&git_dir.join(common.trim()));
            match common.file_name() {
                Some(name) if name == ".git" => name_of(common.parent().unwrap_or(&common)),
                _ => bare_name(&common),
            }
        }
        None => name_of(folder),
    };
    Some(Place {
        repository,
        branch: branch(&git_dir),
    })
}

fn bare_name(folder: &Path) -> String {
    let name = name_of(folder);
    name.strip_suffix(".git").unwrap_or(&name).to_string()
}

fn branch(git_dir: &Path) -> Option<String> {
    let head = small(&git_dir.join("HEAD"))?;
    let head = head.trim();
    if let Some(reference) = head.strip_prefix("ref:") {
        let reference = reference.trim();
        return Some(
            reference
                .strip_prefix("refs/heads/")
                .unwrap_or(reference)
                .to_string(),
        );
    }
    (head.len() >= 40 && head.chars().all(|c| c.is_ascii_hexdigit())).then(|| "HEAD".to_string())
}

fn normalized(path: &Path) -> PathBuf {
    let mut kept = PathBuf::new();
    for part in path.components() {
        match part {
            std::path::Component::ParentDir => {
                kept.pop();
            }
            std::path::Component::CurDir => {}
            other => kept.push(other),
        }
    }
    kept
}

fn small(path: &Path) -> Option<String> {
    let mut text = String::new();
    File::open(path)
        .ok()?
        .take(SMALL)
        .read_to_string(&mut text)
        .ok()?;
    Some(text)
}

fn name_of(folder: &Path) -> String {
    folder
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| folder.display().to_string())
}
