use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub fn real_nvim(name: &str) -> Option<PathBuf> {
    real_nvim_in(name, std::env::var_os("PATH"), std::env::var_os("CI"))
}

pub fn real_nvim_in(name: &str, path: Option<OsString>, ci: Option<OsString>) -> Option<PathBuf> {
    let found = path.and_then(|path| {
        std::env::split_paths(&path)
            .map(|directory| directory.join("nvim"))
            .find(|candidate| is_executable(candidate))
    });
    if found.is_none() {
        assert!(
            ci.is_none_or(|ci| ci.is_empty()),
            "{name}: no nvim on PATH in CI, which must run the real-Neovim tests"
        );
        eprintln!("skipping {name}: no nvim on PATH, so the real-Neovim test has nothing to run");
    }
    found
}

pub fn is_executable(candidate: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    candidate
        .metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}
