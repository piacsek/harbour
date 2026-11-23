use std::path::{Path, PathBuf};

use harbour::tui::{DIR_PICKER, SESSION_COMMAND};

use crate::{AGENT, ARGS_AGENT, FAKE_FZF, PICKER, Server, tools};

const FIXTURE_EDITOR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/editor.sh");
const RESUMABLE_AGENT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/resumable-agent.sh"
);

#[test]
#[ignore]
fn a_syntax_error_stops_harbour_before_the_interface_opens() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "accent = \"cyan\"\neditor = [\"nvim\"\n");
    let server = Server::start("config-syntax", 200, 20);

    server.run(home.path(), &[], &[]);

    let screen = server.wait_for_screen("harbour exited 1");
    assert!(
        screen.contains(&format!("harbour: {}:2: editor: ", path.display())),
        "{screen}"
    );
    assert!(!screen.contains("No session yet"), "{screen}");
}

#[test]
#[ignore]
fn a_wrong_type_stops_harbour_before_the_interface_opens() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "sidebar_width = 0\n");
    let server = Server::start("config-type", 200, 20);

    server.run(home.path(), &[], &[]);

    let screen = server.wait_for_screen("harbour exited 1");
    assert!(
        screen.contains(&format!(
            "harbour: {}:1: sidebar_width: expected a whole number of columns above 0",
            path.display()
        )),
        "{screen}"
    );
}

#[test]
#[ignore]
fn an_unknown_key_stops_harbour_before_the_interface_opens() {
    let home = tempfile::tempdir().unwrap();
    let path = write_config(home.path(), "retention_days = 7\n");
    let server = Server::start("config-unknown", 200, 20);

    server.run(home.path(), &[], &[]);

    let screen = server.wait_for_screen("harbour exited 1");
    assert!(
        screen.contains(&format!(
            "harbour: {}:1: retention_days: unknown key",
            path.display()
        )),
        "{screen}"
    );
}

#[test]
#[ignore]
fn the_demo_never_reads_the_file() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "retention_days = 7\n");
    let server = Server::start("config-demo", 160, 26);

    server.run(home.path(), &[], &["--demo"]);

    let screen = server.wait_for_screen("permission prompt");
    assert!(!screen.contains("retention_days"), "{screen}");
}

#[test]
#[ignore]
fn the_session_command_from_the_file_runs_with_its_arguments() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        &format!("session_command = [{ARGS_AGENT:?}, \"--model\", \"acme\"]\n"),
    );
    let server = Server::start("config-command", 160, 20);
    server.run(home.path(), &[], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme agent args:");
    assert!(
        screen.contains("acme agent args:[--model acme]"),
        "{screen}"
    );
}

#[test]
#[ignore]
fn a_session_command_named_claude_from_the_file_gets_a_session_id_after_its_arguments() {
    let home = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(ARGS_AGENT, Path::new(&tools(home.path())).join("claude")).unwrap();
    write_config(
        home.path(),
        "session_command = [\"claude\", \"--model\", \"acme\"]\n",
    );
    let server = Server::start("config-claude", 160, 20);
    server.run(home.path(), &[], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme agent args:");
    assert!(
        screen.contains("acme agent args:[--model acme --session-id "),
        "{screen}"
    );
}

#[test]
#[ignore]
fn a_conversation_resumes_with_the_session_command_from_the_file() {
    let home = tempfile::tempdir().unwrap();
    let claude = tempfile::tempdir().unwrap();
    let claude_dir = claude.path().display().to_string();
    let env = [("CLAUDE_CONFIG_DIR", claude_dir.as_str())];
    write_config(
        home.path(),
        &format!("session_command = [{RESUMABLE_AGENT:?}, \"--model\", \"acme\"]\n"),
    );
    let server = Server::start("config-resume", 160, 20);
    server.run(home.path(), &env, &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen_all(&["acme agent started", "Fix the flaky login test"]);
    server.send_keys(&["C-/", "q"]);
    server.wait_for_screen("Quit? y/n");
    server.send_keys(&["y"]);
    server.wait_for_screen("harbour exited 0");
    server.run(home.path(), &env, &[]);
    server.wait_for_screen("Enter resumes it");

    server.send_keys(&["Enter"]);

    let resumed = server.wait_for_screen("acme agent args:");
    assert!(
        resumed.contains(
            "acme agent args:[--model acme --resume 0f1e2d3c-4b5a-6978-8a9b-0c1d2e3f4a5b]"
        ),
        "{resumed}"
    );
}

#[test]
#[ignore]
fn the_editor_from_the_file_runs_with_its_arguments() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        &format!("editor = [{FIXTURE_EDITOR:?}, \"--acme-flag\"]\n"),
    );
    let server = Server::start("config-editor", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent in acme-api");

    server.send_keys(&["Enter", "C-l"]);

    let screen = server.wait_for_screen("acme editor in acme-api");
    assert!(screen.contains("first arg:[--acme-flag]"), "{screen}");
    assert!(screen.contains("claude | editor.sh: "), "{screen}");
}

#[test]
#[ignore]
fn a_picker_from_the_file_runs_alone_with_its_arguments() {
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir(home.path().join("acme-web")).unwrap();
    write_config(home.path(), &format!("picker = [{PICKER:?}, \"--dirs\"]\n"));
    let server = Server::start("config-picker", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme picker: type a directory");

    server.send_keys(&["../acme-web", "Enter"]);

    let screen = server.wait_for_screen("acme agent in acme-web");
    assert!(screen.contains("acme agent in acme-web"), "{screen}");
}

#[test]
#[ignore]
fn an_fzf_picker_from_the_file_runs_over_the_list_with_its_arguments() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        &format!("picker = [\"{FAKE_FZF}/fzf\", \"--height=40%\"]\n"),
    );
    let server = Server::start("config-fzf", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme fzf lists:");
    assert!(
        screen.contains("acme fzf args: --height=40% --no-expect"),
        "{screen}"
    );
}

#[test]
#[ignore]
fn an_fzf_picker_from_the_file_that_cannot_start_is_refused() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "picker = [\"/opt/acme/bin/fzf\"]\n");
    let server = Server::start("config-fzf-missing", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("Could not start");
    assert!(
        screen.contains("Could not start /opt/acme/bin/fzf"),
        "{screen}"
    );
    assert!(!screen.contains("acme agent in"), "{screen}");
}

#[test]
#[ignore]
fn an_fzf_picker_variable_that_cannot_start_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("config-fzf-variable-missing", 160, 20);
    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (DIR_PICKER, "/opt/acme/bin/fzf")],
        &[],
    );
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("Could not start");
    assert!(
        screen.contains("Could not start /opt/acme/bin/fzf"),
        "{screen}"
    );
    assert!(!screen.contains("acme agent in"), "{screen}");
}

#[test]
#[ignore]
fn the_accent_from_the_file_colours_the_focused_border() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "accent = \"red\"\n");
    let server = Server::start("config-accent", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");

    let first = coloured(&server);

    assert!(first.contains("\u{1b}[38;5;1m┏"), "{first:?}");
}

#[test]
#[ignore]
fn the_sidebar_width_from_the_file_sets_the_sidebar_columns() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "sidebar_width = 30\n");
    let server = Server::start("config-width", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);

    let screen = server.wait_for_screen("press n to start one");

    let top = screen.lines().next().unwrap_or_default();
    assert_eq!(top.chars().position(|c| c == '┌'), Some(30), "{top}");
}

#[test]
#[ignore]
fn the_nerd_font_from_the_file_puts_icons_in_the_pane_border() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "nerd_font = true\n");
    let server = Server::start("config-nerd-font", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme agent in acme-api");
    let top = screen.lines().next().unwrap_or_default();
    assert!(top.contains(" \u{F0674} claude | \u{E6AE} nvim "), "{top}");
}

#[test]
#[ignore]
fn the_clawd_icon_from_the_file_leads_the_claude_name_in_the_pane_border() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "nerd_font = true\nclaude_icon = \"clawd\"\n");
    let server = Server::start("config-clawd-icon", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme agent in acme-api");
    let top = screen.lines().next().unwrap_or_default();
    assert!(
        top.contains(" \u{100000}  claude | \u{E6AE} nvim "),
        "{top}"
    );
}

#[test]
#[ignore]
fn the_bindings_from_the_file_start_a_session_and_name_the_prefix() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "prefix = \"C-]\"\n[keys]\nnew = \"a\"\n");
    let server = Server::start("config-keys", 200, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("No session yet");

    server.send_keys(&["a"]);
    server.wait_for_screen("acme agent in");
    server.send_keys(&["C-]", "w"]);

    server.wait_for_screen("C-] w does nothing; C-] ? lists the keys");
}

#[test]
#[ignore]
fn the_fact_colours_from_the_file_colour_the_cards_third_line() {
    let home = tempfile::tempdir().unwrap();
    write_config(home.path(), "[colors]\nrepository = \"red\"\n");
    let server = Server::start("config-colors", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent in acme-api");

    let third = coloured_row(&server, 3);
    assert!(third.contains("\u{1b}[38;5;1macme-api"), "{third:?}");
}

fn coloured(server: &Server) -> String {
    let output = server
        .tmux(&["capture-pane", "-p", "-e", "-t", "e2e"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .to_string()
}

fn write_config(home: &Path, text: &str) -> PathBuf {
    let path = home.join(".config/harbour/config.toml");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, text).unwrap();
    path
}

fn coloured_row(server: &Server, row: usize) -> String {
    let output = server
        .tmux(&["capture-pane", "-p", "-e", "-t", "e2e"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .nth(row)
        .unwrap_or_default()
        .to_string()
}
