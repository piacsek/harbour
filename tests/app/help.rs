use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::style::Color;

use crate::{Screen, directory, raw_agent, reading_agent};

#[test]
fn question_mark_shows_the_keys_and_the_tmux_pattern() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(shown.contains("j/k"), "{shown}");
    assert!(
        shown.contains("end the session and its editors, or forget it once it has ended"),
        "{shown}"
    );
    assert!(shown.contains("vim-tmux-navigator"), "{shown}");
}

#[test]
fn question_mark_shows_the_editor_keys() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("from Claude Code, C-l opens its editor"),
        "{shown}"
    );
    assert!(
        shown.contains("In Neovim, C-h goes back only from the leftmost window, in normal mode."),
        "{shown}"
    );
    assert!(shown.contains("the session and its editors"), "{shown}");
    assert!(
        shown.contains("show the session's next editor, one per directory"),
        "{shown}"
    );
}

#[test]
fn question_mark_names_the_setting_for_another_editor() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(shown.contains("HARBOUR_EDITOR"), "{shown}");
}

#[test]
fn question_mark_says_how_to_resume_and_forget_a_stopped_session() {
    let mut screen = Screen::new(reading_agent());
    screen.resize(80, 24);

    screen.press('?');

    let shown = screen.draw();
    assert!(shown.contains("focus the pane or resume;"), "{shown}");
    assert!(shown.contains("or forget it once it has ended"), "{shown}");
    assert!(shown.contains("○ idle, exited, or stopped"), "{shown}");
    assert!(shown.contains("vim-tmux-navigator"), "{shown}");
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
    assert!(shown.contains("in normal mode."), "{shown}");
    assert!(shown.contains("use Alt-Enter for a newline."), "{shown}");
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
    assert!(shown.contains("use Alt-Enter for a newline."), "{shown}");
}

#[test]
fn question_mark_says_that_ctrl_backspace_acts_as_ctrl_h() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("Most terminals send C-Backspace as C-h, so it acts as C-h."),
        "{shown}"
    );
}

#[test]
fn question_mark_names_the_setting_for_another_dir_picker() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(shown.contains("HARBOUR_DIR_PICKER"), "{shown}");
}

#[test]
fn question_mark_says_the_picker_starts_where_harbour_was_opened() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(
        shown.contains("n runs fzf over the directory harbour was opened in and zoxide's list"),
        "{shown}"
    );
}

#[test]
fn the_next_key_hides_the_help_and_does_nothing_else() {
    let mut screen = Screen::new(reading_agent());
    screen.press('?');

    screen.press('n');

    let shown = screen.draw();
    assert!(!shown.contains("vim-tmux-navigator"), "{shown}");
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
    assert!(!shown.contains("vim-tmux-navigator"), "{shown}");
}

#[test]
fn a_paste_hides_the_help() {
    let mut screen = Screen::new(reading_agent());
    screen.press('?');

    screen.app.paste("acme");

    let shown = screen.draw();
    assert!(!shown.contains("vim-tmux-navigator"), "{shown}");
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
