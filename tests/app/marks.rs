use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::keymap::{Action, Keymap};
use ratatui::style::{Color, Modifier};

use crate::side::{agent, editor, side};
use crate::{
    Screen, drawn, focused_char, reading_agent, region, script, starts_focused, unfocused_char,
};

#[test]
fn the_waiting_prefix_shows_on_the_panes_bottom_border() {
    let mut screen = started(App::new(reading_agent()));

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(before_corner(&bottom(&shown)).ends_with(" C-/ "), "{shown}");
    assert!(bottom(&shown).ends_with(unfocused_char), "{shown}");
}

#[test]
fn the_waiting_prefix_is_bold_in_the_accent() {
    let mut screen = started(App::new(reading_agent()).with_accent(Color::Magenta));

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);

    assert_eq!(screen.cell(115, 19).symbol(), "C");
    assert_eq!(screen.cell(115, 19).fg, Color::Magenta);
    assert!(screen.cell(115, 19).modifier.contains(Modifier::BOLD));
}

#[test]
fn the_waiting_prefix_goes_once_the_next_key_comes() {
    let mut screen = started(App::new(reading_agent()));
    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);

    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(!bottom(&region(&shown, 40, 80)).contains("C-/"), "{shown}");
}

#[test]
fn the_waiting_prefix_shows_zoomed_on_the_top_row_before_the_counts() {
    let mut screen = started(App::new(reading_agent()));
    screen.prefixed('z');

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    let top = shown.lines().next().unwrap_or_default();
    assert!(top.ends_with(" C-/ sessions · 1 "), "{shown}");
}

#[test]
fn the_waiting_prefix_shows_side_by_side_on_the_focused_halfs_bottom_border() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(agent(scratch.path())).with_editor(Program::new("acme-edit", editor()));
    let mut screen = started(app);
    side(&mut screen);
    screen.wait_for("acme editor ready");
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(before_corner(&bottom(&shown)).ends_with(" C-/ "), "{shown}");
    assert!(bottom(&shown).ends_with(focused_char), "{shown}");
    assert!(!region(&shown, 0, 60).contains("C-/"), "{shown}");
}

#[test]
fn the_waiting_prefix_shows_on_the_pane_in_the_narrow_layout() {
    let mut screen = started(App::new(reading_agent()));
    screen.resize(50, 20);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(before_corner(&bottom(&shown)).ends_with(" C-/ "), "{shown}");
    assert!(bottom(&shown).ends_with(focused_char), "{shown}");
}

#[test]
fn the_sidebar_shows_the_way_to_the_help_on_its_bottom_border() {
    let mut screen = started(App::new(reading_agent()));

    let shown = screen.draw();

    assert!(starts_focused(&bottom(&shown)), "{shown}");
    assert!(
        before_corner(&region(&shown, 0, 40)).ends_with(" C-/ ? help "),
        "{shown}"
    );
}

#[test]
fn the_way_to_the_help_is_dimmed() {
    let mut screen = started(App::new(reading_agent()));

    screen.draw();

    assert_eq!(screen.cell(28, 19).symbol(), "C");
    assert!(screen.cell(28, 19).modifier.contains(Modifier::DIM));
}

#[test]
fn the_way_to_the_help_fits_the_narrowest_sidebar_with_its_corners() {
    let mut screen = started(App::new(reading_agent()).with_sidebar_width(15));

    let shown = screen.draw();

    let border = bottom(&region(&shown, 0, 15));
    assert_eq!(border.chars().count(), 15, "{shown}");
    assert_eq!(border.trim_matches(drawn), " C-/ ? help ", "{shown}");
}

#[test]
fn the_way_to_the_help_is_left_out_when_the_sidebar_cannot_hold_it_whole() {
    let keymap = Keymap::default().with_prefix(' ');
    let mut screen = started(
        App::new(reading_agent())
            .with_keymap(keymap)
            .with_sidebar_width(15),
    );

    let shown = screen.draw();

    let border = bottom(&region(&shown, 0, 15));
    assert_eq!(border.chars().count(), 15, "{shown}");
    assert!(border.chars().all(drawn), "{shown}");
}

#[test]
fn the_way_to_the_help_names_the_configured_prefix_and_help_key() {
    let keymap = Keymap::default()
        .with_prefix(']')
        .bind(Action::Help, Some('h'))
        .bind(Action::Sidebar, None);
    let mut screen = started(App::new(reading_agent()).with_keymap(keymap));

    let shown = screen.draw();

    assert!(
        before_corner(&region(&shown, 0, 40)).ends_with(" C-] h help "),
        "{shown}"
    );
}

#[test]
fn the_way_to_the_help_is_left_out_while_help_is_unbound() {
    let keymap = Keymap::default().bind(Action::Help, None);
    let mut screen = started(App::new(reading_agent()).with_keymap(keymap));

    let shown = screen.draw();

    assert!(!shown.contains("help"), "{shown}");
}

#[test]
fn with_no_session_the_empty_pane_shows_the_way_to_the_help_too() {
    let mut screen = Screen::new(script("exec sleep 600"));

    let shown = screen.draw();

    assert!(region(&shown, 41, 78).contains("C-/ ? help"), "{shown}");
}

fn started(app: App) -> Screen {
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme");
    screen
}

fn before_corner(text: &str) -> &str {
    text.trim_end_matches(drawn)
}

fn bottom(shown: &str) -> String {
    shown.lines().last().unwrap_or_default().to_string()
}
