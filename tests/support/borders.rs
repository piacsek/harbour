use ratatui::symbols::border;
use ratatui::widgets::BorderType;

const STYLES: [BorderType; 12] = [
    BorderType::Plain,
    BorderType::Rounded,
    BorderType::Double,
    BorderType::Thick,
    BorderType::LightDoubleDashed,
    BorderType::HeavyDoubleDashed,
    BorderType::LightTripleDashed,
    BorderType::HeavyTripleDashed,
    BorderType::LightQuadrupleDashed,
    BorderType::HeavyQuadrupleDashed,
    BorderType::QuadrantInside,
    BorderType::QuadrantOutside,
];

pub const BORDERS: [border::Set<'static>; STYLES.len()] = {
    let mut sets = [border::PLAIN; STYLES.len()];
    let mut at = 0;
    while at < STYLES.len() {
        assert!(
            index(STYLES[at]) == at,
            "STYLES lists every BorderType in index order"
        );
        sets[at] = STYLES[at].to_border_set();
        at += 1;
    }
    assert!(index(BorderType::QuadrantOutside) + 1 == STYLES.len());
    sets
};

const fn index(style: BorderType) -> usize {
    match style {
        BorderType::Plain => 0,
        BorderType::Rounded => 1,
        BorderType::Double => 2,
        BorderType::Thick => 3,
        BorderType::LightDoubleDashed => 4,
        BorderType::HeavyDoubleDashed => 5,
        BorderType::LightTripleDashed => 6,
        BorderType::HeavyTripleDashed => 7,
        BorderType::LightQuadrupleDashed => 8,
        BorderType::HeavyQuadrupleDashed => 9,
        BorderType::QuadrantInside => 10,
        BorderType::QuadrantOutside => 11,
    }
}
