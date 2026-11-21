use std::time::Duration;

use harbour::card::{Card, Ended, Sidebar, age};
use harbour::registry::{Record, Status};
use harbour::repo::Place;
use harbour::text::columns;
use harbour::transcript::Facts;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;

#[test]
fn under_a_minute_reads_less_than_a_minute() {
    assert_eq!(age(Duration::from_secs(0)), "<1m");
    assert_eq!(age(Duration::from_secs(59)), "<1m");
}

#[test]
fn minutes_hours_and_days_are_whole_and_rounded_down() {
    assert_eq!(age(Duration::from_secs(60)), "1m");
    assert_eq!(age(Duration::from_secs(59 * 60 + 59)), "59m");
    assert_eq!(age(Duration::from_secs(3600)), "1h");
    assert_eq!(age(Duration::from_secs(23 * 3600 + 3599)), "23h");
    assert_eq!(age(Duration::from_secs(24 * 3600)), "1d");
    assert_eq!(age(Duration::from_secs(400 * 24 * 3600)), "400d");
}

#[test]
fn a_card_of_emoji_at_the_narrowest_sidebar_keeps_every_line_inside_it() {
    let facts = Facts {
        title: Some("⚠\u{fe0f} ship ⚠\u{fe0f} the 👩\u{200d}💻 api ✅\u{fe0f}".to_string()),
        prompt: Some("✔\u{fe0f} run 🇧🇷 the cafe\u{301} 東京 tests".to_string()),
        branch: Some("fix-⚠\u{fe0f}-東京-🇧🇷-retry".to_string()),
        ..Facts::default()
    };

    let rows = drawn(&facts, 20);

    assert_eq!(rows[0], "  ● ⚠\u{fe0f} ship ⚠\u{fe0f} t… 3m");
    assert_eq!(rows[1], "    ✔\u{fe0f} run 🇧🇷 the c…");
    assert_eq!(rows[2], "    acme-api · fix-…");
}

#[test]
fn a_cjk_prompt_at_the_narrowest_sidebar_is_cut_between_wide_characters() {
    let (rows, _) = sidebar(&[working("東京の テストを 直して"), idle()], 1, 20);

    assert_eq!(rows[1], "    東京の テストを…");
}

#[test]
fn an_unselected_working_card_shows_its_last_prompt_dimmed() {
    let (rows, buffer) = sidebar(&[working("fix the flaky retry test"), idle()], 1, 40);

    assert_eq!(rows[1], "    fix the flaky retry test");
    assert!(buffer[(4, 1)].modifier.contains(Modifier::DIM));
}

#[test]
fn a_selected_working_card_shows_no_second_line() {
    let (rows, _) = sidebar(&[working("fix the flaky retry test"), idle()], 0, 40);

    assert_eq!(rows[1], "▌");
}

#[test]
fn a_selected_blocked_card_shows_no_second_line() {
    let (rows, _) = sidebar(&[blocked(), idle()], 0, 40);

    assert_eq!(rows[1], "▌");
}

#[test]
fn an_unselected_blocked_card_shows_what_it_waits_for() {
    let (rows, _) = sidebar(&[blocked(), idle()], 1, 40);

    assert_eq!(rows[1], "    permission prompt");
}

#[test]
fn a_selected_exited_card_shows_no_second_line() {
    let (rows, _) = sidebar(&[exited(), idle()], 0, 40);

    assert_eq!(rows[1], "▌");
}

#[test]
fn an_unselected_exited_card_shows_its_exit() {
    let (rows, _) = sidebar(&[exited(), idle()], 1, 40);

    assert_eq!(rows[1], "    exited 1");
}

#[test]
fn a_selected_stopped_card_shows_no_second_line() {
    let (rows, _) = sidebar(&[stopped(), idle()], 0, 40);

    assert_eq!(rows[1], "▌");
}

#[test]
fn an_unselected_stopped_card_shows_why_it_cannot_resume() {
    let (rows, _) = sidebar(&[stopped(), idle()], 1, 40);

    assert_eq!(rows[1], "    directory gone");
}

#[test]
fn a_working_card_with_no_prompt_read_yet_shows_no_second_line() {
    let (rows, _) = sidebar(
        &[card(Status::Busy, None, &Facts::default()), idle()],
        1,
        40,
    );

    assert_eq!(rows[1], "");
}

#[test]
fn a_card_below_starts_on_the_same_row_whichever_card_above_is_selected() {
    let (above, _) = sidebar(&[working("fix the retry"), idle()], 0, 40);
    let (below, _) = sidebar(&[working("fix the retry"), idle()], 1, 40);

    assert_eq!(above[4], "  ○ acme-api 1                        3m");
    assert_eq!(below[4], "▌ ○ acme-api 1                        3m");
}

fn drawn(facts: &Facts, width: u16) -> Vec<String> {
    let record = Record {
        status: Status::Busy,
        waiting_for: None,
        status_since: None,
        session_id: None,
        cwd: None,
    };
    let place = Place {
        repository: "acme-api".to_string(),
        branch: None,
    };
    let card = Card::new(
        "acme-api 1",
        None,
        Some(&record),
        &place,
        Some(Duration::from_secs(180)),
        Some(facts),
    );
    let area = Rect::new(0, 0, width, 3);
    let mut buffer = Buffer::empty(area);
    Sidebar::default().draw(&[card], 1, area, &mut buffer);
    (0..3).map(|y| row(&buffer, y, width)).collect()
}

fn row(buffer: &Buffer, y: u16, width: u16) -> String {
    let mut shown = String::new();
    let mut x = 0;
    while x < width {
        let symbol = buffer[(x, y)].symbol();
        shown.push_str(symbol);
        x += u16::try_from(columns(symbol).max(1)).unwrap();
    }
    shown.trim_end().to_string()
}

fn sidebar(cards: &[Card], selected: usize, width: u16) -> (Vec<String>, Buffer) {
    let area = Rect::new(0, 0, width, 7);
    let mut buffer = Buffer::empty(area);
    Sidebar::default().draw(cards, selected, area, &mut buffer);
    let rows = (0..7).map(|y| row(&buffer, y, width)).collect();
    (rows, buffer)
}

fn working(prompt: &str) -> Card {
    let facts = Facts {
        prompt: Some(prompt.to_string()),
        activity: Some("Run the retry tests".to_string()),
        ..Facts::default()
    };
    card(Status::Busy, None, &facts)
}

fn idle() -> Card {
    card(Status::Idle, None, &Facts::default())
}

fn blocked() -> Card {
    card(
        Status::Waiting,
        Some("permission prompt"),
        &Facts::default(),
    )
}

fn exited() -> Card {
    Card::new(
        "acme-api 1",
        Some(Ended::Exited(" 1".to_string())),
        None,
        &place(),
        Some(Duration::from_secs(180)),
        None,
    )
}

fn stopped() -> Card {
    Card::new(
        "acme-api 1",
        Some(Ended::Stopped("directory gone")),
        None,
        &place(),
        None,
        None,
    )
}

fn card(status: Status, waiting_for: Option<&str>, facts: &Facts) -> Card {
    let record = Record {
        status,
        waiting_for: waiting_for.map(str::to_string),
        status_since: None,
        session_id: None,
        cwd: None,
    };
    Card::new(
        "acme-api 1",
        None,
        Some(&record),
        &place(),
        Some(Duration::from_secs(180)),
        Some(facts),
    )
}

fn place() -> Place {
    Place {
        repository: "acme-api".to_string(),
        branch: None,
    }
}
