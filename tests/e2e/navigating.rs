use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use harbour::tui::SESSION_COMMAND;

use super::{AGENT, Server};

#[test]
#[ignore]
fn ctrl_j_in_the_sidebar_moves_tmux_to_the_pane_below_harbour() {
    let home = tempfile::tempdir().unwrap();
    let server = opened("navigating-sidebar", home.path());
    bind(&server);
    let harbour = active(&server);
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent in acme-api");
    let below = split(&server, &harbour, "-v");

    send(&server, &harbour, "C-j");

    assert_eq!(wait_for_active(&server, &below), below);
}

#[test]
#[ignore]
fn ctrl_j_in_claude_code_with_nothing_below_reaches_it_and_ctrl_k_moves_above() {
    let home = tempfile::tempdir().unwrap();
    let server = opened("navigating-claude", home.path());
    bind(&server);
    let harbour = active(&server);
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent in acme-api");
    server.send_keys(&["Enter"]);
    let above = split(&server, &harbour, "-vb");

    send(&server, &harbour, "hello");
    send(&server, &harbour, "C-j");

    let screen = server.wait_for_screen("acme agent got: hello");
    assert!(screen.contains("acme agent got: hello"), "{screen}");
    assert_eq!(active(&server), harbour);
    send(&server, &harbour, "C-k");
    assert_eq!(wait_for_active(&server, &above), above);
}

#[test]
#[ignore]
fn ctrl_j_in_claude_code_reaches_it_when_tmux_binds_no_ctrl_j() {
    let home = tempfile::tempdir().unwrap();
    let server = opened("navigating-unbound", home.path());
    let harbour = active(&server);
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent in acme-api");
    server.send_keys(&["Enter"]);
    split(&server, &harbour, "-v");

    send(&server, &harbour, "hello");
    send(&server, &harbour, "C-j");

    let screen = server.wait_for_screen("acme agent got: hello");
    assert!(screen.contains("acme agent got: hello"), "{screen}");
    assert_eq!(active(&server), harbour);
}

fn bind(server: &Server) {
    for (key, flag) in [("C-j", "-D"), ("C-k", "-U")] {
        let status = server
            .tmux(&["bind-key", "-n", key, "select-pane", flag])
            .status()
            .unwrap();
        assert!(status.success(), "could not bind {key}");
    }
}

fn opened(name: &str, home: &Path) -> Server {
    let bin = home.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::os::unix::fs::symlink(super::neovim::on_path("tmux").unwrap(), bin.join("tmux")).unwrap();
    let server = Server::start(name, 160, 30);
    server.run(home, &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server
}

fn split(server: &Server, pane: &str, flags: &str) -> String {
    let output = server
        .tmux(&[
            "split-window",
            flags,
            "-d",
            "-P",
            "-F",
            "#{pane_id}",
            "-t",
            pane,
            "sleep 600",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "could not split {pane}");
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

fn send(server: &Server, pane: &str, key: &str) {
    let status = server
        .tmux(&["send-keys", "-t", pane, key])
        .status()
        .unwrap();
    assert!(status.success(), "could not send {key}");
    sleep(Duration::from_millis(100));
}

fn active(server: &Server) -> String {
    let output = server
        .tmux(&["display-message", "-p", "-t", "e2e", "#{pane_id}"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

fn wait_for_active(server: &Server, expected: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let pane = active(server);
        if pane == expected || Instant::now() >= deadline {
            return pane;
        }
        sleep(Duration::from_millis(50));
    }
}
