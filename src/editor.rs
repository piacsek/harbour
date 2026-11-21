use std::ffi::OsString;
use std::fs::Permissions;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use tempfile::TempDir;

use crate::log::Log;
use crate::nvim;
use crate::repo;
use crate::session::{Role, Session};

const NORMAL_MODES: [&str; 2] = ["n", "nt"];
const LEFTMOST: &str = "winnr('h') == winnr()";
const RIGHTMOST: &str = "winnr('l') == winnr()";
const UNSAVED: &str = "len(filter(getbufinfo({'bufmodified': 1}), \
                       {_, b -> index(['', 'acwrite'], getbufvar(b.bufnr, '&buftype')) >= 0}))";

const RETRY_OPEN: Duration = Duration::from_millis(10);
const WAITING: &str = "Neovim is waiting for a key; answer it and try again";

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

    pub fn command(command: Vec<OsString>) -> Program {
        let program = Path::new(&command[0]);
        let name = program
            .file_name()
            .unwrap_or(program.as_os_str())
            .to_string_lossy()
            .into_owned();
        Program {
            neovim: name == "nvim",
            name,
            command,
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

    pub fn is_neovim(&self) -> bool {
        self.neovim
    }
}

pub enum Unopened {
    Refused(String),
    Silent,
    Exited,
}

pub struct Editor {
    pub id: usize,
    pub directory: PathBuf,
    pub label: String,
    pub session: Session,
    socket: Option<(PathBuf, TempDir)>,
    patience: Duration,
    log: Log,
}

impl Editor {
    pub fn start(
        program: &Program,
        directory: &Path,
        temporary: &Path,
        id: usize,
        rows: u16,
        cols: u16,
        file: Option<&Path>,
    ) -> io::Result<Editor> {
        let mut command = program.command.clone();
        let private = program.neovim.then(|| {
            tempfile::Builder::new()
                .prefix(SOCKET_PREFIX)
                .rand_bytes(SOCKET_RANDOM)
                .permissions(Permissions::from_mode(0o700))
                .tempdir_in(socket_base(temporary))
                .ok()
        });
        let socket = private.flatten().map(|private| {
            let path = private.path().join("socket");
            command.push("--listen".into());
            command.push(path.clone().into());
            (path, private)
        });
        if let Some(file) = file {
            command.push("--".into());
            command.push(file.into());
        }
        let session = Session::start_as(Role::Editor, &command, directory, rows, cols)?;
        Ok(Editor {
            id,
            directory: directory.to_path_buf(),
            label: label(directory),
            session,
            socket,
            patience: program.patience,
            log: Log::default(),
        })
    }

    pub fn with_log(mut self, log: Log) -> Editor {
        self.log = log;
        self
    }

    pub fn wants_ctrl_h(&self) -> bool {
        self.keeps(LEFTMOST)
    }

    pub fn wants_ctrl_l(&self) -> bool {
        self.keeps(RIGHTMOST)
    }

    fn keeps(&self, edge: &str) -> bool {
        match self.listening() {
            Listening::Blocking => true,
            Listening::Ready(_, _, mode) if !NORMAL_MODES.contains(&mode.as_str()) => true,
            Listening::Ready(socket, deadline, _) => {
                self.evaluated(socket, edge, deadline) == Some(0)
            }
            Listening::No => false,
        }
    }

    pub fn open(&mut self, file: &Path, deadline: Instant) -> Result<(), Unopened> {
        let (socket, _) = self.socket.as_ref().ok_or(Unopened::Silent)?;
        let asked = Instant::now();
        let mut tries = 0;
        let mode = loop {
            tries += 1;
            match nvim::mode(socket, deadline) {
                Some(mode) => break Ok(mode),
                None if self.session.exit().is_some() => break Err(Unopened::Exited),
                None if Instant::now() + RETRY_OPEN < deadline => thread::sleep(RETRY_OPEN),
                None => break Err(Unopened::Silent),
            }
        };
        called(&self.log, self.id, "mode", asked, mode.is_ok(), tries);
        let mode = mode?;
        if mode.waiting() {
            return Err(Unopened::Refused(WAITING.to_string()));
        }
        let asked = Instant::now();
        let edited = nvim::edit(socket, file, deadline);
        called(&self.log, self.id, "edit", asked, edited.is_some(), 1);
        match edited {
            Some(nvim::Edited::Done) => Ok(()),
            Some(nvim::Edited::Refused(error)) => {
                Err(Unopened::Refused(format!("Neovim refused: {error}")))
            }
            Some(nvim::Edited::Waiting) => Err(Unopened::Refused(WAITING.to_string())),
            None => Err(Unopened::Silent),
        }
    }

    pub fn changes(&self) -> Changes {
        let Listening::Ready(socket, deadline, _) = self.listening() else {
            return Changes::Unknown;
        };
        match self.evaluated(socket, UNSAVED, deadline) {
            Some(0) => Changes::Saved,
            Some(_) => Changes::Unsaved,
            None => Changes::Unknown,
        }
    }

    fn listening(&self) -> Listening<'_> {
        let Some((socket, _)) = &self.socket else {
            return Listening::No;
        };
        let asked = Instant::now();
        let deadline = asked + self.patience;
        let mode = nvim::mode(socket, deadline);
        called(&self.log, self.id, "mode", asked, mode.is_some(), 1);
        match mode {
            Some(mode) if mode.blocking => Listening::Blocking,
            Some(mode) => Listening::Ready(socket, deadline, mode.mode),
            None => Listening::No,
        }
    }

    fn evaluated(&self, socket: &Path, expression: &str, deadline: Instant) -> Option<i64> {
        let asked = Instant::now();
        let value = nvim::eval(socket, expression, deadline);
        called(&self.log, self.id, "eval", asked, value.is_some(), 1);
        value
    }
}

fn called(log: &Log, editor: usize, call: &str, asked: Instant, answered: bool, tries: usize) {
    log.write(|| {
        let tries = if tries > 1 {
            format!(" tries={tries}")
        } else {
            String::new()
        };
        format!(
            "neovim call={call} editor={editor} took={}ms answered={}{tries}",
            asked.elapsed().as_millis(),
            if answered { "yes" } else { "no" }
        )
    });
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

pub fn root(directory: &Path, home: Option<&Path>) -> PathBuf {
    if !directory.is_dir() {
        return directory.to_path_buf();
    }
    repo::root(directory, home)
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
