use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::picker::Picker;

use crate::remembering::{ID, Scratch};
use crate::{Screen, pty_slots, reading_agent, script, usage_states};

#[test]
fn a_piped_screen_holds_no_pty_slot() {
    let mut screen = Screen::piped(App::new(told("acme agent")));
    screen.press('n');
    screen.wait_for("acme agent on");

    assert!(!pty_slots::holding());
}

#[test]
fn a_piped_screen_starts_its_agent_without_a_terminal() {
    let mut screen = Screen::piped(App::new(told("acme agent")));

    screen.press('n');

    let shown = screen.wait_for("acme agent on pipes");
    assert!(shown.contains("acme agent on pipes"), "{shown}");
}

#[test]
fn a_piped_screen_resumes_a_stopped_card_without_a_terminal() {
    let scratch = Scratch::new();
    scratch.remember(ID, "acme resumed", 0);
    let mut screen = Screen::piped(App::new(told("acme agent")).with_memory(scratch.memory()));

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("acme agent on pipes");
    assert!(shown.contains("acme agent on pipes"), "{shown}");
}

#[test]
fn a_piped_screen_starts_its_editor_without_a_terminal() {
    let editor = Program::new("acme-edit", told("acme editor"));
    let mut screen = Screen::piped(App::new(reading_agent()).with_editor(editor));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor on pipes");
    assert!(shown.contains("acme editor on pipes"), "{shown}");
}

#[test]
fn a_piped_screen_starts_its_picker_without_a_terminal() {
    let picker = script(
        "if [ -t 2 ]; then echo 'acme picker on a tty' >&2; \
         else echo 'acme picker on pipes' >&2; fi; read -r line",
    );
    let mut screen = Screen::piped(App::new(reading_agent()).with_picker(Picker::Command(picker)));

    screen.press('n');

    let shown = screen.wait_for("acme picker on pipes");
    assert!(shown.contains("acme picker on pipes"), "{shown}");
}

#[test]
fn a_piped_screen_reads_usage_without_a_terminal() {
    let report =
        script("if [ -t 1 ]; then echo 'acme usage on a tty'; else echo 'acme usage on pipes'; fi");
    let mut screen = Screen::piped(usage_states::app().with_usage(report));

    screen.prefixed('u');

    let shown = screen.wait_for("acme usage on pipes");
    assert!(shown.contains("acme usage on pipes"), "{shown}");
}

fn told(name: &str) -> Vec<String> {
    script(&format!(
        "if [ -t 1 ]; then echo '{name} on a tty'; else echo '{name} on pipes'; fi; exec sleep 600"
    ))
}
