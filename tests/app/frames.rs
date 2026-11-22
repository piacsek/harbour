use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::cursor::Shape;
use ratatui::layout::Position;

use crate::{Screen, frames, script};

#[test]
fn a_frame_reaches_the_terminal_in_one_write_and_one_flush() {
    let mut screen = focused(shaped_agent());
    let out = Writes::default();
    let mut frames = frames();

    frames
        .draw(&mut screen.app, &mut Shape::default(), &mut out.clone())
        .unwrap();

    assert_eq!(out.writes(), 1);
    assert_eq!(out.flushes(), 1);
}

#[test]
fn a_frame_that_changes_nothing_writes_nothing() {
    let mut screen = focused(shaped_agent());
    let out = Writes::default();
    let mut frames = frames();
    let mut shape = Shape::default();
    frames
        .draw(&mut screen.app, &mut shape, &mut out.clone())
        .unwrap();
    out.clear();

    frames
        .draw(&mut screen.app, &mut shape, &mut out.clone())
        .unwrap();

    assert_eq!(out.writes(), 0);
    assert_eq!(out.flushes(), 0);
}

#[test]
fn a_frame_where_only_the_cursor_moves_writes_the_move() {
    let mut screen = focused(moving_agent());
    let out = Writes::default();
    let mut frames = frames();
    let mut shape = Shape::default();
    frames
        .draw(&mut screen.app, &mut shape, &mut out.clone())
        .unwrap();
    let before = screen.cursor();
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let moved = cursor_changed(&mut screen, before);
    out.clear();

    frames
        .draw(&mut screen.app, &mut shape, &mut out.clone())
        .unwrap();

    let text = out.text();
    let at = moved.unwrap();
    let moved_to = format!("\x1b[{};{}H\x1b[?25h", at.y + 1, at.x + 1);
    assert_eq!(out.writes(), 1);
    assert!(text.contains(&moved_to), "{text:?}");
}

#[test]
fn a_frame_where_only_the_cursor_goes_hidden_writes_the_hide() {
    let mut screen = focused(hiding_agent());
    let out = Writes::default();
    let mut frames = frames();
    let mut shape = Shape::default();
    frames
        .draw(&mut screen.app, &mut shape, &mut out.clone())
        .unwrap();
    let before = screen.cursor();
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    cursor_changed(&mut screen, before);
    out.clear();

    frames
        .draw(&mut screen.app, &mut shape, &mut out.clone())
        .unwrap();

    let text = out.text();
    assert_eq!(out.writes(), 1);
    assert!(text.starts_with("\x1b[?2026h\x1b[?25l"), "{text:?}");
    assert!(!text.contains('H'), "{text:?}");
    assert!(!text.contains("\x1b[?25h"), "{text:?}");
}

fn focused(agent: Vec<String>) -> Screen {
    let mut screen = Screen::new(agent);
    screen.press('n');
    screen.wait_for("acme agent shaped");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for("┌ sessions");
    screen
}

fn cursor_changed(screen: &mut Screen, before: Option<Position>) -> Option<Position> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        screen.draw();
        let now = screen.cursor();
        if now != before {
            return now;
        }
        assert!(
            Instant::now() < deadline,
            "waited 5 s for the cursor to leave {before:?}"
        );
        sleep(Duration::from_millis(20));
    }
}

fn moving_agent() -> Vec<String> {
    script(
        "stty -echo; printf '\\033[3;5Hacme agent shaped'; read -r line; printf '\\033[6;9H'; exec sleep 600",
    )
}

fn hiding_agent() -> Vec<String> {
    script(
        "stty -echo; printf '\\033[3;5Hacme agent shaped'; read -r line; printf '\\033[?25l'; exec sleep 600",
    )
}

fn shaped_agent() -> Vec<String> {
    script("printf '\\033[6 q\\033[3;5Hacme agent shaped'; exec sleep 600")
}

#[derive(Clone, Default)]
struct Writes(Rc<RefCell<Recorded>>);

#[derive(Default)]
struct Recorded {
    writes: Vec<Vec<u8>>,
    flushes: usize,
}

impl Writes {
    fn clear(&self) {
        *self.0.borrow_mut() = Recorded::default();
    }

    fn text(&self) -> String {
        let written = self.0.borrow().writes.concat();
        String::from_utf8_lossy(&written).into_owned()
    }

    fn writes(&self) -> usize {
        self.0.borrow().writes.len()
    }

    fn flushes(&self) -> usize {
        self.0.borrow().flushes
    }
}

impl io::Write for Writes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().writes.push(bytes.to_vec());
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.borrow_mut().flushes += 1;
        Ok(())
    }
}
