use std::fs;
use std::path::Path;
use std::process::{Command, Output};

#[test]
fn a_directory_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    fs::create_dir(scratch.path().join("acme-api")).unwrap();

    let output = edit(scratch.path(), "acme-api");

    assert_eq!(stderr(&output), "harbour: acme-api is a directory\n");
}

#[test]
fn a_directory_exits_one() {
    let scratch = tempfile::tempdir().unwrap();
    fs::create_dir(scratch.path().join("acme-api")).unwrap();

    assert_eq!(edit(scratch.path(), "acme-api").status.code(), Some(1));
}

#[test]
fn a_file_in_a_missing_folder_is_refused() {
    let scratch = tempfile::tempdir().unwrap();

    let output = edit(scratch.path(), "acme-api/src/main.rs");

    assert_eq!(
        stderr(&output),
        "harbour: acme-api/src/main.rs: No such file or directory\n"
    );
}

#[test]
fn a_file_in_a_missing_folder_exits_one() {
    let scratch = tempfile::tempdir().unwrap();

    let output = edit(scratch.path(), "acme-api/src/main.rs");

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_path_longer_than_4096_bytes_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    let long = format!("{}/main.rs", "a".repeat(4096));

    let output = edit(scratch.path(), &long);

    assert_eq!(
        stderr(&output),
        "harbour: the path is longer than 4096 bytes\n"
    );
}

#[test]
fn with_no_harbour_running_a_file_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("main.rs"), "").unwrap();

    let output = edit(scratch.path(), "main.rs");

    assert_eq!(stderr(&output), "harbour: no harbour is running\n");
}

#[test]
fn with_no_harbour_running_a_file_exits_one() {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("main.rs"), "").unwrap();

    assert_eq!(edit(scratch.path(), "main.rs").status.code(), Some(1));
}

#[test]
fn with_no_harbour_running_nothing_is_printed_on_stdout() {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(scratch.path().join("main.rs"), "").unwrap();

    assert!(edit(scratch.path(), "main.rs").stdout.is_empty());
}

fn edit(scratch: &Path, file: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_harbour"))
        .args(["edit", file])
        .env("HOME", scratch)
        .env("XDG_STATE_HOME", scratch.join("state"))
        .current_dir(scratch)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
