use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;
use harbour::help::KEYS;
use harbour::keymap::{Action, Keymap};
use ratatui::style::Color;

use crate::{Screen, directory, raw_agent, reading_agent};

const LAST: &str = "status unknown: not Claude Code, or not written yet";

#[test]
fn question_mark_shows_the_keys_and_the_last_dot() {
    let mut screen = Screen::new(reading_agent());

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(shown.starts_with("┌ help "), "{shown}");
    assert!(shown.contains("j/k"), "{shown}");
    assert!(
        shown.contains("n/x/X      start / end or forget / end & forget, C-/ too"),
        "{shown}"
    );
    assert!(shown.contains(LAST), "{shown}");
}

#[test]
fn question_mark_shows_the_editor_keys() {
    let mut screen = Screen::new(reading_agent());

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(
        shown.contains("focus or resume; C-l in Claude Code: editor"),
        "{shown}"
    );
    assert!(
        shown.contains("C-/ e      show the next editor, one per directory"),
        "{shown}"
    );
}

#[test]
fn question_mark_shows_the_zoom_key() {
    let mut screen = Screen::new(reading_agent());

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(
        shown.contains("C-/ z      zoom the session to the whole screen, or back"),
        "{shown}"
    );
}

#[test]
fn question_mark_shows_the_side_by_side_key() {
    let mut screen = Screen::new(reading_agent());

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(
        shown.contains("C-/ \\ {}   side by side (120+ columns); swap the halves"),
        "{shown}"
    );
}

#[test]
fn question_mark_shows_the_width_keys() {
    let mut screen = Screen::new(reading_agent());

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(
        shown.contains("C-/ -/=    narrow / widen the sidebar by 2 columns"),
        "{shown}"
    );
}

#[test]
fn question_mark_says_how_to_resume_and_forget_a_stopped_session() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(80, 24);

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(shown.contains("focus or resume;"), "{shown}");
    assert!(shown.contains("end or forget"), "{shown}");
    assert!(shown.contains("○ idle, exited, or stopped"), "{shown}");
}

#[test]
fn question_mark_explains_the_dots() {
    let mut screen = Screen::new(reading_agent());

    screen.prefixed('?');

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

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(shown.contains(KEYS[0].1), "{shown}");
    assert!(shown.contains("status unknown"), "{shown}");
    assert!(shown.contains(LAST), "{shown}");
}

#[test]
fn the_whole_help_shows_on_a_20_row_terminal() {
    let mut screen = Screen::new(reading_agent());

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(shown.contains(KEYS[0].1), "{shown}");
    assert!(shown.contains("status unknown"), "{shown}");
    assert!(shown.contains(LAST), "{shown}");
}

#[test]
fn the_dots_are_the_last_lines_of_the_help() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(80, 30);

    screen.prefixed('?');

    let shown = screen.draw();
    let after: Vec<&str> = shown
        .lines()
        .skip_while(|line| !line.contains(LAST))
        .skip(1)
        .collect();
    assert_eq!(after[0].trim_matches(['│', ' ']), "", "{shown}");
}

#[test]
fn the_help_fills_a_20_row_terminal_with_no_row_to_spare() {
    let mut screen = Screen::new(reading_agent());

    screen.prefixed('?');

    let shown = screen.draw();
    assert_eq!(spare_rows(&shown), 0, "{shown}");
}

#[test]
fn question_mark_shows_the_usage_and_changelog_keys_on_one_row_beside_the_narrowest_sidebar() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(60, 20);

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(
        shown.contains("C-/ u/c    Claude Code's usage / the changelog"),
        "{shown}"
    );
}

#[test]
fn question_mark_shows_the_help_and_quit_keys_on_one_row_beside_the_narrowest_sidebar() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(60, 20);

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(
        shown.contains("C-/ ?/q    this help / quit, asking while anything runs"),
        "{shown}"
    );
}

#[test]
fn question_mark_shows_the_find_and_rename_keys_on_one_row_beside_the_narrowest_sidebar() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(60, 20);

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(
        shown.contains(&format!("{:10} {}", KEYS[7].0, KEYS[7].1)),
        "{shown}"
    );
}

#[test]
fn question_mark_shows_the_ctrl_l_row_on_one_row_in_the_narrow_layout() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(59, 20);

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(
        shown.contains("Enter/C-l  focus or resume; C-l in Claude Code: editor"),
        "{shown}"
    );
}

#[test]
fn the_whole_help_shows_beside_the_narrowest_sidebar() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(60, 20);

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(shown.contains(KEYS[0].1), "{shown}");
    assert!(shown.contains(LAST), "{shown}");
}

#[test]
fn the_whole_help_shows_in_the_narrow_layout_on_20_rows() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(59, 20);

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(shown.contains(KEYS[0].1), "{shown}");
    assert!(shown.contains(LAST), "{shown}");
}

#[test]
fn no_note_shows_in_the_help() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(80, 24);

    screen.prefixed('?');

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
    screen.prefixed('?');

    screen.press('n');

    let shown = screen.draw();
    assert!(!shown.contains(LAST), "{shown}");
    assert!(!shown.contains(&format!("{} 1", directory())), "{shown}");
}

#[test]
fn a_click_in_the_pane_hides_the_help_and_does_not_focus_the_pane() {
    let mut screen = Screen::new(raw_agent("", 1));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.prefixed('?');

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 45, 3);
    screen.press('z');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.press('w');

    let shown = screen.wait_for("acme agent read:w");
    assert!(shown.contains("acme agent read:w"), "{shown}");
    assert!(!shown.contains(LAST), "{shown}");
}

#[test]
fn a_paste_hides_the_help() {
    let mut screen = Screen::new(reading_agent());
    screen.prefixed('?');

    screen.app.paste("acme");

    let shown = screen.draw();
    assert!(!shown.contains(LAST), "{shown}");
}

#[test]
fn the_help_names_a_configured_prefix_in_its_rows() {
    let keymap = Keymap::default().with_prefix(']');
    let mut screen = Screen::of(App::new(reading_agent()).with_keymap(keymap));

    screen.key(KeyCode::Char(']'), KeyModifiers::CONTROL);
    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("C-] z      zoom the session to the whole screen, or back"),
        "{shown}"
    );
    assert!(
        shown.contains("C-] C-] p  the previous session / send C-] to the program"),
        "{shown}"
    );
    assert!(
        shown.contains("n/x/X      start / end or forget / end & forget, C-] too"),
        "{shown}"
    );
}

#[test]
fn the_help_names_the_key_an_action_is_bound_to() {
    let keymap = Keymap::default().bind(Action::Zoom, Some('f'));
    let mut screen = Screen::of(App::new(reading_agent()).with_keymap(keymap));

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(
        shown.contains("C-/ f      zoom the session to the whole screen, or back"),
        "{shown}"
    );
}

#[test]
fn the_help_leaves_out_the_row_of_an_unbound_action() {
    let keymap = Keymap::default().bind(Action::Zoom, None);
    let mut screen = Screen::of(App::new(reading_agent()).with_keymap(keymap));

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(!shown.contains("zoom the session"), "{shown}");
}

#[test]
fn a_pair_row_with_one_side_unbound_shows_the_other_alone() {
    let keymap = Keymap::default()
        .bind(Action::Sidebar, None)
        .bind(Action::End, None)
        .bind(Action::Close, None);
    let mut screen = Screen::of(App::new(reading_agent()).with_keymap(keymap));

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(shown.contains("C-/ l      focus the pane"), "{shown}");
    assert!(
        shown.contains("n          start a session, C-/ too"),
        "{shown}"
    );
}

#[test]
fn the_side_row_without_its_key_still_shows_the_swap_keys() {
    let keymap = Keymap::default()
        .bind(Action::Side, None)
        .bind(Action::SwapLeft, None);
    let mut screen = Screen::of(App::new(reading_agent()).with_keymap(keymap));

    screen.prefixed('?');

    let shown = screen.draw();
    assert!(shown.contains("C-/ }      swap the halves"), "{shown}");
}

#[test]
fn the_space_prefix_is_named_by_its_symbol_so_the_column_keeps_its_width() {
    let keymap = Keymap::default().with_prefix(' ');
    let mut screen = Screen::of(App::new(reading_agent()).with_keymap(keymap));

    screen.key(KeyCode::Char(' '), KeyModifiers::CONTROL);
    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains(
            "C-\u{2423} C-\u{2423} p  the previous session / send C-\u{2423} to the program"
        ),
        "{shown}"
    );
    assert!(
        shown.contains(&format!("{:<10} {}", KEYS[0].0, KEYS[0].1)),
        "{shown}"
    );
}

#[test]
fn a_wide_key_keeps_the_column_of_what_it_does() {
    let keymap = Keymap::default().bind(Action::Zoom, Some('\u{30ba}'));
    let mut screen = Screen::of(App::new(reading_agent()).with_keymap(keymap));

    screen.prefixed('?');

    let shown = screen.draw();
    let zoom = glyph_at(&shown, "zoom the session");
    let editor = glyph_at(&shown, "show the next editor");
    assert_eq!(zoom.0, editor.0, "{shown}");
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
