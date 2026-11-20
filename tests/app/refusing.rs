use std::fs;
use std::path::Path;
use std::process::{Child, Command};

use crossterm::event::{KeyCode, KeyModifiers};

use crate::below;
use crate::remembering::{ID, Scratch, TITLE};

#[test]
fn a_stopped_card_whose_directory_is_gone_says_so() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    fs::remove_dir_all(scratch.checkout()).unwrap();
    let mut screen = scratch.watching(resuming_agent());

    let shown = screen.wait_for(TITLE);

    assert_eq!(below(&shown, TITLE), "directory gone");
}

#[test]
fn enter_on_a_card_whose_directory_is_gone_says_why_and_starts_nothing() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    fs::remove_dir_all(scratch.checkout()).unwrap();
    let mut screen = scratch.watching(resuming_agent());

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("Not a directory");
    assert!(shown.contains("Not a directory"), "{shown}");
    assert!(!shown.contains("acme agent resumed"), "{shown}");
}

#[test]
fn a_stopped_card_whose_transcript_is_gone_is_not_shown_and_its_file_is_kept() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = scratch.watching(resuming_agent());

    let shown = screen.wait_for("No session yet");

    assert!(!shown.contains(TITLE), "{shown}");
    assert_eq!(scratch.memory().recall().len(), 1);
}

#[test]
fn a_stopped_card_whose_transcript_exists_is_shown() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(resuming_agent());

    let shown = screen.wait_for(TITLE);

    assert_eq!(below(&shown, TITLE), "stopped");
}

#[test]
fn enter_on_a_card_whose_transcript_went_away_drops_it_and_says_why() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for(TITLE);
    assert!(before.contains(TITLE), "{before}");
    fs::remove_file(scratch.transcript_file(ID)).unwrap();

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("is gone");
    assert!(shown.contains("This conversation is gone"), "{shown}");
    assert!(!shown.contains("acme agent resumed"), "{shown}");
    assert_eq!(scratch.memory().recall().len(), 1);
}

#[test]
fn a_stopped_card_open_elsewhere_says_so() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let _other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(resuming_agent());

    let shown = screen.wait_for("open elsewhere");

    assert_eq!(below(&shown, TITLE), "open elsewhere");
}

#[test]
fn enter_on_a_card_open_elsewhere_starts_no_second_copy() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let _other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for("open elsewhere");
    assert!(before.contains("open elsewhere"), "{before}");

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("is open elsewhere");
    assert!(
        shown.contains("This conversation is open elsewhere"),
        "{shown}"
    );
    assert!(!shown.contains("acme agent resumed"), "{shown}");
}

#[test]
fn x_on_a_card_open_elsewhere_hides_it_and_keeps_its_file() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let _other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for("open elsewhere");
    assert!(before.contains("open elsewhere"), "{before}");

    screen.press('x');

    let shown = screen.draw();
    assert!(!shown.contains(TITLE), "{shown}");
    assert_eq!(scratch.memory().recall().len(), 1);
}

#[test]
fn a_card_open_elsewhere_says_stopped_once_the_other_one_ends() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for("open elsewhere");
    assert!(before.contains("open elsewhere"), "{before}");

    drop(other);

    let shown = screen.wait_for("stopped");
    assert_eq!(below(&shown, TITLE), "stopped");
}

#[test]
fn a_registry_file_of_an_ended_process_does_not_hold_a_card() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let ended = Elsewhere::open(&scratch.sessions(), ID);
    let pid = ended.pid();
    drop(ended);
    fs::write(
        scratch.sessions().join(format!("{pid}.json")),
        format!(r#"{{"status":"busy","sessionId":"{ID}"}}"#),
    )
    .unwrap();
    let mut screen = scratch.watching(resuming_agent());

    let shown = screen.wait_for(TITLE);

    assert_eq!(below(&shown, TITLE), "stopped");
}

struct Elsewhere {
    child: Child,
}

impl Elsewhere {
    fn open(sessions: &Path, id: &str) -> Elsewhere {
        let child = Command::new("sleep").arg("600").spawn().unwrap();
        fs::write(
            sessions.join(format!("{}.json", child.id())),
            format!(r#"{{"status":"busy","sessionId":"{id}"}}"#),
        )
        .unwrap();
        Elsewhere { child }
    }

    fn pid(&self) -> u32 {
        self.child.id()
    }
}

impl Drop for Elsewhere {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn resuming_agent() -> Vec<String> {
    crate::script("echo \"acme agent resumed $1\"; exec sleep 600")
}
