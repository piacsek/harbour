use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::{Color, Modifier};

use crate::{Screen, below, directory, focused, needs_a_terminal, pane_corner, region, script};

#[test]
fn a_session_that_exited_with_zero_shows_a_hollow_dot_and_exited() {
    let mut screen = Screen::new(script("echo 'acme agent done'; exit 0"));
    screen.press('n');

    let shown = screen.wait_for("exited");

    assert_eq!(below(&shown, &format!("○ {} 1", directory())), "exited");
}

#[test]
fn a_session_that_exited_with_a_code_shows_the_code() {
    let mut screen = Screen::new(script("exit 3"));
    screen.press('n');

    let shown = screen.wait_for("exited 3");

    assert_eq!(below(&shown, &format!("○ {} 1", directory())), "exited 3");
}

#[test]
fn a_session_killed_by_a_signal_shows_the_signal() {
    let mut screen = Screen::new(script("kill -9 $$"));
    screen.press('n');

    let shown = screen.wait_for("Killed");

    assert!(shown.contains("exited (Killed"), "{shown}");
}

#[test]
fn an_exited_session_keeps_its_last_screen_dimmed() {
    let mut screen = Screen::new(script("echo 'acme agent done'; exit 0"));
    screen.press('n');
    let shown = screen.wait_for_all(&["exited", "acme agent done"]);

    assert!(shown.contains("acme agent done"), "{shown}");
    let cell = &screen.terminal.backend().buffer()[(pane_corner(&shown) + 1, 1)];
    assert_eq!(cell.symbol(), "a");
    assert!(cell.modifier.contains(Modifier::DIM));
}

#[test]
fn an_exited_session_keeps_the_cyan_border_undimmed_while_its_pane_has_focus() {
    needs_a_terminal();
    let mut screen = Screen::new(script("read -r line; exit 0"));
    screen.press('n');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.type_text("bye\r");
    let shown = screen.wait_for("exited");

    let pane = pane_corner(&shown);
    assert!(focused(screen.cell(pane, 0).symbol()));
    assert_eq!(screen.cell(pane, 0).fg, Color::Cyan);
    assert!(!screen.cell(pane, 0).modifier.contains(Modifier::DIM));
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
fn a_command_that_cannot_start_is_named_while_other_sessions_run() {
    let scratch = tempfile::tempdir().unwrap();
    let agent = scratch.path().join("acme-agent");
    std::os::unix::fs::symlink("/bin/sh", &agent).unwrap();
    let mut screen = Screen::new(vec![agent.to_string_lossy().into_owned()]);
    screen.press('n');
    screen.wait_for(&format!("{} 1", directory()));
    std::fs::remove_file(&agent).unwrap();

    screen.press('n');

    let shown = screen.draw();
    assert!(shown.contains("Could not start"), "{shown}");
}

#[test]
fn a_command_that_cannot_start_is_named_on_a_narrow_terminal() {
    let mut screen = Screen::new(vec!["acme-agent-missing".to_string()]);
    screen.resize(50, 20);

    screen.press('n');

    let shown = screen.draw();
    assert!(shown.contains("Could not start"), "{shown}");
}

#[test]
fn a_start_that_works_clears_the_failure() {
    let scratch = tempfile::tempdir().unwrap();
    let agent = scratch.path().join("acme-agent");
    let mut screen = Screen::new(vec![agent.to_string_lossy().into_owned()]);
    screen.press('n');
    std::os::unix::fs::symlink("/bin/sh", &agent).unwrap();

    screen.press('n');

    let shown = screen.wait_for(&format!("{} 1", directory()));
    assert!(!shown.contains("Could not start"), "{shown}");
}

#[test]
fn a_long_command_that_cannot_start_is_named_in_full_in_the_pane() {
    let command = format!("/acme/{}acme-agent-tail", "deep/".repeat(20));
    let mut screen = Screen::new(vec![command]);

    screen.press('n');

    let pane = region(&screen.draw(), 41, 78).replace('\n', "");
    assert!(pane.contains("acme-agent-tail"), "{pane}");
}

#[test]
fn a_command_of_wide_characters_that_cannot_start_is_named_in_full_in_the_sidebar() {
    let command = format!("/acme/{}-tail", "東京大阪".repeat(15));
    let mut screen = Screen::new(vec![command.clone()]);

    screen.press('n');

    let sidebar: String = region(&screen.draw(), 1, 38)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert!(sidebar.contains(&command), "{sidebar}");
}

#[test]
fn a_command_that_cannot_start_names_the_cause() {
    needs_a_terminal();
    let scratch = tempfile::tempdir().unwrap();
    let agent = scratch.path().join("acme-agent");
    std::fs::write(&agent, "not a program").unwrap();
    let mut screen = Screen::new(vec![agent.to_string_lossy().into_owned()]);
    screen.resize(400, 20);

    screen.press('n');

    let shown = screen.draw();
    assert!(shown.contains("not executable"), "{shown}");
    assert!(!shown.contains("PATH"), "{shown}");
}

#[test]
fn a_session_that_exits_while_its_pane_has_focus_still_lets_ctrl_h_back() {
    needs_a_terminal();
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
    assert_eq!(below(&shown, &format!("○ {} 1", directory())), "exited");
}
