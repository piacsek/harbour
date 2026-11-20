use std::path::Path;

use harbour::tui::SESSION_COMMAND;

use super::{ARGS_AGENT, Server, tools};

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
