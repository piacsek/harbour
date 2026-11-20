use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use harbour::tui::SESSION_COMMAND;

use super::{AGENT, Server, tools};

const FIXTURE_EDITOR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/editor.sh");

#[test]
#[ignore]
fn harbour_edit_starting_a_neovim_that_never_answers_closes_it_and_exits_three() {
    let home = tempfile::tempdir().unwrap();
    let nvim = Path::new(&tools(home.path())).join("nvim");
    std::os::unix::fs::symlink(FIXTURE_EDITOR, nvim).unwrap();
    let server = Server::start("edit-starts", 320, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    let started = server.wait_for_screen("acme agent in acme-api");
    assert!(started.contains("acme agent in acme-api"), "{started}");
    let file = source_file(home.path(), "main.rs");

    let output = handed(home.path(), None, &file);

    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "harbour: Neovim did not start in time; harbour closed it\n"
    );
    assert_eq!(output.status.code(), Some(3));
}

#[test]
#[ignore]
fn the_demo_never_answers_harbour_edit() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("edit-demo", 160, 26);
    server.run(home.path(), &[], &["--demo"]);
    let screen = server.wait_for_screen("acme agent");
    assert!(screen.contains("acme agent"), "{screen}");
    let file = source_file(home.path(), "main.rs");

    let output = handed(home.path(), None, &file);

    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "harbour: no harbour is running\n"
    );
}

pub fn source_file(home: &Path, name: &str) -> PathBuf {
    let folder = home.join("acme-api/src");
    std::fs::create_dir_all(&folder).unwrap();
    let file = folder.canonicalize().unwrap().join(name);
    std::fs::write(&file, "acme handed line\n").unwrap();
    file
}

pub fn handed(home: &Path, state: Option<&str>, file: &Path) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_harbour"));
    command.arg("edit").arg(file).env("HOME", home);
    match state {
        Some(state) => command.env("XDG_STATE_HOME", state),
        None => command.env_remove("XDG_STATE_HOME"),
    };
    command.output().unwrap()
}
