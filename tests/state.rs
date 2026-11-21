use harbour::registry::{Record, Status};
use harbour::state::{Counts, counts, title};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};

const NARROWEST_TITLE: usize = 18;

#[test]
fn the_title_counts_unknown_sessions_on_their_own() {
    let records = [record(Status::Busy), record(Status::Idle)];
    let sessions = [
        (false, Some(&records[0])),
        (false, Some(&records[1])),
        (true, None),
        (false, None),
    ];

    let line = title(counts(sessions.into_iter()), 60);

    assert_eq!(text(&line), " sessions ● 1 ○ 2 · 1 ");
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

    let line = title(counts(sessions.into_iter()), NARROWEST_TITLE);

    assert_eq!(text(&line), " ●10 ◉10 ○10 ·10 ");
    assert!(line.width() <= NARROWEST_TITLE);
}

#[test]
fn the_title_shows_working_sessions_alone() {
    let line = title(only(|counts| counts.working = 1), 60);

    assert_eq!(text(&line), " sessions ● 1 ");
}

#[test]
fn the_title_shows_blocked_sessions_alone() {
    let line = title(only(|counts| counts.blocked = 1), 60);

    assert_eq!(text(&line), " sessions ◉ 1 ");
}

#[test]
fn the_title_shows_idle_sessions_alone() {
    let line = title(only(|counts| counts.idle = 2), 60);

    assert_eq!(text(&line), " sessions ○ 2 ");
}

#[test]
fn the_title_shows_unknown_sessions_alone() {
    let line = title(only(|counts| counts.unknown = 3), 60);

    assert_eq!(text(&line), " sessions · 3 ");
}

#[test]
fn the_title_shows_all_four_counts_in_order() {
    let all = Counts {
        blocked: 2,
        working: 1,
        idle: 3,
        unknown: 4,
    };

    let line = title(all, 60);

    assert_eq!(text(&line), " sessions ● 1 ◉ 2 ○ 3 · 4 ");
}

#[test]
fn with_no_session_the_title_is_the_word_alone() {
    let line = title(Counts::default(), 60);

    assert_eq!(text(&line), " sessions ");
}

#[test]
fn the_narrow_title_leaves_out_the_zeros() {
    let counts = Counts {
        working: 1,
        idle: 2,
        ..Counts::default()
    };

    let line = title(counts, 10);

    assert_eq!(text(&line), " ●1 ○2 ");
}

#[test]
fn a_title_too_narrow_for_the_word_with_no_session_keeps_the_border_whole() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 7, 3));

    Block::bordered()
        .title(title(Counts::default(), 5))
        .render(buffer.area, &mut buffer);

    assert_eq!(row(&buffer, 0), "┌─────┐");
}

fn record(status: Status) -> Record {
    Record {
        status,
        waiting_for: None,
        status_since: None,
        session_id: None,
        cwd: None,
    }
}

fn text(line: &Line) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

fn only(set: impl FnOnce(&mut Counts)) -> Counts {
    let mut counts = Counts::default();
    set(&mut counts);
    counts
}

fn row(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol())
        .collect()
}
