use super::App;
use crate::group;

impl App {
    pub(super) fn regroup(&mut self) {
        let order = group::order(&self.repositories());
        if order.iter().enumerate().any(|(at, from)| at != *from) {
            self.reorder(&order);
            self.keep_positions();
        }
    }

    pub(super) fn move_card(&mut self, down: bool) {
        if let Some(order) = group::step(&self.repositories(), self.selected, down) {
            self.reorder(&order);
            self.keep_positions();
        }
    }

    pub(super) fn members(&self) -> Vec<group::Member> {
        let repositories = self.repositories();
        (0..repositories.len())
            .map(|at| group::member(&repositories, at))
            .collect()
    }

    fn repositories(&self) -> Vec<&str> {
        self.sessions
            .iter()
            .map(|entry| entry.place.repository.as_str())
            .collect()
    }

    fn reorder(&mut self, order: &[usize]) {
        let mut taken: Vec<_> = std::mem::take(&mut self.sessions)
            .into_iter()
            .map(Some)
            .collect();
        self.selected = order
            .iter()
            .position(|from| *from == self.selected)
            .unwrap_or(self.selected);
        self.sessions = order
            .iter()
            .filter_map(|from| taken[*from].take())
            .collect();
    }
}
