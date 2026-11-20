use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::editor::Program;
use rmpv::Value;

use crate::{Screen, reading_agent, script};

const LEFTMOST_IN_NORMAL_MODE: &str = "mode() ==# 'n' && winnr('h') == winnr()";

#[test]
fn neovim_is_told_to_listen_on_a_socket() {
    let scratch = tempfile::tempdir().unwrap();

    open(scratch.path(), "exec sleep 600");

    let flag = fs::read_to_string(scratch.path().join("flag")).unwrap();
    assert_eq!(flag.trim_end(), "--listen");
}

#[test]
fn the_socket_lives_in_a_directory_only_its_user_can_enter() {
    let scratch = tempfile::tempdir().unwrap();

    let _screen = open(scratch.path(), "exec sleep 600");

    let socket = socket(scratch.path());
    let mode = fs::metadata(socket.parent().unwrap())
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o700);
}

#[test]
fn ctrl_h_asks_neovim_whether_its_leftmost_window_is_in_normal_mode() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");
    let asked = answering(&socket(scratch.path()), Some(1));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let request = asked.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(request, format!("nvim_eval {LEFTMOST_IN_NORMAL_MODE}"));
}

#[test]
fn ctrl_h_in_the_leftmost_window_in_normal_mode_shows_claude_code() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");
    let _asked = answering(&socket(scratch.path()), Some(1));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
    assert!(!shown.contains("acme nvim read"), "{shown}");
}

#[test]
fn ctrl_h_anywhere_else_in_neovim_reaches_neovim() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");
    let _asked = answering(&socket(scratch.path()), Some(0));

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme nvim read:08");
    assert!(shown.contains("acme nvim read:08"), "{shown}");
}

#[test]
fn ctrl_h_shows_claude_code_when_nothing_listens_on_the_socket() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exec sleep 600");

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
}

#[test]
fn ctrl_h_shows_claude_code_soon_when_neovim_never_answers() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open_with(fake_neovim(scratch.path(), "exec sleep 600"));
    let _asked = answering(&socket(scratch.path()), None);

    let before = Instant::now();
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    let waited = before.elapsed();

    let shown = screen.wait_for("acme agent ready");
    assert!(shown.contains("acme agent ready"), "{shown}");
    assert!(waited < Duration::from_millis(500), "{waited:?}");
}

#[test]
fn the_socket_directory_is_removed_once_neovim_exits() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = open(scratch.path(), "exit 0");
    let socket = socket(scratch.path());
    screen.press('q');

    screen.wait_for("acme agent ready");

    assert!(!socket.parent().unwrap().exists());
}

fn open(scratch: &Path, then: &str) -> Screen {
    open_with(fake_neovim(scratch, then).with_patience(Duration::from_secs(5)))
}

fn open_with(neovim: Program) -> Screen {
    let mut screen = Screen::editing(reading_agent(), neovim);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme nvim ready");
    screen
}

fn fake_neovim(scratch: &Path, then: &str) -> Program {
    let scratch = scratch.display();
    Program::neovim(
        "nvim",
        script(&format!(
            "printf '%s\\n' \"$0\" > '{scratch}/flag'; printf '%s\\n' \"$1\" > '{scratch}/socket'; \
             stty raw -echo; echo 'acme nvim ready'; \
             r=$(dd bs=1 count=1 2>/dev/null | od -An -tx1 | tr -d ' \\n'); \
             printf 'acme nvim read:%s' \"$r\"; {then}"
        )),
    )
}

fn socket(scratch: &Path) -> PathBuf {
    PathBuf::from(
        fs::read_to_string(scratch.join("socket"))
            .unwrap()
            .trim_end(),
    )
}

fn answering(socket: &Path, answer: Option<i64>) -> Receiver<String> {
    let listener = UnixListener::bind(socket).unwrap();
    let (asked, requests) = mpsc::channel();
    thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let request = rmpv::decode::read_value(&mut stream).unwrap();
        let fields = request.as_array().unwrap().clone();
        let method = fields[2].as_str().unwrap().to_string();
        let expression = fields[3][0].as_str().unwrap().to_string();
        let _ = asked.send(format!("{method} {expression}"));
        match answer {
            Some(answer) => {
                let reply = Value::Array(vec![
                    Value::from(1),
                    fields[1].clone(),
                    Value::Nil,
                    Value::from(answer),
                ]);
                rmpv::encode::write_value(&mut stream, &reply).unwrap();
            }
            None => thread::sleep(Duration::from_secs(5)),
        }
    });
    requests
}
