use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use harbour::tui::SESSION_COMMAND;

use super::{AGENT, Server};

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
fn a_card_shows_the_title_from_the_sessions_transcript_and_never_its_last_tool_call() {
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

    let screen = server.wait_for_screen("● Fix the flaky login test");
    assert!(screen.contains("● Fix the flaky login test"), "{screen}");
    assert!(!screen.contains("Run the login tests"), "{screen}");
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
    let empty = server.wait_for_screen("press n to start one");
    assert!(empty.contains("press n to start one"), "{empty}");
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

#[test]
#[ignore]
fn a_conversation_comes_back_only_under_the_claude_code_configuration_it_was_made_in() {
    let home = tempfile::tempdir().unwrap();
    let work = tempfile::tempdir().unwrap();
    let personal = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(personal.path().join("projects")).unwrap();
    let work_dir = work.path().display().to_string();
    let personal_dir = personal.path().display().to_string();
    let at_work = [
        (SESSION_COMMAND, RESUMABLE_AGENT),
        ("CLAUDE_CONFIG_DIR", work_dir.as_str()),
    ];
    let at_home = [
        (SESSION_COMMAND, RESUMABLE_AGENT),
        ("CLAUDE_CONFIG_DIR", personal_dir.as_str()),
    ];
    let server = Server::start("configurations", 160, 20);
    server.run(home.path(), &at_work, &[]);
    let empty = server.wait_for_screen("press n to start one");
    assert!(empty.contains("press n to start one"), "{empty}");
    server.send_keys(&["n"]);
    let started = server.wait_for_screen_all(&["acme agent started", "Fix the flaky login test"]);
    assert!(started.contains("Fix the flaky login test"), "{started}");
    server.send_keys(&["q"]);
    let asked = server.wait_for_screen("Quit? y/n");
    assert!(asked.contains("Quit? y/n"), "{asked}");
    server.send_keys(&["y"]);
    let quit = server.wait_for_screen("harbour exited 0");
    assert!(quit.contains("harbour exited 0"), "{quit}");
    let remembered = remembered_files(&home.path().join(".local/state/harbour/sessions"));
    assert_eq!(remembered.len(), 1);
    std::fs::File::options()
        .write(true)
        .open(&remembered[0])
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(31 * 24 * 60 * 60))
        .unwrap();

    server.run(home.path(), &at_home, &[]);
    let elsewhere = server.wait_for_screen("press n to start one");
    assert!(elsewhere.contains("press n to start one"), "{elsewhere}");
    assert!(
        !elsewhere.contains("Fix the flaky login test"),
        "{elsewhere}"
    );
    server.send_keys(&["q"]);
    let left = server.wait_for_screen("harbour exited 0");
    assert!(left.contains("harbour exited 0"), "{left}");

    server.run(home.path(), &at_work, &[]);
    let back = server.wait_for_screen("Enter resumes it");
    assert!(back.contains("Fix the flaky login test"), "{back}");
}

#[test]
#[ignore]
fn the_demo_shows_a_stopped_session_it_remembers() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("demo-stopped", 160, 26);

    server.run(home.path(), &[], &["--demo"]);

    let rows = [
        "○ Tidy the acme-docs links",
        "stopped",
        "acme-docs · main",
        " sessions ● 1 ◉ 1 ○ 2 · 1 ",
    ];
    let screen = server.wait_for_screen_all(&rows);
    assert!(screen.contains(rows[0]), "{screen}");
    assert!(screen.contains(rows[1]), "{screen}");
    assert!(screen.contains(rows[2]), "{screen}");
    assert!(screen.contains(rows[3]), "{screen}");
}

fn remembered_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .flat_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                remembered_files(&path)
            } else if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                vec![path]
            } else {
                Vec::new()
            }
        })
        .collect()
}
