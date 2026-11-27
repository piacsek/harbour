use super::{App, Entry};

pub(super) const NO_PREVIOUS: &str = "No previous session";
pub(super) const PREVIOUS_ENDED: &str = "The previous session has ended";

#[derive(Default)]
pub(super) struct Recent {
    current: Option<Card>,
    previous: Option<Card>,
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
        let current = self.index_of(self.recent.current.as_ref());
        let previous = match current {
            Some(index) if index != self.selected => Some(index),
            _ => self.index_of(self.recent.previous.as_ref()),
        };
        self.recent.previous = previous.map(|index| Card::of(&self.sessions[index]));
        self.recent.current = self.sessions.get(self.selected).map(Card::of);
    }

    pub(super) fn switch_to_previous(&mut self) {
        match self
            .index_of(self.recent.previous.as_ref())
            .filter(|index| *index != self.selected)
        {
            None => self.failed = Some(NO_PREVIOUS.to_string()),
            Some(index) if self.sessions[index].ended() => {
                self.failed = Some(PREVIOUS_ENDED.to_string());
            }
            Some(index) => self.jump(index),
        }
    }

    fn index_of(&self, card: Option<&Card>) -> Option<usize> {
        let card = card?;
        self.sessions.iter().position(|entry| card.is(entry))
    }
}
