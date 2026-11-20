use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const LONGEST_PATH: usize = 4096;

pub fn edit(argument: &OsStr) -> Result<(), String> {
    resolve(argument)?;
    Err("no harbour is running".to_string())
}

fn resolve(argument: &OsStr) -> Result<PathBuf, String> {
    let shown = Path::new(argument).display();
    if argument.as_bytes().len() > LONGEST_PATH {
        return Err(format!("the path is longer than {LONGEST_PATH} bytes"));
    }
    let given = Path::new(argument);
    let resolved = match given.canonicalize() {
        Ok(path) if path.is_dir() => return Err(format!("{shown} is a directory")),
        Ok(path) => path,
        Err(_) => {
            let missing = |error: std::io::Error| format!("{shown}: {}", plain(&error));
            let name = given
                .file_name()
                .ok_or_else(|| format!("{shown} is a directory"))?;
            let folder = match given.parent() {
                Some(parent) if !parent.as_os_str().is_empty() => parent,
                _ => Path::new("."),
            };
            folder.canonicalize().map_err(missing)?.join(name)
        }
    };
    if resolved.as_os_str().as_bytes().len() > LONGEST_PATH {
        return Err(format!("the path is longer than {LONGEST_PATH} bytes"));
    }
    Ok(resolved)
}

fn plain(error: &std::io::Error) -> String {
    let text = error.to_string();
    match text.find(" (os error") {
        Some(end) => text[..end].to_string(),
        None => text,
    }
}
