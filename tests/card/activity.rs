use std::time::Duration;

use harbour::card::{Card, Ended, Sidebar};
use harbour::icons::{BRANCH, PULL_REQUEST, REPOSITORY, SUBAGENTS, TASKS_DONE, TASKS_OPEN};
use harbour::registry::Status;
use harbour::transcript::{Facts, Tasks};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::{at, card, idle, place, row};

#[test]
fn the_fourth_line_shows_the_subagents_then_the_tasks_done() {
    let line = fourth(card(Status::Busy, None, &active(2, 3, 7)), 40);

    assert_eq!(line, "    2 subagents · tasks 3/7");
}

#[test]
fn the_third_line_keeps_only_the_repository_branch_and_pull_request() {
    let buffer = drawn(card(Status::Busy, None, &active(2, 3, 7)), 40);

    assert_eq!(row(&buffer, 2, 40), "    acme-api · fix-retry · #40");
}

#[test]
fn one_subagent_and_no_tasks_shows_the_subagent_alone() {
    let facts = Facts {
        subagents: 1,
        ..Facts::default()
    };

    let line = fourth(card(Status::Busy, None, &facts), 40);

    assert_eq!(line, "    1 subagent");
}

#[test]
fn tasks_and_no_subagents_shows_the_tasks_alone() {
    let line = fourth(card(Status::Busy, None, &active(0, 7, 7)), 40);

    assert_eq!(line, "    tasks 7/7");
}

#[test]
fn with_icons_the_subagents_and_open_tasks_show_behind_their_glyphs() {
    let line = fourth(card(Status::Busy, None, &active(2, 3, 7)).with_icons(), 40);

    assert_eq!(line, "    \u{F160E} 2 \u{F08A8} 3/7");
}

#[test]
fn with_icons_every_task_done_shows_the_checked_glyph() {
    let line = fourth(card(Status::Busy, None, &active(2, 7, 7)).with_icons(), 40);

    assert_eq!(line, "    \u{F160E} 2 \u{F014E} 7/7");
}

#[test]
fn with_icons_the_third_line_keeps_only_its_three_facts() {
    let buffer = drawn(card(Status::Busy, None, &active(2, 3, 7)).with_icons(), 40);

    assert_eq!(
        row(&buffer, 2, 40),
        format!("    {REPOSITORY} acme-api {BRANCH} fix-retry {PULL_REQUEST} #40")
    );
}

#[test]
fn the_subagents_and_open_tasks_are_dimmed() {
    let buffer = drawn(card(Status::Busy, None, &active(2, 3, 7)), 40);

    assert!(
        at(&buffer, 3, "2 subagents")
            .modifier
            .contains(Modifier::DIM)
    );
    assert!(at(&buffer, 3, "tasks 3/7").modifier.contains(Modifier::DIM));
}

#[test]
fn every_task_done_is_green() {
    let buffer = drawn(card(Status::Busy, None, &active(2, 7, 7)), 40);

    assert_eq!(at(&buffer, 3, "tasks").fg, Color::Green);
    assert_eq!(at(&buffer, 3, "7/7").fg, Color::Green);
}

#[test]
fn with_icons_every_task_done_has_its_glyph_and_count_green() {
    let buffer = drawn(card(Status::Busy, None, &active(2, 7, 7)).with_icons(), 40);

    assert_eq!(at(&buffer, 3, TASKS_DONE).fg, Color::Green);
    assert_eq!(at(&buffer, 3, "7/7").fg, Color::Green);
}

#[test]
fn with_icons_the_subagents_and_open_tasks_are_dimmed() {
    let buffer = drawn(card(Status::Busy, None, &active(2, 3, 7)).with_icons(), 40);

    assert!(at(&buffer, 3, SUBAGENTS).modifier.contains(Modifier::DIM));
    assert!(at(&buffer, 3, TASKS_OPEN).modifier.contains(Modifier::DIM));
}

#[test]
fn at_the_narrowest_sidebar_the_fourth_line_is_cut_from_its_end() {
    let line = fourth(card(Status::Busy, None, &active(2, 3, 7)), 20);

    assert_eq!(line, "    2 subagents · t…");
}

#[test]
fn an_exited_card_shows_a_blank_fourth_line() {
    let card = Card::new(
        "acme-api 1",
        Some(Ended::Exited(" 1".to_string())),
        None,
        &place(),
        Some(Duration::from_secs(180)),
        Some(&active(2, 3, 7)),
    );

    let line = fourth(card.with_icons(), 40);

    assert_eq!(line, "");
}

#[test]
fn a_stopped_card_shows_a_blank_fourth_line() {
    let card = Card::new(
        "acme-api 1",
        Some(Ended::Stopped("directory gone")),
        None,
        &place(),
        None,
        Some(&active(2, 3, 7)),
    );

    let line = fourth(card, 40);

    assert_eq!(line, "");
}

#[test]
fn a_card_with_nothing_to_show_keeps_its_fourth_row_before_the_blank_line() {
    let area = Rect::new(0, 0, 30, 10);
    let mut buffer = Buffer::empty(area);

    Sidebar::default().draw(&[idle(), idle()], 1, Color::Cyan, area, &mut buffer);

    assert_eq!(row(&buffer, 3, 30), "");
    assert_eq!(row(&buffer, 4, 30), "");
    assert_eq!(row(&buffer, 5, 30), "▌ ○ acme-api 1              3m");
}

fn active(subagents: usize, done: usize, total: usize) -> Facts {
    Facts {
        branch: Some("fix-retry".to_string()),
        pull_request: Some(40),
        subagents,
        tasks: Some(Tasks { done, total }),
        ..Facts::default()
    }
}

fn drawn(card: Card, width: u16) -> Buffer {
    let area = Rect::new(0, 0, width, 4);
    let mut buffer = Buffer::empty(area);
    Sidebar::default().draw(&[card], 1, Color::Cyan, area, &mut buffer);
    buffer
}

fn fourth(card: Card, width: u16) -> String {
    row(&drawn(card, width), 3, width)
}
