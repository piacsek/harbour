use ratatui::symbols::border;
use ratatui::widgets::BorderType;
use unicode_width::UnicodeWidthChar;

macro_rules! every_style {
    ($($style:ident),+ $(,)?) => {
        pub const BORDERS: [border::Set<'static>; [$(BorderType::$style),+].len()] =
            [$(BorderType::$style.to_border_set()),+];

        const _: () = match BorderType::Plain {
            $(BorderType::$style)|+ => (),
        };
    };
}

every_style!(
    Plain,
    Rounded,
    Double,
    Thick,
    LightDoubleDashed,
    HeavyDoubleDashed,
    LightTripleDashed,
    HeavyTripleDashed,
    LightQuadrupleDashed,
    HeavyQuadrupleDashed,
    QuadrantInside,
    QuadrantOutside,
);

pub fn corner_column(top: &str) -> Option<usize> {
    columns(top)
        .skip(1)
        .find(|(_, _, c)| BORDERS.iter().any(|set| set.top_left.starts_with(*c)))
        .map(|(column, _, _)| column)
}

pub fn columns(line: &str) -> impl Iterator<Item = (usize, usize, char)> + '_ {
    line.char_indices().scan(0, |start, (at, c)| {
        let column = *start;
        *start += c.width().unwrap_or(0);
        Some((column, at, c))
    })
}
