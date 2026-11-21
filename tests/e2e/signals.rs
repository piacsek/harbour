use std::fs::File;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use harbour::tui::SESSION_COMMAND;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use crate::log::wait_for_log;
use crate::{REGISTERED_AGENT, Server, WORKING, harbour_line, quoted, wait_for_status};

const FIFO_AGENT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fifo-agent.sh");

#[test]
#[ignore]
fn term_turns_the_terminal_modes_off_and_ends_harbour_by_term() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-term", 120, 20);
    let extended = extended_keys(&server);
    let pid = home.path().join("harbour.pid");
    server.respawn(&format!(
        "{}; exec sleep 600",
        harbour_line(home.path(), &ended(&pid), &[], &[])
    ));
    server.wait_for_screen("press n to start one");
    assert_eq!(format(&server, "#{mouse_any_flag}"), "1");
    if extended {
        assert_eq!(format(&server, "#{pane_key_mode}"), "Ext 2");
    }

    send(&pid, Signal::SIGTERM);

    let screen = server.wait_for_screen("harbour died of 15 exited 0");
    assert!(screen.contains("harbour died of 15 exited 0"), "{screen}");
    assert_eq!(format(&server, "#{mouse_any_flag}"), "0");
    if extended {
        assert_eq!(format(&server, "#{pane_key_mode}"), "VT10x");
    }
}

#[test]
#[ignore]
fn after_term_the_shell_gets_ctrl_c_as_its_plain_byte() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-ctrl-c", 120, 20);
    extended_keys(&server);
    let pid = home.path().join("harbour.pid");
    server.respawn(&format!(
        "{}; stty raw -echo; echo raw ready; dd bs=1 count=1 2>/dev/null | od -An -tx1; echo read; exec sleep 600",
        harbour_line(home.path(), &ended(&pid), &[], &[])
    ));
    server.wait_for_screen("press n to start one");
    send(&pid, Signal::SIGTERM);
    server.wait_for_screen("raw ready");

    server.send_keys(&["C-c"]);

    let screen = server.wait_for_screen("read");
    assert!(screen.contains(" 03"), "{screen}");
}

#[test]
#[ignore]
fn int_ends_harbour_as_a_quit_does_then_by_int() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-int", 160, 20);
    let pid = started(&server, home.path());

    send(&pid, Signal::SIGINT);

    let screen = server.wait_for_screen("harbour died of 2 exited 0");
    assert!(screen.contains("harbour died of 2 exited 0"), "{screen}");
    assert_eq!(left(home.path(), "status"), Vec::<String>::new());
    assert_eq!(left(home.path(), "edit"), Vec::<String>::new());
    let log = wait_for_log(home.path(), "quit sessions=1");
    assert!(log.contains(" signal name=INT\n"), "{log}");
}

#[test]
#[ignore]
fn hup_ends_harbour_as_a_quit_does_then_by_hup() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-hup", 160, 20);
    let pid = started(&server, home.path());

    send(&pid, Signal::SIGHUP);

    let screen = server.wait_for_screen("harbour died of 1 exited 0");
    assert!(screen.contains("harbour died of 1 exited 0"), "{screen}");
    assert_eq!(left(home.path(), "status"), Vec::<String>::new());
    assert_eq!(left(home.path(), "edit"), Vec::<String>::new());
}

#[test]
#[ignore]
fn two_hups_in_a_row_end_harbour_as_a_quit_does_then_by_hup() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-hup-hup", 160, 20);
    let pid = started(&server, home.path());

    send(&pid, Signal::SIGHUP);
    send(&pid, Signal::SIGHUP);

    let screen = server.wait_for_screen("harbour died of 1 exited 0");
    assert!(screen.contains("harbour died of 1 exited 0"), "{screen}");
    let log = wait_for_log(home.path(), "quit sessions=1");
    assert!(log.contains(" signal name=HUP\n"), "{log}");
}

#[test]
#[ignore]
fn closing_a_pane_whose_interactive_shell_runs_harbour_ends_it_as_a_quit_does() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-shell", 160, 20);
    let script = home.path().join("harbour.sh");
    std::fs::write(
        &script,
        harbour_line(home.path(), "", &[(SESSION_COMMAND, REGISTERED_AGENT)], &[]),
    )
    .unwrap();
    server.respawn("exec /bin/bash --norc --noprofile -i");
    server.send_keys(&[
        &format!("/bin/sh {}", quoted(&script.display().to_string())),
        "Enter",
    ]);
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    wait_for_status(home.path(), WORKING);

    server.respawn("exec sleep 600");

    let log = wait_for_log(home.path(), "quit sessions=1");
    assert!(log.contains("quit sessions=1"), "{log}");
    let status = wait_for_empty(home.path(), "status");
    assert_eq!(status, Vec::<String>::new());
}

#[test]
#[ignore]
fn closing_the_pane_ends_harbour_as_a_quit_does() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-closed", 160, 20);
    started(&server, home.path());

    server.respawn("exec sleep 600");

    let log = wait_for_log(home.path(), "quit sessions=1");
    assert!(log.contains("quit sessions=1"), "{log}");
    let status = wait_for_empty(home.path(), "status");
    let edit = wait_for_empty(home.path(), "edit");
    assert_eq!(status, Vec::<String>::new());
    assert_eq!(edit, Vec::<String>::new());
}

#[test]
#[ignore]
fn a_second_term_ends_a_harbour_whose_loop_is_held() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-held", 160, 20);
    let pid = home.path().join("harbour.pid");
    server.respawn(&format!(
        "{}; exec sleep 600",
        harbour_line(
            home.path(),
            &ended(&pid),
            &[(SESSION_COMMAND, FIFO_AGENT)],
            &[]
        )
    ));
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    let held = held(home.path());
    send(&pid, Signal::SIGTERM);
    sleep(Duration::from_millis(200));

    send(&pid, Signal::SIGTERM);

    let screen = server.wait_for_screen("harbour died of 15 exited 0");
    assert!(screen.contains("harbour died of 15 exited 0"), "{screen}");
    drop(held);
}

fn started(server: &Server, home: &Path) -> PathBuf {
    let pid = home.join("harbour.pid");
    server.respawn(&format!(
        "{}; exec sleep 600",
        harbour_line(
            home,
            &ended(&pid),
            &[(SESSION_COMMAND, REGISTERED_AGENT)],
            &[]
        )
    ));
    server.wait_for_screen("press n to start one");
    server.send_keys(&["n"]);
    wait_for_status(home, WORKING);
    assert_eq!(left(home, "status").len(), 2);
    assert_eq!(left(home, "edit").len(), 1);
    pid
}

fn extended_keys(server: &Server) -> bool {
    let options = [["extended-keys", "on"], ["extended-keys-format", "xterm"]];
    if let Some(option) = options.iter().find(|option| !server.server_option(*option)) {
        eprintln!(
            "not checking the key mode: this tmux cannot set {option:?}; extended-keys-format needs tmux 3.5"
        );
        return false;
    }
    true
}

fn ended(file: &Path) -> String {
    format!(
        "perl -e {} sh -c {} sh",
        quoted(r#"system @ARGV; print "harbour died of ", $? & 127, " exited ", $? >> 8, "\n""#),
        quoted(&format!(
            "echo \"$$ pid-end\" > {}; exec \"$@\"",
            quoted(&file.display().to_string())
        ))
    )
}

fn send(file: &Path, signal: Signal) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let pid = loop {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        if let Some(pid) = text.strip_suffix(" pid-end\n") {
            break pid.parse::<i32>().unwrap();
        }
        assert!(Instant::now() < deadline, "waited 10 s for {file:?}");
        sleep(Duration::from_millis(50));
    };
    kill(Pid::from_raw(pid), signal).unwrap();
}

fn held(home: &Path) -> File {
    let dir = home.join(".claude/sessions");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let fifo = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .next();
        let opened = fifo.and_then(|fifo| {
            std::fs::OpenOptions::new()
                .write(true)
                .custom_flags(nix::libc::O_NONBLOCK)
                .open(fifo)
                .ok()
        });
        if let Some(file) = opened {
            return file;
        }
        assert!(
            Instant::now() < deadline,
            "waited 10 s for harbour to open a registry file in {dir:?}"
        );
        sleep(Duration::from_millis(50));
    }
}

fn format(server: &Server, format: &str) -> String {
    let output = server
        .tmux(&["display-message", "-p", "-t", "e2e", format])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string()
}

fn left(home: &Path, folder: &str) -> Vec<String> {
    let dir = home.join(".local/state/harbour").join(folder);
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

pub fn wait_for_empty(home: &Path, folder: &str) -> Vec<String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let names = left(home, folder);
        if names.is_empty() {
            return names;
        }
        assert!(
            Instant::now() < deadline,
            "waited 10 s for an empty {folder} folder, last held: {names:?}"
        );
        sleep(Duration::from_millis(50));
    }
}
