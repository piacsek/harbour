use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;
use harbour::keymap::{Action, Keymap};
use ratatui::layout::Position;

use crate::logging::{count, logged, read};
use crate::zoom::corners;
use crate::{Screen, reading_agent};

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

fn started(app: App) -> Screen {
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen
}

fn rename(screen: &mut Screen, title: &str) {
    screen.prefixed('r');
    for _ in 0..40 {
        screen.key(KeyCode::Backspace, KeyModifiers::NONE);
    }
    screen.type_text(title);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
}
