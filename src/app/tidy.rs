use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

use super::App;
use crate::session;
use crate::tidy::{self, Card};
use crate::transcript::Transcript;

pub(super) const TIDY_EVERY: Duration = Duration::from_secs(60);

impl App {
    pub fn with_tidy_after(mut self, after: Duration) -> App {
        self.tidy_after = Some(after);
        self
    }

    pub fn with_tidy_every(mut self, every: Duration) -> App {
        self.tidy_every = every;
        self
    }

    pub(super) fn tidy(&mut self) {
        let Some(after) = self.tidy_after else {
            return;
        };
        if self
            .tidied_at
            .is_some_and(|at| at.elapsed() < self.tidy_every)
            || self.busy()
        {
            return;
        }
        self.tidied_at = Some(Instant::now());
        let changed = self.conversations_changed();
        let now = SystemTime::now();
        let cards: Vec<Card> = self
            .sessions
            .iter_mut()
            .map(|entry| Card {
                age: match (&entry.session, entry.exited_at) {
                    (Some(_), exited_at) => exited_at.map(|at| at.elapsed()),
                    (None, _) => entry
                        .remembered
                        .as_ref()
                        .and_then(|id| changed.get(id).copied().flatten())
                        .map(|at| now.duration_since(at).unwrap_or_default()),
                },
                running: entry.running(),
            })
            .collect();
        let due = tidy::due(&cards, self.selected, after);
        for index in due.iter().rev() {
            self.remove_tidied(*index);
        }
        if !due.is_empty() {
            self.log_tidy(due.len());
        }
    }

    fn busy(&self) -> bool {
        self.asking.is_some()
            || self.picking.is_some()
            || self.usage_open()
            || self.renaming()
            || self.prefix_waiting()
    }

    fn conversations_changed(&self) -> HashMap<String, Option<SystemTime>> {
        let stopped = self.sessions.iter().any(|entry| entry.session.is_none());
        self.transcripts
            .as_deref()
            .filter(|_| stopped)
            .and_then(Transcript::changed)
            .unwrap_or_default()
    }

    fn remove_tidied(&mut self, index: usize) {
        let mut removed = self.sessions.remove(index);
        session::end_all(removed.sessions(), self.hangup_grace);
        removed.reservation = None;
        self.forget(&removed);
        if index < self.selected {
            self.selected -= 1;
        }
    }
}
