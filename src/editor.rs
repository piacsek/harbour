use std::ffi::OsString;
use std::fs::Permissions;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tempfile::TempDir;

use crate::nvim;
use crate::session::Session;

const NORMAL_MODE: &str = "n";
const LEFTMOST: &str = "winnr('h') == winnr()";
const UNSAVED: &str = "len(filter(getbufinfo({'bufmodified': 1}), \
                       {_, b -> index(['', 'acwrite'], getbufvar(b.bufnr, '&buftype')) >= 0}))";

const SOCKET_PREFIX: &str = "harbour-nvim";
const SOCKET_RANDOM: usize = 6;
const LONGEST_SOCKET_PATH: usize = 103;

pub struct Program {
    name: String,
    command: Vec<OsString>,
    neovim: bool,
    patience: Duration,
}

impl Program {
    pub fn new(name: &str, command: Vec<String>) -> Program {
        Program {
            name: name.to_string(),
            command: command.into_iter().map(OsString::from).collect(),
            neovim: false,
            patience: nvim::PATIENCE,
        }
    }

    pub fn neovim(name: &str, command: Vec<String>) -> Program {
        Program {
            neovim: true,
            ..Program::new(name, command)
        }
    }

    pub fn named(program: OsString) -> Program {
        let name = Path::new(&program)
            .file_name()
            .unwrap_or(&program)
            .to_string_lossy()
            .into_owned();
        Program {
            neovim: name == "nvim",
            name,
            command: vec![program],
            patience: nvim::PATIENCE,
        }
    }

    pub fn with_patience(mut self, patience: Duration) -> Program {
        self.patience = patience;
        self
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
    patience: Duration,
}

impl Editor {
    pub fn start(
        program: &Program,
        directory: &Path,
        id: usize,
        rows: u16,
        cols: u16,
    ) -> io::Result<Editor> {
        let mut command = program.command.clone();
        let socket = if program.neovim {
            let private = tempfile::Builder::new()
                .prefix(SOCKET_PREFIX)
                .rand_bytes(SOCKET_RANDOM)
                .permissions(Permissions::from_mode(0o700))
                .tempdir_in(socket_base(&std::env::temp_dir()))?;
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
            patience: program.patience,
        })
    }

    pub fn wants_ctrl_h(&self) -> bool {
        match self.listening() {
            Listening::Blocking => true,
            Listening::Ready(_, _, mode) if mode != NORMAL_MODE => true,
            Listening::Ready(socket, deadline, _) => {
                nvim::eval(socket, LEFTMOST, deadline) == Some(0)
            }
            Listening::No => false,
        }
    }

    pub fn changes(&self) -> Changes {
        let Listening::Ready(socket, deadline, _) = self.listening() else {
            return Changes::Unknown;
        };
        match nvim::eval(socket, UNSAVED, deadline) {
            Some(0) => Changes::Saved,
            Some(_) => Changes::Unsaved,
            None => Changes::Unknown,
        }
    }

    fn listening(&self) -> Listening<'_> {
        let Some((socket, _)) = &self.socket else {
            return Listening::No;
        };
        let deadline = Instant::now() + self.patience;
        match nvim::mode(socket, deadline) {
            Some(mode) if mode.blocking => Listening::Blocking,
            Some(mode) => Listening::Ready(socket, deadline, mode.mode),
            None => Listening::No,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Changes {
    Saved,
    Unknown,
    Unsaved,
}

enum Listening<'a> {
    No,
    Blocking,
    Ready(&'a Path, Instant, String),
}

pub fn socket_base(temporary: &Path) -> PathBuf {
    let socket = SOCKET_PREFIX.len() + SOCKET_RANDOM + "/socket".len() + 1;
    if temporary.as_os_str().len() + socket <= LONGEST_SOCKET_PATH {
        temporary.to_path_buf()
    } else {
        PathBuf::from("/tmp")
    }
}

pub fn root(directory: &Path) -> PathBuf {
    if !directory.is_dir() {
        return directory.to_path_buf();
    }
    directory
        .ancestors()
        .find(|folder| folder.join(".git").symlink_metadata().is_ok())
        .unwrap_or(directory)
        .to_path_buf()
}

fn label(directory: &Path) -> String {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    match home.and_then(|home| directory.strip_prefix(home).ok()) {
        Some(below) if below.as_os_str().is_empty() => "~".to_string(),
        Some(below) => format!("~/{}", below.display()),
        None => directory.display().to_string(),
    }
}
