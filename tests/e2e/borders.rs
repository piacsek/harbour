use harbour::app::{FOCUSED_BORDER, UNFOCUSED_BORDER};
use ratatui::symbols::border;
use ratatui::widgets::BorderType;

const BORDERS: [border::Set<'static>; 10] = [
    border::PLAIN,
    border::ROUNDED,
    border::DOUBLE,
    border::THICK,
    border::LIGHT_DOUBLE_DASHED,
    border::HEAVY_DOUBLE_DASHED,
    border::LIGHT_TRIPLE_DASHED,
    border::HEAVY_TRIPLE_DASHED,
    border::LIGHT_QUADRUPLE_DASHED,
    border::HEAVY_QUADRUPLE_DASHED,
];

pub(crate) fn corner(c: char) -> bool {
    BORDERS.iter().any(|set| set.top_left.starts_with(c))
}

pub(crate) fn focused_corner(c: char) -> bool {
    FOCUSED_BORDER.to_border_set().top_left.starts_with(c)
}

pub(crate) fn focused_at(screen: &str, column: usize) -> bool {
    drawn_at(screen, column, FOCUSED_BORDER) && !drawn_at(screen, column, UNFOCUSED_BORDER)
}

pub(crate) fn unfocused_at(screen: &str, column: usize) -> bool {
    drawn_at(screen, column, UNFOCUSED_BORDER) && !drawn_at(screen, column, FOCUSED_BORDER)
}

fn drawn_at(screen: &str, column: usize, style: BorderType) -> bool {
    let set = style.to_border_set();
    let mut lines = screen.lines().map(|line| line.chars().nth(column));
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
    top.split(focused_corner)
        .skip(1)
        .any(|after| after.starts_with(&format!(" {title}")))
}

pub(crate) fn pane_corner(screen: &str) -> Option<usize> {
    screen
        .lines()
        .next()?
        .chars()
        .skip(1)
        .position(corner)
        .map(|column| column + 1)
}
