use std::io;
use std::path::{Path, PathBuf};

use crate::session::Session;

pub struct Program {
    name: String,
    command: Vec<String>,
}

impl Program {
    pub fn new(name: &str, command: Vec<String>) -> Program {
        Program {
            name: name.to_string(),
            command,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub struct Editor {
    pub id: usize,
    pub directory: PathBuf,
    pub label: String,
    pub session: Session,
}

impl Editor {
    pub fn start(
        program: &Program,
        directory: &Path,
        id: usize,
        rows: u16,
        cols: u16,
    ) -> io::Result<Editor> {
        let session = Session::start(&program.command, directory, rows, cols)?;
        Ok(Editor {
            id,
            directory: directory.to_path_buf(),
            label: label(directory),
            session,
        })
    }
}

fn label(directory: &Path) -> String {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    match home.and_then(|home| directory.strip_prefix(home).ok()) {
        Some(below) if below.as_os_str().is_empty() => "~".to_string(),
        Some(below) => format!("~/{}", below.display()),
        None => directory.display().to_string(),
    }
}
