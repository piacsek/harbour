use harbour::registry::Status;
use harbour::transcript::Facts;

use crate::{card, idle, sidebar, stopped};

#[test]
fn a_given_title_shows_over_claude_codes() {
    let facts = Facts {
        title: Some("Fix the flaky login test".to_string()),
        ..Facts::default()
    };
    let renamed = card(Status::Idle, None, &facts).renamed(Some("acme login fix"));

    let (rows, _) = sidebar(&[renamed, idle()], 1, 30);

    assert_eq!(rows[0], "  ○ acme login fix          3m");
}

#[test]
fn a_given_title_shows_on_a_stopped_card() {
    let (rows, _) = sidebar(&[stopped().renamed(Some("acme login fix")), idle()], 1, 30);

    assert_eq!(rows[0], "  ○ acme login fix");
}

#[test]
fn no_given_title_keeps_claude_codes() {
    let facts = Facts {
        title: Some("Fix the flaky login test".to_string()),
        ..Facts::default()
    };
    let kept = card(Status::Idle, None, &facts).renamed(None);

    let (rows, _) = sidebar(&[kept, idle()], 1, 30);

    assert_eq!(rows[0], "  ○ Fix the flaky login te… 3m");
}

#[test]
fn a_given_title_is_cleaned() {
    let (rows, _) = sidebar(
        &[
            stopped().renamed(Some("acme\n login\u{200b}  fix ")),
            idle(),
        ],
        1,
        30,
    );

    assert_eq!(rows[0], "  ○ acme login fix");
}

#[test]
fn a_cjk_given_title_at_the_narrowest_sidebar_is_cut_at_a_whole_character() {
    let (rows, _) = sidebar(
        &[stopped().renamed(Some("東京の会議の準備")), idle()],
        1,
        13,
    );

    assert_eq!(rows[0], "  ○ 東京の会…");
}
