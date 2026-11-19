use crate::{Screen, directory, reading_agent, script};

#[test]
fn x_on_a_running_session_asks_before_ending_it() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.resize(60, 20);
    screen.press('x');

    let shown = screen.draw();
    assert!(shown.contains(" End? y/n "), "{shown}");
    assert!(shown.contains("› ●"), "{shown}");
}

#[test]
fn y_to_the_question_ends_the_session() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.press('x');

    screen.press('y');

    let row = format!("○ {} 1 exited", directory());
    let shown = screen.wait_for(&row);
    assert!(shown.contains(&row), "{shown}");
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
    assert!(shown.contains(&format!("● {} 1", directory())), "{shown}");
    assert!(!shown.contains(&format!("{} 2", directory())), "{shown}");
}

#[test]
fn x_on_an_exited_session_removes_its_row() {
    let mut screen = Screen::new(script("exit 0"));
    screen.press('n');
    screen.press('n');
    screen.wait_for(&format!("○ {} 1 exited", directory()));
    screen.wait_for(&format!("○ {} 2 exited", directory()));

    screen.press('x');

    let shown = screen.draw();
    assert!(
        shown.contains(&format!("› ○ {} 1 exited", directory())),
        "{shown}"
    );
    assert!(!shown.contains(&format!("{} 2", directory())), "{shown}");
    assert!(!shown.contains("y/n"), "{shown}");
}

#[test]
fn q_with_a_running_session_asks_before_quitting() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.resize(60, 20);
    screen.press('q');

    let shown = screen.draw();
    assert!(shown.contains(" Quit? y/n "), "{shown}");
    assert!(screen.app.running());
}

#[test]
fn q_with_only_exited_sessions_quits_without_asking() {
    let mut screen = Screen::new(script("exit 0"));
    screen.press('n');
    screen.wait_for("exited");

    screen.press('q');

    assert!(!screen.app.running());
}

#[test]
fn y_to_the_quit_question_quits() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.press('q');

    screen.press('y');

    assert!(!screen.app.running());
}
