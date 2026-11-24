use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::keymap::{Action, Keymap};

use crate::{Screen, directory, in_order, numbered_agent, reading_agent, region};

#[test]
fn shift_k_moves_the_selected_card_up_and_it_stays_selected() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(Screen::new(numbered_agent(scratch.path())));

    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);

    let sidebar = sidebar(&screen.draw());
    assert!(in_order(&sidebar, &[&card(2), &card(1)]), "{sidebar}");
    assert!(sidebar.contains(&selected(2)), "{sidebar}");
}

#[test]
fn shift_j_moves_the_selected_card_down_and_it_stays_selected() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(Screen::new(numbered_agent(scratch.path())));
    screen.press('k');
    screen.wait_for("acme agent 1");

    screen.key(KeyCode::Char('J'), KeyModifiers::SHIFT);

    let sidebar = sidebar(&screen.draw());
    assert!(in_order(&sidebar, &[&card(2), &card(1)]), "{sidebar}");
    assert!(sidebar.contains(&selected(1)), "{sidebar}");
}

#[test]
fn j_with_the_shift_flag_moves_the_card_down() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(Screen::new(numbered_agent(scratch.path())));
    screen.press('k');
    screen.wait_for("acme agent 1");

    screen.key(KeyCode::Char('j'), KeyModifiers::SHIFT);

    let sidebar = sidebar(&screen.draw());
    assert!(in_order(&sidebar, &[&card(2), &card(1)]), "{sidebar}");
    assert!(sidebar.contains(&selected(1)), "{sidebar}");
}

#[test]
fn capital_k_without_the_shift_flag_moves_the_card_up() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(Screen::new(numbered_agent(scratch.path())));

    screen.press('K');

    let sidebar = sidebar(&screen.draw());
    assert!(in_order(&sidebar, &[&card(2), &card(1)]), "{sidebar}");
    assert!(sidebar.contains(&selected(2)), "{sidebar}");
}

#[test]
fn shift_k_on_the_first_card_does_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(Screen::new(numbered_agent(scratch.path())));
    screen.press('k');
    screen.wait_for("acme agent 1");

    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);

    let sidebar = sidebar(&screen.draw());
    assert!(in_order(&sidebar, &[&card(1), &card(2)]), "{sidebar}");
    assert!(sidebar.contains(&selected(1)), "{sidebar}");
}

#[test]
fn shift_j_on_the_last_card_does_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(Screen::new(numbered_agent(scratch.path())));

    screen.key(KeyCode::Char('J'), KeyModifiers::SHIFT);

    let sidebar = sidebar(&screen.draw());
    assert!(in_order(&sidebar, &[&card(1), &card(2)]), "{sidebar}");
    assert!(sidebar.contains(&selected(2)), "{sidebar}");
}

#[test]
fn shift_j_and_k_with_no_card_do_nothing() {
    let mut screen = Screen::new(reading_agent());

    screen.key(KeyCode::Char('J'), KeyModifiers::SHIFT);
    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);

    let shown = screen.draw();
    assert!(shown.contains("No session yet"), "{shown}");
}

#[test]
fn shift_k_with_the_pane_focused_reaches_the_program_and_moves_nothing() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("acme agent got: K");
    let sidebar = sidebar(&shown);
    assert!(in_order(&sidebar, &[&card(1), &card(2)]), "{sidebar}");
}

#[test]
fn a_binding_of_end_on_capital_k_wins_over_the_move() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(numbered_agent(scratch.path()))
        .with_keymap(Keymap::default().bind(Action::End, Some('K')));
    let mut screen = two_cards(Screen::of(app));

    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);

    let shown = screen.draw();
    assert!(shown.contains("End? y/n"), "{shown}");
    let sidebar = sidebar(&shown);
    assert!(in_order(&sidebar, &[&card(1), &card(2)]), "{sidebar}");
}

#[test]
fn the_prefix_then_1_jumps_to_the_card_moved_first() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(Screen::new(numbered_agent(scratch.path())));
    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);
    screen.press('j');
    screen.wait_for("acme agent 1");

    screen.prefixed('1');

    let shown = screen.wait_for("acme agent 2");
    assert!(!shown.contains("acme agent 1"), "{shown}");
}

fn two_cards(mut screen: Screen) -> Screen {
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");
    screen
}

fn sidebar(shown: &str) -> String {
    region(shown, 1, 38)
}

fn card(n: usize) -> String {
    format!("{} {n}", directory())
}

fn selected(n: usize) -> String {
    format!("▌ · {}", card(n))
}
