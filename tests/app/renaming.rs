use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;
use harbour::keymap::{Action, Keymap};
use ratatui::layout::Position;

use crate::finding::three_sessions;
use crate::logging::{count, logged, read};
use crate::zoom::corners;
use crate::zoom_border::{Claude, started as started_titled, top};
use crate::{Screen, in_order, reading_agent};

#[test]
fn the_prefix_then_r_opens_a_line_at_the_bottom_of_the_sidebar_holding_the_title() {
    let mut screen = started(App::new(reading_agent()));

    screen.prefixed('r');

    let shown = screen.draw();
    let last: String = shown
        .lines()
        .nth(18)
        .unwrap_or_default()
        .chars()
        .skip(1)
        .collect();
    assert!(last.starts_with("Title: acme-home 1 "), "{shown}");
}

#[test]
fn the_cursor_sits_after_the_title() {
    let mut screen = started(App::new(reading_agent()));

    screen.prefixed('r');

    assert_eq!(screen.cursor(), Some(Position::new(19, 18)));
}

#[test]
fn a_typed_title_replaces_the_cards_on_enter() {
    let mut screen = started(App::new(reading_agent()));

    rename(&mut screen, "acme login fix");

    let shown = screen.draw();
    assert!(shown.contains(" acme login fix "), "{shown}");
    assert!(!shown.contains("acme-home 1"), "{shown}");
    assert!(!shown.contains("Title:"), "{shown}");
}

#[test]
fn escape_closes_the_line_and_keeps_the_title() {
    let mut screen = started(App::new(reading_agent()));
    screen.prefixed('r');
    screen.press('z');

    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(shown.contains(" acme-home 1 "), "{shown}");
    assert!(!shown.contains("acme-home 1z"), "{shown}");
    assert!(!shown.contains("Title:"), "{shown}");
}

#[test]
fn an_empty_title_brings_the_first_one_back() {
    let mut screen = started(App::new(reading_agent()));
    rename(&mut screen, "acme login fix");

    rename(&mut screen, "");

    let shown = screen.draw();
    assert!(shown.contains(" acme-home 1 "), "{shown}");
    assert!(!shown.contains("acme login fix"), "{shown}");
}

#[test]
fn a_saved_title_is_cleaned() {
    let mut screen = started(App::new(reading_agent()));

    rename(&mut screen, "  acme   login ");

    let shown = screen.draw();
    assert!(shown.contains(" acme login "), "{shown}");
}

#[test]
fn backspace_takes_a_whole_character_away() {
    let mut screen = started(App::new(reading_agent()));
    screen.prefixed('r');
    screen.type_text("東");

    screen.key(KeyCode::Backspace, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(shown.contains("Title: acme-home 1 "), "{shown}");
}

#[test]
fn from_a_pane_the_typed_keys_never_reach_the_program() {
    let mut screen = started(App::new(reading_agent()));
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.prefixed('r');
    screen.type_text("jk");
    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    screen.type_text("ok\r");

    let shown = screen.wait_for("acme agent got: ok");
    assert!(!shown.contains("jk"), "{shown}");
}

#[test]
fn the_prefix_while_the_line_is_open_types_nothing_and_does_nothing() {
    let mut screen = started(App::new(reading_agent()));
    screen.prefixed('r');

    screen.prefixed('z');

    let shown = screen.draw();
    assert!(shown.contains("Title: acme-home 1z"), "{shown}");
    assert!(corners(&shown) > 0, "{shown}");
}

#[test]
fn a_paste_is_added_cleaned() {
    let mut screen = started(App::new(reading_agent()));
    screen.prefixed('r');

    screen.app.paste(" to\nacme\u{7}");
    let shown = screen.draw();

    assert!(shown.contains("Title: acme-home 1to acme "), "{shown}");
}

#[test]
fn a_mouse_press_leaves_the_line_open() {
    let mut screen = started(App::new(reading_agent()));
    screen.prefixed('r');

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 60, 5);

    let shown = screen.draw();
    assert!(shown.contains("Title: acme-home 1"), "{shown}");
}

#[test]
fn zoomed_the_line_takes_the_bottom_row_of_the_pane() {
    let mut screen = started(App::new(reading_agent()));
    screen.prefixed('z');

    screen.prefixed('r');

    let shown = screen.draw();
    let last = shown.lines().nth(19).unwrap_or_default();
    assert!(last.starts_with("Title: acme-home 1 "), "{shown}");
}

#[test]
fn the_rename_key_moves_with_the_keys_table() {
    let keymap = Keymap::default().bind(Action::Rename, Some('t'));
    let mut screen = started(App::new(reading_agent()).with_keymap(keymap));

    screen.prefixed('t');

    let shown = screen.draw();
    assert!(shown.contains("Title: acme-home 1"), "{shown}");
}

#[test]
fn renaming_logs_the_key_and_never_the_text() {
    let (app, lines) = logged(App::new(reading_agent()));
    let mut screen = started(app);
    read(&lines);

    rename(&mut screen, "jky");

    let logged = read(&lines);
    assert_eq!(count(&logged, "prefix key=rename"), 1, "{logged:#?}");
    assert_eq!(count(&logged, "key name="), 0, "{logged:#?}");
}

#[test]
fn a_given_title_wins_over_claude_codes_on_the_card() {
    let claude = Claude::new("Fix the login");
    let mut screen = Screen::of(claude.app());
    titled(&mut screen);

    rename(&mut screen, "acme login fix");

    let shown = screen.draw();
    assert!(shown.contains(" acme login fix "), "{shown}");
    assert!(!shown.contains("Fix the login"), "{shown}");
}

#[test]
fn a_given_title_leads_the_zoomed_top_row() {
    let claude = Claude::new("Fix the login");
    let mut screen = Screen::of(claude.app());
    titled(&mut screen);
    rename(&mut screen, "acme login fix");

    screen.prefixed('z');

    let top = top(&screen.draw());
    assert!(
        in_order(&top, &["acme login fix", "acme-api · main"]),
        "{top}"
    );
    assert!(!top.contains("Fix the login"), "{top}");
}

#[test]
fn enter_on_the_title_unchanged_keeps_following_claude_codes() {
    let claude = Claude::new("Fix the login");
    let mut screen = Screen::of(claude.app());
    titled(&mut screen);
    screen.prefixed('r');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    claude.retitle("Fix the logout");

    screen.wait_for(" Fix the logout ");
}

#[test]
fn the_session_finder_lists_a_given_title() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    rename(&mut screen, "acme deploy");

    screen.prefixed('s');

    let shown = screen.wait_for("acme fzf ready");
    assert!(shown.contains("2|·|acme deploy |acme-home |"), "{shown}");
}

fn titled(screen: &mut Screen) {
    started_titled(screen);
    screen.wait_for("Fix the login");
}

#[test]
fn shift_with_a_letter_types_its_capital() {
    let mut screen = started(App::new(reading_agent()));
    screen.prefixed('r');

    screen.key(KeyCode::Char('a'), KeyModifiers::SHIFT);

    let shown = screen.draw();
    assert!(shown.contains("Title: acme-home 1A "), "{shown}");
}

#[test]
fn a_terminal_too_short_for_the_line_shows_no_cursor() {
    let mut screen = started(App::new(reading_agent()));
    screen.prefixed('r');

    screen.resize(120, 2);

    assert_eq!(screen.cursor(), None);
}

#[test]
fn the_text_stops_at_the_512_characters_a_title_keeps() {
    let mut screen = started(App::new(reading_agent()));
    screen.prefixed('r');
    screen.app.paste(&"a".repeat(600));

    screen.press('b');

    let shown = screen.draw();
    assert!(!shown.contains("ab"), "{shown}");
}

fn started(app: App) -> Screen {
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen
}

pub(crate) fn rename(screen: &mut Screen, title: &str) {
    screen.prefixed('r');
    for _ in 0..40 {
        screen.key(KeyCode::Backspace, KeyModifiers::NONE);
    }
    screen.type_text(title);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
}
