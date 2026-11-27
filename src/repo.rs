use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
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

pub fn place(directory: &Path, home: Option<&Path>) -> Place {
    nearest(directory, home)
        .map(|(_, place)| place)
        .unwrap_or_else(|| Place {
            repository: name_of(directory),
            branch: None,
        })
}

pub fn root<'a>(directory: &'a Path, home: Option<&Path>) -> Option<&'a Path> {
    nearest(directory, home).map(|(folder, _)| folder)
}

fn nearest<'a>(directory: &'a Path, home: Option<&Path>) -> Option<(&'a Path, Place)> {
    let home = home.and_then(identity);
    directory
        .ancestors()
        .filter(|folder| home.is_none() || identity(folder) != home)
        .find_map(|folder| Some((folder, repository_at(folder)?)))
}

pub fn identity(folder: &Path) -> Option<(u64, u64)> {
    let metadata = fs::metadata(folder).ok()?;
    Some((metadata.dev(), metadata.ino()))
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
    let git_dir = folder.join(path_of(
        target.trim_ascii().strip_prefix(b"gitdir:")?.trim_ascii(),
    ));
    if !git_dir.is_dir() {
        return None;
    }
    let repository = match small(&git_dir.join("commondir")) {
        Some(common) => {
            let common = normalized(&git_dir.join(path_of(common.trim_ascii())));
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
    let head = head.trim_ascii();
    if let Some(reference) = head.strip_prefix(b"ref:") {
        let reference = reference.trim_ascii();
        let name = reference.strip_prefix(b"refs/heads/").unwrap_or(reference);
        return (name != b".invalid").then(|| String::from_utf8_lossy(name).into_owned());
    }
    (head.len() >= 40 && head.iter().all(u8::is_ascii_hexdigit)).then(|| "HEAD".to_string())
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

fn small(path: &Path) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)
        .ok()?
        .take(SMALL)
        .read_to_end(&mut bytes)
        .ok()?;
    Some(bytes)
}

fn path_of(bytes: &[u8]) -> &Path {
    Path::new(OsStr::from_bytes(bytes))
}

fn name_of(folder: &Path) -> String {
    folder
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| folder.display().to_string())
}
