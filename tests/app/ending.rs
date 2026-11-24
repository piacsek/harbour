use std::path::Path;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;

use crate::{
    Screen, below, deadline, directory, hangups, raw_agent, reading_agent, recording_hangups,
    script, staying_after_hangups,
};

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
fn capital_x_on_a_running_session_asks_before_ending_it() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.key(KeyCode::Char('X'), KeyModifiers::SHIFT);

    let shown = screen.draw();
    assert!(shown.contains("End? y/n"), "{shown}");
    assert!(shown.contains(&format!("· {} 1", directory())), "{shown}");
}

#[test]
fn y_to_capital_x_ends_the_session_and_removes_its_card() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen =
        Screen::of(App::new(hanging_up_agent(scratch.path())).with_hangup_grace(deadline()));
    for started in 1..=2 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {started}"));
    }
    screen.key(KeyCode::Char('X'), KeyModifiers::SHIFT);

    screen.press('y');

    let shown = screen.draw();
    assert!(shown.contains(&format!("▌ · {} 1", directory())), "{shown}");
    assert!(!shown.contains(&format!("{} 2", directory())), "{shown}");
    assert!(!shown.contains("y/n"), "{shown}");
    let hangups = hangups(scratch.path());
    assert_eq!(hangups.len(), 1, "{hangups:?}");
    assert_eq!(hangups[0], "2 saw 2 alive", "{hangups:?}");
}

#[test]
fn another_answer_to_capital_x_keeps_the_session_and_its_card() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Char('X'), KeyModifiers::SHIFT);

    screen.press('n');

    let shown = screen.draw();
    assert!(!shown.contains("y/n"), "{shown}");
    assert!(shown.contains(&format!("· {} 1", directory())), "{shown}");
    assert!(!shown.contains("exited"), "{shown}");
}

#[test]
fn capital_x_on_an_exited_session_removes_its_card_without_asking() {
    let mut screen = Screen::new(script("exit 0"));
    screen.press('n');
    screen.press('n');
    screen.wait_for(&format!("○ {} 1", directory()));
    screen.wait_for(&format!("○ {} 2", directory()));

    screen.key(KeyCode::Char('X'), KeyModifiers::SHIFT);

    let shown = screen.draw();
    assert!(shown.contains(&format!("▌ ○ {} 1", directory())), "{shown}");
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
fn quitting_hangs_up_on_every_running_session_before_it_kills_any() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen =
        Screen::of(App::new(hanging_up_agent(scratch.path())).with_hangup_grace(deadline()));
    for started in 1..=5 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {started}"));
    }
    screen.prefixed('q');
    screen.press('y');

    drop(screen);

    let hangups = hangups(scratch.path());
    assert_eq!(hangups.len(), 5, "{hangups:?}");
    assert_eq!(hangups[0], "1 saw 5 alive", "{hangups:?}");
    assert_eq!(hangups[1], "2 saw 5 alive", "{hangups:?}");
    assert_eq!(hangups[2], "3 saw 5 alive", "{hangups:?}");
    assert_eq!(hangups[3], "4 saw 5 alive", "{hangups:?}");
    assert_eq!(hangups[4], "5 saw 5 alive", "{hangups:?}");
}

#[test]
fn quitting_waits_one_grace_in_all_for_sessions_that_ignore_the_hangup() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen =
        Screen::of(App::new(ignoring_agent(scratch.path())).with_hangup_grace(deadline()));
    for started in 1..=3 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {started}"));
    }
    screen.prefixed('q');
    screen.press('y');

    let before = Instant::now();
    drop(screen);
    let elapsed = before.elapsed();

    assert!(elapsed >= deadline(), "{elapsed:?}");
    assert!(elapsed < 2 * deadline(), "{elapsed:?}");
}

fn hanging_up_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         {recording}echo \"acme agent $n\"; {staying}",
        count = count.display(),
        recording = recording_hangups(scratch, "$n"),
        staying = staying_after_hangups(),
    ))
}

fn ignoring_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         trap '' HUP; echo \"acme agent $n\"; {staying}",
        count = count.display(),
        staying = staying_after_hangups(),
    ))
}
