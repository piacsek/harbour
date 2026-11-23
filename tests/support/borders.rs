use ratatui::symbols::border;
use ratatui::widgets::BorderType;

pub const BORDERS: [border::Set<'static>; 12] = [
    set(BorderType::Plain),
    set(BorderType::Rounded),
    set(BorderType::Double),
    set(BorderType::Thick),
    set(BorderType::LightDoubleDashed),
    set(BorderType::HeavyDoubleDashed),
    set(BorderType::LightTripleDashed),
    set(BorderType::HeavyTripleDashed),
    set(BorderType::LightQuadrupleDashed),
    set(BorderType::HeavyQuadrupleDashed),
    set(BorderType::QuadrantInside),
    set(BorderType::QuadrantOutside),
];

const fn set(style: BorderType) -> border::Set<'static> {
    match style {
        BorderType::Plain
        | BorderType::Rounded
        | BorderType::Double
        | BorderType::Thick
        | BorderType::LightDoubleDashed
        | BorderType::HeavyDoubleDashed
        | BorderType::LightTripleDashed
        | BorderType::HeavyTripleDashed
        | BorderType::LightQuadrupleDashed
        | BorderType::HeavyQuadrupleDashed
        | BorderType::QuadrantInside
        | BorderType::QuadrantOutside => style.to_border_set(),
    }
}
