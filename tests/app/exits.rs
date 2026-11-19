use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Modifier;

use crate::{Screen, directory, script};

#[test]
fn a_session_that_exited_with_zero_shows_a_hollow_dot_and_exited() {
    let mut screen = Screen::new(script("echo 'acme agent done'; exit 0"));
    screen.press('n');

    let row = format!("○ {} 1 exited", directory());
    let shown = screen.wait_for(&row);

    assert!(shown.contains(&row), "{shown}");
}

#[test]
fn a_session_that_exited_with_a_code_shows_the_code() {
    let mut screen = Screen::new(script("exit 3"));
    screen.press('n');

    let row = format!("○ {} 1 exited 3", directory());
    let shown = screen.wait_for(&row);

    assert!(shown.contains(&row), "{shown}");
}

#[test]
fn a_session_killed_by_a_signal_shows_the_signal() {
    let mut screen = Screen::new(script("kill -9 $$"));
    screen.resize(180, 20);
    screen.press('n');

    let shown = screen.wait_for("Killed");

    assert!(shown.contains("exited (Killed"), "{shown}");
}

#[test]
fn an_exited_session_keeps_its_last_screen_dimmed() {
    let mut screen = Screen::new(script("echo 'acme agent done'; exit 0"));
    screen.press('n');
    screen.wait_for("exited");

    let shown = screen.draw();

    assert!(shown.contains("acme agent done"), "{shown}");
    let cell = &screen.terminal.backend().buffer()[(41, 1)];
    assert_eq!(cell.symbol(), "a");
    assert!(cell.modifier.contains(Modifier::DIM));
}

#[test]
fn a_command_that_cannot_start_adds_no_row_and_names_the_command() {
    let mut screen = Screen::new(vec!["acme-agent-missing".to_string()]);
    screen.press('n');

    let shown = screen.draw();

    assert!(
        shown.contains("Could not start acme-agent-missing"),
        "{shown}"
    );
    assert!(!shown.contains(&format!("{} 1", directory())), "{shown}");
}

#[test]
fn a_session_that_exits_while_its_pane_has_focus_still_lets_ctrl_h_back() {
    let mut screen = Screen::new(script("read -r line; exit 0"));
    screen.press('n');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("bye\r");
    screen.wait_for("exited");

    screen.type_text("typed after the exit\r");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('n');

    let row = format!("{} 2", directory());
    let shown = screen.wait_for(&row);
    assert!(shown.contains(&row), "{shown}");
    assert!(
        shown.contains(&format!("○ {} 1 exited", directory())),
        "{shown}"
    );
}
