use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::session::Session;

const PRINTING_TO: &str = r#"out=$1; shift; exec "$@" >"$out""#;

pub enum Picker {
    Command(Vec<String>),
}

pub struct Pick {
    session: Session,
    output: PathBuf,
    from: PathBuf,
    _scratch: TempDir,
}

pub enum Picked {
    Chosen(PathBuf),
}

impl Pick {
    pub fn open(picker: &Picker, from: &Path, rows: u16, cols: u16) -> io::Result<Pick> {
        let Picker::Command(command) = picker;
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
        self.session.exit()?;
        let printed = fs::read_to_string(&self.output).unwrap_or_default();
        let line = printed.lines().next().unwrap_or_default();
        Some(Picked::Chosen(self.from.join(line)))
    }
}
