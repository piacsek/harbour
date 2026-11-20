use harbour::text::{clean, columns, fit, wrap};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
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
fn an_emoji_with_a_variation_selector_is_cut_by_the_columns_it_takes() {
    assert_eq!(
        fit("⚠\u{fe0f}⚠\u{fe0f}⚠\u{fe0f} deploy", 6),
        "⚠\u{fe0f}⚠\u{fe0f}…"
    );
}

#[test]
fn a_joined_emoji_sequence_is_cut_whole() {
    assert_eq!(
        fit("👩\u{200d}💻👩\u{200d}💻👩\u{200d}💻 pair", 6),
        "👩\u{200d}💻👩\u{200d}💻…"
    );
}

#[test]
fn a_flag_is_cut_whole() {
    assert_eq!(fit("🇧🇷🇧🇷🇧🇷 release", 6), "🇧🇷🇧🇷…");
}

#[test]
fn a_combining_mark_stays_with_its_letter() {
    assert_eq!(fit("cafe\u{301} cafe\u{301}", 5), "cafe\u{301}…");
}

#[test]
fn no_text_is_ever_cut_wider_than_the_room_it_is_given() {
    assert_eq!(overflows(), Vec::<(String, usize, String)>::new());
}

#[test]
fn text_is_measured_in_the_cells_ratatui_fills() {
    assert_eq!(disagreements(), Vec::<(String, usize, usize)>::new());
}

#[test]
fn a_halfwidth_sound_mark_takes_the_column_ratatui_draws_it_in() {
    assert_eq!(columns("\u{ff76}\u{ff9e}"), filled("\u{ff76}\u{ff9e}"));
}

#[test]
fn an_emoji_with_a_variation_selector_takes_two_columns() {
    assert_eq!(columns("⚠\u{fe0f}"), 2);
}

#[test]
fn wrapped_text_breaks_by_columns() {
    assert_eq!(wrap("acme-api", 3), vec!["acm", "e-a", "pi"]);
}

#[test]
fn wrapped_text_breaks_between_words_where_a_word_fits() {
    assert_eq!(
        wrap("Could not start acme-edit", 12),
        vec!["Could not", "start", "acme-edit"]
    );
}

#[test]
fn a_word_wider_than_the_line_starts_its_own_line_and_breaks_by_columns() {
    assert_eq!(wrap("is acme-api", 4), vec!["is", "acme", "-api"]);
}

#[test]
fn wrapped_wide_characters_are_never_split_across_lines() {
    assert_eq!(wrap("東京大阪", 5), vec!["東京", "大阪"]);
}

#[test]
fn wrapped_emoji_sequences_stay_whole() {
    assert_eq!(
        wrap("👩\u{200d}💻🇧🇷⚠\u{fe0f}", 4),
        vec!["👩\u{200d}💻🇧🇷", "⚠\u{fe0f}"]
    );
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

#[test]
fn the_arabic_letter_mark_is_dropped() {
    assert_eq!(clean("acme\u{61c}api"), "acme api");
}

#[test]
fn a_zero_width_space_is_dropped() {
    assert_eq!(clean("acme\u{200b}api"), "acme api");
}

#[test]
fn a_joiner_that_joins_nothing_is_dropped() {
    assert_eq!(clean("\u{200d}acme\u{200d} api\u{200d}"), "acme api");
}

#[test]
fn a_joiner_inside_an_emoji_sequence_is_kept() {
    assert_eq!(clean("pair 👩\u{200d}💻 now"), "pair 👩\u{200d}💻 now");
}

#[test]
fn a_long_text_is_cleaned_only_as_far_as_a_card_could_show() {
    assert_eq!(clean(&"a".repeat(1 << 20)).len(), 512);
}

#[test]
fn a_letter_with_a_thousand_combining_marks_stays_inside_the_budget() {
    let text = format!("acme {}", "a\u{301}".to_string() + &"\u{301}".repeat(1000));

    assert_eq!(clean(&text), "acme");
}

const PIECES: &[&str] = &[
    "a",
    " ",
    "東",
    "⚠\u{fe0f}",
    "✅",
    "👩\u{200d}💻",
    "🇧🇷",
    "e\u{301}",
    "🚀",
    "\u{fe0f}",
    "\u{200d}",
    "\u{ff76}",
    "\u{ff9e}",
    "\u{ff9f}",
    "\u{ff8a}\u{ff9f}",
    "\u{301}",
    "\u{3099}",
];

fn overflows() -> Vec<(String, usize, String)> {
    let mut seed: u64 = 7;
    let mut next = |below: usize| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize % below
    };
    let mut found = Vec::new();
    for _ in 0..4000 {
        let text: String = (0..next(16)).map(|_| PIECES[next(PIECES.len())]).collect();
        let width = next(20);
        let cut = fit(&text, width);
        if cut.width() > width || columns(&cut) > width {
            found.push((text, width, cut));
        }
    }
    found
}

fn disagreements() -> Vec<(String, usize, usize)> {
    let mut seed: u64 = 11;
    let mut next = |below: usize| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 33) as usize % below
    };
    let mut found = Vec::new();
    for _ in 0..4000 {
        let text: String = (0..next(16)).map(|_| PIECES[next(PIECES.len())]).collect();
        let width = next(20);
        let cut = fit(&text, width);
        for shown in [text, cut] {
            if columns(&shown) != filled(&shown) {
                found.push((shown.clone(), columns(&shown), filled(&shown)));
            }
        }
    }
    found
}

fn filled(text: &str) -> usize {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 200, 1));
    let (x, _) = buffer.set_line(0, 0, &Line::from(text), 200);
    usize::from(x)
}
