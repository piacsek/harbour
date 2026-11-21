use std::os::unix::fs::PermissionsExt;

use harbour::tui::{DIR_PICKER, SESSION_COMMAND};

use super::{AGENT, FAKE_FZF, PICKER, Server, tools};

#[test]
#[ignore]
fn n_in_the_demo_without_fzf_starts_a_session_at_once() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("demo-no-fzf", 160, 20);
    server.run(home.path(), &[], &["--demo"]);
    server.wait_for_screen("acme-infra 5");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme-api 6");
    assert!(screen.contains("acme-api 6"), "{screen}");
    assert!(!screen.contains("Could not start"), "{screen}");
}

#[test]
#[ignore]
fn the_dir_picker_does_not_inherit_the_tmux_of_harbour() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("picker-tmux", 160, 20);
    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (DIR_PICKER, PICKER)],
        &[],
    );
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let screen = server.wait_for_screen("acme picker tmux:[");
    assert!(screen.contains("acme picker tmux:[] pane:[]"), "{screen}");
}

#[test]
#[ignore]
fn n_in_a_directory_deleted_before_launch_starts_nothing_and_names_it() {
    let home = tempfile::tempdir().unwrap();
    let directory = home.path().join("acme-api");
    let server = Server::start("deleted", 160, 20);
    server.launch(
        home.path(),
        "rmdir \"$PWD\" &&",
        &[(SESSION_COMMAND, AGENT)],
        &[],
    );
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    let refusal = format!("Not a directory: {}", directory.display());
    let screen = server.wait_for_screen(&refusal);
    assert!(screen.contains(&refusal), "{screen}");
    assert!(screen.contains("No session yet"), "{screen}");
}

#[test]
#[ignore]
fn n_after_its_directory_is_deleted_starts_nothing_and_names_it() {
    let home = tempfile::tempdir().unwrap();
    let directory = home.path().join("acme-api");
    let server = Server::start("deleted-after", 160, 20);
    server.run(home.path(), &[(SESSION_COMMAND, AGENT)], &[]);
    server.wait_for_screen("press n to start one");
    let shown = directory.canonicalize().unwrap();
    std::fs::remove_dir(&directory).unwrap();

    server.send_keys(&["n"]);

    let refusal = format!("Not a directory: {}", shown.display());
    let screen = server.wait_for_screen(&refusal);
    assert!(screen.contains(&refusal), "{screen}");
    assert!(screen.contains("No session yet"), "{screen}");
}

#[test]
#[ignore]
fn a_picker_that_returns_its_deleted_directory_starts_nothing() {
    let home = tempfile::tempdir().unwrap();
    let directory = home.path().join("acme-api");
    let server = Server::start("deleted-picked", 160, 20);
    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (DIR_PICKER, PICKER)],
        &[],
    );
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    server.wait_for_screen("acme picker: type a directory");
    std::fs::remove_dir(&directory).unwrap();

    server.send_keys(&[&directory.display().to_string(), "Enter"]);

    let refusal = format!("Not a directory: {}", directory.display());
    let screen = server.wait_for_screen(&refusal);
    assert!(screen.contains(&refusal), "{screen}");
    assert!(screen.contains("No session yet"), "{screen}");
}

#[test]
#[ignore]
fn n_with_a_picker_after_its_directory_is_deleted_starts_nothing() {
    let home = tempfile::tempdir().unwrap();
    let directory = home.path().join("acme-api");
    let server = Server::start("deleted-picker", 160, 20);
    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (DIR_PICKER, PICKER)],
        &[],
    );
    server.wait_for_screen("press n to start one");
    let shown = directory.canonicalize().unwrap();
    std::fs::remove_dir(&directory).unwrap();

    server.send_keys(&["n"]);

    let refusal = format!("Not a directory: {}", shown.display());
    let screen = server.wait_for_screen(&refusal);
    assert!(screen.contains(&refusal), "{screen}");
    assert!(screen.contains("No session yet"), "{screen}");
}

#[test]
#[ignore]
fn the_picker_files_live_in_a_directory_only_its_owner_can_open() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("pick-mode", 160, 20);
    server.run(
        home.path(),
        &[(SESSION_COMMAND, AGENT), (DIR_PICKER, PICKER)],
        &[],
    );
    server.wait_for_screen("press n to start one");

    server.send_keys(&["n"]);

    server.wait_for_screen("acme picker: type a directory");
    let picks: Vec<_> = std::fs::read_dir(home.path().join("tmp"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("harbour-pick")
        })
        .collect();
    assert_eq!(picks.len(), 1);
    let mode = std::fs::metadata(&picks[0]).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o700);
}

#[test]
#[ignore]
fn n_in_the_demo_lists_made_up_directories_in_fzf() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("demo-picker", 160, 20);
    let path = format!("{FAKE_FZF}:{}", tools(home.path()));
    server.run(home.path(), &[("PATH", &path)], &["--demo"]);
    server.wait_for_screen("acme-infra 5");
    server.send_keys(&["n"]);

    let screen = server.wait_for_screen_all(&["acme fzf lists:", "/home/acme/web"]);
    assert!(
        screen.contains("acme fzf args: --no-expect --no-print-query --no-read0 --no-print0"),
        "{screen}"
    );
    assert!(screen.contains("acme fzf lists:"), "{screen}");
    assert!(screen.contains("/home/acme/web"), "{screen}");
    let scratch = home
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    assert!(!screen.contains(&scratch), "{screen}");
}
