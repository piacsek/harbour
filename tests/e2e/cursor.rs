use std::thread::sleep;
use std::time::{Duration, Instant};

use harbour::tui::SESSION_COMMAND;

use crate::{Server, focused_on};

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
fn quitting_gives_the_terminal_back_its_default_cursor_shape() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("cursor-quit", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, SHAPED_AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");
    server.send_keys(&["Enter", "5", "Enter"]);
    assert_eq!(wait_for_shape(&server, "bar"), "bar");

    server.send_keys(&["C-h", "C-/", "q", "y"]);

    server.wait_for_screen("harbour exited 0");
    assert_eq!(wait_for_shape(&server, "default"), "default");
}

#[test]
#[ignore]
fn the_terminal_hides_the_cursor_while_the_sidebar_has_focus() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("cursor-sidebar", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, SHAPED_AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");
    server.send_keys(&["Enter"]);
    server.wait_until("focus on claude", |screen| focused_on(screen, "claude"));
    let shown = wait_for_format(&server, "#{cursor_flag}", "1");

    server.send_keys(&["C-h"]);

    server.wait_until("focus on the sidebar", |screen| {
        focused_on(screen, "sessions")
    });
    assert_eq!(shown, "1");
    assert_eq!(wait_for_format(&server, "#{cursor_flag}", "0"), "0");
}

fn wait_for_shape(server: &Server, expected: &str) -> String {
    wait_for_format(server, "#{cursor_shape}", expected)
}

fn wait_for_format(server: &Server, format: &str, expected: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let output = server
            .tmux(&["display-message", "-p", "-t", "e2e", format])
            .output()
            .unwrap();
        let shape = String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string();
        if shape == expected || Instant::now() >= deadline {
            return shape;
        }
        sleep(Duration::from_millis(50));
    }
}
