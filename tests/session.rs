use std::thread::sleep;
use std::time::{Duration, Instant};

use harbour::session::Session;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

const ROWS: u16 = 10;
const COLS: u16 = 200;

#[test]
fn the_pane_shows_what_the_command_prints() {
    let session = start("echo hello from acme; sleep 5");
    let screen = wait_for(&session, "hello from acme");
    assert!(screen.contains("hello from acme"), "{screen}");
}

#[test]
fn the_command_starts_in_the_working_directory_of_harbour() {
    let session = start("echo \"in:$(pwd -P)\"; sleep 5");
    let here = std::env::current_dir().unwrap().canonicalize().unwrap();
    let expected = format!("in:{}", here.display());
    let screen = wait_for(&session, &expected);
    assert!(screen.contains(&expected), "{screen}");
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

fn replied(query: &str, length: usize) -> String {
    format!(
        "stty raw -echo; printf '{query}'; r=$(dd bs=1 count={length} 2>/dev/null); \
         printf 'reply:%s' \"$r\" | tr -d '\\033'; sleep 5"
    )
}

fn start(script: &str) -> Session {
    let command = ["/bin/sh".to_string(), "-c".into(), script.into()];
    Session::start(&command, ROWS, COLS).unwrap()
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
