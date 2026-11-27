use std::path::Path;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::remembering::{ID, OTHER_ID, Scratch, TITLE};
use crate::{Screen, script};

const NONE: &str = "No previous session";
const ENDED: &str = "The previous session has ended";

#[test]
fn the_prefix_twice_switches_to_the_card_selected_before() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    twice(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 2 got:");
    assert!(shown.contains("acme agent 2 got: hi"), "{shown}");
}

#[test]
fn the_prefix_twice_again_switches_back() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    twice(&mut screen);
    screen.wait_for_all(&["acme agent 2"]);

    twice(&mut screen);
    screen.type_text("yo\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: yo"), "{shown}");
}

#[test]
fn ctrl_held_for_the_second_press_switches_too() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('_'), KeyModifiers::CONTROL);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 2 got:");
    assert!(shown.contains("acme agent 2 got: hi"), "{shown}");
}

#[test]
fn a_jump_makes_the_card_it_left_the_previous_one() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.prefixed('1');

    twice(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got:");
    assert!(shown.contains("acme agent 3 got: hi"), "{shown}");
}

#[test]
fn a_moved_card_is_still_the_previous_one() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.prefixed('h');
    screen.press('K');
    screen.press('K');

    twice(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 2 got:");
    assert!(shown.contains("acme agent 2 got: hi"), "{shown}");
}

#[test]
fn with_one_session_the_prefix_twice_says_there_is_no_previous_one() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(lined_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");

    twice(&mut screen);

    let shown = screen.wait_for(NONE);
    assert!(shown.contains(NONE), "{shown}");
}

#[test]
fn selecting_another_card_clears_the_message() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.prefixed('2');
    screen.type_text("bye\r");
    screen.wait_for("exited");
    screen.prefixed('3');
    twice(&mut screen);
    screen.wait_for(ENDED);
    screen.prefixed('h');

    screen.press('k');

    let shown = screen.wait_until("the message gone", |shown| !shown.contains(ENDED));
    assert!(!shown.contains(ENDED), "{shown}");
}

#[test]
fn a_closed_previous_card_leaves_the_one_before_it_as_the_previous() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.prefixed('1');
    screen.prefixed('2');
    screen.prefixed('h');
    screen.press('X');
    screen.press('y');
    screen.wait_until("card 2 removed", |shown| !shown.contains("acme-home 2"));

    twice(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got:");
    assert!(shown.contains("acme agent 1 got: hi"), "{shown}");
}

#[test]
fn an_exited_previous_session_says_it_has_ended_and_stays() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    screen.prefixed('2');
    screen.type_text("bye\r");
    screen.wait_for("exited");
    screen.prefixed('3');

    twice(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for_all(&[ENDED, "acme agent 3 got: hi"]);
    assert!(shown.contains(ENDED), "{shown}");
}

#[test]
fn a_stopped_previous_card_says_it_has_ended() {
    let scratch = Scratch::new();
    scratch.remember(ID, "The first one", 0);
    scratch.remember(OTHER_ID, TITLE, 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(resuming_agent());
    screen.wait_for(TITLE);
    screen.press('j');

    twice(&mut screen);

    let shown = screen.wait_for(ENDED);
    assert!(shown.contains(ENDED), "{shown}");
}

#[test]
fn a_card_that_resumed_is_still_the_previous_one() {
    let scratch = Scratch::new();
    scratch.remember(ID, "The first one", 0);
    scratch.remember(OTHER_ID, TITLE, 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(resuming_agent());
    screen.wait_for(TITLE);
    screen.prefixed('1');
    screen.wait_for(&format!("acme agent --resume {ID}"));
    screen.prefixed('h');
    screen.press('j');

    twice(&mut screen);
    screen.type_text("hello\r");

    let shown = screen.wait_for("acme agent got:");
    assert!(shown.contains("acme agent got: hello"), "{shown}");
}

fn twice(screen: &mut Screen) {
    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
}

fn three_sessions(scratch: &Path) -> Screen {
    let mut screen = Screen::new(lined_agent(scratch));
    for n in 1..=3 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {n}"));
    }
    screen
}

fn lined_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         echo \"acme agent $n\"; \
         while read -r line; do [ \"$line\" = bye ] && exit 0; echo \"acme agent $n got: $line\"; done; \
         exec sleep 600",
        count = count.display()
    ))
}

fn resuming_agent() -> Vec<String> {
    script(
        "echo \"acme agent $0 $1 in $(basename \"$PWD\")\"; \
         while :; do read -r line || continue; echo \"acme agent got: $line\"; done",
    )
}
