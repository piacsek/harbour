use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::memory::{Memory, Remembered};
use ratatui::style::Modifier;
use tempfile::TempDir;

use crate::handoff::last_use;
use crate::{Screen, below, script};

pub(crate) const ID: &str = "0f1e2d3c-4b5a-6978-8a9b-0c1d2e3f4a5b";
pub(crate) const OTHER_ID: &str = "9a8b7c6d-5e4f-4a3b-2c1d-0e9f8a7b6c5d";
pub(crate) const TITLE: &str = "Fix the flaky login test";

#[test]
fn a_remembered_session_comes_back_as_a_stopped_card() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.remember(OTHER_ID, SECOND, 1);
    let mut screen = scratch.screen(started_agent());
    let before = screen.wait_for(SECOND);
    assert!(before.contains(SECOND), "{before}");

    screen.press('j');

    let shown = screen.wait_for_all(&[TITLE, "stopped"]);
    assert_eq!(below(&shown, TITLE), "stopped");
    assert_eq!(below(&shown, SECOND), "");
    assert!(!shown.contains("acme agent started"), "{shown}");
}

#[test]
fn a_stopped_card_says_stopped_dimmed() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.remember(OTHER_ID, SECOND, 1);
    let mut screen = scratch.screen(started_agent());
    let before = screen.wait_for(SECOND);
    assert!(before.contains(SECOND), "{before}");
    screen.press('j');

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
fn resuming_a_stopped_card_counts_as_its_last_use() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.screen(resuming_agent());
    let drawn = screen.draw();
    assert!(drawn.contains(TITLE), "{drawn}");
    sleep(Duration::from_millis(5));
    let resumed_at = SystemTime::now();

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("acme agent --resume");
    assert!(shown.contains("acme agent --resume"), "{shown}");
    assert!(last_use(&mut screen, scratch.dir.path()) >= resumed_at);
}

#[test]
fn a_resumed_session_that_exits_resumes_again_with_enter() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.screen(numbered_resuming_agent(scratch.dir.path()));
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let exited = screen.wait_for_all(&["acme agent run 1", "sessions ○ 1"]);
    assert!(exited.contains("acme agent run 1"), "{exited}");
    assert!(exited.contains("sessions ○ 1"), "{exited}");
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

#[test]
fn enter_on_a_resumed_card_focuses_it_and_starts_no_second_copy() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let count = scratch.dir.path().join("count");
    let mut screen = scratch.screen(counted_resuming_agent(&count));
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let ready = screen.wait_for("acme agent run 1");
    assert!(ready.contains("acme agent run 1"), "{ready}");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("hello\r");

    let shown = screen.wait_for("acme agent got: hello");
    assert!(shown.contains("acme agent got: hello"), "{shown}");
    assert_eq!(fs::read_to_string(&count).unwrap().trim(), "1");
}

#[test]
fn many_stopped_cards_scroll_to_the_selected_one() {
    let scratch = Scratch::new();
    for n in 0..30 {
        let id = format!("00000000-0000-4000-8000-{n:012x}");
        scratch.remember(&id, &format!("Card {n:03}"), n);
    }
    let mut screen = scratch.screen(started_agent());
    let first = screen.wait_for("Card 000");
    assert!(first.contains("Card 000"), "{first}");

    for _ in 0..29 {
        screen.press('j');
    }

    let shown = screen.draw();
    assert!(shown.contains("Card 029"), "{shown}");
    assert!(!shown.contains("Card 000"), "{shown}");
}

#[test]
fn a_title_with_control_and_bidi_characters_from_disk_is_cleaned() {
    let scratch = Scratch::new();
    scratch.remember(ID, "Fix\u{1b}[31m the \u{202e}login", 0);
    let mut screen = scratch.screen(started_agent());

    let shown = screen.wait_for("login");

    assert!(shown.contains("Fix"), "{shown}");
    assert!(!shown.contains('\u{1b}'), "{shown}");
    assert!(!shown.contains('\u{202e}'), "{shown}");
}

pub(crate) struct Scratch {
    pub(crate) dir: TempDir,
}

impl Scratch {
    pub(crate) fn new() -> Scratch {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("acme-api")).unwrap();
        fs::create_dir_all(dir.path().join("claude/sessions")).unwrap();
        fs::create_dir_all(dir.path().join("claude/projects/-home-acme")).unwrap();
        Scratch { dir }
    }

    pub(crate) fn sessions(&self) -> PathBuf {
        self.dir.path().join("claude/sessions")
    }

    pub(crate) fn transcript_file(&self, id: &str) -> PathBuf {
        self.dir
            .path()
            .join(format!("claude/projects/-home-acme/{id}.jsonl"))
    }

    pub(crate) fn transcript(&self, id: &str, line: &str) {
        let file = self.transcript_file(id);
        let mut text = fs::read_to_string(&file).unwrap_or_default();
        text.push_str(line);
        text.push('\n');
        fs::write(file, text).unwrap();
    }

    pub(crate) fn watching(&self, command: Vec<String>) -> Screen {
        Screen::of(
            App::new(command)
                .with_registry(self.sessions())
                .with_transcripts(self.dir.path().join("claude/projects"))
                .with_memory(self.memory()),
        )
    }

    pub(crate) fn memory(&self) -> Memory {
        Memory::new(&self.dir.path().join("state/harbour/sessions"))
    }

    pub(crate) fn checkout(&self) -> PathBuf {
        self.dir.path().join("acme-api")
    }

    pub(crate) fn remember(&self, id: &str, title: &str, position: usize) {
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

const SECOND: &str = "The second one";

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

fn counted_resuming_agent(count: &Path) -> Vec<String> {
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         echo \"acme agent run $n\"; \
         while :; do read -r line || continue; echo \"acme agent got: $line\"; done",
        count = count.display()
    ))
}
