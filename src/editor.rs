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
            session,
        })
    }
}
