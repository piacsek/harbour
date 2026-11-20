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
