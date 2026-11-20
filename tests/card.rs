use std::time::Duration;

use harbour::card::{Card, Sidebar, age};
use harbour::registry::{Record, Status};
use harbour::repo::Place;
use harbour::text::columns;
use harbour::transcript::Facts;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

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
        activity: Some("✔\u{fe0f} run 🇧🇷 the cafe\u{301} 東京 tests".to_string()),
        branch: Some("fix-⚠\u{fe0f}-東京-🇧🇷-retry".to_string()),
        ..Facts::default()
    };

    let rows = drawn(&facts, 20);

    assert_eq!(rows[0], "  ● ⚠\u{fe0f} ship ⚠\u{fe0f} t… 3m");
    assert_eq!(rows[1], "    ✔\u{fe0f} run 🇧🇷 the c…");
    assert_eq!(rows[2], "    acme-api · fix-…");
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
