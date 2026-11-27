use std::time::{Duration, Instant};

use harbour::session::Session;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::{COLS, ROWS, cursor_at, replied, start, wait_for};

#[test]
fn a_session_mid_update_draws_the_frame_before_it() {
    let session = start(&after_enter(&format!("{BEGIN}{HALF}")));
    wait_for(&session, "acme whole");
    let before = Instant::now();

    session.send(b"\r");
    wait_for(&session, "acme half");

    let drawn = drawn_at(&session, before);
    assert!(drawn.contains("acme whole"), "{drawn}");
    assert!(!drawn.contains("acme half"), "{drawn}");
}

#[test]
fn a_session_mid_update_puts_the_cursor_where_the_frame_before_it_had_it() {
    let session = start(&after_enter(&format!("{BEGIN}{HALF}")));
    wait_for(&session, "acme whole");
    let before = Instant::now();

    session.send(b"\r");
    wait_for(&session, "acme half");

    assert_eq!(
        cursor_at(&session, Rect::new(0, 0, COLS, ROWS), before),
        Some(Position::new(14, 2))
    );
}

#[test]
fn a_session_whose_update_ended_draws_the_new_frame() {
    let session = start(&after_enter(&format!(
        "{BEGIN}{HALF}'; sleep 0.2; printf '\\033[2J\\033[1;1Hacme new{END}"
    )));
    wait_for(&session, "acme whole");
    let before = Instant::now();

    session.send(b"\r");
    wait_for(&session, "acme new");

    let drawn = drawn_at(&session, before);
    assert!(drawn.contains("acme new"), "{drawn}");
}

#[test]
fn a_session_whose_begin_marker_came_in_two_reads_draws_the_frame_before_it() {
    let session = start(&after_enter(&format!(
        "\\033[?20'; sleep 0.2; printf '26h{HALF}"
    )));
    wait_for(&session, "acme whole");
    let before = Instant::now();

    session.send(b"\r");
    wait_for(&session, "acme half");

    let drawn = drawn_at(&session, before);
    assert!(drawn.contains("acme whole"), "{drawn}");
}

#[test]
fn a_session_whose_update_never_ends_draws_the_live_screen_after_the_wait() {
    let session = start(&after_enter(&format!("{BEGIN}{HALF}")));
    wait_for(&session, "acme whole");

    session.send(b"\r");
    wait_for(&session, "acme half");

    let drawn = drawn_at(&session, Instant::now() + Duration::from_secs(1));
    assert!(drawn.contains("acme half"), "{drawn}");
}

#[test]
fn a_second_begin_mid_update_keeps_the_frame_before_the_first() {
    let session = start(&after_enter(&format!(
        "{BEGIN}{HALF}{BEGIN}\\033[7;1Hacme more"
    )));
    wait_for(&session, "acme whole");
    let before = Instant::now();

    session.send(b"\r");
    wait_for(&session, "acme more");

    let drawn = drawn_at(&session, before);
    assert!(drawn.contains("acme whole"), "{drawn}");
}

#[test]
fn a_begin_after_an_update_that_never_ended_draws_the_frame_before_the_new_one() {
    let session = start(&after_enter(&format!(
        "{BEGIN}{HALF}'; read -r line; printf '{BEGIN}\\033[7;1Hacme more"
    )));
    wait_for(&session, "acme whole");
    session.send(b"\r");
    wait_for(&session, "acme half");
    let before = Instant::now();

    session.send(b"\r");
    wait_for(&session, "acme more");

    let drawn = drawn_at(&session, before);
    assert!(drawn.contains("acme half"), "{drawn}");
    assert!(!drawn.contains("acme more"), "{drawn}");
}

#[test]
fn a_query_for_synchronized_output_is_answered_reset() {
    let session = start(&replied("\\033[?2026$p", 11));
    let expected = "reply:[?2026;2$y";
    let screen = wait_for(&session, expected);
    assert!(screen.contains(expected), "{screen}");
}

#[test]
fn a_query_for_synchronized_output_mid_update_is_answered_set() {
    let session = start(&replied("\\033[?2026h\\033[?2026$p", 11));
    let expected = "reply:[?2026;1$y";
    let screen = wait_for(&session, expected);
    assert!(screen.contains(expected), "{screen}");
}

const BEGIN: &str = "\\033[?2026h";
const END: &str = "\\033[?2026l";
const HALF: &str = "\\033[2J\\033[6;9Hacme half";

fn after_enter(update: &str) -> String {
    format!(
        "stty -echo; printf '\\033[3;5Hacme whole'; read -r line; printf '{update}'; exec sleep 600"
    )
}

fn drawn_at(session: &Session, now: Instant) -> String {
    let area = Rect::new(0, 0, COLS, ROWS);
    let mut buffer = Buffer::empty(area);
    session.draw(area, &mut buffer, now);
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
