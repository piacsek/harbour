use std::path::Path;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::picker::Picker;
use ratatui::layout::Position;
use ratatui::style::Modifier;

use crate::{Screen, home, pane_corner, script};

#[test]
fn the_terminal_cursor_sits_on_the_pane_cursor_cell() {
    let mut screen = Screen::new(shaped_agent("6"));
    screen.press('n');
    let shown = screen.wait_for("acme agent shaped");

    let corner = pane_corner(&shown);

    assert_eq!(screen.cursor(), Some(Position::new(corner + 22, 3)));
}

#[test]
fn the_app_asks_for_the_shape_the_shown_program_set() {
    let mut screen = Screen::new(shaped_agent("6"));
    screen.press('n');
    screen.wait_for("acme agent shaped");

    assert_eq!(screen.app.cursor_shape(), 6);
}

#[test]
fn a_program_that_hid_its_cursor_shows_none() {
    let mut screen = Screen::new(script(
        "printf '\\033[?25lacme agent hidden'; exec sleep 600",
    ));
    screen.press('n');
    screen.wait_for("acme agent hidden");

    assert_eq!(screen.cursor(), None);
}

#[test]
fn with_no_session_the_terminal_shows_no_cursor() {
    let screen = Screen::new(shaped_agent("6"));

    assert_eq!(screen.cursor(), None);
    assert_eq!(screen.app.cursor_shape(), 0);
}

#[test]
fn the_help_view_shows_no_cursor_and_the_default_shape() {
    let mut screen = Screen::new(shaped_agent("6"));
    screen.press('n');
    screen.wait_for("acme agent shaped");

    screen.press('?');

    screen.wait_for(" help ");
    assert_eq!(screen.cursor(), None);
    assert_eq!(screen.app.cursor_shape(), 0);
}

#[test]
fn a_question_leaves_the_cursor_in_the_pane() {
    let mut screen = Screen::new(shaped_agent("6"));
    screen.press('n');
    let shown = screen.wait_for("acme agent shaped");
    let corner = pane_corner(&shown);

    screen.press('x');

    screen.wait_for("End? y/n");
    assert_eq!(screen.cursor(), Some(Position::new(corner + 22, 3)));
    assert_eq!(screen.app.cursor_shape(), 6);
}

#[test]
fn the_picker_popup_shows_its_own_cursor_and_shape() {
    let mut screen = Screen::picking(shaped_agent("6"), Picker::Command(typed_picker()));
    started_through_the_picker(&mut screen, &home());

    screen.press('n');

    screen.wait_for("acme picker");
    assert_eq!(screen.cursor(), Some(Position::new(13, 4)));
    assert_eq!(screen.app.cursor_shape(), 0);
}

#[test]
fn another_session_shown_takes_its_own_shape() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(first_shaped_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");
    let second = screen.app.cursor_shape();

    screen.press('k');

    screen.wait_for("acme agent 1");
    assert_eq!(second, 0);
    assert_eq!(screen.app.cursor_shape(), 6);
}

#[test]
fn the_narrowest_sidebar_keeps_the_cursor_in_the_pane() {
    let mut screen = Screen::at(App::new(shaped_agent("6")), home(), 60);
    screen.press('n');
    let shown = screen.wait_for("acme agent shaped");

    let corner = pane_corner(&shown);

    assert_eq!(corner, 20);
    assert_eq!(screen.cursor(), Some(Position::new(corner + 22, 3)));
}

#[test]
fn a_narrow_terminal_showing_only_the_sidebar_shows_no_cursor() {
    let mut screen = Screen::at(App::new(shaped_agent("6")), home(), 50);
    screen.press('n');
    screen.wait_for("acme-home 1");

    assert_eq!(screen.cursor(), None);
}

#[test]
fn a_wide_character_under_the_cursor_is_drawn_whole_and_unstyled() {
    let mut screen = Screen::new(script("printf 'acme 東京\\033[2D'; exec sleep 600"));
    screen.press('n');
    let shown = screen.wait_for("京");
    let corner = pane_corner(&shown);

    let cell = screen.cell(corner + 8, 1);

    assert_eq!(screen.cursor(), Some(Position::new(corner + 8, 1)));
    assert_eq!(cell.symbol(), "京");
    assert!(!cell.modifier.contains(Modifier::REVERSED));
}

fn shaped_agent(shape: &str) -> Vec<String> {
    script(&format!(
        "printf '\\033[{shape} q\\033[3;5Hacme agent shaped'; exec sleep 600"
    ))
}

fn first_shaped_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         if [ $n = 1 ]; then printf '\\033[6 q'; fi; \
         echo \"acme agent $n\"; exec sleep 600",
        count = count.display()
    ))
}

fn typed_picker() -> Vec<String> {
    script("echo 'acme picker' >&2; read -r line; printf '%s\\n' \"$line\"")
}

fn started_through_the_picker(screen: &mut Screen, directory: &Path) {
    screen.press('n');
    screen.wait_for("acme picker");
    screen.type_text(&format!("{}\r", directory.display()));
    screen.wait_for("acme agent shaped");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.wait_for("┏ sessions");
}
