use harbour::tui::SESSION_COMMAND;

use super::Server;

const TITLED_AGENT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/titled-agent.sh"
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
