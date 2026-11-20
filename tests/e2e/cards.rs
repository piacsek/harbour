use harbour::tui::SESSION_COMMAND;

use super::{AGENT, Server};

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

#[test]
#[ignore]
fn a_home_directory_kept_in_git_is_not_the_repository_of_a_folder_below_it() {
    let home = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(home.path().join(".git")).unwrap();
    std::fs::write(
        home.path().join(".git/HEAD"),
        "ref: refs/heads/acme-dotfiles\n",
    )
    .unwrap();
    let server = Server::start("home-repository", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme-api 1");
    assert!(screen.contains("acme-api 1"), "{screen}");
    assert!(!screen.contains("acme-dotfiles"), "{screen}");
}
