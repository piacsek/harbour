use std::time::SystemTime;

use super::{App, Entry, refusal};
use crate::memory::Memory;
use crate::repo;
use crate::session::Session;

pub(super) const STOPPED: &str = "stopped";
pub(super) const RESUME: &str = "Stopped. Enter resumes it, x forgets it.";

impl App {
    pub fn with_memory(mut self, memory: Memory) -> App {
        for remembered in memory.recall() {
            self.sessions.push(Entry {
                id: 0,
                name: remembered.title,
                session: None,
                started_at: SystemTime::now(),
                record: None,
                place: repo::place(&remembered.directory),
                directory: remembered.directory,
                exited_at: None,
                transcript: None,
                remembered: Some(remembered.session_id),
            });
        }
        self.memory = Some(memory);
        self
    }

    pub(super) fn resume(&mut self) -> bool {
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return false;
        };
        let Some(id) = entry.remembered.clone() else {
            return false;
        };
        if entry
            .session
            .as_mut()
            .is_some_and(|session| session.exit().is_none())
        {
            return false;
        }
        let mut command = self.command.clone();
        command.extend(["--resume".to_string(), id]);
        let started_at = SystemTime::now();
        match Session::start(
            &command,
            &entry.directory,
            self.pane.height,
            self.pane.width,
        ) {
            Ok(session) => {
                self.started += 1;
                entry.id = self.started;
                entry.session = Some(session);
                entry.started_at = started_at;
                entry.record = None;
                entry.exited_at = None;
                entry.transcript = None;
                self.failed = None;
                self.pane_focused = true;
            }
            Err(error) => {
                self.failed = Some(refusal(command.first().map_or("", String::as_str), &error));
            }
        }
        true
    }

    pub(super) fn forget(&self, remembered: Option<String>) {
        if let (Some(memory), Some(id)) = (&self.memory, remembered) {
            memory.forget(&id);
        }
    }
}
