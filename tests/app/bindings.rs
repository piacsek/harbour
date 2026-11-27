use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::keymap::{Action, Keymap};

use crate::zoom::corners;
use crate::{Screen, raw_reader, script};

#[test]
fn a_bound_key_after_the_prefix_runs_its_action() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = started(
        Keymap::default().bind(Action::Zoom, Some('f')),
        scratch.path(),
    );

    screen.prefixed('f');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert_eq!(corners(&shown), 0, "{shown}");
}

#[test]
fn the_default_key_of_a_moved_action_does_nothing_after_the_prefix() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(lined_agent(scratch.path()))
        .with_keymap(Keymap::default().bind(Action::Zoom, Some('f')));
    let mut screen = Screen::wide(app);
    screen.press('n');
    screen.wait_for("acme agent 1");

    screen.prefixed('z');

    screen.wait_for("C-/ z does nothing; C-/ ? lists the keys");
}

#[test]
fn an_unbound_action_does_nothing_after_the_prefix() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(lined_agent(scratch.path()))
        .with_keymap(Keymap::default().bind(Action::Zoom, None));
    let mut screen = Screen::wide(app);
    screen.press('n');
    screen.wait_for("acme agent 1");

    screen.prefixed('z');

    screen.wait_for("C-/ z does nothing; C-/ ? lists the keys");
}

#[test]
fn a_configured_prefix_starts_the_sequence() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = started(Keymap::default().with_prefix(']'), scratch.path());

    screen.key(KeyCode::Char(']'), KeyModifiers::CONTROL);
    screen.press('z');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert_eq!(corners(&shown), 0, "{shown}");
}

#[test]
fn ctrl_semicolon_then_p_sends_claude_code_a_semicolon() {
    let app = App::new(raw_reader(2)).with_keymap(Keymap::default().with_prefix(';'));
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char(';'), KeyModifiers::CONTROL);
    screen.press('p');
    screen.press('z');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 3b 7a"), "{shown}");
}

#[test]
fn ctrl_held_through_a_ctrl_semicolon_prefix_runs_the_action() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = started(Keymap::default().with_prefix(';'), scratch.path());

    screen.key(KeyCode::Char(';'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('z'), KeyModifiers::CONTROL);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 1 got: hi");
    assert_eq!(corners(&shown), 0, "{shown}");
}

#[test]
fn a_configured_prefix_is_named_in_the_unknown_key_message() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(lined_agent(scratch.path())).with_keymap(Keymap::default().with_prefix(']'));
    let mut screen = Screen::wide(app);

    screen.key(KeyCode::Char(']'), KeyModifiers::CONTROL);
    screen.press('w');

    screen.wait_for("C-] w does nothing; C-] ? lists the keys");
}

#[test]
fn c_slash_does_not_start_the_sequence_once_another_prefix_is_set() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = started(Keymap::default().with_prefix(']'), scratch.path());
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.press('z');
    screen.type_text("\r");

    screen.wait_for("acme agent 1 got: z");
}

#[test]
fn the_new_binding_starts_a_session_from_the_sidebar() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(lined_agent(scratch.path()))
        .with_keymap(Keymap::default().bind(Action::New, Some('a')));
    let mut screen = Screen::of(app);

    screen.press('a');

    screen.wait_for("acme agent 1");
}

#[test]
fn n_does_nothing_in_the_sidebar_once_new_is_bound_elsewhere() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(lined_agent(scratch.path()))
        .with_keymap(Keymap::default().bind(Action::New, Some('a')));
    let mut screen = Screen::of(app);

    screen.press('n');
    sleep(Duration::from_millis(300));

    let shown = screen.draw();
    assert!(!shown.contains("acme agent"), "{shown}");
}

#[test]
fn the_end_binding_asks_to_end_from_the_sidebar() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = started(
        Keymap::default().bind(Action::End, Some('d')),
        scratch.path(),
    );

    screen.press('d');

    screen.wait_for("End? y/n");
}

fn started(keymap: Keymap, scratch: &Path) -> Screen {
    let mut screen = Screen::of(App::new(lined_agent(scratch)).with_keymap(keymap));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen
}

fn lined_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         echo \"acme agent $n\"; \
         while read -r line; do echo \"acme agent $n got: $line\"; done; exec sleep 600",
        count = count.display()
    ))
}
