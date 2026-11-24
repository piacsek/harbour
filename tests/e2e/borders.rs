use harbour::app::{FOCUSED_BORDER, UNFOCUSED_BORDER};
use ratatui::widgets::BorderType;
use unicode_width::UnicodeWidthChar;

#[path = "../support/borders.rs"]
mod styles;
use styles::BORDERS;

pub(crate) fn corner(c: char) -> bool {
    BORDERS.iter().any(|set| set.top_left.starts_with(c))
}

pub(crate) fn focused_corner(c: char) -> bool {
    FOCUSED_BORDER.to_border_set().top_left.starts_with(c)
        && !UNFOCUSED_BORDER.to_border_set().top_left.starts_with(c)
}

pub(crate) fn focused_at(screen: &str, column: usize) -> bool {
    drawn_at(screen, column, FOCUSED_BORDER) && !drawn_at(screen, column, UNFOCUSED_BORDER)
}

pub(crate) fn unfocused_at(screen: &str, column: usize) -> bool {
    drawn_at(screen, column, UNFOCUSED_BORDER) && !drawn_at(screen, column, FOCUSED_BORDER)
}

fn drawn_at(screen: &str, column: usize, style: BorderType) -> bool {
    let set = style.to_border_set();
    let mut lines = screen.lines().map(|line| at_column(line, column));
    let top = lines.next().flatten();
    let side = lines.next().flatten();
    top.is_some_and(|c| set.top_left.starts_with(c))
        && side.is_some_and(|c| set.vertical_left.starts_with(c))
}

pub(crate) fn vertical(c: char) -> bool {
    BORDERS.iter().any(|set| set.vertical_left.starts_with(c))
}

pub(crate) fn focused_on(screen: &str, title: &str) -> bool {
    let top = screen.lines().next().unwrap_or_default();
    let label = format!(" {title}");
    columns(top).any(|(column, at, c)| {
        corner(c) && top[at + c.len_utf8()..].starts_with(&label) && focused_at(screen, column)
    })
}

fn at_column(line: &str, column: usize) -> Option<char> {
    columns(line)
        .find(|(start, _, _)| *start == column)
        .map(|(_, _, c)| c)
}

pub(crate) fn pane_corner(screen: &str) -> Option<usize> {
    columns(screen.lines().next()?)
        .skip(1)
        .find(|(_, _, c)| corner(*c))
        .map(|(column, _, _)| column)
}

fn columns(line: &str) -> impl Iterator<Item = (usize, usize, char)> + '_ {
    line.char_indices().scan(0, |start, (at, c)| {
        let column = *start;
        *start += c.width().unwrap_or(0);
        Some((column, at, c))
    })
}
