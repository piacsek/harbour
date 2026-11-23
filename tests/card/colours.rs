use std::time::Duration;

use harbour::card::{Card, Ended};
use harbour::icons::{BRANCH, NEOVIM, PULL_REQUEST, REPOSITORY, SUBAGENTS};
use harbour::registry::Status;
use harbour::repo::Place;
use harbour::transcript::Facts;
use ratatui::style::{Color, Modifier};

use crate::{all_facts, alone, at, card, every_fact, exited, idle, place, row, sidebar, stopped};

#[test]
fn the_repository_is_blue() {
    let buffer = alone(card(Status::Busy, None, &all_facts()), 60);

    assert_eq!(at(&buffer, 2, "acme-api").fg, Color::Blue);
}

#[test]
fn the_branch_is_magenta() {
    let buffer = alone(card(Status::Busy, None, &all_facts()), 60);

    assert_eq!(at(&buffer, 2, "fix-retry").fg, Color::Magenta);
}

#[test]
fn the_pull_request_is_cyan() {
    let buffer = alone(card(Status::Busy, None, &all_facts()), 60);

    assert_eq!(at(&buffer, 2, "#40").fg, Color::Cyan);
}

#[test]
fn the_subagents_are_dimmed() {
    let buffer = alone(card(Status::Busy, None, &all_facts()), 60);

    assert!(
        at(&buffer, 2, "2 subagents")
            .modifier
            .contains(Modifier::DIM)
    );
}

#[test]
fn the_separators_are_dimmed() {
    let buffer = alone(card(Status::Busy, None, &all_facts()), 60);

    assert!(at(&buffer, 2, "· fix").modifier.contains(Modifier::DIM));
}

#[test]
fn the_facts_themselves_are_not_dimmed() {
    let buffer = alone(card(Status::Busy, None, &all_facts()), 60);

    assert!(!at(&buffer, 2, "fix-retry").modifier.contains(Modifier::DIM));
}

#[test]
fn a_cut_branch_keeps_its_colour_up_to_the_ellipsis() {
    let buffer = alone(card(Status::Busy, None, &all_facts()), 20);

    assert_eq!(row(&buffer, 2, 20), "    acme-api · fix-…");
    assert_eq!(at(&buffer, 2, "fix-").fg, Color::Magenta);
    assert_eq!(at(&buffer, 2, "…").fg, Color::Magenta);
}

#[test]
fn a_cjk_branch_at_the_narrowest_sidebar_keeps_its_colour_up_to_the_ellipsis() {
    let facts = Facts {
        branch: Some("修正-東京".to_string()),
        ..Facts::default()
    };

    let buffer = alone(card(Status::Idle, None, &facts), 18);

    assert_eq!(row(&buffer, 2, 18), "    acme-api · 修…");
    assert_eq!(at(&buffer, 2, "修").fg, Color::Magenta);
    assert_eq!(at(&buffer, 2, "…").fg, Color::Magenta);
}

#[test]
fn a_selected_card_keeps_its_fact_colours_under_the_bold() {
    let (_, buffer) = sidebar(&[card(Status::Busy, None, &all_facts()), idle()], 0, 60);

    assert_eq!(at(&buffer, 2, "fix-retry").fg, Color::Magenta);
    assert!(
        at(&buffer, 2, "fix-retry")
            .modifier
            .contains(Modifier::BOLD)
    );
}

#[test]
fn with_icons_each_glyph_takes_its_facts_colour() {
    let buffer = alone(every_fact(&place()), 60);

    assert_eq!(at(&buffer, 2, REPOSITORY).fg, Color::Blue);
    assert_eq!(at(&buffer, 2, BRANCH).fg, Color::Magenta);
    assert_eq!(at(&buffer, 2, PULL_REQUEST).fg, Color::Cyan);
    assert!(at(&buffer, 2, SUBAGENTS).modifier.contains(Modifier::DIM));
}

#[test]
fn with_icons_each_fact_keeps_its_colour_behind_its_glyph() {
    let buffer = alone(every_fact(&place()), 60);

    assert_eq!(at(&buffer, 2, "acme-api").fg, Color::Blue);
    assert_eq!(at(&buffer, 2, "fix-retry").fg, Color::Magenta);
    assert_eq!(at(&buffer, 2, "#40").fg, Color::Cyan);
}

#[test]
fn with_icons_the_editor_glyph_is_in_the_default_foreground() {
    let buffer = alone(every_fact(&place()), 60);

    assert_eq!(at(&buffer, 2, NEOVIM).fg, Color::Reset);
}

#[test]
fn with_icons_a_dropped_branch_glyph_leaves_an_ellipsis_in_the_branch_colour() {
    let place = Place {
        repository: "日本語-acme".to_string(),
        branch: None,
    };

    let buffer = alone(every_fact(&place), 20);

    assert_eq!(
        row(&buffer, 2, 20),
        format!("    {REPOSITORY} 日本語-acme …")
    );
    assert_eq!(at(&buffer, 2, "日本語").fg, Color::Blue);
    assert_eq!(at(&buffer, 2, "…").fg, Color::Magenta);
}

#[test]
fn the_age_is_dimmed() {
    let buffer = alone(idle(), 40);

    assert!(at(&buffer, 0, "3m").modifier.contains(Modifier::DIM));
}

#[test]
fn the_title_is_not_dimmed() {
    let buffer = alone(idle(), 40);

    assert!(
        !at(&buffer, 0, "acme-api 1")
            .modifier
            .contains(Modifier::DIM)
    );
}

#[test]
fn a_clean_exit_is_dimmed() {
    let buffer = alone(ended(Ended::Exited(String::new())), 40);

    assert!(at(&buffer, 1, "exited").modifier.contains(Modifier::DIM));
}

#[test]
fn an_exit_with_a_code_is_red() {
    let buffer = alone(ended(Ended::Exited(" 1".to_string())), 40);

    assert_eq!(at(&buffer, 1, "exited 1").fg, Color::Red);
}

#[test]
fn an_exit_by_a_signal_is_red() {
    let buffer = alone(ended(Ended::Exited(" (KILL)".to_string())), 40);

    assert_eq!(at(&buffer, 1, "exited (KILL)").fg, Color::Red);
}

#[test]
fn the_unknown_dot_is_in_the_default_foreground() {
    let card = Card::new("acme-api 1", None, None, &place(), None, None);

    let buffer = alone(card, 40);

    assert_eq!(at(&buffer, 0, "·").fg, Color::Reset);
}

#[test]
fn an_unselected_exited_card_is_dimmed_on_every_row() {
    let (_, buffer) = sidebar(&[idle(), exited()], 0, 40);

    assert!(
        at(&buffer, 4, "acme-api 1")
            .modifier
            .contains(Modifier::DIM)
    );
    assert!(at(&buffer, 5, "exited 1").modifier.contains(Modifier::DIM));
    assert!(at(&buffer, 6, "acme-api").modifier.contains(Modifier::DIM));
}

#[test]
fn an_unselected_exited_card_keeps_its_red_exit_under_the_dim() {
    let (_, buffer) = sidebar(&[idle(), exited()], 0, 40);

    assert_eq!(at(&buffer, 5, "exited 1").fg, Color::Red);
}

#[test]
fn an_unselected_stopped_card_is_dimmed() {
    let (_, buffer) = sidebar(&[idle(), stopped()], 0, 40);

    assert!(
        at(&buffer, 4, "acme-api 1")
            .modifier
            .contains(Modifier::DIM)
    );
}

#[test]
fn a_selected_exited_card_is_not_dimmed() {
    let (_, buffer) = sidebar(&[idle(), exited()], 1, 40);

    assert!(
        !at(&buffer, 4, "acme-api 1")
            .modifier
            .contains(Modifier::DIM)
    );
}

#[test]
fn a_running_card_is_not_dimmed() {
    let (_, buffer) = sidebar(&[exited(), idle()], 0, 40);

    assert!(
        !at(&buffer, 4, "acme-api 1")
            .modifier
            .contains(Modifier::DIM)
    );
}

fn ended(how: Ended) -> Card {
    Card::new(
        "acme-api 1",
        Some(how),
        None,
        &place(),
        Some(Duration::from_secs(180)),
        None,
    )
}
