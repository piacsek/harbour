use tempfile::TempDir;

use crate::{RAW_AGENT, SESSION_COMMAND, Server};

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
    server.run(home.path(), &[(SESSION_COMMAND, RAW_AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");
    server.send_keys(&["Enter"]);
    server.wait_for_screen("read:");
    Some((home, server))
}
