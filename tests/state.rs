use harbour::registry::{Record, Status};
use harbour::state::{counts, title};
use ratatui::text::Line;

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

    let line = title(counts(sessions.into_iter()), NARROWEST_TITLE);

    assert_eq!(text(&line), " ●10 ◉10 ○10 ·10 ");
    assert!(line.width() <= NARROWEST_TITLE);
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
