use tempfile::TempDir;

use crate::{
    EDITOR, RAW_AGENT, SESSION_COMMAND, Server, corner, focused_at, in_order, pane_corner,
    unfocused_at,
};

const XTERM: &str = "\\033[>4;2m";
const KITTY: &str = "\\033[>1u";
const XTERM_ONE: &str = "\\033[>4;1m";

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
fn ctrl_escape_reaches_an_editor_that_asked_for_modify_other_keys_as_ctrl_escape() {
    let Some((_home, server)) = asking_raw_editor("asked-xterm-ctrl-esc", "csi-u", XTERM) else {
        return;
    };

    server.send_keys(&["C-Escape"]);

    let screen = server.wait_for_screen("read:^[27;5;27~");
    assert!(screen.contains("read:^[27;5;27~"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_escape_reaches_an_editor_that_asked_for_the_kitty_protocol_as_ctrl_escape() {
    let Some((_home, server)) = asking_raw_editor("asked-kitty-ctrl-esc", "xterm", KITTY) else {
        return;
    };

    server.send_keys(&["C-Escape"]);

    let screen = server.wait_for_screen("read:^[27;5u");
    assert!(screen.contains("read:^[27;5u"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_zero_reaches_an_editor_that_asked_for_modify_other_keys_level_two_as_ctrl_zero() {
    let Some((_home, server)) = asking_raw_editor("asked-xterm-ctrl-0", "xterm", XTERM) else {
        return;
    };

    server.send_keys(&["C-0"]);

    let screen = server.wait_for_screen("read:^[27;5;48~");
    assert!(screen.contains("read:^[27;5;48~"), "{screen}");
}

#[test]
#[ignore]
fn the_prefix_never_reaches_an_editor_that_asked_for_modify_other_keys_level_two() {
    let Some((_home, server)) = asking_raw_editor("asked-xterm-jump", "xterm", XTERM) else {
        return;
    };

    server.send_keys(&["C-/", "1", "z"]);

    let screen = server.wait_for_screen("read:z");
    assert!(screen.contains("read:z"), "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_a_digit_jumps_between_cards_with_extended_keys_on() {
    let Some((_home, server)) = two_raw_agents("ext-xterm-jump", "xterm") else {
        return;
    };

    server.send_keys(&["C-/", "1", "a"]);
    server.wait_for_screen("read:a");
    server.send_keys(&["C-/", "2", "b"]);

    let screen = server.wait_for_screen("read:b");
    assert!(screen.contains("read:b"), "{screen}");
    assert!(!screen.contains("read:a"), "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_a_digit_with_ctrl_held_jumps_between_cards_with_extended_keys_on() {
    let Some((_home, server)) = two_raw_agents("ext-xterm-ctrl-jump", "xterm") else {
        return;
    };

    server.send_keys(&["C-/", "C-1", "a"]);
    server.wait_for_screen("read:a");
    server.send_keys(&["C-/", "C-2", "b"]);

    let screen = server.wait_for_screen("read:b");
    assert!(screen.contains("read:b"), "{screen}");
    assert!(!screen.contains("read:a"), "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_nine_jumps_to_the_last_card_in_the_csi_u_form() {
    let Some((_home, server)) = two_raw_agents("ext-csi-u-jump", "csi-u") else {
        return;
    };

    server.send_keys(&["C-/", "1", "a"]);
    server.wait_for_screen("read:a");
    server.send_keys(&["C-/", "9", "b"]);

    let screen = server.wait_for_screen("read:b");
    assert!(screen.contains("read:b"), "{screen}");
    assert!(!screen.contains("read:a"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_a_and_ctrl_escape_reach_an_editor_that_asked_for_modify_other_keys_level_one_as_xterm_sends_them()
 {
    let Some((_home, server)) = asking_raw_editor("asked-xterm-one", "xterm", XTERM_ONE) else {
        return;
    };

    server.send_keys(&["C-a", "C-Escape"]);

    let screen = server.wait_for_screen("read:@^[27;5;27~");
    assert!(screen.contains("read:@^[27;5;27~"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_shift_p_reaches_an_editor_that_asked_for_the_kitty_protocol_apart_from_ctrl_p() {
    let Some((_home, server)) = asking_raw_editor("asked-kitty-ctrl-shift-p", "csi-u", KITTY)
    else {
        return;
    };

    server.send_keys(&["C-S-p", "C-p", "z"]);

    let screen = server.wait_for_screen("read:^[112;6u^[112;5uz");
    assert!(screen.contains("read:^[112;6u^[112;5uz"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_closing_bracket_reaches_an_editor_that_asked_for_the_kitty_protocol_as_ctrl_closing_bracket_with_extended_keys_off()
 {
    let Some((_home, server)) =
        started_raw_editor("asked-kitty-ctrl-bracket-off", "off", "csi-u", KITTY)
    else {
        return;
    };

    server.send_keys(&["C-]", "z"]);

    let screen = server.wait_for_screen("read:^[93;5uz");
    assert!(screen.contains("read:^[93;5uz"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_opening_bracket_reaches_an_editor_that_asked_for_the_kitty_protocol_apart_from_escape() {
    let Some((_home, server)) = asking_raw_editor("asked-kitty-ctrl-bracket", "csi-u", KITTY)
    else {
        return;
    };

    server.send_keys(&["C-[", "z"]);

    let screen = server.wait_for_screen("read:^[91;5uz");
    assert!(screen.contains("read:^[91;5uz"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_i_reaches_an_editor_that_asked_for_modify_other_keys_apart_from_tab() {
    let Some((_home, server)) = asking_raw_editor("asked-xterm-ctrl-i", "xterm", XTERM) else {
        return;
    };

    server.send_keys(&["C-i", "z"]);

    let screen = server.wait_for_screen("read:^[27;5;105~z");
    assert!(screen.contains("read:^[27;5;105~z"), "{screen}");
}

#[test]
#[ignore]
fn claude_code_that_asked_for_the_kitty_protocol_gets_the_keys_it_gets_without_asking() {
    let Some((_home, server)) = asking_raw_agent("asked-kitty-agent", "csi-u", KITTY) else {
        return;
    };

    server.send_keys(&["C-Escape", "S-Enter", "z"]);

    let screen = server.wait_for_screen("read:^^|z");
    assert!(screen.contains("read:^^|z"), "{screen}");
}

#[test]
#[ignore]
fn ctrl_one_and_ctrl_three_reach_the_session_as_one_and_escape_with_extended_keys_off() {
    let Some((_home, server)) = focused_raw_agent("ext-off-ctrl-digits", "off", "xterm") else {
        return;
    };

    server.send_keys(&["C-1", "C-3", "z"]);

    let screen = server.wait_for_screen("read:1^z");
    assert!(screen.contains("read:1^z"), "{screen}");
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

#[test]
#[ignore]
fn ctrl_m_ctrl_shift_h_and_ctrl_shift_l_act_as_enter_ctrl_h_and_ctrl_l_with_extended_keys_on() {
    let Some((_home, server)) = raw_harbour("ext-xterm-shifted-controls", "on", "xterm", "") else {
        return;
    };
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");

    server.send_keys(&["C-m", "z"]);
    server.wait_for_screen("read:z");
    server.send_keys(&["C-S-h", "n"]);
    server.wait_for_screen("acme-api 2");
    server.send_keys(&["Enter", "C-S-l"]);

    let screen = server.wait_until("the raw editor beside claude", |screen| {
        in_order(screen, &["claude ", " raw-agent.sh: "])
    });
    assert!(
        in_order(&screen, &["claude ", " raw-agent.sh: "]),
        "{screen}"
    );
}

#[test]
#[ignore]
fn the_prefix_then_z_zooms_the_selected_session_with_extended_keys_on() {
    let Some((_home, server)) = two_raw_agents("ext-xterm-zoom", "xterm") else {
        return;
    };

    server.send_keys(&["C-/", "z", "a"]);

    let screen = server.wait_until("the zoomed pane", |screen| {
        corners(screen) == 0 && screen.contains("read:a")
    });
    assert_eq!(corners(&screen), 0, "{screen}");
    assert!(screen.contains("read:a"), "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_z_zooms_the_selected_session_in_the_csi_u_form() {
    let Some((_home, server)) = two_raw_agents("ext-csi-u-zoom", "csi-u") else {
        return;
    };

    server.send_keys(&["C-/", "z", "a"]);

    let screen = server.wait_until("the zoomed pane", |screen| {
        corners(screen) == 0 && screen.contains("read:a")
    });
    assert_eq!(corners(&screen), 0, "{screen}");
    assert!(screen.contains("read:a"), "{screen}");
}

#[test]
#[ignore]
fn alt_z_reaches_the_session_as_escape_z_with_extended_keys_off() {
    let Some((_home, server)) = focused_raw_agent("ext-off-alt-z", "off", "xterm") else {
        return;
    };

    server.send_keys(&["M-z"]);

    let screen = server.wait_for_screen("read:^z");
    assert!(screen.contains("read:^z"), "{screen}");
    assert_eq!(corners(&screen), 2, "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_backslash_shows_claude_code_beside_its_editor_with_extended_keys_on() {
    let Some((_home, server)) = focused_raw_agent("ext-xterm-side", "on", "xterm") else {
        return;
    };

    server.send_keys(&["C-/", "\\", "a"]);

    let screen = server.wait_until("side by side", |screen| {
        pane_corner(screen) == Some(80) && screen.contains("read:a")
    });
    assert_eq!(pane_corner(&screen), Some(80), "{screen}");
    assert_eq!(corners(&screen), 2, "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_backslash_shows_claude_code_beside_its_editor_in_the_csi_u_form() {
    let Some((_home, server)) = focused_raw_agent("ext-csi-u-side", "on", "csi-u") else {
        return;
    };

    server.send_keys(&["C-/", "\\", "a"]);

    let screen = server.wait_until("side by side", |screen| {
        pane_corner(screen) == Some(80) && screen.contains("read:a")
    });
    assert_eq!(pane_corner(&screen), Some(80), "{screen}");
    assert_eq!(corners(&screen), 2, "{screen}");
}

#[test]
#[ignore]
fn alt_backslash_reaches_the_session_as_escape_backslash_with_extended_keys_off() {
    let Some((_home, server)) = focused_raw_agent("ext-off-side", "off", "xterm") else {
        return;
    };

    server.send_keys(&["M-\\"]);

    let screen = server.wait_for_screen("read:^\\");
    assert!(screen.contains("read:^\\"), "{screen}");
    assert_ne!(pane_corner(&screen), Some(80), "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_equals_widens_the_sidebar_with_extended_keys_on() {
    let Some((_home, server)) = focused_raw_agent("ext-xterm-wider", "on", "xterm") else {
        return;
    };

    server.send_keys(&["C-/", "=", "a"]);

    let screen = server.wait_until("the wider sidebar", |screen| {
        pane_corner(screen) == Some(55) && screen.contains("read:a")
    });
    assert_eq!(pane_corner(&screen), Some(55), "{screen}");
    assert!(screen.contains("read:a"), "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_minus_narrows_the_sidebar_in_the_csi_u_form() {
    let Some((_home, server)) = focused_raw_agent("ext-csi-u-narrower", "on", "csi-u") else {
        return;
    };

    server.send_keys(&["C-/", "-", "a"]);

    let screen = server.wait_until("the narrower sidebar", |screen| {
        pane_corner(screen) == Some(51) && screen.contains("read:a")
    });
    assert_eq!(pane_corner(&screen), Some(51), "{screen}");
    assert!(screen.contains("read:a"), "{screen}");
}

#[test]
#[ignore]
fn alt_minus_reaches_the_session_as_escape_minus_with_extended_keys_off() {
    let Some((_home, server)) = focused_raw_agent("ext-off-alt-minus", "off", "xterm") else {
        return;
    };

    server.send_keys(&["M--"]);

    let screen = server.wait_for_screen("read:^-");
    assert!(screen.contains("read:^-"), "{screen}");
    assert_eq!(pane_corner(&screen), Some(53), "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_closing_brace_puts_claude_code_on_the_right_with_extended_keys_on() {
    let Some((_home, server)) = side_by_side_raw_agent("ext-xterm-swap", "xterm") else {
        return;
    };

    server.send_keys(&["C-/", "}", "a"]);

    let screen = server.wait_until("the swapped halves", |screen| {
        unfocused_at(screen, 0) && screen.contains("read:a")
    });
    assert!(unfocused_at(&screen, 0), "{screen}");
    assert_eq!(pane_corner(&screen), Some(80), "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_opening_brace_puts_claude_code_back_on_the_left_in_the_csi_u_form() {
    let Some((_home, server)) = side_by_side_raw_agent("ext-csi-u-swap", "csi-u") else {
        return;
    };
    server.send_keys(&["C-/", "}"]);
    server.wait_until("the swapped halves", |screen| unfocused_at(screen, 0));

    server.send_keys(&["C-/", "{", "a"]);

    let screen = server.wait_until("the halves back", |screen| {
        focused_at(screen, 0) && screen.contains("read:a")
    });
    assert!(focused_at(&screen, 0), "{screen}");
    assert_eq!(pane_corner(&screen), Some(80), "{screen}");
}

#[test]
#[ignore]
fn alt_opening_brace_reaches_the_session_as_escape_brace_with_extended_keys_off() {
    let Some((_home, server)) = focused_raw_agent("ext-off-swap", "off", "xterm") else {
        return;
    };

    server.send_keys(&["M-{"]);

    let screen = server.wait_for_screen("read:^{");
    assert!(screen.contains("read:^{"), "{screen}");
}

fn two_raw_agents(name: &str, format: &str) -> Option<(TempDir, Server)> {
    let (home, server) = raw_harbour(name, "on", format, "")?;
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme-api 2");
    Some((home, server))
}

fn side_by_side_raw_agent(name: &str, format: &str) -> Option<(TempDir, Server)> {
    let (home, server) = focused_raw_agent(name, "on", format)?;
    server.send_keys(&["C-/", "\\"]);
    server.wait_until("side by side", |screen| pane_corner(screen) == Some(80));
    Some((home, server))
}

fn focused_raw_agent(name: &str, extended: &str, format: &str) -> Option<(TempDir, Server)> {
    started_raw_agent(name, extended, format, "")
}

fn asking_raw_agent(name: &str, format: &str, asked: &str) -> Option<(TempDir, Server)> {
    started_raw_agent(name, "on", format, asked)
}

fn asking_raw_editor(name: &str, format: &str, asked: &str) -> Option<(TempDir, Server)> {
    started_raw_editor(name, "on", format, asked)
}

fn started_raw_agent(
    name: &str,
    extended: &str,
    format: &str,
    asked: &str,
) -> Option<(TempDir, Server)> {
    let (home, server) = raw_harbour(name, extended, format, asked)?;
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent ready");
    server.send_keys(&["Enter"]);
    server.wait_for_screen("read:");
    Some((home, server))
}

fn started_raw_editor(
    name: &str,
    extended: &str,
    format: &str,
    asked: &str,
) -> Option<(TempDir, Server)> {
    let (home, server) = started_raw_agent(name, extended, format, asked)?;
    server.send_keys(&["C-l"]);
    server.wait_until("the raw editor beside claude", |screen| {
        in_order(screen, &["claude ", " raw-agent.sh: "])
    });
    server.wait_for_screen("read:");
    Some((home, server))
}

fn raw_harbour(name: &str, extended: &str, format: &str, asked: &str) -> Option<(TempDir, Server)> {
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
        &[
            (SESSION_COMMAND, RAW_AGENT),
            (EDITOR, RAW_AGENT),
            ("ACME_KEYS_ASKED", asked),
        ],
        &[],
    );
    server.wait_for_screen("press n to start one");
    Some((home, server))
}

fn corners(screen: &str) -> usize {
    screen
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|c| corner(*c))
        .count()
}
