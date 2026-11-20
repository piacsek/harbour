use std::fs;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::memory::{Memory, Remembered};
use ratatui::style::Modifier;
use tempfile::TempDir;

use crate::{Screen, below, script};

const ID: &str = "0f1e2d3c-4b5a-6978-8a9b-0c1d2e3f4a5b";
const OTHER_ID: &str = "9a8b7c6d-5e4f-4a3b-2c1d-0e9f8a7b6c5d";
const TITLE: &str = "Fix the flaky login test";

#[test]
fn a_remembered_session_comes_back_as_a_stopped_card() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.screen(started_agent());

    let shown = screen.wait_for(TITLE);

    assert_eq!(below(&shown, TITLE), "stopped");
    assert!(!shown.contains("acme agent started"), "{shown}");
}

#[test]
fn a_stopped_card_says_stopped_dimmed() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.screen(started_agent());

    let shown = screen.wait_for("stopped");
    let row = shown
        .lines()
        .position(|line| line.contains("stopped"))
        .unwrap();
    let column = shown.lines().nth(row).unwrap().find("stopped").unwrap();

    assert!(
        screen
            .cell(u16::try_from(column).unwrap(), u16::try_from(row).unwrap())
            .modifier
            .contains(Modifier::DIM)
    );
}

#[test]
fn the_pane_of_a_stopped_card_says_how_to_resume_it() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.screen(started_agent());

    let shown = screen.wait_for("Enter resumes it");

    assert!(
        shown.contains("Stopped. Enter resumes it, x forgets it."),
        "{shown}"
    );
}

#[test]
fn stopped_cards_come_back_in_their_order() {
    let scratch = Scratch::new();
    scratch.remember(OTHER_ID, "The second one", 1);
    scratch.remember(ID, "The first one", 0);
    let mut screen = scratch.screen(started_agent());

    let shown = screen.wait_for_all(&["The first one", "The second one"]);

    assert!(
        shown.find("The first one").unwrap() < shown.find("The second one").unwrap(),
        "{shown}"
    );
}

#[test]
fn enter_resumes_a_stopped_card_in_its_directory() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.screen(resuming_agent());

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for(&format!("acme agent --resume {ID} in acme-api"));
    assert!(
        shown.contains(&format!("acme agent --resume {ID} in acme-api")),
        "{shown}"
    );
}

#[test]
fn ctrl_l_resumes_a_stopped_card_and_focuses_its_pane() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.screen(resuming_agent());

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let ready = screen.wait_for("acme agent --resume");
    assert!(ready.contains("acme agent --resume"), "{ready}");
    screen.type_text("hello\r");

    let shown = screen.wait_for("acme agent got: hello");
    assert!(shown.contains("acme agent got: hello"), "{shown}");
}

#[test]
fn a_resumed_card_is_no_longer_stopped() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.screen(resuming_agent());

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("acme agent --resume");
    assert!(shown.contains("acme agent --resume"), "{shown}");
    assert!(!shown.contains("stopped"), "{shown}");
}

#[test]
fn a_resumed_session_that_exits_resumes_again_with_enter() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.screen(numbered_resuming_agent(scratch.dir.path()));
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let exited = screen.wait_for_all(&["acme agent run 1", "exited"]);
    assert!(exited.contains("acme agent run 1"), "{exited}");
    assert!(exited.contains("exited"), "{exited}");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("acme agent run 2");
    assert!(shown.contains("acme agent run 2"), "{shown}");
}

#[test]
fn x_forgets_a_stopped_card() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.screen(started_agent());
    let before = screen.wait_for(TITLE);
    assert!(before.contains(TITLE), "{before}");

    screen.press('x');

    let shown = screen.draw();
    assert!(!shown.contains(TITLE), "{shown}");
    assert_eq!(scratch.memory().recall(), Vec::new());
}

#[test]
fn x_on_a_stopped_card_forgets_only_that_one() {
    let scratch = Scratch::new();
    scratch.remember(ID, "The first one", 0);
    scratch.remember(OTHER_ID, "The second one", 1);
    let mut screen = scratch.screen(started_agent());
    let before = screen.wait_for("The second one");
    assert!(before.contains("The second one"), "{before}");

    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains("The second one"), "{shown}");
    let kept = scratch.memory().recall();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].session_id, OTHER_ID);
}

#[test]
fn n_still_starts_a_new_session_beside_stopped_cards() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.screen(started_agent());

    screen.press('n');

    let shown = screen.wait_for("acme agent started");
    assert!(shown.contains("acme agent started"), "{shown}");
    assert!(shown.contains(TITLE), "{shown}");
}

struct Scratch {
    dir: TempDir,
}

impl Scratch {
    fn new() -> Scratch {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("acme-api")).unwrap();
        Scratch { dir }
    }

    fn memory(&self) -> Memory {
        Memory::new(&self.dir.path().join("state/harbour/sessions"))
    }

    fn checkout(&self) -> PathBuf {
        self.dir.path().join("acme-api")
    }

    fn remember(&self, id: &str, title: &str, position: usize) {
        self.memory()
            .keep(&Remembered {
                session_id: id.to_string(),
                directory: self.checkout(),
                title: title.to_string(),
                position,
            })
            .unwrap();
    }

    fn screen(&self, command: Vec<String>) -> Screen {
        Screen::of(App::new(command).with_memory(self.memory()))
    }
}

fn started_agent() -> Vec<String> {
    script("echo 'acme agent started'; exec sleep 600")
}

fn resuming_agent() -> Vec<String> {
    script(
        "echo \"acme agent $0 $1 in $(basename \"$PWD\")\"; \
         while :; do read -r line || continue; echo \"acme agent got: $line\"; done",
    )
}

fn numbered_resuming_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         echo \"acme agent run $n\"; sleep 0.3; exit 0",
        count = count.display()
    ))
}
