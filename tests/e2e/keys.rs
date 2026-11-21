use tempfile::TempDir;

use crate::{RAW_AGENT, SESSION_COMMAND, Server};

const XTERM: &str = "\\033[>4;2m";
const KITTY: &str = "\\033[>1u";

#[test]
#[ignore]
fn shift_enter_reaches_the_session_as_a_new_line_in_the_xterm_form() {
    let Some((_home, server)) = focused_raw_agent("ext-xterm-shift", "on", "xterm") else {
        return;
    };

    server.send_keys(&["S-Enter"]);

    let screen = server.wait_for_screen("read:^|");
    assert!(screen.contains("read:^|"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_enter_reaches_the_session_as_a_new_line_in_the_xterm_form() {
    let Some((_home, server)) = focused_raw_agent("ext-xterm-ctrl", "on", "xterm") else {
        return;
    };

    server.send_keys(&["C-Enter"]);

    let screen = server.wait_for_screen("read:^|");
    assert!(screen.contains("read:^|"), "{screen}");
}

#[test]
#[ignore]
fn shift_enter_reaches_the_session_as_a_new_line_in_the_csi_u_form() {
    let Some((_home, server)) = focused_raw_agent("ext-csi-u-shift", "on", "csi-u") else {
        return;
    };

    server.send_keys(&["S-Enter"]);

    let screen = server.wait_for_screen("read:^|");
    assert!(screen.contains("read:^|"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_enter_reaches_the_session_as_a_new_line_with_extended_keys_always_on() {
    let Some((_home, server)) = focused_raw_agent("ext-always-ctrl", "always", "xterm") else {
        return;
    };

    server.send_keys(&["C-Enter"]);

    let screen = server.wait_for_screen("read:^|");
    assert!(screen.contains("read:^|"), "{screen}");
}

#[test]
#[ignore]
fn shift_enter_reaches_the_session_as_enter_with_extended_keys_off() {
    let Some((_home, server)) = focused_raw_agent("ext-off-shift", "off", "xterm") else {
        return;
    };

    server.send_keys(&["S-Enter", "z"]);

    let screen = server.wait_for_screen("read:|z");
    assert!(screen.contains("read:|z"), "{screen}");
}

#[test]
#[ignore]
fn shift_tab_alt_and_ctrl_letters_reach_the_session_unchanged_in_the_xterm_form() {
    let Some((_home, server)) = focused_raw_agent("ext-xterm-legacy", "on", "xterm") else {
        return;
    };

    server.send_keys(&["BTab", "M-x", "C-a", "Enter"]);

    let screen = server.wait_for_screen("read:^[Z^x@|");
    assert!(screen.contains("read:^[Z^x@|"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_h_gives_the_keys_back_to_the_sidebar_with_extended_keys_on() {
    let Some((_home, server)) = focused_raw_agent("ext-xterm-ctrl-h", "on", "xterm") else {
        return;
    };

    server.send_keys(&["C-h", "n"]);

    let screen = server.wait_for_screen("acme-api 2");
    assert!(screen.contains("acme-api 2"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_backspace_gives_the_keys_back_to_the_sidebar_in_the_csi_u_form() {
    let Some((_home, server)) = focused_raw_agent("ext-csi-u-ctrl-bspace", "on", "csi-u") else {
        return;
    };

    server.send_keys(&["C-BSpace", "n"]);

    let screen = server.wait_for_screen("acme-api 2");
    assert!(screen.contains("acme-api 2"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_escape_reaches_a_session_that_asked_for_modify_other_keys_as_ctrl_escape() {
    let Some((_home, server)) = asking_raw_agent("asked-xterm-ctrl-esc", "csi-u", XTERM) else {
        return;
    };

    server.send_keys(&["C-Escape"]);

    let screen = server.wait_for_screen("read:^[27;5;27~");
    assert!(screen.contains("read:^[27;5;27~"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_escape_reaches_a_session_that_asked_for_the_kitty_protocol_as_ctrl_escape() {
    let Some((_home, server)) = asking_raw_agent("asked-kitty-ctrl-esc", "xterm", KITTY) else {
        return;
    };

    server.send_keys(&["C-Escape"]);

    let screen = server.wait_for_screen("read:^[27;5u");
    assert!(screen.contains("read:^[27;5u"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_three_reaches_a_session_that_asked_for_modify_other_keys_as_ctrl_three() {
    let Some((_home, server)) = asking_raw_agent("asked-xterm-ctrl-3", "xterm", XTERM) else {
        return;
    };

    server.send_keys(&["C-3"]);

    let screen = server.wait_for_screen("read:^[27;5;51~");
    assert!(screen.contains("read:^[27;5;51~"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_shift_p_reaches_a_session_that_asked_for_the_kitty_protocol_apart_from_ctrl_p() {
    let Some((_home, server)) = asking_raw_agent("asked-kitty-ctrl-shift-p", "csi-u", KITTY) else {
        return;
    };

    server.send_keys(&["C-S-p", "C-p", "z"]);

    let screen = server.wait_for_screen("read:^[112;6uz");
    assert!(screen.contains("read:^[112;6uz"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_three_reaches_a_session_that_asked_for_nothing_as_escape() {
    let Some((_home, server)) = asking_raw_agent("asked-nothing-ctrl-3", "xterm", "") else {
        return;
    };

    server.send_keys(&["C-3", "z"]);

    let screen = server.wait_for_screen("read:^z");
    assert!(screen.contains("read:^z"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_h_gives_the_keys_back_to_the_sidebar_from_a_session_that_asked_for_modify_other_keys() {
    let Some((_home, server)) = asking_raw_agent("asked-xterm-ctrl-h", "xterm", XTERM) else {
        return;
    };

    server.send_keys(&["C-h", "n"]);

    let screen = server.wait_for_screen("acme-api 2");
    assert!(screen.contains("acme-api 2"), "{screen}");
}

fn focused_raw_agent(name: &str, extended: &str, format: &str) -> Option<(TempDir, Server)> {
    started_raw_agent(name, extended, format, "")
}

fn asking_raw_agent(name: &str, format: &str, asked: &str) -> Option<(TempDir, Server)> {
    started_raw_agent(name, "on", format, asked)
}

fn started_raw_agent(
    name: &str,
    extended: &str,
    format: &str,
    asked: &str,
) -> Option<(TempDir, Server)> {
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
        &[(SESSION_COMMAND, RAW_AGENT), ("ACME_KEYS_ASKED", asked)],
        &[],
    );
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");
    server.send_keys(&["Enter"]);
    server.wait_for_screen("read:");
    Some((home, server))
}
