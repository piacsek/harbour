use std::time::Duration;

use harbour::card::{age, clean, fit};
use unicode_width::UnicodeWidthStr;

#[test]
fn text_that_fits_is_kept_whole() {
    assert_eq!(fit("acme-api", 8), "acme-api");
}

#[test]
fn text_too_wide_is_cut_with_an_ellipsis() {
    assert_eq!(fit("permission prompt", 14), "permission pr…");
}

#[test]
fn wide_characters_are_counted_by_the_columns_they_take() {
    assert_eq!(fit("東京大阪名古屋横浜", 7), "東京大…");
}

#[test]
fn a_wide_character_that_would_not_fit_is_left_out_whole() {
    assert_eq!(fit("acme-東京-api", 7), "acme-…");
    assert_eq!(fit("acme-東京-api", 7).width(), 6);
}

#[test]
fn an_emoji_is_never_split() {
    assert_eq!(fit("ship 🚀🚀🚀", 8), "ship 🚀…");
}

#[test]
fn no_room_leaves_nothing() {
    assert_eq!(fit("acme", 0), "");
}

#[test]
fn one_column_holds_only_the_ellipsis() {
    assert_eq!(fit("acme", 1), "…");
}

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
fn line_breaks_and_tabs_fold_into_one_space() {
    assert_eq!(clean("fix the\n\n  retry\ttest\r\n"), "fix the retry test");
}

#[test]
fn an_escape_sequence_never_reaches_the_terminal_raw() {
    assert_eq!(clean("acme\u{1b}[31mred\u{7}"), "acme [31mred");
}

#[test]
fn control_characters_of_every_range_are_dropped() {
    assert_eq!(clean("a\u{0}b\u{9b}c\u{7f}d"), "a b c d");
}

#[test]
fn bidirectional_overrides_are_dropped() {
    assert_eq!(clean("acme\u{202e}ipa"), "acme ipa");
}
