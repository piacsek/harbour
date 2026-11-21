use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use harbour::tui::SESSION_COMMAND;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use crate::{REGISTERED_AGENT, Server, WORKING, harbour_line, quoted, wait_for_status};

#[test]
#[ignore]
fn term_turns_the_terminal_modes_off_and_exits_143() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-term", 120, 20);
    if !extended_keys(&server) {
        return;
    }
    let pid = home.path().join("harbour.pid");
    server.respawn(&format!(
        "{}; echo \"harbour exited $?\"; stty raw -echo; dd bs=1 count=1 2>/dev/null | od -An -tx1; exec sleep 600",
        harbour_line(home.path(), &pid_written(&pid), &[], &[])
    ));
    server.wait_for_screen("press n to start one");
    assert_eq!(modes(&server), "Ext 2|1");

    send(&pid, Signal::SIGTERM);

    let screen = server.wait_for_screen("harbour exited 143");
    assert!(screen.contains("harbour exited 143"), "{screen}");
    assert_eq!(modes(&server), "VT10x|0");
}

#[test]
#[ignore]
fn after_term_the_shell_gets_ctrl_c_as_its_plain_byte() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-ctrl-c", 120, 20);
    if !extended_keys(&server) {
        return;
    }
    let pid = home.path().join("harbour.pid");
    server.respawn(&format!(
        "{}; echo \"harbour exited $?\"; stty raw -echo; dd bs=1 count=1 2>/dev/null | od -An -tx1; echo read; exec sleep 600",
        harbour_line(home.path(), &pid_written(&pid), &[], &[])
    ));
    server.wait_for_screen("press n to start one");
    send(&pid, Signal::SIGTERM);
    server.wait_for_screen("harbour exited 143");

    server.send_keys(&["C-c"]);

    let screen = server.wait_for_screen("read");
    assert!(screen.contains(" 03"), "{screen}");
}

#[test]
#[ignore]
fn int_ends_harbour_as_a_quit_does_and_exits_130() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-int", 160, 20);
    let pid = started(&server, home.path());

    send(&pid, Signal::SIGINT);

    let screen = server.wait_for_screen("harbour exited 130");
    assert!(screen.contains("harbour exited 130"), "{screen}");
    assert_eq!(left(home.path(), "status"), Vec::<String>::new());
    assert_eq!(left(home.path(), "edit"), Vec::<String>::new());
}

#[test]
#[ignore]
fn hup_ends_harbour_as_a_quit_does_and_exits_129() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-hup", 160, 20);
    let pid = started(&server, home.path());

    send(&pid, Signal::SIGHUP);

    let screen = server.wait_for_screen("harbour exited 129");
    assert!(screen.contains("harbour exited 129"), "{screen}");
    assert_eq!(left(home.path(), "status"), Vec::<String>::new());
    assert_eq!(left(home.path(), "edit"), Vec::<String>::new());
}

#[test]
#[ignore]
fn closing_the_pane_ends_harbour_as_a_quit_does() {
    let home = tempfile::tempdir().unwrap();
    let server = Server::start("signal-closed", 160, 20);
    started(&server, home.path());

    server.respawn("exec sleep 600");

    let status = wait_for_empty(home.path(), "status");
    let edit = wait_for_empty(home.path(), "edit");
    assert_eq!(status, Vec::<String>::new());
    assert_eq!(edit, Vec::<String>::new());
}

fn started(server: &Server, home: &Path) -> std::path::PathBuf {
    let pid = home.join("harbour.pid");
    server.respawn(&format!(
        "{}; echo \"harbour exited $?\"; exec sleep 600",
        harbour_line(
            home,
            &pid_written(&pid),
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
        eprintln!("skipping: this tmux cannot set {option:?}; extended-keys-format needs tmux 3.5");
        return false;
    }
    true
}

fn pid_written(file: &Path) -> String {
    format!(
        "sh -c {} sh",
        quoted(&format!(
            "echo $$ > {}; exec \"$@\"",
            quoted(&file.display().to_string())
        ))
    )
}

fn send(file: &Path, signal: Signal) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let pid = loop {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        if let Ok(pid) = text.trim().parse::<i32>() {
            break pid;
        }
        assert!(Instant::now() < deadline, "waited 10 s for {file:?}");
        sleep(Duration::from_millis(50));
    };
    kill(Pid::from_raw(pid), signal).unwrap();
}

fn modes(server: &Server) -> String {
    let output = server
        .tmux(&[
            "display-message",
            "-p",
            "-t",
            "e2e",
            "#{pane_key_mode}|#{mouse_any_flag}",
        ])
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
