use std::time::Duration;

use harbour::card::{Card, Sidebar};
use harbour::icons::BRANCH;
use harbour::registry::{Record, Status};
use harbour::repo::Place;
use harbour::transcript::Facts;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::{idle, row};

#[test]
fn a_group_draws_its_repository_and_a_rule_above_its_first_card() {
    let rows = drawn(&[idle().in_group(true), idle().in_group(false)], 1, 30, 9);

    assert_eq!(rows[0], format!("  acme-api {}", "─".repeat(19)));
    assert_eq!(rows[1], "  ○ acme-api 1              3m");
}

#[test]
fn the_cards_of_a_group_keep_a_blank_line_between_them() {
    let rows = drawn(&[idle().in_group(true), idle().in_group(false)], 1, 30, 9);

    assert_eq!(rows[4], "");
    assert_eq!(rows[5], "▌ ○ acme-api 1              3m");
}

#[test]
fn a_card_after_a_group_starts_after_the_blank_line() {
    let cards = [idle().in_group(true), idle().in_group(false), other()];

    let rows = drawn(&cards, 2, 30, 13);

    assert_eq!(rows[8], "");
    assert_eq!(rows[9], "▌ ○ acme-web 1              3m");
    assert_eq!(rows[11], "▌   acme-web");
}

#[test]
fn the_header_is_dimmed() {
    let buffer = buffer(&[idle().in_group(true), idle().in_group(false)], 1, 30, 9);

    assert!(buffer[(2, 0)].modifier.contains(Modifier::DIM));
    assert!(buffer[(12, 0)].modifier.contains(Modifier::DIM));
}

#[test]
fn the_header_is_never_drawn_as_selected() {
    let buffer = buffer(&[idle().in_group(true), idle().in_group(false)], 0, 30, 9);

    assert_eq!(
        row(&buffer, 0, 30),
        format!("  acme-api {}", "─".repeat(19))
    );
    assert!(!buffer[(2, 0)].modifier.contains(Modifier::BOLD));
}

#[test]
fn a_card_in_a_group_drops_the_repository_from_its_third_line() {
    let cards = [branched("fix-login").in_group(true), idle().in_group(false)];

    let rows = drawn(&cards, 1, 30, 9);

    assert_eq!(rows[3], "    fix-login");
}

#[test]
fn a_card_in_a_group_with_no_branch_has_an_empty_third_line() {
    let rows = drawn(&[idle().in_group(true), idle().in_group(false)], 0, 30, 9);

    assert_eq!(rows[7], "");
}

#[test]
fn with_icons_a_card_in_a_group_keeps_its_branch_glyph_and_no_repository_glyph() {
    let cards = [
        branched("fix-login").with_icons().in_group(true),
        idle().in_group(false),
    ];

    let rows = drawn(&cards, 1, 30, 9);

    assert_eq!(rows[3], format!("    {BRANCH} fix-login"));
}

#[test]
fn at_the_narrowest_sidebar_a_long_name_is_cut_and_shows_no_rule() {
    let cards = [
        named("acme-platform-api").in_group(true),
        idle().in_group(false),
    ];

    let rows = drawn(&cards, 1, 13, 9);

    assert_eq!(rows[0], "  acme-platf…");
}

#[test]
fn at_the_narrowest_sidebar_a_name_that_leaves_no_column_after_its_space_shows_no_rule() {
    let cards = [named("acme-infra").in_group(true), idle().in_group(false)];

    let rows = drawn(&cards, 1, 13, 9);

    assert_eq!(rows[0], "  acme-infra");
}

#[test]
fn at_the_narrowest_sidebar_a_short_name_keeps_the_rule_to_the_edge() {
    let rows = drawn(&[idle().in_group(true), idle().in_group(false)], 1, 13, 9);

    assert_eq!(rows[0], "  acme-api ──");
}

#[test]
fn a_cjk_name_at_the_narrowest_sidebar_is_cut_between_wide_characters() {
    let cards = [
        named("東京リポジトリ").in_group(true),
        idle().in_group(false),
    ];

    let rows = drawn(&cards, 1, 13, 9);

    assert_eq!(rows[0], "  東京リポジ…");
}

#[test]
fn a_short_cjk_name_keeps_the_rule_to_the_edge_by_columns() {
    let cards = [named("東京リポ").in_group(true), idle().in_group(false)];

    let rows = drawn(&cards, 1, 13, 9);

    assert_eq!(rows[0], "  東京リポ ──");
}

#[test]
fn scrolling_up_to_a_groups_first_card_shows_its_header() {
    let cards = [other(), idle().in_group(true), idle().in_group(false)];
    let mut sidebar = Sidebar::default();
    scrolled(&mut sidebar, &cards, 2, 4);

    let rows = scrolled(&mut sidebar, &cards, 1, 4);

    assert_eq!(rows[0], format!("  acme-api {}", "─".repeat(19)));
    assert_eq!(rows[1], "▌ ○ acme-api 1              3m");
}

#[test]
fn scrolling_down_to_a_groups_first_card_shows_its_header() {
    let cards = [other(), idle().in_group(true), idle().in_group(false)];
    let mut sidebar = Sidebar::default();
    scrolled(&mut sidebar, &cards, 0, 4);

    let rows = scrolled(&mut sidebar, &cards, 1, 4);

    assert_eq!(rows[0], format!("  acme-api {}", "─".repeat(19)));
    assert_eq!(rows[1], "▌ ○ acme-api 1              3m");
}

#[test]
fn a_sidebar_too_short_for_the_header_keeps_the_selected_card_whole() {
    let cards = [other(), idle().in_group(true), idle().in_group(false)];
    let mut sidebar = Sidebar::default();
    scrolled(&mut sidebar, &cards, 2, 3);

    let rows = scrolled(&mut sidebar, &cards, 1, 3);

    assert_eq!(rows[0], "▌ ○ acme-api 1              3m");
}

fn drawn(cards: &[Card], selected: usize, width: u16, height: u16) -> Vec<String> {
    let buffer = buffer(cards, selected, width, height);
    (0..height).map(|y| row(&buffer, y, width)).collect()
}

fn scrolled(sidebar: &mut Sidebar, cards: &[Card], selected: usize, height: u16) -> Vec<String> {
    let area = Rect::new(0, 0, 30, height);
    let mut buffer = Buffer::empty(area);
    sidebar.draw(cards, selected, Color::Cyan, area, &mut buffer);
    (0..height).map(|y| row(&buffer, y, 30)).collect()
}

fn buffer(cards: &[Card], selected: usize, width: u16, height: u16) -> Buffer {
    let area = Rect::new(0, 0, width, height);
    let mut buffer = Buffer::empty(area);
    Sidebar::default().draw(cards, selected, Color::Cyan, area, &mut buffer);
    buffer
}

fn other() -> Card {
    placed(
        "acme-web 1",
        &Place {
            repository: "acme-web".to_string(),
            branch: None,
        },
        &Facts::default(),
    )
}

fn named(repository: &str) -> Card {
    placed(
        "acme-api 1",
        &Place {
            repository: repository.to_string(),
            branch: None,
        },
        &Facts::default(),
    )
}

fn branched(branch: &str) -> Card {
    let facts = Facts {
        branch: Some(branch.to_string()),
        ..Facts::default()
    };
    placed(
        "acme-api 1",
        &Place {
            repository: "acme-api".to_string(),
            branch: None,
        },
        &facts,
    )
}

fn placed(name: &str, place: &Place, facts: &Facts) -> Card {
    let record = Record {
        status: Status::Idle,
        waiting_for: None,
        status_since: None,
        session_id: None,
        cwd: None,
    };
    Card::new(
        name,
        None,
        Some(&record),
        place,
        Some(Duration::from_secs(180)),
        Some(facts),
    )
}
