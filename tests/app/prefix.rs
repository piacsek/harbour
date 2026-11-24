use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;
use harbour::editor::Program;
use harbour::keymap::{Action, Keymap};
use harbour::picker::Picker;

use crate::editor_input::raw_editor;
use crate::finding::FINDER;
use crate::logging::{count, logged, only, read};
use crate::side::{agent, editor};
use crate::usage_states::app;
use crate::zoom::{corners, editing_session, raw_reader};
use crate::{Screen, focused_title, home, numbered_agent, pane_corner, region, script, settled};

#[test]
fn the_prefix_then_z_from_the_sidebar_zooms_the_selected_session() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    screen.prefixed('z');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 3 got: hi");
    assert_eq!(corners(&shown), 0, "{shown}");
}

#[test]
fn the_prefix_then_z_from_an_editor_zooms_that_editor() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing_session(scratch.path());

    screen.prefixed('z');

    let shown = screen.draw();
    assert_eq!(corners(&shown), 0, "{shown}");
    assert!(shown.contains("acme editor ready"), "{shown}");
}

#[test]
fn the_prefix_twice_sends_claude_code_the_legacy_byte() {
    let mut screen = focused(raw_reader(1));

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 1f"), "{shown}");
}

#[test]
fn the_prefix_twice_sends_an_editor_the_form_it_asked_for() {
    let mut screen = Screen::editing(raw_reader(1), raw_editor("\\033[>4;2m", 10));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read:[27;5;47~"), "{shown}");
}

#[test]
fn alt_with_the_prefix_sends_claude_code_esc_then_starts_the_prefix() {
    let mut screen = focused(raw_reader(1));

    screen.key(
        KeyCode::Char('_'),
        KeyModifiers::ALT | KeyModifiers::CONTROL,
    );
    screen.press('z');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 1b"), "{shown}");
    assert_eq!(corners(&screen.draw()), 0, "{shown}");
}

#[test]
fn alt_with_the_prefix_sends_an_editor_esc_in_its_form_then_starts_the_prefix() {
    let mut screen = Screen::editing(raw_reader(1), raw_editor("\\033[>1u", 5));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");

    screen.key(
        KeyCode::Char('_'),
        KeyModifiers::ALT | KeyModifiers::CONTROL,
    );
    screen.press('z');

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read:[27u"), "{shown}");
    assert_eq!(corners(&screen.draw()), 0, "{shown}");
}

#[test]
fn ctrl_underscore_is_the_prefix_too() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    screen.key(
        KeyCode::Char('_'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );
    screen.press('z');

    assert_eq!(corners(&screen.draw()), 0);
}

#[test]
fn escape_after_the_prefix_cancels_it_and_the_next_key_reaches_the_program() {
    let mut screen = focused(raw_reader(1));

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Esc, KeyModifiers::NONE);
    screen.press('z');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 7a"), "{shown}");
}

#[test]
fn an_unknown_key_after_the_prefix_is_swallowed_and_says_so() {
    let mut screen = Screen::wide(App::new(raw_reader(1)));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.prefixed('w');
    screen.press('z');

    let shown = screen.wait_for("acme agent read:");
    assert!(
        shown.contains("C-/ w does nothing; C-/ ? lists the keys"),
        "{shown}"
    );
    assert!(shown.contains("acme agent read: 7a"), "{shown}");
}

#[test]
fn the_unknown_key_message_names_the_configured_help_key() {
    let keymap = Keymap::default()
        .bind(Action::Sidebar, None)
        .bind(Action::Help, Some('h'));
    let mut screen = Screen::wide(App::new(raw_reader(1)).with_keymap(keymap));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.prefixed('w');

    screen.wait_for("C-/ w does nothing; C-/ h lists the keys");
}

#[test]
fn the_unknown_key_message_points_nowhere_while_help_is_unbound() {
    let keymap = Keymap::default().bind(Action::Help, None);
    let mut screen = Screen::wide(App::new(raw_reader(1)).with_keymap(keymap));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.prefixed('w');

    let shown = screen.wait_for("C-/ w does nothing");
    assert!(!shown.contains("lists the keys"), "{shown}");
}

#[test]
fn a_digit_after_the_prefix_jumps_to_that_card_and_9_to_the_last() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    screen.prefixed('1');
    screen.type_text("one\r");
    screen.wait_for("acme agent 1 got: one");
    screen.prefixed('9');
    screen.type_text("last\r");

    screen.wait_for("acme agent 3 got: last");
}

#[test]
fn a_digit_after_the_prefix_with_no_session_does_nothing() {
    let mut screen = Screen::new(script("exec sleep 600"));
    let before = screen.draw();

    screen.prefixed('3');

    assert_eq!(screen.draw(), before);
}

#[test]
fn h_after_the_prefix_focuses_the_sidebar_from_an_editor() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing_session(scratch.path());

    screen.prefixed('h');
    screen.press('j');

    assert!(!screen.draw().contains("acme editor read:"));
}

#[test]
fn l_after_the_prefix_focuses_the_pane_from_the_sidebar() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    screen.prefixed('l');
    screen.type_text("hi\r");

    screen.wait_for("acme agent 3 got: hi");
}

#[test]
fn n_after_the_prefix_from_claude_code_starts_a_session() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(lined_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.prefixed('n');

    screen.wait_for("acme agent 2");
}

#[test]
fn q_after_the_prefix_from_claude_code_asks_to_quit() {
    let mut screen = focused(raw_reader(1));

    screen.prefixed('q');

    screen.wait_for("Quit? y/n");
}

#[test]
fn x_after_the_prefix_from_claude_code_asks_to_end_it() {
    let mut screen = focused(raw_reader(1));

    screen.prefixed('x');

    screen.wait_for("End? y/n");
}

#[test]
fn capital_x_after_the_prefix_from_claude_code_asks_to_end_it() {
    let mut screen = focused(raw_reader(1));

    screen.prefixed('X');

    screen.wait_for("End and remove? y/n");
}

#[test]
fn q_after_the_prefix_while_zoomed_shows_the_question_over_the_pane() {
    let mut screen = focused(raw_reader(1));
    screen.prefixed('z');

    screen.prefixed('q');

    screen.wait_for("Quit? y/n");
}

#[test]
fn x_after_the_prefix_side_by_side_shows_the_question_over_the_pane() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = on_the_editor(scratch.path());
    screen.prefixed('\\');

    screen.prefixed('x');

    screen.wait_for("End? y/n");
}

#[test]
fn x_after_the_prefix_in_the_narrow_layout_shows_the_question_over_the_pane() {
    let mut screen = focused(raw_reader(1));
    screen.resize(50, 20);

    screen.prefixed('x');

    screen.wait_for("End? y/n");
}

#[test]
fn question_mark_after_the_prefix_from_a_pane_shows_the_help() {
    let mut screen = focused(raw_reader(1));

    screen.prefixed('?');

    screen.wait_for("zoom the session to the whole screen");
}

#[test]
fn minus_after_the_prefix_then_minus_again_soon_narrows_the_sidebar_twice() {
    let mut screen = started(30);

    screen.prefixed('-');
    screen.press('-');

    assert_eq!(pane_corner(&screen.draw()), 26);
}

#[test]
fn minus_after_the_prefix_then_minus_after_half_a_second_narrows_it_once() {
    let mut screen = started(30);

    screen.prefixed('-');
    sleep(Duration::from_millis(600));
    screen.press('-');

    assert_eq!(pane_corner(&screen.draw()), 28);
}

#[test]
fn minus_after_a_width_key_that_changed_nothing_reaches_the_session() {
    let mut screen = focused(raw_reader(1));
    screen.prefixed('z');

    screen.prefixed('-');
    screen.press('-');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 2d"), "{shown}");
}

#[test]
fn equals_after_the_prefix_widens_the_sidebar() {
    let mut screen = started(30);

    screen.prefixed('=');

    assert_eq!(pane_corner(&screen.draw()), 32);
}

#[test]
fn a_paste_after_the_prefix_cancels_it_and_reaches_the_program() {
    let mut screen = focused(raw_reader(2));

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.app.paste("a");
    screen.press('z');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 61 7a"), "{shown}");
}

#[test]
fn backslash_after_the_prefix_from_an_editor_goes_side_by_side_on_the_editor_half() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = on_the_editor(scratch.path());

    screen.prefixed('\\');
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme editor got: hi");
    assert_eq!(pane_corner(&shown), 60, "{shown}");
}

#[test]
fn a_closing_brace_after_the_prefix_from_the_editor_half_swaps_the_halves() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = on_the_editor(scratch.path());
    screen.prefixed('\\');

    screen.prefixed('}');

    let shown = screen.draw();
    assert!(
        region(&shown, 0, 60).contains("acme editor ready"),
        "{shown}"
    );
}

#[test]
fn a_closing_brace_after_the_prefix_does_nothing_when_not_side_by_side() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = on_the_editor(scratch.path());
    let before = screen.wait_for_stable();

    screen.prefixed('}');

    let after = settled(&mut screen, |shown| shown != before);
    assert_eq!(after, before);
}

#[test]
fn s_after_the_prefix_from_an_editor_opens_the_session_finder() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(agent(scratch.path()))
        .with_editor(Program::new("acme-edit", editor()))
        .with_picker(Picker::Command(vec![FINDER.into()]));
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");

    screen.prefixed('s');

    screen.wait_until("the session finder", |shown| {
        shown.lines().any(|line| focused_title(line) == "sessions")
    });
}

#[test]
fn u_after_the_prefix_from_claude_code_opens_the_usage_popup() {
    let mut screen = Screen::of(app().with_usage(script("echo 'acme session: 12% spent'")));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.prefixed('u');

    screen.wait_for("acme session: 12% spent");
}

#[test]
fn c_after_the_prefix_from_claude_code_shows_the_changelog() {
    let mut screen = Screen::of(
        App::new(raw_reader(1)).with_changelog("# Changelog\n\n## v1.0.0\n\n- acme note\n"),
    );
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.prefixed('c');

    screen.wait_for("acme note");
}

#[test]
fn the_prefix_then_question_mark_closes_the_help_and_sends_nothing() {
    let mut screen = focused(raw_reader(1));
    screen.prefixed('?');
    screen.wait_for("zoom the session to the whole screen");

    screen.prefixed('?');
    screen.press('z');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 7a"), "{shown}");
    assert!(
        !shown.contains("zoom the session to the whole screen"),
        "{shown}"
    );
}

#[test]
fn the_prefix_then_c_closes_the_changelog_and_sends_nothing() {
    let mut screen = focused_app(
        App::new(raw_reader(1)).with_changelog("# Changelog\n\n## v1.0.0\n\n- acme note\n"),
    );
    screen.prefixed('c');
    screen.wait_for("acme note");

    screen.prefixed('c');
    screen.press('z');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 7a"), "{shown}");
    assert!(!shown.contains("acme note"), "{shown}");
}

#[test]
fn the_prefix_then_z_closes_the_help_and_zooms() {
    let mut screen = focused(raw_reader(1));
    screen.prefixed('?');
    screen.wait_for("zoom the session to the whole screen");

    screen.prefixed('z');

    let shown = screen.draw();
    assert!(
        !shown.contains("zoom the session to the whole screen"),
        "{shown}"
    );
    assert_eq!(corners(&shown), 0, "{shown}");
}

#[test]
fn a_click_after_the_prefix_cancels_it() {
    let mut screen = focused(raw_reader(1));

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 60, 5);
    screen.press('z');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 7a"), "{shown}");
}

#[test]
fn the_prefix_and_its_action_are_logged_by_name() {
    let (app, lines) = logged(App::new(raw_reader(1)));
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.prefixed('z');

    let lines = read(&lines);
    assert!(only(&lines, " prefix\n").ends_with(" prefix\n"));
    assert!(only(&lines, " prefix key=zoom").ends_with(" prefix key=zoom\n"));
}

#[test]
fn an_unknown_key_after_the_prefix_is_logged_without_its_name() {
    let (app, lines) = logged(App::new(raw_reader(1)));
    let mut screen = Screen::of(app);

    screen.prefixed('%');

    let lines = read(&lines);
    assert_eq!(count(&lines, " prefix key=other"), 1);
    assert_eq!(count(&lines, "%"), 0);
}

#[test]
fn escape_after_the_prefix_is_logged_as_a_cancel() {
    let (app, lines) = logged(App::new(raw_reader(1)));
    let mut screen = Screen::of(app);

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    assert_eq!(count(&read(&lines), " prefix cancelled"), 1);
}

#[test]
fn the_prefix_sent_on_is_logged_as_its_byte_count() {
    let (app, lines) = logged(App::new(raw_reader(1)));
    let mut screen = focused_app(app);

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);

    let lines = read(&lines);
    assert_eq!(count(&lines, " input to=session session=1 bytes=1"), 1);
    assert_eq!(count(&lines, "key name=other"), 0);
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
         while read -r line; do echo \"acme agent $n got: $line\"; done; exec sleep 600",
        count = count.display()
    ))
}

fn focused(command: Vec<String>) -> Screen {
    focused_app(App::new(command))
}

fn focused_app(app: App) -> Screen {
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen
}

fn started(width: u16) -> Screen {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::at(
        App::new(numbered_agent(scratch.path())).with_sidebar_width(width),
        home(),
        120,
    );
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen
}

fn on_the_editor(scratch: &Path) -> Screen {
    let mut screen = Screen::editing(agent(scratch), Program::new("acme-edit", editor()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");
    screen
}
