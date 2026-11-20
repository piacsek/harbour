use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::session::Session;

const PRINTING_TO: &str = r#"out=$1; shift; exec "$@" >"$out""#;

pub enum Picker {
    Command(Vec<String>),
}

impl Picker {
    pub fn program(&self) -> &str {
        let Picker::Command(command) = self;
        &command[0]
    }
}

pub struct Pick {
    session: Session,
    output: PathBuf,
    from: PathBuf,
    _scratch: TempDir,
}

pub enum Picked {
    Cancelled,
    Chosen(PathBuf),
    Refused(String),
}

impl Pick {
    pub fn open(picker: &Picker, from: &Path, rows: u16, cols: u16) -> io::Result<Pick> {
        let Picker::Command(command) = picker;
        startable(picker.program(), from)?;
        let scratch = tempfile::Builder::new().prefix("harbour-pick").tempdir()?;
        let output = scratch.path().join("picked");
        let mut wrapped = vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            PRINTING_TO.to_string(),
            "harbour-picker".to_string(),
            output.to_string_lossy().into_owned(),
        ];
        wrapped.extend(command.iter().cloned());
        let session = Session::start(&wrapped, from, rows, cols)?;
        Ok(Pick {
            session,
            output,
            from: from.to_path_buf(),
            _scratch: scratch,
        })
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.session.resize(rows, cols);
    }

    pub fn picked(&mut self) -> Option<Picked> {
        if !self.session.exit()?.success() {
            return Some(Picked::Cancelled);
        }
        let printed = fs::read_to_string(&self.output).unwrap_or_default();
        match printed.lines().next() {
            None | Some("") => Some(Picked::Cancelled),
            Some(line) if self.from.join(line).is_dir() => {
                Some(Picked::Chosen(self.from.join(line)))
            }
            Some(line) => Some(Picked::Refused(format!("Not a directory: {line}"))),
        }
    }
}

fn startable(program: &str, from: &Path) -> io::Result<()> {
    if program.contains('/') {
        return if executable(&from.join(program)) {
            Ok(())
        } else {
            Err(io::Error::other("not an executable file"))
        };
    }
    let path = env::var_os("PATH").unwrap_or_default();
    if env::split_paths(&path).any(|directory| executable(&directory.join(program))) {
        Ok(())
    } else {
        Err(io::Error::other("is it installed and on PATH?"))
    }
}

fn executable(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}
