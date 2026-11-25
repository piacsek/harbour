use std::collections::HashSet;
use std::fs;
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

    pub fn with_tidy_clock(mut self, clock: Box<dyn Fn() -> SystemTime>) -> App {
        self.tidy_clock = clock;
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
        let now = (self.tidy_clock)();
        let mut cards = Vec::with_capacity(self.sessions.len());
        for index in 0..self.sessions.len() {
            let changed = match &self.sessions[index].session {
                Some(_) => self.sessions[index].exited_wall,
                None => self.conversation_changed(index),
            };
            cards.push(Card {
                age: changed.map(|at| now.duration_since(at).unwrap_or_default()),
                running: self.sessions[index].running(),
            });
        }
        let due = tidy::due(&cards, self.selected, after);
        if due.is_empty() {
            return;
        }
        let held = self.held_ids();
        for index in due.iter().rev() {
            self.remove_tidied(*index, &held);
        }
        self.keep_positions();
        self.log_tidy(due.len());
    }

    fn busy(&self) -> bool {
        self.asking.is_some()
            || self.picking.is_some()
            || self.usage_open()
            || self.renaming()
            || self.prefix_waiting()
    }

    fn conversation_changed(&mut self, index: usize) -> Option<SystemTime> {
        let id = self.sessions[index].remembered.clone()?;
        let projects = self.transcripts.as_deref()?;
        let changed = |path: &std::path::Path| {
            fs::metadata(path)
                .ok()
                .filter(fs::Metadata::is_file)
                .and_then(|meta| meta.modified().ok())
        };
        if let Some(at) = self
            .transcript_paths
            .get(&id)
            .and_then(|path| changed(path))
        {
            return Some(at);
        }
        let path = Transcript::located(projects, &id)?;
        let at = changed(&path);
        self.transcript_paths.insert(id, path);
        at
    }

    fn remove_tidied(&mut self, index: usize, held: &HashSet<String>) {
        let mut removed = self.sessions.remove(index);
        session::end_all(removed.sessions(), self.hangup_grace);
        removed.reservation = None;
        self.forget_unless(&removed, held);
        if let Some(id) = &removed.remembered {
            self.transcript_paths.remove(id);
        }
        if index < self.selected {
            self.selected -= 1;
        }
    }
}
