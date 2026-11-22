use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::cursor::Shape;

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

fn focused(agent: Vec<String>) -> Screen {
    let mut screen = Screen::new(agent);
    screen.press('n');
    screen.wait_for("acme agent shaped");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for("┌ sessions");
    screen
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
