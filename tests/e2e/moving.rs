use std::fs;
use std::path::{Path, PathBuf};

use harbour::tui::{DIR_PICKER, SESSION_COMMAND};

use super::{AGENT, PICKER, Server};

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

fn opened(name: &str, home: &Path) -> Server {
    let server = Server::start(name, 160, 20);
    server.exec(home, &[(SESSION_COMMAND, AGENT), (DIR_PICKER, PICKER)]);
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
