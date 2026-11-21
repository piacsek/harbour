use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::changelog::{CHANGELOG, shown};

use crate::{Screen, directory, reading_agent};

#[test]
fn c_shows_the_changelog_in_place_of_the_pane() {
    let mut screen = reader(RELEASES);

    screen.press('c');

    let shown = screen.draw();
    assert!(shown.contains(" changelog "), "{shown}");
    assert!(shown.contains("v0.2.0 (2026-11-02)"), "{shown}");
    assert!(shown.contains("- acme deploys retry twice"), "{shown}");
    assert!(shown.contains("v0.1.0 (2026-11-01)"), "{shown}");
}

#[test]
fn shift_c_shows_the_changelog_too() {
    let mut screen = reader(RELEASES);

    screen.key(KeyCode::Char('C'), KeyModifiers::SHIFT);

    let shown = screen.draw();
    assert!(shown.contains("v0.2.0 (2026-11-02)"), "{shown}");
}

#[test]
fn c_with_ctrl_shows_nothing() {
    let mut screen = reader(RELEASES);

    screen.key(KeyCode::Char('c'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(!shown.contains("v0.2.0 (2026-11-02)"), "{shown}");
}

#[test]
fn c_with_alt_shows_nothing() {
    let mut screen = reader(RELEASES);

    screen.key(KeyCode::Char('c'), KeyModifiers::ALT);

    let shown = screen.draw();
    assert!(!shown.contains("v0.2.0 (2026-11-02)"), "{shown}");
}

#[test]
fn the_view_shows_the_bundled_changelog_by_default() {
    let mut screen = Screen::new(reading_agent());
    let first = shown(CHANGELOG, 118).remove(0).to_string();

    screen.press('c');

    let shown = screen.draw();
    assert!(shown.contains(&format!("│{first}")), "{shown}");
}

#[test]
fn the_view_of_a_build_without_releases_says_so() {
    let mut screen = reader("# Changelog\n");

    screen.press('c');

    let shown = screen.draw();
    assert!(
        shown.contains("This build has no release notes."),
        "{shown}"
    );
}

#[test]
fn the_next_key_closes_the_view_and_does_nothing_else() {
    let mut screen = reader(RELEASES);
    screen.press('c');

    screen.press('n');

    let shown = screen.draw();
    assert!(!shown.contains("v0.2.0 (2026-11-02)"), "{shown}");
    assert!(!shown.contains(&format!("{} 1", directory())), "{shown}");
}

#[test]
fn q_closes_the_view_and_does_not_quit() {
    let mut screen = reader(RELEASES);
    screen.press('c');

    screen.press('q');

    let shown = screen.draw();
    assert!(!shown.contains("v0.2.0 (2026-11-02)"), "{shown}");
    assert!(screen.app.running());
}

const RELEASES: &str = "# Changelog\n\n## v0.2.0 (2026-11-02)\n- acme deploys retry twice\n\n## v0.1.0 (2026-11-01)\n- acme starts\n";

fn reader(changelog: &str) -> Screen {
    Screen::of(App::new(reading_agent()).with_changelog(changelog))
}
