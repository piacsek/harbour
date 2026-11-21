use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;

use crate::{Screen, home, numbered_agent, pane_corner};

#[test]
fn the_sidebar_takes_the_width_it_is_given() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = sized(numbered_agent(scratch.path()), 30, home(), 120);

    screen.press('n');

    let shown = screen.wait_for("acme agent 1");
    assert_eq!(pane_corner(&shown), 30);
}

#[test]
fn the_pane_keeps_at_least_half_the_width() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = sized(numbered_agent(scratch.path()), 100, home(), 120);

    screen.press('n');

    let shown = screen.wait_for("acme agent 1");
    assert_eq!(pane_corner(&shown), 60);
}

#[test]
fn a_resize_keeps_the_width() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = sized(numbered_agent(scratch.path()), 40, home(), 120);
    screen.press('n');
    screen.wait_for("acme agent 1");

    screen.resize(160, 20);

    assert_eq!(pane_corner(&screen.draw()), 40);
}

#[test]
fn the_narrowest_two_column_terminal_keeps_a_one_column_sidebar() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = sized(numbered_agent(scratch.path()), 1, home(), 60);

    screen.press('n');

    let shown = screen.wait_for("acme agent 1");
    assert_eq!(pane_corner(&shown), 1);
}

#[test]
fn below_sixty_columns_one_side_shows_at_a_time_whatever_the_width() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = sized(numbered_agent(scratch.path()), 20, home(), 120);
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.resize(59, 16);

    assert_eq!(screen.cell(0, 0).symbol(), "┏");
    assert_eq!(screen.cell(58, 0).symbol(), "┓");
}

#[test]
fn a_cjk_name_stays_inside_a_narrow_sidebar() {
    let scratch = tempfile::tempdir().unwrap();
    let place = scratch.path().join("東京の倉庫アプリ");
    std::fs::create_dir_all(&place).unwrap();
    let mut screen = sized(numbered_agent(scratch.path()), 12, place, 120);

    screen.press('n');

    screen.wait_for("acme agent 1");
    assert_eq!(screen.cell(11, 1).symbol(), "┃");
    assert_eq!(screen.cell(12, 0).symbol(), "┌");
}

fn sized(command: Vec<String>, width: u16, home: PathBuf, columns: u16) -> Screen {
    Screen::at(App::new(command).with_sidebar_width(width), home, columns)
}
