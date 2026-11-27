use std::io;
use std::path::Path;

use harbour::session::{Role, Session, Transport};

use crate::{COLS, ROWS, alive, pid_shown, still_alive_after_a_while, wait_for};

#[test]
fn a_piped_session_shows_what_its_command_prints() {
    let session = piped("echo hello from acme; sleep 5");

    let screen = wait_for(&session, "hello from acme");

    assert!(screen.contains("hello from acme"), "{screen}");
}

#[test]
fn a_piped_session_shows_what_its_command_prints_on_stderr() {
    let session = piped("echo acme went wrong >&2; sleep 5");

    let screen = wait_for(&session, "acme went wrong");

    assert!(screen.contains("acme went wrong"), "{screen}");
}

#[test]
fn a_piped_session_starts_each_new_line_at_the_left_edge() {
    let session = piped("printf 'acme one\\nacme two\\n'; sleep 5");

    let screen = wait_for(&session, "acme two");

    assert!(
        screen.lines().nth(1).unwrap().starts_with("acme two"),
        "{screen}"
    );
}

#[test]
fn what_is_sent_to_a_piped_session_reaches_its_input() {
    let session = piped("echo ready; read -r line; echo \"got:$line\"; sleep 5");
    wait_for(&session, "ready");

    session.send(b"acme-api\n");

    let screen = wait_for(&session, "got:acme-api");
    assert!(screen.contains("got:acme-api"), "{screen}");
}

#[test]
fn a_piped_session_gives_its_exit_status() {
    let mut session = piped("echo bye; exit 3");
    wait_for(&session, "bye");

    let code = exited(&mut session);

    assert_eq!(code, 3);
}

#[test]
fn a_piped_session_gives_the_pid_of_its_command() {
    let session = piped("echo \"pid:$$ shown\"; sleep 5");

    let pid = pid_shown(&session);

    assert_eq!(session.pid().unwrap().to_string(), pid);
}

#[test]
fn ending_a_piped_session_ends_what_its_command_started() {
    let mut session = piped("sleep 600 & echo \"pid:$! shown\"; exec sleep 600");
    let pid = pid_shown(&session);
    assert!(alive(&pid));

    session.end();

    assert!(!still_alive_after_a_while(&pid));
}

#[test]
fn a_piped_session_is_drained_once_its_output_is_read_to_its_end() {
    let session = piped("echo acme done");
    wait_for(&session, "acme done");

    let drained = drained_soon(&session);

    assert!(drained);
}

#[test]
fn a_piped_session_in_a_directory_that_is_gone_starts_nothing_and_is_named() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path().join("acme-gone");

    let refused = piped_in("sleep 5", &directory).err().unwrap();

    assert_eq!(
        refused.to_string(),
        format!("Not a directory: {}", directory.display())
    );
}

#[test]
fn a_piped_session_of_a_missing_program_says_to_install_it() {
    let command = ["acme-missing-agent".to_string()];

    let refused = Session::start_over(
        Transport::Pipes,
        Role::Agent,
        &command,
        &std::env::temp_dir(),
        ROWS,
        COLS,
    )
    .err()
    .unwrap();

    assert_eq!(refused.to_string(), "is it installed and on PATH?");
}

fn piped(script: &str) -> Session {
    piped_in(script, &std::env::temp_dir()).unwrap()
}

fn piped_in(script: &str, directory: &Path) -> io::Result<Session> {
    let command = ["/bin/sh".to_string(), "-c".into(), script.into()];
    Session::start_over(
        Transport::Pipes,
        Role::Agent,
        &command,
        directory,
        ROWS,
        COLS,
    )
}

fn exited(session: &mut Session) -> u32 {
    let deadline = std::time::Instant::now() + 5 * crate::deadline();
    loop {
        if let Some(status) = session.exit() {
            return status.exit_code();
        }
        assert!(
            std::time::Instant::now() < deadline,
            "the session never exited"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

fn drained_soon(session: &Session) -> bool {
    let deadline = std::time::Instant::now() + 5 * crate::deadline();
    while !session.drained() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    session.drained()
}
