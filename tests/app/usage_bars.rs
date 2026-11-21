use crossterm::event::{KeyCode, KeyModifiers};

use crate::usage_states::app;
use crate::{Screen, script};

const ROW: &str = "  Session   ███░░░░░░░░░░░░░░░░░░░░░   12%   resets 7:20pm";

#[test]
fn a_finished_report_that_parses_shows_a_bar_per_limit() {
    let mut screen = Screen::of(app().with_usage(report("")));

    screen.press('u');

    let shown = screen.wait_for(ROW);
    assert!(shown.contains("  Week      ██"), "{shown}");
    assert!(!shown.contains("Current session: 12% used"), "{shown}");
}

#[test]
fn a_report_still_running_shows_its_text_as_it_arrives() {
    let mut screen = Screen::of(app().with_usage(report("exec sleep 600")));

    screen.press('u');

    let shown = screen.wait_for("Current week (all models): 90% used");
    assert!(shown.contains("Current session: 12% used"), "{shown}");
    assert!(!shown.contains(ROW), "{shown}");
}

#[test]
fn the_bars_end_with_the_hint() {
    let mut screen = Screen::of(app().with_usage(report("")));

    screen.press('u');

    let shown = screen.wait_for(ROW);
    assert!(shown.contains("t raw text · Esc close"), "{shown}");
}

#[test]
fn t_shows_the_report_as_printed_and_names_the_way_back() {
    let mut screen = Screen::of(app().with_usage(report("")));
    screen.press('u');
    screen.wait_for(ROW);

    screen.press('t');

    let shown = screen.draw();
    assert!(shown.contains("Current session: 12% used"), "{shown}");
    assert!(shown.contains(" t bars "), "{shown}");
    assert!(!shown.contains(ROW), "{shown}");
}

#[test]
fn t_again_brings_the_bars_back() {
    let mut screen = Screen::of(app().with_usage(report("")));
    screen.press('u');
    screen.wait_for(ROW);
    screen.press('t');

    screen.press('T');

    let shown = screen.draw();
    assert!(shown.contains(ROW), "{shown}");
    assert!(!shown.contains(" t bars "), "{shown}");
}

#[test]
fn ctrl_t_leaves_the_bars_shown() {
    let mut screen = Screen::of(app().with_usage(report("")));
    screen.press('u');
    screen.wait_for(ROW);

    screen.key(KeyCode::Char('t'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(shown.contains(ROW), "{shown}");
}

#[test]
fn a_popup_opened_again_starts_on_the_bars() {
    let mut screen = Screen::of(app().with_usage(report("")));
    screen.press('u');
    screen.wait_for(ROW);
    screen.press('t');
    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    screen.press('u');

    let shown = screen.wait_for(ROW);
    assert!(!shown.contains(" t bars "), "{shown}");
}

#[test]
fn t_while_the_report_loads_goes_to_its_process() {
    let reading = script("read line; echo \"acme read $line\"; exec sleep 600");
    let mut screen = Screen::of(app().with_usage(reading));
    screen.press('u');

    screen.press('t');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("acme read t");
    assert!(!shown.contains(" t bars "), "{shown}");
}

#[test]
fn t_on_a_report_that_does_not_parse_changes_nothing() {
    let mut screen = Screen::of(app().with_usage(script("echo 'acme usage: not logged in'")));
    screen.press('u');
    screen.wait_for("acme usage: not logged in");
    std::thread::sleep(std::time::Duration::from_millis(600));

    screen.press('t');

    let shown = screen.draw();
    assert!(shown.contains("acme usage: not logged in"), "{shown}");
    assert!(!shown.contains(" t bars "), "{shown}");
    assert!(!shown.contains("t raw text"), "{shown}");
}

#[test]
fn the_bars_are_drawn_again_for_each_width_while_open() {
    let mut screen = Screen::of(app().with_usage(report("")));
    screen.press('u');
    screen.wait_for(ROW);

    screen.resize(60, 20);
    let narrow = screen.draw();
    screen.resize(120, 20);
    let wide = screen.draw();

    assert!(
        narrow.contains("  Session   ░░░░   12%   resets 7:20pm"),
        "{narrow}"
    );
    assert!(wide.contains(ROW), "{wide}");
}

fn report(then: &str) -> Vec<String> {
    script(&format!(
        "printf 'You are on the acme team plan\\n\\n'; \
         printf 'Current session: 12%% used · resets 7:20pm (Acme/Harbour)\\n'; \
         printf 'Current week (all models): 90%% used · resets Sun 5am (Acme/Harbour)\\n'; \
         {then}"
    ))
}
