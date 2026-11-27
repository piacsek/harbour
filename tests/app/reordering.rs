use std::path::Path;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::keymap::{Action, Keymap};
use harbour::memory::Remembered;

use crate::directory::folder;
use crate::grouping::{cycling, remember};
use crate::refusing::Elsewhere;
use crate::remembering::{ID, OTHER_ID, Scratch, TITLE};
use crate::{
    Screen, directory, in_order, needs_a_terminal, numbered_agent, reading_agent, region, script,
};

const OTHER_TITLE: &str = "Add the acme export";
const THIRD_ID: &str = "5c4d3e2f-1a0b-4c9d-8e7f-6a5b4c3d2e1f";
const THIRD_TITLE: &str = "Rename the acme jobs";

#[test]
fn shift_k_moves_the_selected_card_up_and_it_stays_selected() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(App::new(numbered_agent(scratch.path())), scratch.path());

    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);

    let sidebar = sidebar(&screen.draw());
    assert!(in_order(&sidebar, &[&card(2), &card(1)]), "{sidebar}");
    assert!(sidebar.contains(&selected(2)), "{sidebar}");
}

#[test]
fn shift_j_moves_the_selected_card_down_and_it_stays_selected() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(App::new(numbered_agent(scratch.path())), scratch.path());
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
    let mut screen = two_cards(App::new(numbered_agent(scratch.path())), scratch.path());
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
    let mut screen = two_cards(App::new(numbered_agent(scratch.path())), scratch.path());

    screen.press('K');

    let sidebar = sidebar(&screen.draw());
    assert!(in_order(&sidebar, &[&card(2), &card(1)]), "{sidebar}");
    assert!(sidebar.contains(&selected(2)), "{sidebar}");
}

#[test]
fn shift_k_on_the_first_card_does_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(App::new(numbered_agent(scratch.path())), scratch.path());
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
    let mut screen = two_cards(App::new(numbered_agent(scratch.path())), scratch.path());

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
    needs_a_terminal();
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("acme agent got: K");
    let sidebar = sidebar(&shown);
    assert!(
        in_order(
            &sidebar,
            &[&format!("{} 1", directory()), &format!("{} 2", directory())]
        ),
        "{sidebar}"
    );
}

#[test]
fn a_binding_of_end_on_capital_k_wins_over_the_move() {
    let scratch = tempfile::tempdir().unwrap();
    let app = App::new(numbered_agent(scratch.path()))
        .with_keymap(Keymap::default().bind(Action::End, Some('K')));
    let mut screen = two_cards(app, scratch.path());

    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);

    let shown = screen.draw();
    assert!(shown.contains("End? y/n"), "{shown}");
    let sidebar = sidebar(&shown);
    assert!(in_order(&sidebar, &[&card(1), &card(2)]), "{sidebar}");
}

#[test]
fn the_prefix_then_1_jumps_to_the_card_moved_first() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = two_cards(App::new(numbered_agent(scratch.path())), scratch.path());
    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);
    screen.press('j');
    screen.wait_for("acme agent 1");

    screen.prefixed('1');

    let shown = screen.wait_for("acme agent 2");
    assert!(!shown.contains("acme agent 1"), "{shown}");
}

#[test]
fn two_stopped_cards_swapped_come_back_in_the_new_order() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    remember(&scratch, OTHER_ID, OTHER_TITLE, "acme-web", 1);
    let mut screen = stopped(&scratch);

    screen.key(KeyCode::Char('J'), KeyModifiers::SHIFT);

    let recalled = scratch.memory().recall();
    assert_eq!(recalled[0].session_id, OTHER_ID);
    assert_eq!(recalled[1].session_id, ID);
}

#[test]
fn a_moved_card_whose_file_was_forgotten_writes_no_file_back() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.remember(OTHER_ID, OTHER_TITLE, 1);
    let mut screen = stopped(&scratch);
    scratch.memory().forget(ID);

    screen.key(KeyCode::Char('J'), KeyModifiers::SHIFT);

    assert!(!scratch.memory().holds(ID));
    assert_eq!(scratch.memory().recall()[0].session_id, OTHER_ID);
}

#[test]
fn a_moved_card_open_elsewhere_saves_only_its_new_position() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    remember(&scratch, OTHER_ID, OTHER_TITLE, "acme-web", 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    let _other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(script("exec sleep 600"));
    screen.wait_for("open elsewhere");

    screen.key(KeyCode::Char('J'), KeyModifiers::SHIFT);

    assert_eq!(position(&scratch, ID), 1);
    assert_eq!(position(&scratch, OTHER_ID), 0);
    assert_eq!(recalled(&scratch, ID).title, TITLE);
    assert_eq!(recalled(&scratch, ID).directory, scratch.checkout());
}

#[test]
fn a_move_keeps_the_message_of_a_card_open_elsewhere() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.remember(OTHER_ID, OTHER_TITLE, 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    let _other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(script("exec sleep 600"));
    screen.wait_for("open elsewhere");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.wait_for("This conversation is open elsewhere");

    screen.key(KeyCode::Char('J'), KeyModifiers::SHIFT);

    let shown = screen.draw();
    assert!(
        shown.contains("This conversation is open elsewhere"),
        "{shown}"
    );
}

#[test]
fn a_move_saves_every_card_in_the_order_shown() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 5);
    scratch.remember(OTHER_ID, OTHER_TITLE, 6);
    scratch.remember(THIRD_ID, THIRD_TITLE, 7);
    let mut screen = stopped(&scratch);
    screen.wait_for(THIRD_TITLE);
    screen.press('j');
    screen.press('j');

    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);

    let recalled = scratch.memory().recall();
    assert_eq!(recalled[0].session_id, ID);
    assert_eq!(recalled[1].session_id, THIRD_ID);
    assert_eq!(recalled[2].session_id, OTHER_ID);
}

#[test]
fn a_move_keeps_a_title_another_harbour_saved_since() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    remember(&scratch, OTHER_ID, OTHER_TITLE, "acme-web", 1);
    let mut screen = stopped(&scratch);
    scratch.remember(ID, "Retitled in acme", 0);

    screen.key(KeyCode::Char('J'), KeyModifiers::SHIFT);

    let recalled = scratch.memory().recall();
    assert_eq!(recalled[1].session_id, ID);
    assert_eq!(recalled[1].title, "Retitled in acme");
}

fn two_cards(app: App, scratch: &Path) -> Screen {
    let one = folder(scratch, "acme-one");
    let two = folder(scratch, "acme-two");
    let mut screen = Screen::of(app.with_picker(cycling(scratch, &[&one, &two])));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('n');
    screen.wait_for("acme agent 2");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen
}

fn sidebar(shown: &str) -> String {
    region(shown, 1, 38)
}

fn card(n: usize) -> String {
    format!("acme-{} {n}", ["one", "two"][n - 1])
}

fn selected(n: usize) -> String {
    format!("▌ · {}", card(n))
}

fn stopped(scratch: &Scratch) -> Screen {
    let mut screen = Screen::of(App::new(script("exec sleep 600")).with_memory(scratch.memory()));
    screen.wait_for(TITLE);
    screen.wait_for(OTHER_TITLE);
    screen
}

fn position(scratch: &Scratch, id: &str) -> usize {
    recalled(scratch, id).position
}

fn recalled(scratch: &Scratch, id: &str) -> Remembered {
    scratch
        .memory()
        .recall()
        .into_iter()
        .find(|remembered| remembered.session_id == id)
        .unwrap()
}
