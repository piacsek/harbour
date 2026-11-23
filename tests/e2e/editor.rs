use std::path::Path;

use harbour::tui::{EDITOR, SESSION_COMMAND};

use super::{AGENT, ARGS_AGENT, Server, focused_at, in_order, tools, unfocused_at};

const FIXTURE_EDITOR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/editor.sh");

#[test]
#[ignore]
fn the_default_claude_is_started_with_a_session_id() {
    let home = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(ARGS_AGENT, Path::new(&tools(home.path())).join("claude")).unwrap();
    let server = Server::start("session-id", 160, 20);
    server.run(home.path(), &[], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme agent args:");
    assert!(
        screen.contains("acme agent args:[--session-id "),
        "{screen}"
    );
}

#[test]
#[ignore]
fn c_l_opens_the_editor_harbour_editor_names_without_arguments() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("named-editor", 160, 20);

    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (EDITOR, FIXTURE_EDITOR)],
        &[],
    );

    let screen = editing(&server);
    assert!(
        screen.contains("acme editor in acme-api first arg:[]"),
        "{screen}"
    );
    assert!(in_order(&screen, &["claude ", " editor.sh: "]), "{screen}");
}

#[test]
#[ignore]
fn without_harbour_editor_c_l_opens_nvim_listening_on_a_socket() {
    let home = tempfile::tempdir().unwrap();
    let nvim = Path::new(&tools(home.path())).join("nvim");
    std::os::unix::fs::symlink(FIXTURE_EDITOR, nvim).unwrap();
    let server = Server::start("default-editor", 160, 20);

    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);

    let screen = editing(&server);
    assert!(screen.contains("first arg:[--listen]"), "{screen}");
    assert!(in_order(&screen, &["claude ", " nvim: "]), "{screen}");
}

#[test]
#[ignore]
fn an_editor_whose_file_name_is_nvim_listens_on_a_socket() {
    let home = tempfile::tempdir().unwrap();
    let elsewhere = home.path().join("acme-tools");
    std::fs::create_dir(&elsewhere).unwrap();
    let nvim = elsewhere.join("nvim");
    std::os::unix::fs::symlink(FIXTURE_EDITOR, &nvim).unwrap();
    let server = Server::start("nvim-path", 160, 20);

    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (EDITOR, nvim.to_str().unwrap())],
        &[],
    );

    let screen = editing(&server);
    assert!(screen.contains("first arg:[--listen]"), "{screen}");
}

#[test]
#[ignore]
fn nvim_starts_without_a_socket_once_its_temporary_directory_is_gone() {
    let home = tempfile::tempdir().unwrap();
    let elsewhere = home.path().join("acme-tools");
    std::fs::create_dir(&elsewhere).unwrap();
    let nvim = elsewhere.join("nvim");
    std::os::unix::fs::symlink(FIXTURE_EDITOR, &nvim).unwrap();
    let temporary = home.path().join("acme-tmp");
    std::fs::create_dir(&temporary).unwrap();
    let server = Server::start("nvim-no-tmp", 160, 20);
    server.run(
        home.path(),
        &[
            (SESSION_COMMAND, AGENT),
            (EDITOR, nvim.to_str().unwrap()),
            ("TMPDIR", temporary.to_str().unwrap()),
        ],
        &[],
    );
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent in acme-api");
    std::fs::remove_dir_all(&temporary).unwrap();

    server.send_keys(&["Enter", "C-l"]);

    let screen = server.wait_for_screen("acme editor in acme-api");
    assert!(screen.contains("first arg:[]"), "{screen}");
    assert!(!screen.contains("Could not start"), "{screen}");
}

#[test]
#[ignore]
fn nvim_inside_tmux_keeps_the_tmux_of_harbour_and_its_session_does_not() {
    let home = tempfile::tempdir().unwrap();
    let nvim = Path::new(&tools(home.path())).join("nvim");
    std::os::unix::fs::symlink(FIXTURE_EDITOR, nvim).unwrap();
    let server = Server::start("nvim-tmux", 200, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    let session = server.wait_for_screen("acme agent in acme-api, tmux:[");
    assert!(
        session.contains("acme agent in acme-api, tmux:[] pane:[]"),
        "{session}"
    );

    server.send_keys(&["Enter", "C-l"]);

    let expected = editor_line(&harbours_tmux(&server));
    let screen = server.wait_for_screen(&expected);
    assert!(screen.contains(&expected), "{screen}");
}

#[test]
#[ignore]
fn the_editor_harbour_editor_names_keeps_the_tmux_of_harbour() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("named-editor-tmux", 200, 20);

    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (EDITOR, FIXTURE_EDITOR)],
        &[],
    );
    editing(&server);

    let expected = editor_line(&harbours_tmux(&server));
    let screen = server.wait_for_screen(&expected);
    assert!(screen.contains(&expected), "{screen}");
}

#[test]
#[ignore]
fn an_editor_of_a_harbour_outside_tmux_sees_no_tmux() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("editor-no-tmux", 200, 20);

    server.launch(
        home.path(),
        "env -u TMUX -u TMUX_PANE",
        &[(SESSION_COMMAND, AGENT), (EDITOR, FIXTURE_EDITOR)],
        &[],
    );
    editing(&server);

    let screen = server.wait_for_screen("acme editor tmux:[");
    assert!(screen.contains("acme editor tmux:[] pane:[]"), "{screen}");
}

fn editing(server: &Server) -> String {
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme agent in acme-api");
    server.send_keys(&["Enter", "C-l"]);
    server.wait_for_screen("acme editor in acme-api")
}

#[test]
#[ignore]
fn c_l_in_the_demo_opens_a_made_up_editor() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("demo-editor", 160, 26);
    server.run(home.path(), &[], &["--demo"]);
    server.wait_for_screen("acme agent");

    server.send_keys(&["Enter", "C-l"]);

    let screen = server.wait_for_screen("src/retry.rs");
    assert!(screen.contains("pub fn retry"), "{screen}");
    assert!(in_order(&screen, &["claude ", " nvim: "]), "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_backslash_in_the_demo_shows_its_made_up_editor_beside_claude_code() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("demo-side", 160, 26);
    server.run(home.path(), &[], &["--demo"]);
    server.wait_for_screen("acme agent");

    server.send_keys(&["C-/", "\\"]);

    let screen = server.wait_until("the demo editor beside claude code", |screen| {
        screen.contains("pub fn retry") && claude_focused_beside_the_editor(screen)
    });
    assert!(claude_focused_beside_the_editor(&screen), "{screen}");
    assert!(screen.contains("pub fn retry"), "{screen}");
}

#[test]
#[ignore]
fn the_prefix_then_u_in_the_demo_shows_a_made_up_usage_report() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("demo-usage", 160, 26);
    server.run(home.path(), &[], &["--demo"]);
    server.wait_for_screen("acme agent");

    server.send_keys(&["C-/", "u"]);

    let screen = server.wait_for_screen_all(&[
        " usage ",
        "12%   resets 3:40pm",
        "74%   resets Mon 9:00am",
        "Last 7d · 6815 requests · 41 sessions",
        "subagents   Explore 22% · Plan 4%",
        "t raw text · Esc close",
    ]);
    assert!(screen.contains("t raw text · Esc close"), "{screen}");
}

fn harbours_tmux(server: &Server) -> (String, String) {
    let output = server
        .tmux(&[
            "display-message",
            "-p",
            "-t",
            "e2e",
            "#{socket_path},#{pid},#{session_id} #{pane_id}",
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "display-message of harbour's pane failed: {stderr}"
    );
    let line = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    let (tmux, pane) = line.split_once(' ').unwrap_or((&line, ""));
    assert!(
        !pane.is_empty(),
        "display-message printed no pane id: {line}"
    );
    let (server, session) = tmux.rsplit_once(',').unwrap();
    let session = session.strip_prefix('$').unwrap_or(session);
    (format!("{server},{session}"), pane.to_string())
}

fn editor_line((tmux, pane): &(String, String)) -> String {
    format!("acme editor tmux:[{tmux}] pane:[{pane}]")
}

fn claude_focused_beside_the_editor(screen: &str) -> bool {
    focused_at(screen, 0) && unfocused_at(screen, 80)
}
