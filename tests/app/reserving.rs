use crossterm::event::{KeyCode, KeyModifiers};

use crate::below;
use crate::remembering::{ID, OTHER_ID, Scratch, TITLE};

#[test]
fn a_conversation_resumed_in_one_harbour_is_open_elsewhere_in_another() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, USER);
    let mut first = scratch.watching(resuming_agent());
    let mut second = scratch.watching(resuming_agent());

    first.key(KeyCode::Enter, KeyModifiers::NONE);

    let resumed = first.wait_for("acme agent resumed");
    assert!(resumed.contains("acme agent resumed"), "{resumed}");
    let shown = second.wait_for("open elsewhere");
    assert_eq!(below(&shown, TITLE), "open elsewhere");
}

#[test]
fn enter_in_a_second_harbour_starts_no_second_copy() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, USER);
    let mut first = scratch.watching(resuming_agent());
    let mut second = scratch.watching(resuming_agent());
    first.key(KeyCode::Enter, KeyModifiers::NONE);
    let resumed = first.wait_for("acme agent resumed");
    assert!(resumed.contains("acme agent resumed"), "{resumed}");

    second.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = second.wait_for("is open elsewhere");
    assert!(
        shown.contains("This conversation is open elsewhere"),
        "{shown}"
    );
    assert!(!shown.contains("acme agent resumed"), "{shown}");
}

#[test]
fn two_harbours_resume_different_conversations() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.remember(OTHER_ID, "The second one", 1);
    scratch.transcript(ID, USER);
    scratch.transcript(OTHER_ID, USER);
    let mut first = scratch.watching(resuming_agent());
    let mut second = scratch.watching(resuming_agent());
    first.key(KeyCode::Enter, KeyModifiers::NONE);
    let resumed = first.wait_for(&format!("acme agent resumed {ID}"));
    assert!(
        resumed.contains(&format!("acme agent resumed {ID}")),
        "{resumed}"
    );

    second.press('j');
    second.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = second.wait_for(&format!("acme agent resumed {OTHER_ID}"));
    assert!(
        shown.contains(&format!("acme agent resumed {OTHER_ID}")),
        "{shown}"
    );
}

#[test]
fn a_conversation_ended_in_one_harbour_resumes_in_another() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, USER);
    let mut first = scratch.watching(resuming_agent());
    let mut second = scratch.watching(resuming_agent());
    first.key(KeyCode::Enter, KeyModifiers::NONE);
    let held = second.wait_for("open elsewhere");
    assert!(held.contains("open elsewhere"), "{held}");
    first.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    first.press('x');
    first.press('y');
    let stopped = second.wait_for("stopped");
    assert_eq!(below(&stopped, TITLE), "stopped");

    second.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = second.wait_for("acme agent resumed");
    assert!(shown.contains("acme agent resumed"), "{shown}");
}

const USER: &str = r#"{"type":"user","message":{}}"#;

fn resuming_agent() -> Vec<String> {
    crate::script("echo \"acme agent resumed $1\"; exec sleep 600")
}
