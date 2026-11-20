use harbour::registry::{Record, Status};
use harbour::state::{row, title};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::Widget;

const NARROWEST_ROW: usize = 16;
const NARROWEST_TITLE: usize = 18;

#[test]
fn a_wide_character_name_at_the_narrowest_sidebar_still_shows_part_of_the_reason() {
    let record = waiting("permission prompt");

    let line = row("東京大阪名古屋横浜", None, Some(&record), NARROWEST_ROW);

    assert_eq!(line.spans[1].content, " 東京大…");
    assert!(shown(&line, NARROWEST_ROW).contains(" perm"));
}

#[test]
fn a_mixed_width_name_is_cut_before_a_wide_character_that_would_not_fit() {
    let record = waiting("permission prompt");

    let line = row("acme-東京-api", None, Some(&record), NARROWEST_ROW);

    assert_eq!(line.spans[1].content, " acme-…");
    assert!(shown(&line, NARROWEST_ROW).contains(" perm"));
}

#[test]
fn a_wide_character_name_that_fits_beside_the_reason_is_kept_whole() {
    let record = waiting("input");

    let line = row("東京", None, Some(&record), NARROWEST_ROW);

    assert_eq!(line.spans[1].content, " 東京");
}

#[test]
fn the_title_counts_unknown_sessions_on_their_own() {
    let records = [record(Status::Busy), record(Status::Idle)];
    let sessions = [
        (false, Some(&records[0])),
        (false, Some(&records[1])),
        (true, None),
        (false, None),
    ];

    let line = title(sessions.into_iter(), 60);

    assert_eq!(text(&line), " sessions ● 1 ◉ 0 ○ 2 · 1 ");
}

#[test]
fn the_narrowest_title_keeps_every_count_when_each_is_past_nine() {
    let records = [
        record(Status::Busy),
        record(Status::Waiting),
        record(Status::Idle),
    ];
    let sessions: Vec<(bool, Option<&Record>)> = (0..40)
        .map(|index| match index % 4 {
            3 => (false, None),
            kind => (false, Some(&records[kind])),
        })
        .collect();

    let line = title(sessions.into_iter(), NARROWEST_TITLE);

    assert_eq!(text(&line), " ●10 ◉10 ○10 ·10 ");
    assert!(line.width() <= NARROWEST_TITLE);
}

fn waiting(reason: &str) -> Record {
    Record {
        status: Status::Waiting,
        waiting_for: Some(reason.to_string()),
    }
}

fn record(status: Status) -> Record {
    Record {
        status,
        waiting_for: None,
    }
}

fn text(line: &Line) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn shown(line: &Line, width: usize) -> String {
    let area = Rect::new(0, 0, u16::try_from(width).unwrap(), 1);
    let mut buffer = Buffer::empty(area);
    line.clone().render(area, &mut buffer);
    buffer.content.iter().map(|cell| cell.symbol()).collect()
}
