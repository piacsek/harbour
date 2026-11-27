use harbour::state::State;
use harbour::tui::DIR_PICKER;
use tempfile::TempDir;

use crate::{RAW_AGENT, REGISTERED_AGENT, SESSION_COMMAND, Server};

const FINDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/finder/fzf");

#[test]
#[ignore]
fn the_prefix_then_s_lists_the_sessions_with_extended_keys_off() {
    let Some((_home, server)) = focused_raw_agent("find-prefix", "off", "xterm") else {
        return;
    };

    server.send_keys(&["C-/", "s"]);

    let screen = server.wait_for_screen("acme fzf ready");
    assert!(screen.contains("acme fzf ready"), "{screen}");
    assert!(!screen.contains("read:^"), "{screen}");
}

#[test]
#[ignore]
fn alt_s_reaches_the_session_as_escape_s_with_extended_keys_off() {
    let Some((_home, server)) = focused_raw_agent("find-off", "off", "xterm") else {
        return;
    };

    server.send_keys(&["M-s"]);

    let screen = server.wait_for_screen("read:^s");
    assert!(screen.contains("read:^s"), "{screen}");
    assert!(!screen.contains("acme fzf"), "{screen}");
}

#[test]
#[ignore]
fn the_finder_draws_a_working_sessions_dot_in_yellow() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("find-colour", 160, 20);
    server.run(
        home.path(),
        &[(SESSION_COMMAND, REGISTERED_AGENT), (DIR_PICKER, FINDER)],
        &[],
    );
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent registered");
    server.wait_for_screen(State::Working.glyph());

    server.send_keys(&["C-/", "s"]);

    let dot = State::Working.glyph();
    server.wait_for_screen(&format!("0|{dot}|"));
    let coloured = coloured_screen(&server);
    assert!(
        coloured.contains(&format!("0|\u{1b}[38;5;3m{dot}\u{1b}[39m|")),
        "{coloured:?}"
    );
}

fn focused_raw_agent(name: &str, extended: &str, format: &str) -> Option<(TempDir, Server)> {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start(name, 160, 20);
    let options = [
        ["extended-keys", extended],
        ["extended-keys-format", format],
    ];
    if let Some(option) = options.iter().find(|option| !server.server_option(*option)) {
        eprintln!(
            "skipping {name}: this tmux cannot set {option:?}; extended-keys-format needs tmux 3.5"
        );
        return None;
    }
    server.run(
        home.path(),
        &[(SESSION_COMMAND, RAW_AGENT), (DIR_PICKER, FINDER)],
        &[],
    );
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");
    Some((home, server))
}

fn coloured_screen(server: &Server) -> String {
    let output = server
        .tmux(&["capture-pane", "-p", "-e", "-t", "e2e"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).to_string()
}
