use std::time::Duration;

use crate::real_neovim::{held, reached};
use crate::{Screen, failure, reading_agent, script};

#[test]
fn a_wait_for_text_that_never_shows_fails_naming_it_and_showing_the_screen() {
    let mut screen = Screen::new(reading_agent());

    let failure = failure(|| {
        screen.wait_for("acme never shown");
    });

    assert!(failure.contains(r#"for ["acme never shown"]"#), "{failure}");
    assert!(failure.contains("No session yet"), "{failure}");
}

#[test]
fn a_wait_for_a_screen_that_never_stops_changing_fails_showing_the_last_screen() {
    let mut screen = Screen::new(script(
        "i=0; while :; do i=$((i+1)); echo \"acme tick $i\"; sleep 0.05; done",
    ));
    screen.press('n');
    screen.wait_for("acme tick");

    let failure = failure(|| {
        screen.wait_for_stable();
    });

    assert!(
        failure.contains("for the screen to stop changing"),
        "{failure}"
    );
    assert!(failure.contains("acme tick"), "{failure}");
}

#[test]
fn a_wait_for_a_mode_never_reached_fails_naming_it_and_the_last_mode() {
    let scratch = tempfile::tempdir().unwrap();

    let failure = failure(|| {
        reached(
            &scratch.path().join("socket"),
            "mode \"acme\"",
            Duration::from_millis(200),
            |mode| mode.mode == "acme",
        );
    });

    assert!(failure.contains(r#"for mode "acme""#), "{failure}");
    assert!(failure.contains("last mode: none"), "{failure}");
}

#[test]
fn a_wait_for_an_expression_never_true_fails_naming_it_and_the_last_value() {
    let scratch = tempfile::tempdir().unwrap();

    let failure = failure(|| {
        held(
            &scratch.path().join("socket"),
            "acme == 1",
            Duration::from_millis(200),
        );
    });

    assert!(failure.contains("for acme == 1"), "{failure}");
    assert!(failure.contains("last value: None"), "{failure}");
}

#[test]
fn a_wait_for_focus_never_reached_fails_naming_the_pane_and_showing_the_screen() {
    let mut screen = Screen::new(reading_agent());

    let failure = failure(|| {
        screen.wait_for_focused("acme never focused");
    });

    assert!(
        failure.contains("for focus on acme never focused"),
        "{failure}"
    );
    assert!(failure.contains("No session yet"), "{failure}");
}

#[test]
fn a_wait_for_a_pane_without_focus_never_reached_fails_naming_it_and_showing_the_screen() {
    let mut screen = Screen::new(reading_agent());

    let failure = failure(|| {
        screen.wait_for_unfocused("acme never shown");
    });

    assert!(
        failure.contains("for acme never shown without focus"),
        "{failure}"
    );
    assert!(failure.contains("No session yet"), "{failure}");
}
