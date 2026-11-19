use crate::{Screen, directory, reading_agent};

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
fn the_next_key_hides_the_help_and_does_nothing_else() {
    let mut screen = Screen::new(reading_agent());
    screen.press('?');

    screen.press('n');

    let shown = screen.draw();
    assert!(!shown.contains("vim-tmux-navigator"), "{shown}");
    assert!(!shown.contains(&format!("{} 1", directory())), "{shown}");
}
