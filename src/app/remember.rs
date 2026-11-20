use std::collections::HashSet;
use std::time::SystemTime;

use ratatui::style::Stylize;
use ratatui::text::Line;

use super::{App, Entry, refusal, user_home};
use crate::memory::{Memory, Remembered};
use crate::registry;
use crate::repo;
use crate::session::Session;
use crate::text;
use crate::transcript::Transcript;

pub(super) const STOPPED: &str = "stopped";
pub(super) const RESUME: &str = "Stopped. Enter resumes it, x forgets it.";
const DIRECTORY_GONE: &str = "directory gone";
const ELSEWHERE: &str = "open elsewhere";
const GONE: &str = "This conversation is gone: Claude Code no longer has it.";
const HELD: &str = "This conversation is open elsewhere: close it there first.";
const COME_BACK: &str = "Conversations come back stopped.";

impl App {
    pub fn with_memory(mut self, memory: Memory) -> App {
        self.memory = Some(memory);
        self
    }

    fn recall(&mut self) {
        let Some(memory) = &self.memory else {
            return;
        };
        self.recalled = true;
        for remembered in memory.recall() {
            if !self.transcript_exists(&remembered.session_id) {
                continue;
            }
            self.sessions.push(Entry {
                id: 0,
                name: remembered.title,
                session: None,
                started_at: SystemTime::now(),
                record: None,
                place: repo::place(&remembered.directory, user_home().as_deref()),
                exited_at: None,
                transcript: None,
                editors: Vec::new(),
                editor: None,
                on_editor: false,
                seen: remembered.directory.clone(),
                followed: false,
                directory: remembered.directory,
                remembered: Some(remembered.session_id),
                kept: None,
                why: STOPPED,
            });
        }
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
        if !self.transcript_exists(&id) {
            self.sessions.remove(self.selected);
            self.selected = self.selected.min(self.sessions.len().saturating_sub(1));
            self.failed = Some(GONE.to_string());
            return true;
        }
        if self.open_ids().contains(&id) {
            self.failed = Some(HELD.to_string());
            return true;
        }
        let mut command = self.command.clone();
        let transcript = self
            .transcripts
            .as_deref()
            .map(|projects| Transcript::new(projects, &id));
        command.extend(["--resume".to_string(), id]);
        let started_at = SystemTime::now();
        let entry = &mut self.sessions[self.selected];
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
                entry.transcript = transcript;
                entry.seen = entry.directory.clone();
                entry.followed = false;
                self.failed = None;
                self.pane_focused = true;
            }
            Err(error) => {
                self.failed = Some(refusal(command.first().map_or("", String::as_str), &error));
            }
        }
        true
    }

    pub(super) fn remember_all(&mut self) {
        if !self.recalled {
            self.recall();
        }
        let open = if self.sessions.iter().any(|entry| entry.session.is_none()) {
            self.open_ids()
        } else {
            HashSet::new()
        };
        let Some(memory) = &self.memory else {
            return;
        };
        for (position, entry) in self.sessions.iter_mut().enumerate() {
            let Some(session) = entry.session.as_mut() else {
                entry.why = if !entry.directory.is_dir() {
                    DIRECTORY_GONE
                } else if entry
                    .remembered
                    .as_ref()
                    .is_some_and(|id| open.contains(id))
                {
                    ELSEWHERE
                } else {
                    STOPPED
                };
                continue;
            };
            if session.exit().is_some() {
                continue;
            }
            let Some(transcript) = entry.transcript.as_ref().filter(|found| found.is_found())
            else {
                continue;
            };
            let wanted = Remembered {
                session_id: transcript.session_id().to_string(),
                directory: entry.directory.clone(),
                title: transcript
                    .facts()
                    .title
                    .clone()
                    .unwrap_or_else(|| entry.name.clone()),
                position,
            };
            if entry.kept.as_ref() == Some(&wanted) || memory.keep(&wanted).is_err() {
                continue;
            }
            if let Some(old) = entry
                .remembered
                .as_ref()
                .filter(|old| **old != wanted.session_id)
            {
                memory.forget(old);
            }
            entry.remembered = Some(wanted.session_id.clone());
            entry.kept = Some(wanted);
        }
    }

    pub(super) fn forget(&self, entry: &Entry) {
        if let (Some(memory), Some(id)) = (&self.memory, &entry.remembered)
            && !self.open_ids().contains(id)
        {
            memory.forget(id);
        }
    }

    pub(super) fn has_process(&self) -> bool {
        self.sessions
            .get(self.selected)
            .is_some_and(|entry| entry.session.is_some())
    }

    pub(super) fn quit_note(&self, width: u16) -> Vec<Line<'static>> {
        if self.memory.is_none() {
            return Vec::new();
        }
        text::wrap(COME_BACK, usize::from(width))
            .into_iter()
            .map(|line| Line::from(line).dim())
            .collect()
    }

    fn transcript_exists(&self, id: &str) -> bool {
        self.transcripts
            .as_deref()
            .is_none_or(|projects| Transcript::exists(projects, id))
    }

    fn open_ids(&self) -> HashSet<String> {
        self.registry
            .as_deref()
            .map(registry::open_ids)
            .unwrap_or_default()
    }
}
