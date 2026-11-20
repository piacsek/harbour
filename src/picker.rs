use std::env;
use std::ffi::{OsStr, OsString};
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
  [ -z "$here" ] || printf '%s\n' "$here"
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
    Fuzzy {
        fzf: Vec<String>,
        list: Vec<String>,
        here_first: bool,
    },
}

impl Picker {
    pub fn program(&self) -> &str {
        match self {
            Picker::Command(command) => &command[0],
            Picker::Fuzzy { fzf, .. } => &fzf[0],
        }
    }

    pub fn command(&self, from: &Path, output: &Path) -> Vec<OsString> {
        match self {
            Picker::Command(command) => wrapping(PRINTING_TO, output, &[], command),
            Picker::Fuzzy {
                fzf,
                list,
                here_first,
            } => {
                let here = if *here_first {
                    from.as_os_str().to_owned()
                } else {
                    OsString::new()
                };
                let before = [here, fzf.len().to_string().into()];
                let mut command = fzf.clone();
                command.extend(list.iter().cloned());
                wrapping(LISTING_INTO, output, &before, &command)
            }
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
        match (picker, startable(picker.program(), from)) {
            (Picker::Command(_), Err(error)) => return Err(error),
            (Picker::Fuzzy { .. }, Err(_)) => return Ok(None),
            (_, Ok(())) => {}
        }
        let wrapped = picker.command(from, &output);
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
        Some(Picked::Chosen(chosen.canonicalize().unwrap_or(chosen)))
    }
}

fn wrapping(script: &str, output: &Path, before: &[OsString], command: &[String]) -> Vec<OsString> {
    let mut wrapped: Vec<OsString> = vec![
        "/bin/sh".into(),
        "-c".into(),
        script.into(),
        "harbour-picker".into(),
        output.as_os_str().to_owned(),
    ];
    wrapped.extend(before.iter().cloned());
    wrapped.extend(command.iter().map(OsString::from));
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
