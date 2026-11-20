use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::TempDir;

use crate::session::Session;

const PRINTING_TO: &str = r#"out=$1; shift; exec "$@" >"$out""#;
const READING_AND_PRINTING_TO: &str = r#"list=$1; out=$2; shift 2; exec "$@" <"$list" >"$out""#;

pub enum Picker {
    Command(Vec<String>),
    Fuzzy { fzf: Vec<String>, list: Vec<String> },
}

impl Picker {
    pub fn program(&self) -> &str {
        match self {
            Picker::Command(command) => &command[0],
            Picker::Fuzzy { fzf, .. } => &fzf[0],
        }
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
    pub fn open(picker: &Picker, from: &Path, rows: u16, cols: u16) -> io::Result<Option<Pick>> {
        let scratch = tempfile::Builder::new().prefix("harbour-pick").tempdir()?;
        let output = scratch.path().join("picked");
        let wrapped = match picker {
            Picker::Command(command) => {
                startable(picker.program(), from)?;
                wrapping(PRINTING_TO, &[], &output, command)
            }
            Picker::Fuzzy { fzf, list } => {
                if startable(picker.program(), from).is_err() {
                    return Ok(None);
                }
                let listed = scratch.path().join("listed");
                fs::write(&listed, listing(list, from))?;
                wrapping(READING_AND_PRINTING_TO, &[&listed], &output, fzf)
            }
        };
        let session = Session::start(&wrapped, from, rows, cols)?;
        Ok(Some(Pick {
            session,
            output,
            from: from.to_path_buf(),
            _scratch: scratch,
        }))
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

fn wrapping(script: &str, before: &[&Path], output: &Path, command: &[String]) -> Vec<String> {
    let mut wrapped = vec![
        "/bin/sh".to_string(),
        "-c".to_string(),
        script.to_string(),
        "harbour-picker".to_string(),
    ];
    wrapped.extend(
        before
            .iter()
            .map(|path| path.to_string_lossy().into_owned()),
    );
    wrapped.push(output.to_string_lossy().into_owned());
    wrapped.extend(command.iter().cloned());
    wrapped
}

fn listing(list: &[String], from: &Path) -> String {
    let here = from.to_string_lossy();
    let ranked = Command::new(&list[0])
        .args(&list[1..])
        .current_dir(from)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
        .unwrap_or_default();
    let mut listed = format!("{here}\n");
    for line in ranked
        .lines()
        .filter(|line| !line.is_empty() && *line != here)
    {
        listed.push_str(line);
        listed.push('\n');
    }
    listed
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
