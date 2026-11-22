use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};

use crate::{Screen, below, directory, numbered_agent, raw_agent, reading_agent, script};

#[test]
fn x_on_a_running_session_asks_before_ending_it() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.resize(60, 20);
    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains("End? y/n"), "{shown}");
    assert!(shown.contains("▌ ·"), "{shown}");
}

#[test]
fn y_to_the_question_ends_the_session() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.press('x');

    screen.press('y');

    let shown = screen.wait_for("exited");
    let doing = below(&shown, &format!("○ {} 1", directory()));
    assert!(doing.starts_with("exited"), "{shown}");
    assert!(!shown.contains("y/n"), "{shown}");
}

#[test]
fn another_answer_to_the_question_keeps_the_session_running() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.press('x');

    screen.press('n');

    let shown = screen.draw();
    assert!(!shown.contains("y/n"), "{shown}");
    assert!(shown.contains(&format!("· {} 1", directory())), "{shown}");
    assert!(!shown.contains("exited"), "{shown}");
    assert!(!shown.contains(&format!("{} 2", directory())), "{shown}");
}

#[test]
fn x_on_an_exited_session_removes_its_row() {
    let mut screen = Screen::new(script("exit 0"));
    screen.press('n');
    screen.press('n');
    screen.wait_for(&format!("○ {} 1", directory()));
    screen.wait_for(&format!("○ {} 2", directory()));

    screen.press('x');

    let shown = screen.draw();
    let first = format!("▌ ○ {} 1", directory());
    assert!(shown.contains(&first), "{shown}");
    assert_eq!(below(&shown, &first), "exited");
    assert!(!shown.contains(&format!("{} 2", directory())), "{shown}");
    assert!(!shown.contains("y/n"), "{shown}");
}

#[test]
fn ctrl_y_to_the_question_keeps_the_session_running() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.press('x');

    screen.key(KeyCode::Char('y'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(!shown.contains("y/n"), "{shown}");
    assert!(shown.contains(&format!("· {} 1", directory())), "{shown}");
    assert!(!shown.contains("exited"), "{shown}");
}

#[test]
fn alt_y_to_the_quit_question_keeps_harbour_running() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.prefixed('q');

    screen.key(KeyCode::Char('y'), KeyModifiers::ALT);

    assert!(screen.app.running());
}

#[test]
fn shift_y_to_the_quit_question_quits() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.prefixed('q');

    screen.key(KeyCode::Char('Y'), KeyModifiers::SHIFT);

    assert!(!screen.app.running());
}

#[test]
fn q_with_a_running_session_asks_before_quitting() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.resize(60, 20);
    screen.prefixed('q');

    let shown = screen.draw();
    assert!(shown.contains("Quit? y/n"), "{shown}");
    assert!(screen.app.running());
}

#[test]
fn q_with_only_exited_sessions_quits_without_asking() {
    let mut screen = Screen::new(script("exit 0"));
    screen.press('n');
    screen.wait_for("exited");

    screen.prefixed('q');

    assert!(!screen.app.running());
}

#[test]
fn y_to_the_quit_question_quits() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.prefixed('q');

    screen.press('y');

    assert!(!screen.app.running());
}

#[test]
fn a_click_in_the_pane_cancels_the_question_and_does_not_focus_the_pane() {
    let mut screen = Screen::new(raw_agent("", 1));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.press('x');

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 45, 3);
    screen.press('z');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.press('w');

    let shown = screen.wait_for("acme agent read:w");
    assert!(shown.contains("acme agent read:w"), "{shown}");
    assert!(!shown.contains("y/n"), "{shown}");
}

#[test]
fn quitting_ends_every_running_session_within_one_grace_period() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    for started in 1..=5 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {started}"));
    }
    screen.prefixed('q');
    screen.press('y');
    let started = Instant::now();

    drop(screen);

    assert!(started.elapsed() < Duration::from_millis(600));
}
