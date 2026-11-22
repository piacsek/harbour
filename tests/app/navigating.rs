use std::path::Path;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::tmux::Tmux;

use crate::logging::{logged, only, read};
use crate::{Screen, numbered_agent, script};

const BELOW: &str = "list-keys -T root C-j\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_bottom} #{pane_tty}\nselect-pane -D -t %7\n";

#[test]
fn ctrl_j_in_the_sidebar_moves_to_the_tmux_pane_below_and_keeps_the_selection() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(scratch.path(), tmux(scratch.path(), "110"));

    screen.key(KeyCode::Char('j'), KeyModifiers::CONTROL);

    assert_eq!(calls(scratch.path()), BELOW);
    let shown = screen.wait_for("acme agent 1");
    assert!(!shown.contains("acme agent 2"), "{shown}");
}

#[test]
fn ctrl_k_in_the_sidebar_moves_to_the_tmux_pane_above() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(scratch.path(), tmux(scratch.path(), "110"));
    screen.key(KeyCode::Char('j'), KeyModifiers::NONE);
    screen.wait_for("acme agent 2");
    std::fs::remove_file(scratch.path().join("calls")).ok();

    screen.key(KeyCode::Char('k'), KeyModifiers::CONTROL);

    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root C-k\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_top} #{pane_tty}\nselect-pane -U -t %7\n"
    );
    let shown = screen.wait_for("acme agent 2");
    assert!(!shown.contains("acme agent 1"), "{shown}");
}

#[test]
fn ctrl_j_in_the_sidebar_at_the_bottom_of_the_window_selects_the_next_card() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(scratch.path(), tmux(scratch.path(), "111"));

    screen.key(KeyCode::Char('j'), KeyModifiers::CONTROL);

    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root C-j\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_bottom} #{pane_tty}\n"
    );
    screen.wait_for("acme agent 2");
}

#[test]
fn ctrl_j_in_the_sidebar_outside_tmux_selects_the_next_card() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");
    screen.press('k');
    screen.wait_for("acme agent 1");

    screen.key(KeyCode::Char('j'), KeyModifiers::CONTROL);

    screen.wait_for("acme agent 2");
}

#[test]
fn ctrl_shift_j_moves_as_ctrl_j_does() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(scratch.path(), tmux(scratch.path(), "110"));

    screen.key(
        KeyCode::Char('J'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );

    assert_eq!(calls(scratch.path()), BELOW);
}

#[test]
fn ctrl_shift_j_in_the_sidebar_at_the_bottom_of_the_window_selects_the_next_card() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(scratch.path(), tmux(scratch.path(), "111"));

    screen.key(
        KeyCode::Char('J'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );

    screen.wait_for("acme agent 2");
}

#[test]
fn ctrl_j_in_claude_code_reaches_it_when_tmux_binds_no_ctrl_j() {
    let unbound = Tmux::with_program(
        vec![
            "/bin/sh".into(),
            "-c".into(),
            "case $1 in list-keys) exit 1 ;; display-message) echo '110 /dev/acme-tty' ;; esac"
                .into(),
            "tmux".into(),
        ],
        "%7".into(),
        "/dev/acme-tty".into(),
        Duration::from_secs(5),
    );
    let mut screen = focused(App::new(reader("agent")).with_tmux(unbound));

    screen.key(KeyCode::Char('j'), KeyModifiers::CONTROL);

    screen.wait_for("acme agent read: 0a");
}

#[test]
fn ctrl_j_in_claude_code_moves_to_the_tmux_pane_below_and_sends_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = focused(App::new(reader("agent")).with_tmux(tmux(scratch.path(), "110")));

    screen.key(KeyCode::Char('j'), KeyModifiers::CONTROL);
    screen.press('z');

    assert_eq!(calls(scratch.path()), BELOW);
    screen.wait_for("acme agent read: 7a");
}

#[test]
fn ctrl_j_in_claude_code_at_the_bottom_of_the_window_reaches_claude_code() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = focused(App::new(reader("agent")).with_tmux(tmux(scratch.path(), "111")));

    screen.key(KeyCode::Char('j'), KeyModifiers::CONTROL);

    screen.wait_for("acme agent read: 0a");
}

#[test]
fn ctrl_j_in_claude_code_reaches_it_when_tmux_fails() {
    let failing = Tmux::with_program(
        vec!["/nonexistent/acme-tmux".into()],
        "%7".into(),
        "/dev/acme-tty".into(),
        Duration::from_secs(5),
    );
    let mut screen = focused(App::new(reader("agent")).with_tmux(failing));

    screen.key(KeyCode::Char('j'), KeyModifiers::CONTROL);

    screen.wait_for("acme agent read: 0a");
}

#[test]
fn ctrl_j_in_a_shown_editor_reaches_the_editor_and_tmux_is_not_asked() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(reader("agent"))
        .with_editor(Program::new("acme-edit", reader("editor")))
        .with_tmux(tmux(scratch.path(), "110"));
    let mut screen = focused(app);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");

    screen.key(KeyCode::Char('j'), KeyModifiers::CONTROL);

    screen.wait_for("acme editor read: 0a");
    assert_eq!(calls(scratch.path()), "");
}

#[test]
fn ctrl_j_under_the_help_closes_it_and_tmux_is_not_asked() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(scratch.path(), tmux(scratch.path(), "110"));
    screen.press('?');
    screen.wait_for(" help ");

    screen.key(KeyCode::Char('j'), KeyModifiers::CONTROL);

    assert_eq!(calls(scratch.path()), "");
    let shown = screen.wait_for("acme agent 1");
    assert!(!shown.contains(" help "), "{shown}");
}

#[test]
fn a_move_to_the_tmux_pane_below_is_logged_in_place_of_the_key() {
    let scratch = tempfile::tempdir().unwrap();
    let (app, lines) = logged(App::new(reader("agent")).with_tmux(tmux(scratch.path(), "110")));
    let mut screen = focused(app);
    read(&lines);

    screen.key(KeyCode::Char('j'), KeyModifiers::CONTROL);

    let lines = read(&lines);
    only(&lines, " tmux pane=below\n");
    assert!(
        !lines.iter().any(|line| line.contains(" input ")),
        "{lines:#?}"
    );
    assert!(
        !lines.iter().any(|line| line.contains(" key ")),
        "{lines:#?}"
    );
}

#[test]
fn a_move_to_the_tmux_pane_above_is_logged() {
    let scratch = tempfile::tempdir().unwrap();
    let (app, lines) = logged(App::new(reader("agent")).with_tmux(tmux(scratch.path(), "110")));
    let mut screen = focused(app);
    read(&lines);

    screen.key(KeyCode::Char('k'), KeyModifiers::CONTROL);

    only(&read(&lines), " tmux pane=above\n");
}

fn two_cards(scratch: &Path, tmux: Tmux) -> Screen {
    let mut screen = Screen::of(App::new(numbered_agent(scratch)).with_tmux(tmux));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");
    screen.press('k');
    screen.wait_for("acme agent 1");
    std::fs::remove_file(scratch.join("calls")).ok();
    screen
}

fn focused(app: App) -> Screen {
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen
}

fn tmux(scratch: &Path, edge: &str) -> Tmux {
    let calls = scratch.join("calls");
    Tmux::with_program(
        vec![
            "/bin/sh".into(),
            "-c".into(),
            format!(
                "echo \"$*\" >> '{}'; case $1 in display-message) echo '{edge} /dev/acme-tty' ;; esac",
                calls.display()
            )
            .into(),
            "tmux".into(),
        ],
        "%7".into(),
        "/dev/acme-tty".into(),
        Duration::from_secs(5),
    )
}

fn calls(scratch: &Path) -> String {
    std::fs::read_to_string(scratch.join("calls")).unwrap_or_default()
}

fn reader(name: &str) -> Vec<String> {
    script(&format!(
        "stty raw -echo; echo 'acme {name} ready' >&2; \
         dd bs=1 count=1 2>/dev/null | od -An -tx1 | tr -s ' \\n' '  ' | \
         sed 's/^/acme {name} read:/' >&2; exec sleep 600"
    ))
}
