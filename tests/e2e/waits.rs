use harbour::app::FOCUSED_BORDER;

use crate::{Server, focused_at, pane_corner};

#[test]
#[ignore]
fn a_server_holds_a_pty_slot() {
    let _server = Server::start("slot", 80, 10);

    assert!(crate::pty_slots::holding());
}

#[test]
fn a_wide_character_before_a_corner_counts_two_columns() {
    let set = FOCUSED_BORDER.to_border_set();
    let screen = format!(
        "修{}{}\n  {}",
        set.top_left, set.horizontal_top, set.vertical_left
    );
    assert!(focused_at(&screen, 2));
}

#[test]
fn a_wide_title_before_the_pane_corner_counts_two_columns() {
    let corner = FOCUSED_BORDER.to_border_set().top_left;
    let screen = format!("{corner}修{corner}");
    assert_eq!(pane_corner(&screen), Some(3));
}
