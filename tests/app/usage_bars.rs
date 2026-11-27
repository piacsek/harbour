use crossterm::event::{KeyCode, KeyModifiers};

use crate::logging::{count, logged, read};
use crate::usage_states::app;
use crate::{Screen, script};

const ROW: &str = "  Session   ███░░░░░░░░░░░░░░░░░░░░░   12%   resets 7:20pm";

#[test]
fn a_finished_report_that_parses_shows_a_bar_per_limit() {
    let mut screen = Screen::of(app().with_usage(report("")));

    screen.prefixed('u');

    let shown = screen.wait_for(ROW);
    assert!(shown.contains("  Week      ██"), "{shown}");
    assert!(!shown.contains("Current session: 12% used"), "{shown}");
}

#[test]
fn a_report_still_running_shows_none_of_its_text() {
    let mut screen = Screen::of(app().with_usage(report("exec sleep 600")));
    screen.prefixed('u');
    screen.press('t');
    screen.wait_for("Current week (all models): 90% used");

    screen.press('t');

    let shown = screen.draw();
    assert!(shown.contains("reading usage…"), "{shown}");
    assert!(!shown.contains("Current session: 12% used"), "{shown}");
    assert!(!shown.contains(ROW), "{shown}");
}

#[test]
fn the_text_never_shows_between_the_animation_and_the_bars() {
    let mut screen = Screen::of(app().with_usage(report("sleep 0.4")));
    let mut animated = 0;
    let mut text = 0;

    screen.prefixed('u');
    screen.wait_until("the bars", |shown| {
        animated += usize::from(shown.contains("reading usage…"));
        text += usize::from(shown.contains("Current session: 12% used"));
        shown.contains(ROW)
    });

    assert!(animated > 0);
    assert_eq!(text, 0);
}

#[test]
fn the_bars_end_with_the_hint() {
    let mut screen = Screen::of(app().with_usage(report("")));

    screen.prefixed('u');

    let shown = screen.wait_for(ROW);
    assert!(shown.contains("t raw text · Esc close"), "{shown}");
}

#[test]
fn t_shows_the_report_as_printed_and_names_the_way_back() {
    let mut screen = Screen::of(app().with_usage(report("")));
    screen.prefixed('u');
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
    screen.prefixed('u');
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
    screen.prefixed('u');
    screen.wait_for(ROW);

    screen.key(KeyCode::Char('t'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(shown.contains(ROW), "{shown}");
}

#[test]
fn a_popup_opened_again_starts_on_the_bars() {
    let mut screen = Screen::of(app().with_usage(report("")));
    screen.prefixed('u');
    screen.wait_for(ROW);
    screen.press('t');
    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    screen.prefixed('u');

    let shown = screen.wait_for(ROW);
    assert!(!shown.contains(" t bars "), "{shown}");
}

#[test]
fn t_while_the_report_loads_shows_its_text_as_it_arrives() {
    let mut screen = Screen::of(app().with_usage(report("exec sleep 600")));
    screen.prefixed('u');

    screen.press('t');

    let shown = screen.wait_for("Current week (all models): 90% used");
    assert!(shown.contains("Current session: 12% used"), "{shown}");
    assert!(!shown.contains("reading usage…"), "{shown}");
    assert!(!shown.contains(" t bars "), "{shown}");
}

#[test]
fn shift_t_while_the_report_loads_shows_its_text_too() {
    let mut screen = Screen::of(app().with_usage(report("exec sleep 600")));
    screen.prefixed('u');

    screen.key(KeyCode::Char('T'), KeyModifiers::SHIFT);

    let shown = screen.wait_for("Current week (all models): 90% used");
    assert!(!shown.contains("reading usage…"), "{shown}");
}

#[test]
fn ctrl_t_while_the_report_loads_goes_to_its_process() {
    let reading = script("read line; echo \"acme read $line\" | tr -d '\\024'; exec sleep 600");
    let mut screen = Screen::of(app().with_usage(reading));
    screen.prefixed('u');

    screen.key(KeyCode::Char('t'), KeyModifiers::CONTROL);
    screen.press('x');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.press('t');

    let shown = screen.wait_for("acme read x");
    assert!(!shown.contains("reading usage…"), "{shown}");
}

#[test]
fn a_report_that_finishes_after_t_was_pressed_while_loading_keeps_its_text() {
    let mut screen = Screen::of(app().with_usage(report("sleep 0.4")));
    screen.prefixed('u');
    screen.press('t');

    let shown = screen.wait_for(" t bars ");

    assert!(shown.contains("Current session: 12% used"), "{shown}");
    assert!(!shown.contains(ROW), "{shown}");
}

#[test]
fn a_popup_opened_again_after_t_while_loading_starts_on_the_animation() {
    let mut screen = Screen::of(app().with_usage(report("exec sleep 600")));
    screen.prefixed('u');
    screen.press('t');
    screen.wait_for("Current session: 12% used");
    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    screen.prefixed('u');

    let shown = screen.draw();
    assert!(shown.contains("reading usage…"), "{shown}");
    assert!(!shown.contains("Current session: 12% used"), "{shown}");
}

#[test]
fn t_while_loading_is_logged_as_the_popups_own_event_and_not_as_a_key() {
    let (app, lines) = logged(app().with_usage(report("exec sleep 600")));
    let mut screen = Screen::of(app);
    screen.prefixed('u');

    screen.press('t');
    screen.press('t');
    screen.draw();

    let lines = read(&lines);
    assert_eq!(count(&lines, " usage view=raw\n"), 1);
    assert_eq!(count(&lines, " usage view=bars\n"), 1);
    assert_eq!(count(&lines, " input to=usage"), 0);
}

#[test]
fn ctrl_z_in_the_usage_popup_logs_no_input() {
    let (app, lines) = logged(app().with_usage(report("exec sleep 600")));
    let mut screen = Screen::of(app);
    screen.prefixed('u');

    screen.key(KeyCode::Char('z'), KeyModifiers::CONTROL);
    screen.draw();

    let lines = read(&lines);
    assert_eq!(count(&lines, " input to=usage"), 0);
}

#[test]
fn t_on_a_report_that_does_not_parse_changes_nothing() {
    let failing = script("echo 'acme usage: not logged in'; exit 3");
    let mut screen = Screen::of(app().with_usage(failing));
    screen.prefixed('u');
    screen.wait_for(" exit 3 ");

    screen.press('t');

    let shown = screen.draw();
    assert!(shown.contains("acme usage: not logged in"), "{shown}");
    assert!(!shown.contains(" t bars "), "{shown}");
    assert!(!shown.contains("t raw text"), "{shown}");
}

#[test]
fn t_after_a_report_that_does_not_parse_finished_goes_to_its_process() {
    let failing = script("echo 'acme usage: not logged in'; exit 3");
    let (app, lines) = logged(app().with_usage(failing));
    let mut screen = Screen::of(app);
    screen.prefixed('u');
    screen.wait_for(" exit 3 ");

    screen.press('t');

    let lines = read(&lines);
    assert_eq!(count(&lines, " usage view=raw\n"), 0);
    assert_eq!(count(&lines, " input to=usage"), 1);
}

#[test]
fn the_bars_are_drawn_again_for_each_width_while_open() {
    let mut screen = Screen::of(app().with_usage(report("")));
    screen.prefixed('u');
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

#[test]
fn switching_views_is_logged_as_the_popups_own_event_and_not_as_a_key() {
    let (app, lines) = logged(app().with_usage(report("")));
    let mut screen = Screen::of(app);
    screen.prefixed('u');
    screen.wait_for(ROW);

    screen.press('t');
    screen.press('t');
    screen.draw();

    let lines = read(&lines);
    assert_eq!(count(&lines, " usage view=raw\n"), 1);
    assert_eq!(count(&lines, " usage view=bars\n"), 1);
    assert_eq!(count(&lines, " prefix key=usage\n"), 1);
    assert_eq!(count(&lines, " key name="), 0);
    assert_eq!(count(&lines, "acme"), 0);
}

#[cfg(target_os = "linux")]
#[test]
fn the_bars_stay_once_shown_though_something_the_report_started_prints_later() {
    let late = "trap '' HUP; (sleep 1.5; printf '\\033[2J\\033[Hacme later\\n') &";
    let mut screen = Screen::of(app().with_usage(report(late)));
    screen.prefixed('u');
    screen.wait_for(ROW);
    screen.press('t');
    screen.wait_for("acme later");

    screen.press('t');

    let shown = screen.draw();
    assert!(shown.contains(ROW), "{shown}");
}

fn report(then: &str) -> Vec<String> {
    script(&format!(
        "printf 'You are on the acme team plan\\n\\n'; \
         printf 'Current session: 12%% used · resets 7:20pm (Acme/Harbour)\\n'; \
         printf 'Current week (all models): 90%% used · resets Sun 5am (Acme/Harbour)\\n'; \
         {then}"
    ))
}
