use std::time::Duration;

pub struct Card {
    pub age: Option<Duration>,
    pub running: bool,
}

pub fn due(cards: &[Card], selected: usize, after: Duration) -> Vec<usize> {
    cards
        .iter()
        .enumerate()
        .filter(|(index, card)| {
            *index != selected && !card.running && card.age.is_some_and(|age| age >= after)
        })
        .map(|(index, _)| index)
        .collect()
}
