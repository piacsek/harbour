use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};

use crate::{Screen, directory, raw_agent, reading_agent};

#[test]
fn question_mark_shows_the_keys_and_the_tmux_pattern() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(shown.contains("j/k"), "{shown}");
    assert!(
        shown.contains("end the session, or remove it once it has exited"),
        "{shown}"
    );
    assert!(shown.contains("vim-tmux-navigator"), "{shown}");
}

#[test]
fn question_mark_says_that_ctrl_backspace_focuses_the_sidebar() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(shown.contains("C-Backspace"), "{shown}");
}

#[test]
fn question_mark_names_the_setting_for_another_dir_picker() {
    let mut screen = Screen::new(reading_agent());

    screen.press('?');

    let shown = screen.draw();
    assert!(shown.contains("HARBOUR_DIR_PICKER"), "{shown}");
}

#[test]
fn the_next_key_hides_the_help_and_does_nothing_else() {
    let mut screen = Screen::wide(reading_agent());
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
