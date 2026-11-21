use std::path::{Path, PathBuf};

use harbour::tui::SESSION_COMMAND;

use crate::{AGENT, ARGS_AGENT, FAKE_FZF, PICKER, Server};

const FIXTURE_EDITOR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/editor.sh");

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
fn the_session_command_variable_outranks_the_file() {
    let home = tempfile::tempdir().unwrap();
    write_config(
        home.path(),
        &format!("session_command = [{ARGS_AGENT:?}, \"--model\", \"acme\"]\n"),
    );
    let server = Server::start("config-command-variable", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme agent in acme-api");
    assert!(!screen.contains("acme agent args:"), "{screen}");
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
