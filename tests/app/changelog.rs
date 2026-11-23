use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;
use harbour::changelog::{CHANGELOG, shown};
use harbour::handoff::{Reply, Verb};

use crate::handing::answering;
use crate::handoff::{app_with, file, harbour, recording_neovim, soon, start_in};
use crate::logging::{count, logged, only, read};
use crate::{Screen, directory, reading_agent};

#[test]
fn c_shows_the_changelog_over_the_whole_screen() {
    let mut screen = reader(RELEASES);

    screen.prefixed('c');

    let shown = screen.draw();
    assert!(shown.contains(" changelog "), "{shown}");
    assert!(shown.contains("v0.2.0 (2026-11-02)"), "{shown}");
    assert!(shown.contains("- acme deploys retry twice"), "{shown}");
    assert!(shown.contains("v0.1.0 (2026-11-01)"), "{shown}");
}

#[test]
fn shift_c_shows_the_changelog_too() {
    let mut screen = reader(RELEASES);

    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
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
    let expected: Vec<String> = shown(CHANGELOG, 118)
        .iter()
        .take(18)
        .map(|line| format!("│{:<118}│", line.to_string()))
        .collect();

    screen.prefixed('c');

    let drawn = screen.draw();
    let rows: Vec<&str> = drawn.lines().skip(1).take(expected.len()).collect();
    assert_eq!(rows, expected, "{drawn}");
}

#[test]
fn the_view_of_a_build_without_releases_says_so() {
    let mut screen = reader("# Changelog\n");

    screen.prefixed('c');

    let shown = screen.draw();
    assert!(
        shown.contains("This build has no release notes."),
        "{shown}"
    );
}

#[test]
fn the_next_key_closes_the_view_and_does_nothing_else() {
    let mut screen = reader(RELEASES);
    screen.prefixed('c');

    screen.press('n');

    let shown = screen.draw();
    assert!(!shown.contains("v0.2.0 (2026-11-02)"), "{shown}");
    assert!(!shown.contains(&format!("{} 1", directory())), "{shown}");
}

#[test]
fn q_closes_the_view_and_does_not_quit() {
    let mut screen = reader(RELEASES);
    screen.prefixed('c');

    screen.press('q');

    let shown = screen.draw();
    assert!(!shown.contains("v0.2.0 (2026-11-02)"), "{shown}");
    assert!(screen.app.running());
}

#[test]
fn the_view_opens_at_the_top() {
    let mut screen = reader(&long());

    screen.prefixed('c');

    let shown = screen.draw();
    assert!(shown.contains("│Changelog"), "{shown}");
    assert!(shown.contains("- acme step 15"), "{shown}");
    assert!(!shown.contains("- acme step 16"), "{shown}");
}

#[test]
fn j_scrolls_one_line_on() {
    let mut screen = reader(&long());
    screen.prefixed('c');

    screen.press('j');

    let shown = screen.draw();
    assert!(!shown.contains("│Changelog"), "{shown}");
    assert!(shown.contains("- acme step 16"), "{shown}");
}

#[test]
fn down_scrolls_one_line_on() {
    let mut screen = reader(&long());
    screen.prefixed('c');

    screen.key(KeyCode::Down, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(!shown.contains("│Changelog"), "{shown}");
    assert!(shown.contains("- acme step 16"), "{shown}");
}

#[test]
fn k_scrolls_one_line_back() {
    let mut screen = reader(&long());
    screen.prefixed('c');
    screen.press('j');

    screen.press('k');

    let shown = screen.draw();
    assert!(shown.contains("│Changelog"), "{shown}");
    assert!(!shown.contains("- acme step 16"), "{shown}");
}

#[test]
fn up_scrolls_one_line_back() {
    let mut screen = reader(&long());
    screen.prefixed('c');
    screen.press('j');

    screen.key(KeyCode::Up, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(shown.contains("│Changelog"), "{shown}");
    assert!(!shown.contains("- acme step 16"), "{shown}");
}

#[test]
fn shift_j_scrolls_one_line_on_under_caps_lock() {
    let mut screen = reader(&long());
    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('C'), KeyModifiers::SHIFT);

    screen.key(KeyCode::Char('J'), KeyModifiers::SHIFT);

    let shown = screen.draw();
    assert!(!shown.contains("│Changelog"), "{shown}");
    assert!(shown.contains("- acme step 16"), "{shown}");
}

#[test]
fn shift_k_scrolls_one_line_back_under_caps_lock() {
    let mut screen = reader(&long());
    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('C'), KeyModifiers::SHIFT);
    screen.press('j');

    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);

    let shown = screen.draw();
    assert!(shown.contains("│Changelog"), "{shown}");
    assert!(!shown.contains("- acme step 16"), "{shown}");
}

#[test]
fn ctrl_shift_d_scrolls_half_the_view_on_under_caps_lock() {
    let mut screen = reader(&long());
    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('C'), KeyModifiers::SHIFT);

    screen.key(
        KeyCode::Char('D'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );

    let shown = screen.draw();
    assert!(!shown.contains("- acme step 06"), "{shown}");
    assert!(shown.contains("- acme step 07"), "{shown}");
}

#[test]
fn ctrl_shift_u_scrolls_half_the_view_back_under_caps_lock() {
    let mut screen = reader(&long());
    screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
    screen.key(KeyCode::Char('C'), KeyModifiers::SHIFT);
    screen.key(KeyCode::Char('d'), KeyModifiers::CONTROL);

    screen.key(
        KeyCode::Char('U'),
        KeyModifiers::CONTROL | KeyModifiers::SHIFT,
    );

    let shown = screen.draw();
    assert!(shown.contains("│Changelog"), "{shown}");
    assert!(!shown.contains("- acme step 16"), "{shown}");
}

#[test]
fn k_at_the_top_keeps_the_view_open() {
    let mut screen = reader(&long());
    screen.prefixed('c');

    screen.press('k');

    let shown = screen.draw();
    assert!(shown.contains(" changelog "), "{shown}");
    assert!(shown.contains("│Changelog"), "{shown}");
}

#[test]
fn ctrl_d_scrolls_half_the_view_on() {
    let mut screen = reader(&long());
    screen.prefixed('c');

    screen.key(KeyCode::Char('d'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(!shown.contains("- acme step 06"), "{shown}");
    assert!(shown.contains("- acme step 07"), "{shown}");
    assert!(shown.contains("- acme step 24"), "{shown}");
    assert!(!shown.contains("- acme step 25"), "{shown}");
}

#[test]
fn ctrl_u_scrolls_half_the_view_back() {
    let mut screen = reader(&long());
    screen.prefixed('c');
    screen.key(KeyCode::Char('d'), KeyModifiers::CONTROL);

    screen.key(KeyCode::Char('u'), KeyModifiers::CONTROL);

    let shown = screen.draw();
    assert!(shown.contains("│Changelog"), "{shown}");
    assert!(!shown.contains("- acme step 16"), "{shown}");
}

#[test]
fn the_last_line_never_leaves_the_bottom() {
    let mut screen = reader(&long());
    screen.prefixed('c');

    for _ in 0..5 {
        screen.key(KeyCode::Char('d'), KeyModifiers::CONTROL);
    }

    let shown = screen.draw();
    assert!(!shown.contains("- acme step 12"), "{shown}");
    assert!(shown.contains("- acme step 13"), "{shown}");
    assert!(shown.contains("- acme step 30"), "{shown}");
}

#[test]
fn j_on_a_text_shorter_than_the_view_does_nothing() {
    let mut screen = reader(RELEASES);
    screen.prefixed('c');

    screen.press('j');

    let shown = screen.draw();
    assert!(shown.contains("│Changelog"), "{shown}");
    assert!(shown.contains("- acme starts"), "{shown}");
}

#[test]
fn the_view_opens_at_the_top_again_after_scrolling() {
    let mut screen = reader(&long());
    screen.prefixed('c');
    screen.key(KeyCode::Char('d'), KeyModifiers::CONTROL);
    screen.press('x');

    screen.prefixed('c');

    let shown = screen.draw();
    assert!(shown.contains("│Changelog"), "{shown}");
}

#[test]
fn a_taller_terminal_keeps_the_last_line_at_the_bottom() {
    let mut screen = reader(&long());
    screen.prefixed('c');
    for _ in 0..5 {
        screen.key(KeyCode::Char('d'), KeyModifiers::CONTROL);
    }
    screen.draw();

    screen.resize(120, 30);

    let shown = screen.draw();
    assert!(!shown.contains("- acme step 02"), "{shown}");
    assert!(shown.contains("- acme step 03"), "{shown}");
    assert!(shown.contains("- acme step 30"), "{shown}");
}

#[test]
fn the_wheel_scrolls_one_line_on() {
    let mut screen = reader(&long());
    screen.prefixed('c');

    screen.mouse(MouseEventKind::ScrollDown, 45, 3);

    let shown = screen.draw();
    assert!(!shown.contains("│Changelog"), "{shown}");
    assert!(shown.contains("- acme step 16"), "{shown}");
}

#[test]
fn the_wheel_scrolls_one_line_back() {
    let mut screen = reader(&long());
    screen.prefixed('c');
    screen.press('j');
    screen.press('j');

    screen.mouse(MouseEventKind::ScrollUp, 45, 3);

    let shown = screen.draw();
    assert!(!shown.contains("- acme step 17"), "{shown}");
    assert!(shown.contains("- acme step 16"), "{shown}");
    assert!(shown.contains(" changelog "), "{shown}");
}

#[test]
fn page_down_scrolls_a_whole_view_on() {
    let mut screen = reader(&longer());
    screen.prefixed('c');

    screen.key(KeyCode::PageDown, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(!shown.contains("- acme step 15"), "{shown}");
    assert!(shown.contains("- acme step 16"), "{shown}");
    assert!(shown.contains("- acme step 33"), "{shown}");
    assert!(!shown.contains("- acme step 34"), "{shown}");
}

#[test]
fn page_up_scrolls_a_whole_view_back() {
    let mut screen = reader(&longer());
    screen.prefixed('c');
    screen.key(KeyCode::PageDown, KeyModifiers::NONE);
    screen.press('j');

    screen.key(KeyCode::PageUp, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(!shown.contains("│Changelog"), "{shown}");
    assert!(shown.contains("- acme step 16"), "{shown}");
    assert!(!shown.contains("- acme step 17"), "{shown}");
}

#[test]
fn a_click_closes_the_view() {
    let mut screen = reader(RELEASES);
    screen.prefixed('c');

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 45, 3);

    let shown = screen.draw();
    assert!(!shown.contains(" changelog "), "{shown}");
}

#[test]
fn a_paste_closes_the_view() {
    let mut screen = reader(RELEASES);
    screen.prefixed('c');

    screen.app.paste("acme");

    let shown = screen.draw();
    assert!(!shown.contains(" changelog "), "{shown}");
}

#[test]
fn the_prefix_then_c_while_the_help_shows_closes_it_and_shows_the_changelog() {
    let mut screen = reader(RELEASES);
    screen.prefixed('?');

    screen.prefixed('c');

    let shown = screen.draw();
    assert!(!shown.contains("┌ help "), "{shown}");
    assert!(shown.contains(" changelog "), "{shown}");
}

#[test]
fn c_from_the_pane_goes_to_the_session() {
    let mut screen = reader(RELEASES);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.press('c');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("acme agent got: c");
    assert!(shown.contains("acme agent got: c"), "{shown}");
    assert!(!shown.contains(" changelog "), "{shown}");
}

#[test]
fn an_open_of_harbour_edit_closes_the_view_and_shows_the_file() {
    let scratch = tempfile::tempdir().unwrap();
    let _asked = answering(scratch.path());
    let mut screen = harbour(scratch.path());
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.prefixed('c');

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Opened);
    let shown = screen.wait_for("acme nvim ready");
    assert!(!shown.contains(" changelog "), "{shown}");
}

#[test]
fn opening_the_view_logs_it_in_place_of_the_key() {
    let (app, lines) = logged(App::new(reading_agent()).with_changelog(&long()));
    let mut screen = Screen::of(app);

    screen.prefixed('c');

    let lines = read(&lines);
    only(&lines, " changelog shown");
    assert_eq!(count(&lines, " key name="), 0);
}

#[test]
fn the_keys_inside_the_view_log_nothing() {
    let (app, lines) = logged(App::new(reading_agent()).with_changelog(&long()));
    let mut screen = Screen::of(app);
    screen.prefixed('c');

    screen.press('j');
    screen.key(KeyCode::Char('d'), KeyModifiers::CONTROL);
    screen.press('k');
    screen.key(KeyCode::PageDown, KeyModifiers::NONE);

    let lines = read(&lines);
    only(&lines, " changelog shown");
    assert_eq!(count(&lines, " key name="), 0);
    assert_eq!(count(&lines, " changelog closed"), 0);
}

#[test]
fn a_key_that_closes_the_view_logs_it_closed() {
    let (app, lines) = logged(App::new(reading_agent()).with_changelog(RELEASES));
    let mut screen = Screen::of(app);
    screen.prefixed('c');

    screen.press('x');

    let lines = read(&lines);
    only(&lines, " changelog closed");
    assert_eq!(count(&lines, " key name="), 0);
}

#[test]
fn a_paste_that_closes_the_view_logs_it_closed() {
    let (app, lines) = logged(App::new(reading_agent()).with_changelog(RELEASES));
    let mut screen = Screen::of(app);
    screen.prefixed('c');

    screen.app.paste("acme");

    let lines = read(&lines);
    only(&lines, " changelog closed");
}

#[test]
fn a_click_that_closes_the_view_logs_it_closed() {
    let (app, lines) = logged(App::new(reading_agent()).with_changelog(RELEASES));
    let mut screen = Screen::of(app);
    screen.prefixed('c');

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 45, 3);

    let lines = read(&lines);
    only(&lines, " changelog closed");
}

#[test]
fn an_open_of_harbour_edit_that_closes_the_view_logs_it_closed() {
    let scratch = tempfile::tempdir().unwrap();
    let _asked = answering(scratch.path());
    let (app, lines) = logged(app_with(scratch.path(), recording_neovim(scratch.path())));
    let mut screen = Screen::of(app);
    start_in(&mut screen, scratch.path(), "acme-api", 1);
    screen.prefixed('c');

    let reply = screen
        .app
        .hand(Verb::Open, &file(scratch.path(), "acme-api"), soon());

    assert_eq!(reply, Reply::Opened);
    let lines = read(&lines);
    only(&lines, " changelog closed");
}

const RELEASES: &str = "# Changelog\n\n## v0.2.0 (2026-11-02)\n- acme deploys retry twice\n\n## v0.1.0 (2026-11-01)\n- acme starts\n";

fn reader(changelog: &str) -> Screen {
    Screen::of(App::new(reading_agent()).with_changelog(changelog))
}

fn long() -> String {
    let steps: String = (1..=30).map(|n| format!("- acme step {n:02}\n")).collect();
    format!("# Changelog\n\n## v0.1.0 (2026-11-01)\n{steps}")
}

fn longer() -> String {
    let steps: String = (1..=50).map(|n| format!("- acme step {n:02}\n")).collect();
    format!("# Changelog\n\n## v0.1.0 (2026-11-01)\n{steps}")
}
