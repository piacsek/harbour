use harbour::app::{FOCUSED_BORDER, UNFOCUSED_BORDER};
use ratatui::symbols::border;
use ratatui::widgets::BorderType;

const BORDERS: [border::Set<'static>; 12] = [
    BorderType::Plain.to_border_set(),
    BorderType::Rounded.to_border_set(),
    BorderType::Double.to_border_set(),
    BorderType::Thick.to_border_set(),
    BorderType::LightDoubleDashed.to_border_set(),
    BorderType::HeavyDoubleDashed.to_border_set(),
    BorderType::LightTripleDashed.to_border_set(),
    BorderType::HeavyTripleDashed.to_border_set(),
    BorderType::LightQuadrupleDashed.to_border_set(),
    BorderType::HeavyQuadrupleDashed.to_border_set(),
    BorderType::QuadrantInside.to_border_set(),
    BorderType::QuadrantOutside.to_border_set(),
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
    let label = format!(" {title}");
    top.char_indices().enumerate().any(|(column, (at, c))| {
        corner(c) && top[at + c.len_utf8()..].starts_with(&label) && focused_at(screen, column)
    })
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
