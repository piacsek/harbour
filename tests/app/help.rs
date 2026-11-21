use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::style::Color;

use crate::{Screen, directory, raw_agent, reading_agent};

const POINTER: &str = "harbour --help has the rest.";

#[test]
fn question_mark_shows_the_keys_and_where_the_notes_went() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(shown.contains("j/k"), "{shown}");
    assert!(
        shown.contains("end the session and its editors, or forget it once it has ended"),
        "{shown}"
    );
    assert!(shown.contains(POINTER), "{shown}");
}

#[test]
fn question_mark_shows_the_editor_keys() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("focus or resume; from Claude Code, its editor"),
        "{shown}"
    );
    assert!(shown.contains("the session and its editors"), "{shown}");
    assert!(
        shown.contains("show the next editor, one per directory"),
        "{shown}"
    );
}

#[test]
fn question_mark_shows_the_zoom_key() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("M-z        zoom the session to the whole screen, or back"),
        "{shown}"
    );
}

#[test]
fn question_mark_says_how_to_resume_and_forget_a_stopped_session() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(80, 24);

    screen.press('?');

    let shown = screen.draw();
    assert!(shown.contains("focus or resume;"), "{shown}");
    assert!(shown.contains("or forget it once it has ended"), "{shown}");
    assert!(shown.contains("○ idle, exited, or stopped"), "{shown}");
}

#[test]
fn question_mark_explains_the_dots() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("● working: thinking, or running a command"),
        "{shown}"
    );
    assert!(
        shown.contains("◉ blocked, waiting on you: the card says what for"),
        "{shown}"
    );
    assert!(shown.contains("○ idle, exited, or stopped"), "{shown}");
    assert!(
        shown.contains("· status unknown: not Claude Code, or not written yet"),
        "{shown}"
    );
    let (x, y) = glyph_at(&shown, "◉ blocked");
    assert_eq!(screen.cell(x, y).fg, Color::Red);
}

#[test]
fn the_whole_help_shows_on_an_80_by_24_terminal() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(80, 24);

    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("select the next / previous session"),
        "{shown}"
    );
    assert!(shown.contains("status unknown"), "{shown}");
    assert!(shown.contains(POINTER), "{shown}");
}

#[test]
fn the_whole_help_shows_on_a_20_row_terminal() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("select the next / previous session"),
        "{shown}"
    );
    assert!(shown.contains("status unknown"), "{shown}");
    assert!(shown.contains(POINTER), "{shown}");
}

#[test]
fn the_help_leaves_a_row_for_a_new_key_on_a_20_row_terminal() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert_eq!(spare_rows(&shown), 1, "{shown}");
}

#[test]
fn question_mark_shows_the_usage_key_on_one_row_beside_the_narrowest_sidebar() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(60, 20);

    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("u          show Claude Code's usage and limits"),
        "{shown}"
    );
}

#[test]
fn question_mark_shows_the_find_key_on_one_row_beside_the_narrowest_sidebar() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(60, 20);

    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("M-s        jump to a session by part of its name"),
        "{shown}"
    );
}

#[test]
fn question_mark_shows_the_ctrl_l_row_on_one_row_in_the_narrow_layout() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(59, 20);

    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("C-l/Enter  focus or resume; from Claude Code, its editor"),
        "{shown}"
    );
}

#[test]
fn the_whole_help_shows_beside_the_narrowest_sidebar() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(60, 20);

    screen.press('?');

    let shown = screen.draw();
    assert!(shown.contains("select the next / previous"), "{shown}");
    assert!(shown.contains(POINTER), "{shown}");
}

#[test]
fn the_whole_help_shows_in_the_narrow_layout_on_20_rows() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(59, 20);

    screen.press('?');

    let shown = screen.draw();
    assert!(shown.contains("select the next / previous"), "{shown}");
    assert!(shown.contains(POINTER), "{shown}");
}

#[test]
fn no_note_shows_in_the_help() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(80, 24);

    screen.press('?');

    let shown = screen.draw();
    assert!(!shown.contains("HARBOUR_DIR_PICKER"), "{shown}");
    assert!(!shown.contains("HARBOUR_EDITOR"), "{shown}");
    assert!(!shown.contains("HARBOUR_LOG"), "{shown}");
    assert!(!shown.contains("C-Backspace"), "{shown}");
    assert!(!shown.contains("vim-tmux-navigator"), "{shown}");
    assert!(!shown.contains("Alt-Enter"), "{shown}");
}

#[test]
fn the_next_key_hides_the_help_and_does_nothing_else() {
    let mut screen = Screen::new(reading_agent());
    screen.press('?');

    screen.press('n');

    let shown = screen.draw();
    assert!(!shown.contains(POINTER), "{shown}");
    assert!(!shown.contains(&format!("{} 1", directory())), "{shown}");
}

#[test]
fn a_click_in_the_pane_hides_the_help_and_does_not_focus_the_pane() {
    let mut screen = Screen::new(raw_agent("", 1));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.press('?');

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 45, 3);
    screen.press('z');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.press('w');

    let shown = screen.wait_for("acme agent read:w");
    assert!(shown.contains("acme agent read:w"), "{shown}");
    assert!(!shown.contains(POINTER), "{shown}");
}

#[test]
fn a_paste_hides_the_help() {
    let mut screen = Screen::new(reading_agent());
    screen.press('?');

    screen.app.paste("acme");

    let shown = screen.draw();
    assert!(!shown.contains(POINTER), "{shown}");
}

fn glyph_at(shown: &str, text: &str) -> (u16, u16) {
    let (y, line) = shown
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains(text))
        .unwrap();
    let x = line[..line.find(text).unwrap()].chars().count();
    (u16::try_from(x).unwrap(), u16::try_from(y).unwrap())
}

fn spare_rows(shown: &str) -> usize {
    shown
        .lines()
        .filter(|line| line.trim_matches(['│', ' ']).is_empty() && line.starts_with('│'))
        .count()
}
