use harbour::usage;
use ratatui::style::{Color, Modifier};

#[test]
fn the_patch_sweeps_one_cell_a_step_from_the_left() {
    let bar = |step| row(&usage::loading(step, 78, 17, Color::Cyan), 7);

    assert_eq!(bar(1), format!("{}▓█▓▒{}", pad(27), empty(20)));
    assert_eq!(bar(5), format!("{}{}▒▓█▓▒{}", pad(27), empty(3), empty(16)));
    assert_eq!(bar(9), format!("{}{}▒▓█▓▒{}", pad(27), empty(7), empty(12)));
    assert_eq!(
        bar(13),
        format!("{}{}▒▓█▓▒{}", pad(27), empty(11), empty(8))
    );
    assert_eq!(
        bar(17),
        format!("{}{}▒▓█▓▒{}", pad(27), empty(15), empty(4))
    );
    assert_eq!(bar(21), format!("{}{}▒▓█▓▒", pad(27), empty(19)));
}

#[test]
fn the_patch_starts_again_after_the_last_cell() {
    let lines = usage::loading(24, 78, 17, Color::Cyan);

    assert_eq!(row(&lines, 7), format!("{}█▓▒{}", pad(27), empty(21)));
}

#[test]
fn the_patch_leaves_the_bar_at_its_right_end() {
    let lines = usage::loading(23, 78, 17, Color::Cyan);

    assert_eq!(row(&lines, 7), format!("{}{}▒▓█", pad(27), empty(21)));
}

#[test]
fn the_label_is_centred_one_blank_row_under_the_bar() {
    let lines = usage::loading(0, 78, 17, Color::Cyan);

    assert_eq!(row(&lines, 8), "");
    assert_eq!(row(&lines, 9), format!("{}reading usage…", pad(32)));
}

#[test]
fn the_bar_and_the_label_are_centred_in_the_rows() {
    let lines = usage::loading(0, 78, 17, Color::Cyan);

    assert_eq!(lines.len(), 17);
    assert_eq!(row(&lines, 6), "");
    assert_eq!(row(&lines, 10), "");
    assert_eq!(row(&lines, 15), "");
}

#[test]
fn the_last_row_is_the_hint() {
    let lines = usage::loading(0, 78, 28, Color::Cyan);

    assert_eq!(lines.len(), 28);
    assert_eq!(row(&lines, 27), "  Esc close");
    assert_eq!(row(&lines, 12), format!("{}█▓▒{}", pad(27), empty(21)));
}

#[test]
fn four_rows_keep_the_hint_under_the_label() {
    let lines = usage::loading(0, 78, 4, Color::Cyan);

    assert_eq!(lines.len(), 4);
    assert_eq!(row(&lines, 0), format!("{}█▓▒{}", pad(27), empty(21)));
    assert_eq!(row(&lines, 1), "");
    assert_eq!(row(&lines, 2), format!("{}reading usage…", pad(32)));
    assert_eq!(row(&lines, 3), "  Esc close");
}

#[test]
fn three_rows_drop_the_hint_first() {
    let lines = usage::loading(0, 78, 3, Color::Cyan);

    assert_eq!(lines.len(), 3);
    assert_eq!(row(&lines, 0), format!("{}█▓▒{}", pad(27), empty(21)));
    assert_eq!(row(&lines, 1), "");
    assert_eq!(row(&lines, 2), format!("{}reading usage…", pad(32)));
}

#[test]
fn two_rows_show_the_label_alone() {
    let lines = usage::loading(0, 78, 2, Color::Cyan);

    assert_eq!(lines.len(), 2);
    assert_eq!(row(&lines, 0), format!("{}reading usage…", pad(32)));
    assert_eq!(row(&lines, 1), "");
}

#[test]
fn one_row_shows_the_label_alone() {
    let lines = usage::loading(0, 78, 1, Color::Cyan);

    assert_eq!(lines.len(), 1);
    assert_eq!(row(&lines, 0), format!("{}reading usage…", pad(32)));
}

#[test]
fn no_rows_show_nothing() {
    let lines = usage::loading(0, 78, 0, Color::Cyan);

    assert_eq!(lines.len(), 0);
}

#[test]
fn a_narrower_popup_shrinks_the_bar_as_the_limit_bars_shrink() {
    let lines = usage::loading(1, 60, 17, Color::Cyan);

    assert_eq!(row(&lines, 7), format!("{}▓█▓▒{}", pad(27), empty(2)));
    assert_eq!(row(&lines, 9), format!("{}reading usage…", pad(23)));
}

#[test]
fn the_bar_never_has_fewer_than_four_cells() {
    let lines = usage::loading(2, 20, 17, Color::Cyan);

    assert_eq!(row(&lines, 7), format!("{}▒▓█▓", pad(8)));
}

#[test]
fn a_label_wider_than_the_popup_is_cut() {
    let lines = usage::loading(0, 10, 17, Color::Cyan);

    assert_eq!(row(&lines, 9), "reading u…");
}

#[test]
fn the_patch_takes_the_accent_and_the_rest_is_dim() {
    let lines = usage::loading(1, 78, 17, Color::Magenta);

    let bar = &lines[7];
    let bright = bar.spans.iter().find(|span| span.content == "█").unwrap();
    let edge = bar.spans.iter().find(|span| span.content == "▒").unwrap();
    let rest = bar.spans.iter().find(|span| span.content == "░").unwrap();
    assert_eq!(bright.style.fg, Some(Color::Magenta));
    assert_eq!(edge.style.fg, Some(Color::Magenta));
    assert!(rest.style.add_modifier.contains(Modifier::DIM));
}

fn row(lines: &[ratatui::text::Line<'static>], at: usize) -> String {
    let text: String = lines[at]
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect();
    text.trim_end().to_string()
}

fn pad(columns: usize) -> String {
    " ".repeat(columns)
}

fn empty(cells: usize) -> String {
    "░".repeat(cells)
}
