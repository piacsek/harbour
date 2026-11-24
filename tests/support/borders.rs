use ratatui::symbols::border;
use ratatui::widgets::BorderType;

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
