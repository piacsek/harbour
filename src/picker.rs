use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::session::Session;

const PRINTING_TO: &str = r#"out=$1; shift; "$@" >"$out""#;
const LISTING_INTO: &str = r#"out=$1; here=$2; n=$3; shift 3
listed="${out%/*}/listed"
mkfifo "$listed" || exit 126
(
  shift "$n"
  printf '%s\n' "$here"
  "$@" </dev/null 2>/dev/null | grep -vxF -e "$here" -e ''
) >"$listed" &
for argument do
  shift
  [ "$n" -gt 0 ] && set -- "$@" "$argument"
  n=$((n - 1))
done
"$@" <"$listed" >"$out""#;

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
    program: String,
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
                wrapping(PRINTING_TO, &output, &[], command)
            }
            Picker::Fuzzy { fzf, list } => {
                if startable(picker.program(), from).is_err() {
                    return Ok(None);
                }
                let before = [from.to_string_lossy().into_owned(), fzf.len().to_string()];
                let mut command = fzf.clone();
                command.extend(list.iter().cloned());
                wrapping(LISTING_INTO, &output, &before, &command)
            }
        };
        let session = Session::start(&wrapped, from, rows, cols)?;
        Ok(Some(Pick {
            session,
            program: picker.program().to_string(),
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
        let status = self.session.exit()?;
        if let code @ (126 | 127) = status.exit_code() {
            return Some(Picked::Refused(format!(
                "Could not start {}: it exited with {code}",
                self.program
            )));
        }
        if !status.success() {
            return Some(Picked::Cancelled);
        }
        let printed = fs::read(&self.output).unwrap_or_default();
        let line = printed
            .split(|byte| *byte == b'\n')
            .next()
            .unwrap_or_default();
        if line.is_empty() {
            return Some(Picked::Cancelled);
        }
        let chosen = self.from.join(OsStr::from_bytes(line));
        if chosen.is_dir() {
            Some(Picked::Chosen(chosen))
        } else {
            Some(Picked::Refused(format!(
                "Not a directory: {}",
                String::from_utf8_lossy(line)
            )))
        }
    }
}

fn wrapping(script: &str, output: &Path, before: &[String], command: &[String]) -> Vec<String> {
    let mut wrapped = vec![
        "/bin/sh".to_string(),
        "-c".to_string(),
        script.to_string(),
        "harbour-picker".to_string(),
        output.to_string_lossy().into_owned(),
    ];
    wrapped.extend(before.iter().cloned());
    wrapped.extend(command.iter().cloned());
    wrapped
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
