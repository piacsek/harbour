use std::path::Path;

use harbour::tui::{EDITOR, SESSION_COMMAND};

use super::{AGENT, ARGS_AGENT, Server, tools};

const FIXTURE_EDITOR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/editor.sh");

#[test]
#[ignore]
fn the_default_claude_is_started_with_a_session_id() {
    let home = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(ARGS_AGENT, Path::new(&tools(home.path())).join("claude")).unwrap();
    let server = Server::start("session-id", 160, 20);
    server.run(home.path(), &[], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme agent args:");
    assert!(
        screen.contains("acme agent args:[--session-id "),
        "{screen}"
    );
}

#[test]
#[ignore]
fn a_session_command_is_started_without_a_session_id() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("no-session-id", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, ARGS_AGENT)], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme agent args:");
    assert!(screen.contains("acme agent args:[]"), "{screen}");
}

#[test]
#[ignore]
fn c_l_opens_the_editor_harbour_editor_names_without_arguments() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("named-editor", 160, 20);

    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (EDITOR, FIXTURE_EDITOR)],
        &[],
    );

    let screen = editing(&server);
    assert!(
        screen.contains("acme editor in acme-api first arg:[]"),
        "{screen}"
    );
    assert!(screen.contains("claude | editor.sh: "), "{screen}");
}

#[test]
#[ignore]
fn without_harbour_editor_c_l_opens_nvim_listening_on_a_socket() {
    let home = tempfile::tempdir().unwrap();
    let nvim = Path::new(&tools(home.path())).join("nvim");
    std::os::unix::fs::symlink(FIXTURE_EDITOR, nvim).unwrap();
    let server = Server::start("default-editor", 160, 20);

    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);

    let screen = editing(&server);
    assert!(screen.contains("first arg:[--listen]"), "{screen}");
    assert!(screen.contains("claude | nvim: "), "{screen}");
}

#[test]
#[ignore]
fn an_empty_editor_setting_counts_as_unset() {
    let home = tempfile::tempdir().unwrap();
    let nvim = Path::new(&tools(home.path())).join("nvim");
    std::os::unix::fs::symlink(FIXTURE_EDITOR, nvim).unwrap();
    let server = Server::start("empty-editor", 160, 20);

    server.run(home.path(), &[(SESSION_COMMAND, AGENT), (EDITOR, "")], &[]);

    let screen = editing(&server);
    assert!(screen.contains("first arg:[--listen]"), "{screen}");
}

#[test]
#[ignore]
fn an_editor_whose_file_name_is_nvim_listens_on_a_socket() {
    let home = tempfile::tempdir().unwrap();
    let elsewhere = home.path().join("acme-tools");
    std::fs::create_dir(&elsewhere).unwrap();
    let nvim = elsewhere.join("nvim");
    std::os::unix::fs::symlink(FIXTURE_EDITOR, &nvim).unwrap();
    let server = Server::start("nvim-path", 160, 20);

    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (EDITOR, nvim.to_str().unwrap())],
        &[],
    );

    let screen = editing(&server);
    assert!(screen.contains("first arg:[--listen]"), "{screen}");
}

fn editing(server: &Server) -> String {
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent in acme-api");
    server.send_keys(&["Enter", "C-l"]);
    server.wait_for_screen("acme editor in acme-api")
}
