use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use harbour::app::App;
use harbour::input::ESCAPED;

use crate::usage_states::place;
use crate::zoom::{editing_session, raw_reader};
use crate::{Screen, home, numbered_agent, pane_corner, reading_agent, script, sized_agent};

#[test]
fn alt_minus_from_the_sidebar_makes_it_two_columns_narrower() {
    let mut screen = started(30);

    narrower(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 28);
}

#[test]
fn alt_equals_from_the_sidebar_makes_it_two_columns_wider() {
    let mut screen = started(30);

    wider(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 32);
}

#[test]
fn alt_shift_minus_narrows_it_on_a_layout_where_minus_needs_shift() {
    let mut screen = started(30);

    screen.key(KeyCode::Char('-'), KeyModifiers::ALT | KeyModifiers::SHIFT);

    assert_eq!(pane_corner(&screen.draw()), 28);
}

#[test]
fn alt_shift_equals_widens_it_on_a_layout_where_equals_needs_shift() {
    let mut screen = started(30);

    screen.key(KeyCode::Char('='), KeyModifiers::ALT | KeyModifiers::SHIFT);

    assert_eq!(pane_corner(&screen.draw()), 32);
}

#[test]
fn alt_equals_from_the_claude_code_pane_widens_the_sidebar_and_keeps_the_pane_focused() {
    let mut screen = Screen::at(
        App::new(reading_agent()).with_sidebar_width(30),
        home(),
        120,
    );
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    wider(&mut screen);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent got: hi");
    assert_eq!(pane_corner(&shown), 32);
}

#[test]
fn held_keys_move_the_border_two_columns_a_press() {
    let mut screen = started(30);

    wider(&mut screen);
    wider(&mut screen);
    wider(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 36);
}

#[test]
fn the_session_never_receives_the_width_keys() {
    let mut screen = Screen::new(raw_reader(1));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    narrower(&mut screen);
    wider(&mut screen);
    screen.press('a');

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 61"), "{shown}");
}

#[test]
fn every_session_is_told_its_new_width() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::at(
        App::new(sized_agent(scratch.path())).with_sidebar_width(30),
        home(),
        120,
    );
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");

    wider(&mut screen);
    screen.wait_for("size:18 86");
    screen.press('k');

    screen.wait_for_all(&["acme agent 1", "size:18 86"]);
}

#[test]
fn with_no_session_the_border_moves_too() {
    let mut screen = Screen::at(
        App::new(reading_agent()).with_sidebar_width(30),
        home(),
        120,
    );

    wider(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 32);
}

#[test]
fn alt_minus_stops_at_fifteen_columns() {
    let mut screen = started(16);

    narrower(&mut screen);
    narrower(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 15);
}

#[test]
fn alt_minus_leaves_a_configured_width_under_fifteen_as_it_is() {
    let mut screen = started(10);

    narrower(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 10);
}

#[test]
fn alt_equals_widens_a_configured_width_under_fifteen() {
    let mut screen = started(10);

    wider(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 12);
}

#[test]
fn alt_equals_stops_at_half_the_terminal() {
    let mut screen = started(59);

    wider(&mut screen);
    wider(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 60);
}

#[test]
fn alt_equals_at_half_the_terminal_keeps_the_width_for_a_wider_terminal() {
    let mut screen = started(60);

    wider(&mut screen);
    screen.resize(200, 20);

    assert_eq!(pane_corner(&screen.draw()), 60);
}

#[test]
fn under_sixty_columns_the_width_keys_do_nothing() {
    let mut screen = started(30);
    screen.resize(59, 20);

    wider(&mut screen);
    screen.resize(120, 20);

    assert_eq!(pane_corner(&screen.draw()), 30);
}

#[test]
fn while_zoomed_the_width_keys_do_nothing_and_the_width_is_back_after() {
    let mut screen = started(30);
    zoom(&mut screen);

    wider(&mut screen);
    zoom(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 30);
}

#[test]
fn with_the_help_shown_the_width_keys_do_nothing() {
    let mut screen = started(30);
    screen.press('?');

    wider(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 30);
}

#[test]
fn with_a_question_shown_the_width_keys_do_nothing() {
    let mut screen = started(30);
    screen.press('q');

    wider(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 30);
}

#[test]
fn with_the_usage_popup_shown_the_width_keys_do_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::at(
        App::new(numbered_agent(scratch.path()))
            .with_sidebar_width(30)
            .with_usage_place(Some(place()))
            .with_usage(script("echo 'acme session: 12% spent'")),
        home(),
        120,
    );
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('u');
    screen.wait_for("acme session: 12% spent");

    wider(&mut screen);
    screen.key(KeyCode::Esc, KeyModifiers::NONE);

    assert_eq!(pane_corner(&screen.draw()), 30);
}

#[test]
fn alt_minus_while_the_editor_has_focus_reaches_the_editor() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = editing_session(scratch.path());

    narrower(&mut screen);

    let shown = screen.wait_for("acme editor read:");
    assert!(shown.contains("acme editor read: 1b"), "{shown}");
    assert_eq!(pane_corner(&shown), 40);
}

#[test]
fn alt_minus_read_after_an_escape_reaches_the_session() {
    let mut screen = Screen::new(raw_reader(1));
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.app.key(KeyEvent::new_with_kind_and_state(
        KeyCode::Char('-'),
        KeyModifiers::ALT,
        KeyEventKind::Press,
        ESCAPED,
    ));

    let shown = screen.wait_for("acme agent read:");
    assert!(shown.contains("acme agent read: 1b"), "{shown}");
    assert_eq!(pane_corner(&shown), 40);
}

#[test]
fn a_narrower_terminal_cuts_the_width_and_keeps_it_for_later() {
    let mut screen = started(50);
    wider(&mut screen);

    screen.resize(80, 20);
    let cut = pane_corner(&screen.draw());
    screen.resize(120, 20);

    assert_eq!(cut, 40);
    assert_eq!(pane_corner(&screen.draw()), 52);
}

#[test]
fn a_key_after_a_cut_starts_from_the_cut_width() {
    let mut screen = started(60);
    screen.resize(100, 20);

    narrower(&mut screen);

    assert_eq!(pane_corner(&screen.draw()), 48);
}

#[test]
fn a_cjk_name_stays_whole_in_the_narrowest_sidebar() {
    let scratch = tempfile::tempdir().unwrap();
    let place = scratch.path().join("東京の倉庫アプリ");
    std::fs::create_dir_all(&place).unwrap();
    let mut screen = Screen::at(
        App::new(numbered_agent(scratch.path())).with_sidebar_width(17),
        place,
        120,
    );
    screen.press('n');
    screen.wait_for("acme agent 1");

    narrower(&mut screen);
    narrower(&mut screen);

    screen.draw();
    assert_eq!(screen.cell(11, 1).symbol(), "倉");
    assert_eq!(screen.cell(13, 1).symbol(), "…");
    assert_eq!(screen.cell(14, 1).symbol(), "┃");
    assert_eq!(screen.cell(15, 0).symbol(), "┌");
}

#[test]
fn an_exited_session_keeps_its_last_line_whole_after_the_width_keys() {
    let mut screen = exited_at(88);

    wider(&mut screen);
    wider(&mut screen);
    wider(&mut screen);
    narrower(&mut screen);
    narrower(&mut screen);
    narrower(&mut screen);

    let shown = screen.draw();
    assert!(shown.contains(&line(88)), "{shown}");
}

#[test]
fn an_exited_session_keeps_its_last_line_whole_after_the_terminal_narrows_and_widens() {
    let mut screen = exited_at(88);

    screen.resize(100, 20);
    screen.resize(120, 20);

    let shown = screen.draw();
    assert!(shown.contains(&line(88)), "{shown}");
}

#[test]
fn a_session_that_exits_while_zoomed_keeps_its_last_line_for_a_wider_terminal() {
    let mut screen = Screen::at(
        App::new(printing_at(120)).with_sidebar_width(30),
        home(),
        120,
    );
    screen.press('n');
    zoom(&mut screen);
    screen.wait_for("acme done");
    screen.wait_until("the zoom to end", |shown| !shown.contains(&line(120)));

    screen.resize(240, 20);

    let shown = screen.draw();
    assert!(shown.contains(&line(120)), "{shown}");
}

fn started(width: u16) -> Screen {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::at(
        App::new(numbered_agent(scratch.path())).with_sidebar_width(width),
        home(),
        120,
    );
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen
}

fn narrower(screen: &mut Screen) {
    screen.key(KeyCode::Char('-'), KeyModifiers::ALT);
}

fn wider(screen: &mut Screen) {
    screen.key(KeyCode::Char('='), KeyModifiers::ALT);
}

fn zoom(screen: &mut Screen) {
    screen.key(KeyCode::Char('z'), KeyModifiers::ALT);
}

fn exited_at(columns: u16) -> Screen {
    let mut screen = Screen::at(
        App::new(printing_at(columns)).with_sidebar_width(30),
        home(),
        120,
    );
    screen.press('n');
    screen.wait_for("acme done");
    screen.wait_for("exited");
    screen
}

fn printing_at(columns: u16) -> Vec<String> {
    script(&format!(
        "while [ \"$(stty size | cut -d' ' -f2)\" != {columns} ]; do sleep 0.05; done; \
         printf '%s\\nacme done\\n' '{}'",
        line(columns)
    ))
}

fn line(columns: u16) -> String {
    format!("{}Z", "a".repeat(usize::from(columns) - 1))
}
