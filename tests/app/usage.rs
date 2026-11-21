use std::path::Path;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;

use crate::editor_ending::gone;
use crate::keeping::{remembered_after_a_while, titled};
use crate::remembering::{ID, Scratch, TITLE};
use crate::{Screen, home, reading_agent, script};

const ARGS_AGENT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/args-agent.sh");

#[test]
fn u_opens_a_popup_with_the_usage_report() {
    let mut screen = Screen::of(App::new(reading_agent()).with_usage(report()));

    screen.press('u');

    let shown = screen.wait_for("acme session: 12% used");
    assert!(shown.contains(" usage "), "{shown}");
    assert!(shown.contains("No session yet"), "{shown}");
}

#[test]
fn esc_closes_the_popup() {
    let mut screen = Screen::of(App::new(reading_agent()).with_usage(report()));
    screen.press('u');
    screen.wait_for("acme session: 12% used");

    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
    assert!(!shown.contains("acme session: 12% used"), "{shown}");
}

#[test]
fn q_closes_the_popup_and_does_not_quit() {
    let mut screen = Screen::of(App::new(reading_agent()).with_usage(report()));
    screen.press('u');
    screen.wait_for("acme session: 12% used");

    screen.press('q');

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
    assert!(screen.app.running());
}

#[test]
fn closing_the_popup_leaves_no_card_no_remembered_file_and_no_process_in_the_wide_layout() {
    leaves_nothing_behind(200);
}

#[test]
fn closing_the_popup_leaves_no_card_no_remembered_file_and_no_process_in_the_narrow_layout() {
    leaves_nothing_behind(50);
}

#[test]
fn the_popup_runs_the_session_commands_claude_for_a_usage_report_without_its_arguments() {
    let scratch = tempfile::tempdir().unwrap();
    let claude = scratch.path().join("claude");
    std::os::unix::fs::symlink(ARGS_AGENT, &claude).unwrap();
    let command = vec![
        claude.display().to_string(),
        "--model".to_string(),
        "acme-model".to_string(),
    ];
    let mut screen = Screen::of(App::new(command));

    screen.press('u');

    let shown = screen.wait_for("acme agent args:");
    assert!(
        shown.contains("acme agent args:[-p --no-session-persistence /usage]"),
        "{shown}"
    );
}

#[test]
fn a_session_command_that_is_not_claude_opens_no_popup_and_says_why() {
    let mut screen = Screen::new(reading_agent());

    screen.press('u');

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
    assert!(
        shown.contains("Usage needs Claude Code; the session command runs /bin/sh"),
        "{shown}"
    );
}

#[test]
fn a_claude_that_cannot_start_opens_no_popup_and_says_so() {
    let scratch = tempfile::tempdir().unwrap();
    let claude = scratch.path().join("claude");
    let mut screen = Screen::new(vec![claude.display().to_string()]);

    screen.press('u');

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
    assert!(shown.contains("Could not start"), "{shown}");
}

fn leaves_nothing_behind(width: u16) {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let app = App::new(reading_agent())
        .with_registry(scratch.sessions())
        .with_transcripts(scratch.dir.path().join("claude/projects"))
        .with_memory(scratch.memory())
        .naming_sessions()
        .with_usage(registered_report(&scratch.sessions(), scratch.dir.path()));
    let mut screen = Screen::at(app, home(), width);
    screen.press('u');
    screen.wait_for("acme session: 12% used");
    let pid = std::fs::read_to_string(scratch.dir.path().join("pid")).unwrap();
    let kept = remembered_after_a_while(&mut screen, &scratch.memory());
    assert_eq!(kept, Vec::new());

    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(gone(pid.trim()));
    assert_eq!(scratch.memory().recall(), Vec::new());
}

fn report() -> Vec<String> {
    script("echo 'acme session: 12% used'; echo 'acme week: 30% used'")
}

fn registered_report(sessions: &Path, scratch: &Path) -> Vec<String> {
    script(&format!(
        "echo $$ > '{}/pid'; \
         printf '%s' '{{\"status\":\"busy\",\"sessionId\":\"{ID}\"}}' > '{}'/$$.json; \
         echo 'acme session: 12% used'; exec sleep 600",
        scratch.display(),
        sessions.display()
    ))
}
