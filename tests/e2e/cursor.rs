use std::thread::sleep;
use std::time::{Duration, Instant};

use harbour::tui::SESSION_COMMAND;

use crate::Server;

const SHAPED_AGENT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/shaped-agent.sh"
);

#[test]
#[ignore]
fn the_terminal_takes_the_cursor_shape_of_the_shown_session() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("cursor-shape", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, SHAPED_AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");

    server.send_keys(&["Enter", "6", "Enter"]);

    server.wait_for_screen("acme agent shape 6");
    assert_eq!(wait_for_shape(&server, "bar"), "bar");
}

#[test]
#[ignore]
fn another_session_shown_gives_the_terminal_its_shape() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("cursor-switch", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, SHAPED_AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");
    server.send_keys(&["Enter", "4", "Enter"]);
    wait_for_shape(&server, "underline");

    server.send_keys(&["C-h", "n"]);

    server.wait_for_screen("acme-api 2");
    assert_eq!(wait_for_shape(&server, "default"), "default");
    server.send_keys(&["k"]);
    assert_eq!(wait_for_shape(&server, "underline"), "underline");
}

#[test]
#[ignore]
fn quitting_gives_the_terminal_back_its_default_cursor_shape() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("cursor-quit", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, SHAPED_AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");
    server.send_keys(&["Enter", "5", "Enter"]);
    wait_for_shape(&server, "bar");

    server.send_keys(&["C-h", "q", "y"]);

    server.wait_for_screen("harbour exited 0");
    assert_eq!(wait_for_shape(&server, "default"), "default");
}

fn wait_for_shape(server: &Server, expected: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = server
            .tmux(&["display-message", "-p", "-t", "e2e", "#{cursor_shape}"])
            .output()
            .unwrap();
        let shape = String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string();
        if shape == expected {
            return shape;
        }
        assert!(
            Instant::now() < deadline,
            "waited 10 s for the cursor shape {expected}, last shape: {shape}"
        );
        sleep(Duration::from_millis(50));
    }
}
