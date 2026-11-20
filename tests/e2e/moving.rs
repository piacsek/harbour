use std::fs;
use std::path::{Path, PathBuf};

use harbour::tui::{DIR_PICKER, EDITOR, SESSION_COMMAND};

use super::{AGENT, PICKER, REGISTERED_AGENT, Server};

const TITLED_AGENT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/titled-agent.sh"
);
const FIXTURE_EDITOR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/editor.sh");

#[test]
#[ignore]
fn a_session_inside_a_repository_moves_the_pane_path_to_its_root() {
    let home = tempfile::tempdir().unwrap();
    let web = repository(home.path(), "acme-web");
    let server = opened("moving-root", home.path());

    start(&server, &web.join("src"), "src 1");

    let root = web.canonicalize().unwrap();
    assert_eq!(server.wait_for_path(&root), root);
}

#[test]
#[ignore]
fn moving_the_selection_moves_the_pane_path() {
    let home = tempfile::tempdir().unwrap();
    let web = repository(home.path(), "acme-web");
    let docs = repository(home.path(), "acme-docs");
    let server = opened("moving-selection", home.path());
    start(&server, &web, "acme-web 1");
    start(&server, &docs, "acme-docs 2");
    let docs = docs.canonicalize().unwrap();
    assert_eq!(server.wait_for_path(&docs), docs);

    server.send_keys(&["k"]);

    let web = web.canonicalize().unwrap();
    assert_eq!(server.wait_for_path(&web), web);
}

#[test]
#[ignore]
fn a_session_outside_any_repository_moves_the_pane_path_to_its_directory() {
    let home = tempfile::tempdir().unwrap();
    let notes = home.path().join("acme-notes");
    fs::create_dir_all(&notes).unwrap();
    let server = opened("moving-plain", home.path());

    start(&server, &notes, "acme-notes 1");

    let notes = notes.canonicalize().unwrap();
    assert_eq!(server.wait_for_path(&notes), notes);
}

#[test]
#[ignore]
fn a_directory_that_is_gone_moves_the_pane_path_back_where_harbour_was_opened() {
    let home = tempfile::tempdir().unwrap();
    let gone = home.path().join("acme-gone");
    fs::create_dir_all(&gone).unwrap();
    let server = opened("moving-gone", home.path());
    let opened_at = home.path().join("acme-api").canonicalize().unwrap();
    start(&server, &gone, "acme-gone 1");
    let gone = gone.canonicalize().unwrap();
    assert_eq!(server.wait_for_path(&gone), gone);

    fs::remove_dir_all(&gone).unwrap();

    assert_eq!(server.wait_for_path(&opened_at), opened_at);
}

#[test]
#[ignore]
fn removing_the_last_session_moves_the_pane_path_back_where_harbour_was_opened() {
    let home = tempfile::tempdir().unwrap();
    let web = repository(home.path(), "acme-web");
    let server = opened("moving-removed", home.path());
    let opened_at = home.path().join("acme-api").canonicalize().unwrap();
    start(&server, &web, "acme-web 1");
    let web = web.canonicalize().unwrap();
    assert_eq!(server.wait_for_path(&web), web);

    server.send_keys(&["x", "y"]);
    let screen = server.wait_for_screen("exited");
    assert!(screen.contains("exited"), "{screen}");
    server.send_keys(&["x"]);

    assert_eq!(server.wait_for_path(&opened_at), opened_at);
}

#[test]
#[ignore]
fn a_relative_claude_config_dir_is_read_from_where_harbour_was_opened() {
    let home = tempfile::tempdir().unwrap();
    let api = repository(home.path(), "acme-api");
    let server = Server::start("moving-config", 160, 20);
    server.exec(
        home.path(),
        "cd src &&",
        &[
            (SESSION_COMMAND, REGISTERED_AGENT),
            ("CLAUDE_CONFIG_DIR", "acme-config"),
        ],
        &[],
    );
    let screen = server.wait_for_screen("press n to start one");
    assert!(screen.contains("press n to start one"), "{screen}");

    server.send_keys(&["n"]);

    let root = api.canonicalize().unwrap();
    assert_eq!(server.wait_for_path(&root), root);
    let screen = server.wait_for_screen("● src 1");
    assert!(screen.contains("● src 1"), "{screen}");
}

#[test]
#[ignore]
fn a_relative_claude_config_dir_holds_the_transcripts_once_harbour_moved() {
    let home = tempfile::tempdir().unwrap();
    let api = repository(home.path(), "acme-api");
    let server = Server::start("moving-transcripts", 160, 20);
    server.exec(
        home.path(),
        "cd src &&",
        &[
            (SESSION_COMMAND, TITLED_AGENT),
            ("CLAUDE_CONFIG_DIR", "acme-config"),
        ],
        &[],
    );
    let screen = server.wait_for_screen("press n to start one");
    assert!(screen.contains("press n to start one"), "{screen}");

    server.send_keys(&["n"]);

    let root = api.canonicalize().unwrap();
    assert_eq!(server.wait_for_path(&root), root);
    let screen = server.wait_for_screen("Fix the flaky login test");
    assert!(screen.contains("● Fix the flaky login test"), "{screen}");
}

#[test]
#[ignore]
fn a_relative_tmpdir_still_holds_the_picker_once_harbour_moved() {
    let home = tempfile::tempdir().unwrap();
    let web = repository(home.path(), "acme-web");
    let docs = repository(home.path(), "acme-docs");
    let server = Server::start("moving-tmp-picker", 160, 20);
    fs::create_dir_all(home.path().join("acme-api/acme-tmp")).unwrap();
    server.exec(
        home.path(),
        "",
        &[
            (SESSION_COMMAND, AGENT),
            (DIR_PICKER, PICKER),
            ("TMPDIR", "acme-tmp"),
        ],
        &[],
    );
    let screen = server.wait_for_screen("press n to start one");
    assert!(screen.contains("press n to start one"), "{screen}");
    start(&server, &web, "acme-web 1");
    let web = web.canonicalize().unwrap();
    assert_eq!(server.wait_for_path(&web), web);

    start(&server, &docs, "acme-docs 2");

    let screen = server.screen();
    assert!(!screen.contains("Could not"), "{screen}");
}

#[test]
#[ignore]
fn a_relative_tmpdir_still_holds_the_neovim_socket_once_harbour_moved() {
    let home = tempfile::tempdir().unwrap();
    let web = repository(home.path(), "acme-web");
    let tools = home.path().join("acme-tools");
    fs::create_dir_all(&tools).unwrap();
    let nvim = tools.join("nvim");
    std::os::unix::fs::symlink(FIXTURE_EDITOR, &nvim).unwrap();
    let server = Server::start("moving-tmp-editor", 160, 20);
    fs::create_dir_all(home.path().join("acme-api/acme-tmp")).unwrap();
    server.exec(
        home.path(),
        "",
        &[
            (SESSION_COMMAND, AGENT),
            (DIR_PICKER, PICKER),
            (EDITOR, nvim.to_str().unwrap()),
            ("TMPDIR", "acme-tmp"),
        ],
        &[],
    );
    let screen = server.wait_for_screen("press n to start one");
    assert!(screen.contains("press n to start one"), "{screen}");
    start(&server, &web, "acme-web 1");
    let web = web.canonicalize().unwrap();
    assert_eq!(server.wait_for_path(&web), web);

    server.send_keys(&["Enter", "C-l"]);

    let screen = server.wait_for_screen("acme editor in acme-web");
    assert!(screen.contains("first arg:[--listen]"), "{screen}");
}

fn opened(name: &str, home: &Path) -> Server {
    let server = Server::start(name, 160, 20);
    server.exec(
        home,
        "",
        &[(SESSION_COMMAND, AGENT), (DIR_PICKER, PICKER)],
        &[],
    );
    let screen = server.wait_for_screen("press n to start one");
    assert!(screen.contains("press n to start one"), "{screen}");
    server
}

fn start(server: &Server, directory: &Path, row: &str) {
    server.send_keys(&["n"]);
    let screen = server.wait_for_screen("acme picker: type a directory");
    assert!(screen.contains("acme picker: type a directory"), "{screen}");
    server.send_keys(&[&directory.display().to_string(), "Enter"]);
    let screen = server.wait_for_screen(row);
    assert!(screen.contains(row), "{screen}");
    server.send_keys(&["C-h"]);
}

fn repository(home: &Path, name: &str) -> PathBuf {
    let root = home.join(name);
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    root
}
