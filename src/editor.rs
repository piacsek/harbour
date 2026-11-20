use std::ffi::OsString;
use std::fs::Permissions;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::nvim;
use crate::session::Session;

const LEFTMOST_IN_NORMAL_MODE: &str = "mode() ==# 'n' && winnr('h') == winnr()";

pub struct Program {
    name: String,
    command: Vec<String>,
    neovim: bool,
}

impl Program {
    pub fn new(name: &str, command: Vec<String>) -> Program {
        Program {
            name: name.to_string(),
            command,
            neovim: false,
        }
    }

    pub fn neovim(name: &str, command: Vec<String>) -> Program {
        Program {
            neovim: true,
            ..Program::new(name, command)
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
    socket: Option<(PathBuf, TempDir)>,
}

impl Editor {
    pub fn start(
        program: &Program,
        directory: &Path,
        id: usize,
        rows: u16,
        cols: u16,
    ) -> io::Result<Editor> {
        let mut command: Vec<OsString> = program.command.iter().map(OsString::from).collect();
        let socket = if program.neovim {
            let private = tempfile::Builder::new()
                .prefix("harbour-nvim")
                .permissions(Permissions::from_mode(0o700))
                .tempdir()?;
            let path = private.path().join("socket");
            command.push("--listen".into());
            command.push(path.clone().into());
            Some((path, private))
        } else {
            None
        };
        let session = Session::start(&command, directory, rows, cols)?;
        Ok(Editor {
            id,
            directory: directory.to_path_buf(),
            label: label(directory),
            session,
            socket,
        })
    }

    pub fn wants_ctrl_h(&self) -> bool {
        self.socket
            .as_ref()
            .and_then(|(path, _)| nvim::eval(path, LEFTMOST_IN_NORMAL_MODE))
            == Some(0)
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
