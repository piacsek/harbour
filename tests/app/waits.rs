use std::time::Duration;

use crate::real_neovim::{held, reached};
use crate::{Screen, failure, reading_agent};

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
