use std::sync::mpsc::{self, Receiver};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::keymap::{Action, Keymap};
use harbour::log::Log;
use harbour::picker::Picker;

use crate::finding::FINDER;
use crate::{Screen, home, reading_agent, script};

#[test]
fn the_first_frame_logs_the_start_with_the_version() {
    let (app, lines) = logged(App::new(script("exec sleep 600")));

    Screen::of(app);

    let start = only(&read(&lines), " start ");
    assert!(
        start.contains(&format!("version={}", env!("CARGO_PKG_VERSION"))),
        "{start}"
    );
}

#[test]
fn the_start_names_the_size_of_the_terminal() {
    let (app, lines) = logged(App::new(script("exec sleep 600")));

    Screen::of(app);

    assert!(only(&read(&lines), " start ").contains(" size=120x20"));
}

#[test]
fn the_start_names_the_session_command_picker_and_editor() {
    let (app, lines) = logged(App::new(script("exec sleep 600")));

    Screen::of(app);

    let start = only(&read(&lines), " start ");
    assert!(start.contains(r#" command="/bin/sh""#), "{start}");
    assert!(start.contains(" picker=none"), "{start}");
    assert!(start.contains(" editor=none"), "{start}");
}

#[test]
fn the_start_is_logged_once() {
    let (app, lines) = logged(App::new(script("exec sleep 600")));
    let mut screen = Screen::of(app);

    screen.draw();
    screen.draw();

    assert_eq!(count(&read(&lines), " start "), 1);
}

#[test]
fn a_key_harbour_acts_on_is_logged_by_its_name() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);

    screen.press('n');

    only(&read(&lines), r#" key name="n""#);
}

#[test]
fn a_sidebar_key_moved_by_the_keys_table_is_logged_by_its_name() {
    let keymap = Keymap::default().bind(Action::New, Some('a'));
    let (app, lines) = logged(App::new(reading_agent()).with_keymap(keymap));
    let mut screen = Screen::of(app);

    screen.press('a');

    only(&read(&lines), r#" key name="a""#);
}

#[test]
fn a_sidebar_key_with_shift_is_logged_by_its_name() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);

    screen.key(KeyCode::Char('N'), KeyModifiers::SHIFT);

    only(&read(&lines), r#" key name="N""#);
}

#[test]
fn a_control_key_harbour_acts_on_is_logged_with_its_modifier() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    only(&read(&lines), r#" key name="C-l""#);
}

#[test]
fn a_jump_after_the_prefix_is_logged_by_its_digit() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);

    screen.prefixed('3');

    only(&read(&lines), " prefix key=3\n");
}

#[test]
fn the_zoom_key_is_logged_by_its_name() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = focused(app);

    screen.prefixed('z');

    only(&read(&lines), " prefix key=zoom\n");
}

#[test]
fn the_side_by_side_key_is_logged_by_its_name() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = focused(app);

    screen.prefixed('\\');

    only(&read(&lines), " prefix key=side\n");
}

#[test]
fn the_swap_keys_are_logged_by_their_names() {
    let editor = Program::new("acme-edit", script("exec sleep 600"));
    let (app, lines) = logged(App::new(reading_agent()).with_editor(editor));
    let mut screen = focused(app);
    screen.prefixed('\\');

    screen.prefixed('}');
    screen.prefixed('{');

    let lines = read(&lines);
    assert_eq!(count(&lines, " prefix key=swap_right\n"), 1);
    assert_eq!(count(&lines, " prefix key=swap_left\n"), 1);
}

#[test]
fn the_find_key_is_logged_by_its_name_and_what_is_typed_into_the_list_is_not() {
    let picker = Picker::Command(vec![FINDER.into()]);
    let (app, lines) = logged(App::new(reading_agent()).with_picker(picker));
    let mut screen = focused(app);

    screen.prefixed('s');
    screen.wait_for("acme fzf ready");
    screen.type_text("acme-secret");

    let lines = read(&lines);
    only(&lines, " prefix key=find\n");
    assert_eq!(count(&lines, "acme-secret"), 0);
}

#[test]
fn the_narrower_key_is_logged_by_its_name() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = focused(app);

    screen.prefixed('-');

    only(&read(&lines), " prefix key=narrower\n");
}

#[test]
fn the_wider_key_is_logged_by_its_name() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = focused(app);

    screen.prefixed('=');

    only(&read(&lines), " prefix key=wider\n");
}

#[test]
fn a_wider_key_that_only_hides_the_help_is_logged_without_its_name() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.press('?');

    screen.prefixed('=');

    let lines = read(&lines);
    assert_eq!(count(&lines, " key name=other"), 1);
    assert_eq!(count(&lines, "M-="), 0);
}

#[test]
fn a_key_harbour_binds_nothing_to_is_logged_without_its_name() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);

    screen.press('z');

    let lines = read(&lines);
    only(&lines, " key name=other");
    assert_eq!(count(&lines, "z\""), 0);
}

#[test]
fn keys_typed_into_a_session_are_logged_as_byte_counts_only() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = focused(app);

    screen.type_text("acme-secret");

    let lines = read(&lines);
    assert_eq!(count(&lines, " input to=session session=1 bytes=1"), 11);
    assert_eq!(count(&lines, "acme-secret"), 0);
    assert_eq!(count(&lines, " key "), 2);
}

#[test]
fn a_paste_into_a_session_is_logged_as_its_byte_count_only() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = focused(app);

    screen.app.paste("acme-secret\r");
    screen.draw();

    let lines = read(&lines);
    only(&lines, " input to=session session=1 bytes=12");
    assert_eq!(count(&lines, "acme-secret"), 0);
}

#[test]
fn keys_typed_into_the_picker_are_logged_as_byte_counts_only() {
    let picker = Picker::Command(script("echo 'acme picker' >&2; read -r line"));
    let (app, lines) = logged(App::new(reading_agent()).with_picker(picker));
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme picker");

    screen.type_text("acme-secret");

    let lines = read(&lines);
    assert_eq!(count(&lines, " input to=picker bytes=1"), 11);
    assert_eq!(count(&lines, "acme-secret"), 0);
}

#[test]
fn keys_typed_into_an_editor_are_logged_with_the_editor() {
    let editor = Program::new(
        "acme-edit",
        script("echo 'acme editor ready'; exec cat >/dev/null"),
    );
    let (app, lines) = logged(App::new(reading_agent()).with_editor(editor));
    let mut screen = focused(app);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor ready");

    screen.type_text("acme-secret");

    let lines = read(&lines);
    assert_eq!(
        count(&lines, " input to=editor session=1 editor=2 bytes=1"),
        11
    );
    assert_eq!(count(&lines, "acme-secret"), 0);
}

#[test]
fn a_key_harbour_acts_on_after_a_paste_is_logged_by_name() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = focused(app);
    screen.app.paste("acme-secret");
    read(&lines);

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    only(&read(&lines), r#" key name="C-h""#);
}

#[test]
fn a_paste_logs_the_bytes_sent_with_its_bracketed_paste_marks() {
    let agent = script("printf '\\033[?2004h'; echo 'acme agent ready'; exec cat");
    let (app, lines) = logged(App::new(agent));
    let mut screen = focused(app);
    read(&lines);

    screen.app.paste("acme");

    let input = only(&read(&lines), " input ");
    assert!(
        input.ends_with(" input to=session session=1 bytes=16\n"),
        "{input}"
    );
}

#[test]
fn a_started_session_is_logged_as_selected_with_its_directory() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);

    screen.press('n');

    let selected = only(&read(&lines), " selected ");
    assert!(selected.contains(" session=1 "), "{selected}");
    assert!(
        selected.ends_with(&format!(" directory={:?}\n", home())),
        "{selected}"
    );
}

#[test]
fn a_move_of_the_selection_is_logged_once() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.press('n');
    read(&lines);

    screen.press('k');
    screen.draw();

    only(&read(&lines), " selected session=1 ");
}

#[test]
fn the_focus_moving_to_a_session_is_logged() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    only(&read(&lines), " focus to=pane shown=session\n");
}

#[test]
fn the_focus_moving_back_to_the_sidebar_is_logged() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = focused(app);
    read(&lines);

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);

    only(&read(&lines), " focus to=sidebar\n");
}

#[test]
fn a_session_that_cannot_start_logs_the_refusal_once() {
    let (app, lines) = logged(App::new(vec!["/acme/missing-agent".to_string()]));
    let mut screen = Screen::of(app);

    screen.press('n');
    screen.draw();

    let refused = only(&read(&lines), " refused ");
    assert!(
        refused.contains(r#" message="Could not start /acme/missing-agent: "#),
        "{refused}"
    );
}

#[test]
fn q_with_no_session_logs_the_quit() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);

    screen.prefixed('q');

    only(&read(&lines), " quit sessions=0\n");
}

#[test]
fn a_stop_logs_its_cause_before_the_quit() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = Screen::of(app);
    screen.draw();
    read(&lines);

    screen.app.stop("signal name=TERM");

    let lines = read(&lines);
    assert_eq!(lines.len(), 2, "{lines:#?}");
    assert!(lines[0].ends_with(" signal name=TERM\n"), "{lines:#?}");
    assert!(lines[1].ends_with(" quit sessions=0\n"), "{lines:#?}");
}

pub(crate) fn focused(app: App) -> Screen {
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen
}

pub(crate) fn logged(app: App) -> (App, Receiver<String>) {
    let (sender, lines) = mpsc::sync_channel(4096);
    (app.with_log(Log::sending(sender)), lines)
}

pub(crate) fn read(lines: &Receiver<String>) -> Vec<String> {
    lines.try_iter().collect()
}

pub(crate) fn count(lines: &[String], event: &str) -> usize {
    lines.iter().filter(|line| line.contains(event)).count()
}

pub(crate) fn only(lines: &[String], event: &str) -> String {
    let found: Vec<&String> = lines.iter().filter(|line| line.contains(event)).collect();
    assert_eq!(found.len(), 1, "{event:?} in {lines:#?}");
    found[0].clone()
}
