use harbour::tui::SESSION_COMMAND;

use super::Server;

const TITLED_AGENT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/titled-agent.sh"
);

const RESUMABLE_AGENT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/resumable-agent.sh"
);

#[test]
#[ignore]
fn a_card_shows_the_title_and_activity_from_the_sessions_transcript() {
    let home = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let config_dir = config.path().display().to_string();
    let server = Server::start("cards", 160, 20);
    server.run(
        home.path(),
        &[
            (SESSION_COMMAND, TITLED_AGENT),
            ("CLAUDE_CONFIG_DIR", &config_dir),
        ],
        &[],
    );
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen_all(&["Fix the flaky login test", "Run the login tests"]);
    assert!(screen.contains("● Fix the flaky login test"), "{screen}");
    assert!(screen.contains("Run the login tests"), "{screen}");
}

#[test]
#[ignore]
fn a_session_comes_back_stopped_after_quitting_and_resumes_with_enter() {
    let home = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    let config_dir = config.path().display().to_string();
    let env = [
        (SESSION_COMMAND, RESUMABLE_AGENT),
        ("CLAUDE_CONFIG_DIR", config_dir.as_str()),
    ];
    let server = Server::start("resume", 160, 20);
    server.run(home.path(), &env, &[]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    let started = server.wait_for_screen_all(&["acme agent started", "Fix the flaky login test"]);
    assert!(started.contains("Fix the flaky login test"), "{started}");
    server.send_keys(&["q"]);
    let asked = server.wait_for_screen("Quit? y/n");
    assert!(
        asked.contains("Conversations come back stopped."),
        "{asked}"
    );
    server.send_keys(&["y"]);
    let quit = server.wait_for_screen("harbour exited 0");
    assert!(quit.contains("harbour exited 0"), "{quit}");

    server.run(home.path(), &env, &[]);
    let stopped = server.wait_for_screen("Enter resumes it");
    assert!(stopped.contains("Fix the flaky login test"), "{stopped}");
    server.send_keys(&["Enter"]);

    let resumed = server.wait_for_screen("acme agent resumed 0f1e2d3c-4b5a-6978-8a9b-0c1d2e3f4a5b");
    assert!(
        resumed.contains("acme agent resumed 0f1e2d3c-4b5a-6978-8a9b-0c1d2e3f4a5b"),
        "{resumed}"
    );
}
