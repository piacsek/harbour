use std::fs;
use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, SystemTime};

use crossterm::event::{KeyCode, KeyModifiers};

use crate::below;
use crate::remembering::{ID, OTHER_ID, Scratch, TITLE};

#[test]
fn a_stopped_card_whose_directory_is_gone_says_so() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    another(&scratch);
    fs::remove_dir_all(scratch.checkout()).unwrap();
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for(SECOND);
    assert!(before.contains(SECOND), "{before}");

    screen.press('j');

    let shown = screen.wait_for("directory gone");
    assert_eq!(below(&shown, TITLE), "directory gone");
    assert_eq!(below(&shown, SECOND), "");
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
    another(&scratch);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for(SECOND);
    assert!(before.contains(SECOND), "{before}");

    screen.press('j');

    let shown = screen.wait_for_all(&[TITLE, "stopped"]);
    assert_eq!(below(&shown, TITLE), "stopped");
    assert_eq!(below(&shown, SECOND), "");
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
    another(&scratch);
    let _other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for(SECOND);
    assert!(before.contains(SECOND), "{before}");

    screen.press('j');

    let shown = screen.wait_for("open elsewhere");
    assert_eq!(below(&shown, TITLE), "open elsewhere");
    assert_eq!(below(&shown, SECOND), "");
}

#[test]
fn enter_on_a_card_open_elsewhere_starts_no_second_copy() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let _other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for(CLOSE_IT_THERE);
    assert!(before.contains(CLOSE_IT_THERE), "{before}");

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
    let before = screen.wait_for(CLOSE_IT_THERE);
    assert!(before.contains(CLOSE_IT_THERE), "{before}");

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
    another(&scratch);
    let other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for(SECOND);
    assert!(before.contains(SECOND), "{before}");
    screen.press('j');
    let held = screen.wait_for("open elsewhere");
    assert_eq!(below(&held, TITLE), "open elsewhere");

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
    another(&scratch);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for(SECOND);
    assert!(before.contains(SECOND), "{before}");

    screen.press('j');

    let shown = screen.wait_for_all(&[TITLE, "stopped"]);
    assert_eq!(below(&shown, TITLE), "stopped");
}

#[test]
fn the_pane_of_a_card_whose_directory_is_gone_says_x_forgets_it() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    fs::remove_dir_all(scratch.checkout()).unwrap();
    let mut screen = scratch.watching(resuming_agent());

    let shown = screen.wait_for("x forgets it");

    assert!(
        shown.contains("Its directory is gone. x forgets it."),
        "{shown}"
    );
}

#[test]
fn the_pane_of_a_card_open_elsewhere_says_to_close_it_there() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let _other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(resuming_agent());

    let shown = screen.wait_for("close it there");

    assert!(
        shown.contains("Open elsewhere: close it there, then Enter resumes it here."),
        "{shown}"
    );
}

#[test]
fn the_held_message_goes_once_another_card_is_selected() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.remember(OTHER_ID, "The second one", 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    let _other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for(CLOSE_IT_THERE);
    assert!(before.contains(CLOSE_IT_THERE), "{before}");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let held = screen.wait_for("This conversation is open elsewhere");
    assert!(
        held.contains("This conversation is open elsewhere"),
        "{held}"
    );

    screen.press('j');

    let shown = screen.draw();
    assert!(
        !shown.contains("This conversation is open elsewhere"),
        "{shown}"
    );
}

#[test]
fn the_held_message_goes_once_the_card_can_be_resumed() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for(CLOSE_IT_THERE);
    assert!(before.contains(CLOSE_IT_THERE), "{before}");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let held = screen.wait_for("This conversation is open elsewhere");
    assert!(
        held.contains("This conversation is open elsewhere"),
        "{held}"
    );

    drop(other);

    let shown = screen.wait_until(
        "the card can resume and the held message is gone",
        |shown| shown.contains(RESUMES) && !shown.contains("This conversation is open elsewhere"),
    );
    assert!(shown.contains(RESUMES), "{shown}");
    assert!(
        !shown.contains("This conversation is open elsewhere"),
        "{shown}"
    );
    assert_eq!(below(&shown, TITLE), "");
}

#[test]
fn the_gone_message_goes_once_another_card_is_selected() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.remember(OTHER_ID, "The second one", 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(resuming_agent());
    let before = screen.wait_for("The second one");
    assert!(before.contains("The second one"), "{before}");
    fs::remove_file(scratch.transcript_file(ID)).unwrap();
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let gone = screen.wait_for("This conversation is gone");
    assert!(gone.contains("This conversation is gone"), "{gone}");

    screen.press('k');

    let shown = screen.draw();
    assert!(!shown.contains("This conversation is gone"), "{shown}");
}

#[test]
fn a_remembered_file_named_like_an_option_is_not_shown() {
    let scratch = Scratch::new();
    let memory = scratch.dir.path().join("state/harbour/sessions");
    fs::create_dir_all(&memory).unwrap();
    fs::write(
        memory.join("--settings.json"),
        format!(
            r#"{{"version":1,"sessionId":"--settings","directory":"{}","title":"Forged","position":0}}"#,
            scratch.checkout().display()
        ),
    )
    .unwrap();
    scratch.transcript("--settings", r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(resuming_agent());

    let shown = screen.wait_for("No session yet");

    assert!(!shown.contains("Forged"), "{shown}");
}

#[test]
fn a_stale_card_whose_transcript_is_gone_is_forgotten() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let file = scratch
        .dir
        .path()
        .join(format!("state/harbour/sessions/{ID}.json"));
    fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(31 * 24 * 60 * 60))
        .unwrap();
    let mut screen = scratch.watching(resuming_agent());

    let shown = screen.wait_for("No session yet");

    assert!(!shown.contains(TITLE), "{shown}");
    assert_eq!(scratch.memory().recall(), Vec::new());
}

#[test]
fn a_stale_card_is_kept_while_its_claude_code_configuration_is_gone() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let file = scratch
        .dir
        .path()
        .join(format!("state/harbour/sessions/{ID}.json"));
    fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(31 * 24 * 60 * 60))
        .unwrap();
    fs::remove_dir_all(scratch.dir.path().join("claude")).unwrap();
    let mut screen = scratch.watching(resuming_agent());

    let shown = screen.wait_for("No session yet");

    assert!(!shown.contains(TITLE), "{shown}");
    assert_eq!(scratch.memory().recall().len(), 1);
}

#[test]
fn at_most_a_hundred_cards_come_back() {
    let scratch = Scratch::new();
    for n in 0..101 {
        let id = format!("00000000-0000-4000-8000-{n:012x}");
        scratch.remember(&id, &format!("Card {n:03}"), n);
        scratch.transcript(&id, r#"{"type":"user","message":{}}"#);
    }
    let mut screen = scratch.watching(resuming_agent());
    let first = screen.wait_for("Card 000");
    assert!(first.contains("Card 000"), "{first}");

    for _ in 0..120 {
        screen.press('j');
    }

    let shown = screen.draw();
    assert!(shown.contains("Card 099"), "{shown}");
    assert!(!shown.contains("Card 100"), "{shown}");
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

const SECOND: &str = "The second one";
const CLOSE_IT_THERE: &str = "Open elsewhere: close it there";
const RESUMES: &str = "Stopped. Enter resumes it, x forgets it.";

fn another(scratch: &Scratch) {
    scratch.remember(OTHER_ID, SECOND, 1);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
}

fn resuming_agent() -> Vec<String> {
    crate::script("echo \"acme agent resumed $1\"; exec sleep 600")
}
