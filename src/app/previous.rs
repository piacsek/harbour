use super::{App, Entry};

pub(super) const NO_PREVIOUS: &str = "No previous session";
pub(super) const PREVIOUS_ENDED: &str = "The previous session has ended";

const REMEMBERED: usize = 16;

#[derive(Default)]
pub(super) struct Recent {
    cards: Vec<Card>,
}

struct Card {
    id: usize,
    remembered: Option<String>,
}

impl Card {
    fn of(entry: &Entry) -> Card {
        Card {
            id: entry.id,
            remembered: entry.remembered.clone(),
        }
    }

    fn is(&self, entry: &Entry) -> bool {
        (self.id != 0 && self.id == entry.id)
            || (self.remembered.is_some() && self.remembered == entry.remembered)
    }
}

impl App {
    pub(super) fn note_selection(&mut self) {
        let mut indices: Vec<usize> = Vec::new();
        if self.selected < self.sessions.len() {
            indices.push(self.selected);
        }
        for index in self
            .recent
            .cards
            .iter()
            .filter_map(|card| self.index_of(card))
        {
            if !indices.contains(&index) {
                indices.push(index);
            }
        }
        indices.truncate(REMEMBERED);
        self.recent.cards = indices
            .into_iter()
            .map(|index| Card::of(&self.sessions[index]))
            .collect();
    }

    pub(super) fn switch_to_previous(&mut self) {
        match self
            .recent
            .cards
            .iter()
            .filter_map(|card| self.index_of(card))
            .find(|index| *index != self.selected)
        {
            None => self.failed = Some(NO_PREVIOUS.to_string()),
            Some(index) if self.sessions[index].ended() => {
                self.failed = Some(PREVIOUS_ENDED.to_string());
            }
            Some(index) => self.jump(index),
        }
    }

    fn index_of(&self, card: &Card) -> Option<usize> {
        self.sessions.iter().position(|entry| card.is(entry))
    }
}
