use std::collections::HashSet;
use std::time::SystemTime;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Clear, Paragraph, Widget};

use super::{App, Entry, Question, refusal};
use crate::memory::{Memory, Remembered};
use crate::registry;
use crate::repo;
use crate::session::Session;
use crate::text::fit;
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
                place: repo::place(&remembered.directory),
                directory: remembered.directory,
                exited_at: None,
                transcript: None,
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

    pub(super) fn follow(&mut self) {
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

    pub(super) fn note_quit(&self, area: Rect, buffer: &mut Buffer) {
        if self.memory.is_none() || !matches!(self.asking, Some(Question::Quit)) || area.is_empty()
        {
            return;
        }
        let row = Rect {
            y: area.bottom() - 1,
            height: 1,
            ..area
        };
        Clear.render(row, buffer);
        Paragraph::new(fit(COME_BACK, usize::from(row.width)))
            .style(Style::default().add_modifier(Modifier::DIM))
            .render(row, buffer);
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
