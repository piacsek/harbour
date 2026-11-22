use std::path::Path;

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;

use crate::editor_ending::gone;
use crate::keeping::{remembered_after_a_while, titled};
use crate::logging::{count, logged, only, read};
use crate::remembering::{ID, Scratch, TITLE};
use crate::usage_states::{app, place};
use crate::{Screen, home, pane_corner, raw_agent, script};

const ARGS_AGENT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/args-agent.sh");

#[test]
fn u_opens_a_popup_with_the_usage_report() {
    let mut screen = Screen::of(app().with_usage(report()));

    screen.prefixed('u');

    let shown = screen.wait_for("acme session: 12% spent");
    assert!(shown.contains(" usage "), "{shown}");
    assert!(shown.contains("No session yet"), "{shown}");
}

#[test]
fn esc_closes_the_popup() {
    let mut screen = Screen::of(app().with_usage(report()));
    screen.prefixed('u');
    screen.wait_for("acme session: 12% spent");

    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
    assert!(!shown.contains("acme session: 12% spent"), "{shown}");
}

#[test]
fn q_closes_the_popup_and_does_not_quit() {
    let mut screen = Screen::of(app().with_usage(report()));
    screen.prefixed('u');
    screen.wait_for("acme session: 12% spent");

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
    let mut screen = Screen::of(App::new(command).with_usage_place(Some(place())));

    screen.prefixed('u');
    screen.press('t');

    let shown = screen.wait_for("acme agent args:");
    assert!(
        shown.contains(
            "acme agent args:[-p --no-session-persistence --setting-sources user /usage]"
        ),
        "{shown}"
    );
}

#[test]
fn a_session_command_that_is_not_claude_opens_no_popup_and_says_why() {
    let mut screen = Screen::of(app());

    screen.prefixed('u');

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
    let app = App::new(vec![claude.display().to_string()]).with_usage_place(Some(place()));
    let mut screen = Screen::of(app);

    screen.prefixed('u');

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
    assert!(shown.contains("Could not start"), "{shown}");
}

#[test]
fn keys_while_the_popup_is_open_reach_only_the_popup() {
    let mut screen = Screen::of(app().with_usage(reading_report()));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.prefixed('u');
    screen.press('t');
    screen.wait_for("acme usage ready");

    screen.type_text("nxjk?u\r");

    let shown = screen.wait_for("acme usage got: nxjk?u");
    assert!(!shown.contains("? y/n"), "{shown}");
    screen.key(KeyCode::Esc, KeyModifiers::NONE);
    let after = typed_into_the_session_after(&mut screen);
    assert!(!after.contains("acme agent got: nxjk?u"), "{after}");
    assert!(!after.contains("acme-home 2"), "{after}");
    assert!(!after.contains(" help "), "{after}");
}

#[test]
fn a_paste_while_the_popup_is_open_reaches_only_the_popup() {
    let mut screen = Screen::of(app().with_usage(reading_report()));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.prefixed('u');
    screen.press('t');
    screen.wait_for("acme usage ready");

    screen.app.paste("acme pasted\r");

    let shown = screen.wait_for("acme usage got: acme pasted");
    assert!(shown.contains("acme usage got: acme pasted"), "{shown}");
    screen.key(KeyCode::Esc, KeyModifiers::NONE);
    let after = typed_into_the_session_after(&mut screen);
    assert!(!after.contains("acme agent got: acme pasted"), "{after}");
}

#[test]
fn a_click_while_the_popup_is_open_reaches_only_the_popup() {
    let mut screen = Screen::of(
        App::new(raw_agent("\\033[?1000h", 6))
            .with_usage_place(Some(place()))
            .with_usage(clicked_report()),
    );
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.prefixed('u');
    screen.press('t');
    screen.wait_for("acme usage ready");

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 110, 5);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 110, 5);
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 50, 10);
    screen.mouse(MouseEventKind::Up(MouseButton::Left), 50, 10);

    let shown = screen.wait_for("acme usage read:");
    assert!(shown.contains("acme usage read:[M"), "{shown}");
    screen.key(KeyCode::Esc, KeyModifiers::NONE);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("abcdef");
    let after = screen.wait_for("acme agent read:");
    assert!(after.contains("acme agent read:abcdef"), "{after}");
}

#[test]
fn u_is_logged_by_name_and_what_reaches_the_popup_by_its_byte_count_only() {
    let (app, lines) = logged(app().with_usage(reading_report()));
    let mut screen = Screen::of(app);
    screen.prefixed('u');
    screen.press('t');
    screen.wait_for("acme usage ready");

    screen.type_text("acme-hidden");
    screen.app.paste("acme-pasted");
    screen.draw();

    let lines = read(&lines);
    only(&lines, " prefix key=usage\n");
    assert_eq!(count(&lines, " input to=usage bytes=1\n"), 11);
    only(&lines, " input to=usage bytes=11\n");
    assert_eq!(count(&lines, "acme-hidden"), 0);
    assert_eq!(count(&lines, "acme-pasted"), 0);
}

#[test]
fn closing_the_popup_is_logged_as_the_popups_own_event_and_not_as_a_key() {
    let (app, lines) = logged(app().with_usage(report()));
    let mut screen = Screen::of(app);
    screen.prefixed('u');
    screen.wait_for("acme session: 12% spent");
    screen.press('q');
    screen.prefixed('u');
    screen.wait_for("acme session: 12% spent");

    screen.key(KeyCode::Esc, KeyModifiers::NONE);
    screen.draw();

    let lines = read(&lines);
    assert_eq!(count(&lines, " usage closed\n"), 2);
    assert_eq!(count(&lines, " prefix key=usage\n"), 2);
    assert_eq!(count(&lines, " key name="), 0);
    assert_eq!(count(&lines, r#" key name="q""#), 0);
}

#[test]
fn shift_u_opens_the_popup() {
    let mut screen = Screen::of(app().with_usage(report()));

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('U'), KeyModifiers::SHIFT);

    let shown = screen.wait_for("acme session: 12% spent");
    assert!(shown.contains(" usage "), "{shown}");
}

#[test]
fn u_with_caps_lock_opens_the_popup() {
    let mut screen = Screen::of(app().with_usage(report()));

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.press('U');

    let shown = screen.wait_for("acme session: 12% spent");
    assert!(shown.contains(" usage "), "{shown}");
}

#[test]
fn shift_q_closes_the_popup_and_does_not_quit() {
    let mut screen = Screen::of(app().with_usage(report()));
    screen.prefixed('u');
    screen.wait_for("acme session: 12% spent");

    screen.key(KeyCode::Char('Q'), KeyModifiers::SHIFT);

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
    assert!(screen.app.running());
}

#[test]
fn q_with_caps_lock_closes_the_popup_and_does_not_quit() {
    let mut screen = Screen::of(app().with_usage(report()));
    screen.prefixed('u');
    screen.wait_for("acme session: 12% spent");

    screen.press('Q');

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
    assert!(screen.app.running());
}

#[test]
fn ctrl_u_opens_no_popup() {
    let mut screen = Screen::of(app().with_usage(report()));

    screen.key(KeyCode::Char('u'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
}

#[test]
fn alt_u_opens_no_popup() {
    let mut screen = Screen::of(app().with_usage(report()));

    screen.key(KeyCode::Char('u'), KeyModifiers::ALT);

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
}

#[test]
fn alt_z_while_the_popup_is_open_does_not_zoom_the_session() {
    let mut screen = Screen::of(app().with_usage(reading_report()));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.prefixed('u');
    screen.press('t');
    screen.wait_for("acme usage ready");

    screen.prefixed('z');
    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(pane_corner(&shown) > 0, "{shown}");
}

fn leaves_nothing_behind(width: u16) {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let app = app()
        .with_registry(scratch.sessions())
        .with_transcripts(scratch.dir.path().join("claude/projects"))
        .with_memory(scratch.memory())
        .naming_sessions()
        .with_usage(registered_report(&scratch.sessions(), scratch.dir.path()));
    let mut screen = Screen::at(app, home(), width);
    screen.prefixed('u');
    screen.press('t');
    screen.wait_for("acme session: 12% spent");
    let pid = std::fs::read_to_string(scratch.dir.path().join("pid")).unwrap();
    let kept = remembered_after_a_while(&mut screen, &scratch.memory());
    assert_eq!(kept, Vec::new());

    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(shown.contains("No session yet"), "{shown}");
    assert!(gone(pid.trim()));
    assert_eq!(scratch.memory().recall(), Vec::new());
}

fn typed_into_the_session_after(screen: &mut Screen) -> String {
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("acme after\r");
    screen.wait_for("acme agent got: acme after")
}

fn reading_report() -> Vec<String> {
    script(
        "echo 'acme usage ready'; \
         while read -r line; do echo \"acme usage got: $line\"; done; exec sleep 600",
    )
}

fn clicked_report() -> Vec<String> {
    script(
        "stty raw -echo; printf '\\033[?1000h'; echo 'acme usage ready'; \
         r=$(dd bs=1 count=6 2>/dev/null); \
         printf 'acme usage read:%s' \"$r\" | tr -d '\\033'; exec sleep 600",
    )
}

fn report() -> Vec<String> {
    script("echo 'acme session: 12% spent'; echo 'acme week: 30% spent'")
}

fn registered_report(sessions: &Path, scratch: &Path) -> Vec<String> {
    script(&format!(
        "echo $$ > '{}/pid'; \
         printf '%s' '{{\"status\":\"busy\",\"sessionId\":\"{ID}\"}}' > '{}'/$$.json; \
         echo 'acme session: 12% spent'; exec sleep 600",
        scratch.display(),
        sessions.display()
    ))
}
