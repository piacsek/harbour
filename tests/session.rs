use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::{Duration, Instant};

use harbour::session::Session;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use vt100::{MouseProtocolEncoding, MouseProtocolMode};

const ROWS: u16 = 10;
const COLS: u16 = 200;

#[test]
fn the_pane_shows_what_the_command_prints() {
    let session = start("echo hello from acme; sleep 5");
    let screen = wait_for(&session, "hello from acme");
    assert!(screen.contains("hello from acme"), "{screen}");
}

#[test]
fn the_command_starts_in_the_directory_it_is_given() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path().join("acme-web");
    std::fs::create_dir(&directory).unwrap();
    let command = [
        "/bin/sh".to_string(),
        "-c".into(),
        "echo \"in:${PWD##*/}\"; sleep 5".into(),
    ];
    let session = Session::start(&command, &directory, ROWS, COLS).unwrap();
    let screen = wait_for(&session, "in:acme-web");
    assert!(screen.contains("in:acme-web"), "{screen}");
}

#[test]
fn a_directory_that_is_gone_starts_nothing_and_is_named() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path().join("acme-gone");
    let command = ["/bin/sh".to_string(), "-c".into(), "sleep 5".into()];

    let refused = Session::start(&command, &directory, ROWS, COLS)
        .err()
        .unwrap();

    assert_eq!(
        refused.to_string(),
        format!("Not a directory: {}", directory.display())
    );
}

#[test]
fn the_command_sees_its_directory_in_pwd() {
    let scratch = tempfile::tempdir().unwrap();
    let directory = scratch.path().join("acme-web");
    std::fs::create_dir(&directory).unwrap();
    let command = ["/usr/bin/printenv".to_string(), "PWD".into()];
    let session = Session::start(&command, &directory, ROWS, COLS).unwrap();
    let screen = wait_for(&session, "/acme-web");
    assert!(screen.contains("/acme-web"), "{screen}");
}

#[test]
fn the_command_is_told_it_runs_in_a_truecolor_xterm() {
    let session = start("echo \"term:$TERM/$COLORTERM\"; sleep 5");
    let expected = "term:xterm-256color/truecolor";
    let screen = wait_for(&session, expected);
    assert!(screen.contains(expected), "{screen}");
}

#[test]
fn a_device_attributes_query_is_answered() {
    let session = start(&replied("\\033[c", 9));
    let expected = "reply:[?62;22c";
    let screen = wait_for(&session, expected);
    assert!(screen.contains(expected), "{screen}");
}

#[test]
fn a_secondary_device_attributes_query_is_answered() {
    let session = start(&replied("\\033[>c", 10));
    let expected = "reply:[>0;10;1c";
    let screen = wait_for(&session, expected);
    assert!(screen.contains(expected), "{screen}");
}

#[test]
fn a_cursor_position_query_is_answered_with_the_position() {
    let session = start(&replied("ab\\033[6n", 6));
    let expected = "reply:[1;3R";
    let screen = wait_for(&session, expected);
    assert!(screen.contains(expected), "{screen}");
}

#[test]
fn a_status_query_is_answered_with_ok() {
    let session = start(&replied("\\033[5n", 4));
    let expected = "reply:[0n";
    let screen = wait_for(&session, expected);
    assert!(screen.contains(expected), "{screen}");
}

#[test]
fn a_foreground_colour_query_is_answered() {
    let session = start(&replied("\\033]10;?\\033\\\\", 25));
    let expected = "reply:]10;rgb:d0d0/d0d0/d0d0";
    let screen = wait_for(&session, expected);
    assert!(screen.contains(expected), "{screen}");
}

#[test]
fn a_background_colour_query_is_answered() {
    let session = start(&replied("\\033]11;?\\033\\\\", 25));
    let expected = "reply:]11;rgb:1c1c/1c1c/1c1c";
    let screen = wait_for(&session, expected);
    assert!(screen.contains(expected), "{screen}");
}

#[test]
fn a_resize_tells_the_command_its_new_size() {
    let mut session =
        start("trap 'echo \"size:$(stty size)\"' WINCH; echo ready; while :; do sleep 0.1; done");
    wait_for(&session, "ready");

    session.resize(8, 150);

    let screen = wait_for(&session, "size:8 150");
    assert!(screen.contains("size:8 150"), "{screen}");
}

#[test]
fn a_resize_to_nothing_keeps_the_last_size() {
    let mut session = start("echo ready; sleep 5");
    wait_for(&session, "ready");

    session.resize(0, 0);
    session.resize(ROWS, COLS);

    let screen = wait_for(&session, "ready");
    assert!(screen.contains("ready"), "{screen}");
}

#[test]
fn a_session_started_with_no_room_still_runs() {
    let command = [
        "/bin/sh".to_string(),
        "-c".into(),
        "echo ready; sleep 5".into(),
    ];
    let mut session = Session::start(&command, &here(), 0, 0).unwrap();

    session.resize(ROWS, COLS);

    let screen = wait_for(&session, "ready");
    assert!(screen.contains("ready"), "{screen}");
}

#[test]
fn a_session_started_one_row_high_survives_a_wrapped_line() {
    let command = [
        "/bin/sh".to_string(),
        "-c".into(),
        "printf 'acme-api wraps'; echo; sleep 1; echo ready; sleep 5".into(),
    ];
    let mut session = Session::start(&command, &here(), 1, 4).unwrap();
    sleep(Duration::from_millis(300));

    session.resize(ROWS, COLS);

    let screen = wait_for(&session, "ready");
    assert!(screen.contains("ready"), "{screen}");
}

#[test]
fn a_session_resized_to_one_row_survives_a_wrapped_line() {
    let mut session =
        start("sleep 0.3; printf 'acme-api wraps'; echo; sleep 1; echo ready; sleep 5");
    session.resize(1, 4);
    sleep(Duration::from_millis(600));

    session.resize(ROWS, COLS);

    let screen = wait_for(&session, "ready");
    assert!(screen.contains("ready"), "{screen}");
}

#[test]
fn a_session_started_one_column_wide_survives_a_wide_character() {
    let command = [
        "/bin/sh".to_string(),
        "-c".into(),
        "printf '\\344\\270\\226'; echo; sleep 1; echo ready; sleep 5".into(),
    ];
    let mut session = Session::start(&command, &here(), 10, 1).unwrap();
    sleep(Duration::from_millis(300));

    session.resize(ROWS, COLS);

    let screen = wait_for(&session, "ready");
    assert!(screen.contains("ready"), "{screen}");
}

#[test]
fn a_session_resized_to_one_column_survives_a_wide_character() {
    let mut session =
        start("sleep 0.3; printf '\\344\\270\\226'; echo; sleep 1; echo ready; sleep 5");
    session.resize(ROWS, 1);
    sleep(Duration::from_millis(600));

    session.resize(ROWS, COLS);

    let screen = wait_for(&session, "ready");
    assert!(screen.contains("ready"), "{screen}");
}

#[test]
fn a_session_survives_a_wide_character_cut_by_a_resize() {
    let mut session = Session::start(&wide_at_the_edge(), &here(), ROWS, 160).unwrap();
    wait_for(&session, "ready");
    session.resize(ROWS, 159);

    session.send(b"\r");
    sleep(Duration::from_millis(300));
    session.send(b"\r");

    let screen = wait_for(&session, "redrew 159");
    assert!(screen.contains("redrew 159"), "{screen}");
}

#[test]
fn a_session_whose_screen_was_reset_is_asked_to_repaint() {
    let mut session = Session::start(&wide_at_the_edge(), &here(), ROWS, 160).unwrap();
    wait_for(&session, "ready");
    session.resize(ROWS, 159);
    wait_for(&session, "repainted");
    session.send(b"\r");
    sleep(Duration::from_millis(300));

    session.resize(ROWS, 159);

    let screen = wait_for(&session, "repainted");
    assert!(screen.contains("repainted"), "{screen}");
    assert!(!screen.contains("ready"), "{screen}");
}

#[test]
fn a_session_whose_screen_was_reset_keeps_the_modes_its_program_turned_on() {
    let mut session = Session::start(&moded_wide_at_the_edge(), &here(), ROWS, 160).unwrap();
    wait_for(&session, "ready");
    session.resize(ROWS, 159);
    session.send(b"x");
    sleep(Duration::from_millis(300));

    session.paste("P1\rP2");

    let screen = wait_for(&session, "^[200~P1/P2^[201~");
    assert!(screen.contains("^[200~P1/P2^[201~"), "{screen}");
    assert!(!screen.contains('█'), "{screen}");
    assert_eq!(
        session.mouse_protocol(),
        (MouseProtocolMode::PressRelease, MouseProtocolEncoding::Sgr)
    );
}

#[test]
fn dropping_a_session_ends_a_command_that_ignores_hangup() {
    let session = start("trap '' HUP; echo \"pid:$$ shown\"; while :; do sleep 1; done");
    let pid = pid_shown(&session);

    drop(session);

    assert!(!alive(&pid), "{pid} is still running");
}

#[test]
fn dropping_a_session_ends_what_its_command_started() {
    let session = start("(while :; do sleep 1; done) & echo \"pid:$! shown\"; wait");
    let pid = pid_shown(&session);

    drop(session);

    assert!(!still_alive_after_a_while(&pid), "{pid} is still running");
}

#[test]
fn ending_a_session_ends_what_its_command_started_even_when_it_ignores_hangup() {
    let mut session = start("(trap '' HUP; exec sleep 7771) & echo \"pid:$! shown\"; wait");
    let pid = pid_shown(&session);

    session.end();

    assert!(!still_alive_after_a_while(&pid), "{pid} is still running");
}

#[test]
fn dropping_a_session_ends_what_its_command_started_even_when_it_ignores_hangup() {
    let session = start("(trap '' HUP; exec sleep 7772) & echo \"pid:$! shown\"; wait");
    let pid = pid_shown(&session);

    drop(session);

    assert!(!still_alive_after_a_while(&pid), "{pid} is still running");
}

#[test]
fn a_large_paste_into_a_command_that_does_not_read_returns_at_once() {
    let session = start("stty raw -echo; echo raw; exec sleep 600");
    wait_for(&session, "raw");
    let (done, finished) = mpsc::channel();

    thread::spawn(move || {
        session.paste(&"acme ".repeat(20_000));
        let _ = done.send(session.application_cursor());
    });

    assert!(finished.recv_timeout(Duration::from_secs(2)).is_ok());
}

fn replied(query: &str, length: usize) -> String {
    format!(
        "stty raw -echo; printf '{query}'; r=$(dd bs=1 count={length} 2>/dev/null); \
         printf 'reply:%s' \"$r\" | tr -d '\\033'; sleep 5"
    )
}

fn wide_at_the_edge() -> [String; 3] {
    let script = "trap 'printf \"\\033[5;1Hrepainted\"' WINCH; \
                  cols=$(stty size | cut -d' ' -f2); \
                  printf '\\033[1;%dH\\343\\201\\202' $((cols-1)); \
                  printf '\\033[3;1Hready'; \
                  while :; do \
                  read -r line || continue; \
                  cols=$(stty size | cut -d' ' -f2); \
                  printf '\\033[1;1H%*s' \"$cols\" ''; \
                  printf '\\033[4;1Hredrew %s' \"$cols\"; \
                  done";
    ["/bin/sh".to_string(), "-c".into(), script.into()]
}

fn moded_wide_at_the_edge() -> [String; 3] {
    let script = "printf '\\033[?2004h\\033[?1000h\\033[?1006h\\033[?25l'; \
                  trap 'printf \"\\033[1;1H%*s\" \"$(stty size | cut -d\" \" -f2)\" \"\"' WINCH; \
                  cols=$(stty size | cut -d' ' -f2); \
                  printf '\\033[1;%dH\\343\\201\\202' $((cols-1)); \
                  printf '\\033[3;1Hready\\033[5;1H'; \
                  stty raw -echo; \
                  while :; do dd bs=1 count=1 2>/dev/null | tr '\\033\\r' '^/'; done";
    ["/bin/sh".to_string(), "-c".into(), script.into()]
}

fn start(script: &str) -> Session {
    let command = ["/bin/sh".to_string(), "-c".into(), script.into()];
    Session::start(&command, &here(), ROWS, COLS).unwrap()
}

fn wait_for(session: &Session, text: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let screen = rendered(session);
        if screen.contains(text) || Instant::now() > deadline {
            return screen;
        }
        sleep(Duration::from_millis(20));
    }
}

fn rendered(session: &Session) -> String {
    let area = Rect::new(0, 0, COLS, ROWS);
    let mut buffer = Buffer::empty(area);
    session.render(area, &mut buffer);
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn pid_shown(session: &Session) -> String {
    let screen = wait_for(session, " shown");
    screen
        .split("pid:")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap()
        .to_string()
}

fn alive(pid: &str) -> bool {
    Command::new("kill")
        .args(["-0", pid])
        .status()
        .unwrap()
        .success()
}

fn still_alive_after_a_while(pid: &str) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while alive(pid) && Instant::now() < deadline {
        sleep(Duration::from_millis(50));
    }
    alive(pid)
}

fn here() -> PathBuf {
    std::env::current_dir().unwrap()
}
