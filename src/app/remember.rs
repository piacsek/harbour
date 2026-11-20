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
const DIRECTORY_GONE: &str = "directory gone";
const ELSEWHERE: &str = "open elsewhere";
const GONE: &str = "This conversation is gone: Claude Code no longer has it.";
const HELD: &str = "This conversation is open elsewhere: close it there first.";
const COME_BACK: &str = "Conversations come back stopped.";
const UNRESERVED: &str = "Could not reserve this conversation";
const RECALLED: usize = 100;

pub(super) fn pane(why: &str) -> &'static str {
    match why {
        DIRECTORY_GONE => "Its directory is gone. x forgets it.",
        ELSEWHERE => "Open elsewhere: close it there, then Enter resumes it here.",
        _ => "Stopped. Enter resumes it, x forgets it.",
    }
}

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
        let existing = match self.transcripts.as_deref() {
            None => None,
            Some(projects) => match Transcript::existing(projects) {
                None => return,
                found => found,
            },
        };
        let mut recalled = Vec::new();
        for remembered in memory.recall() {
            if existing
                .as_ref()
                .is_some_and(|ids| !ids.contains(&remembered.session_id))
            {
                memory.forget_stale(&remembered.session_id);
            } else if recalled.len() < RECALLED {
                recalled.push(remembered);
            }
        }
        for remembered in recalled {
            self.sessions.push(Entry {
                id: 0,
                name: remembered.title,
                session: None,
                started_at: SystemTime::now(),
                used_at: SystemTime::now(),
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
                reservation: None,
            });
        }
    }

    pub(super) fn select(&mut self, index: usize) {
        self.selected = index;
        self.clear_refusal();
    }

    fn clear_refusal(&mut self) {
        if self
            .failed
            .as_deref()
            .is_some_and(|failed| failed == GONE || failed == HELD)
        {
            self.failed = None;
        }
    }

    pub(super) fn resume(&mut self) -> bool {
        self.settle_exited();
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
        self.clear_refusal();
        if !self.transcript_exists(&id) {
            let entry = &mut self.sessions[self.selected];
            if entry.running() {
                if let Some(memory) = &self.memory {
                    leave(memory, entry);
                }
                self.failed = Some(GONE.to_string());
                return true;
            }
            self.sessions.remove(self.selected);
            self.selected = self.selected.min(self.sessions.len().saturating_sub(1));
            self.failed = Some(GONE.to_string());
            return true;
        }
        if self.held_ids().contains(&id) {
            self.failed = Some(HELD.to_string());
            return true;
        }
        let reservation = match self.memory.as_ref().map(|memory| memory.reserve(&id)) {
            Some(Ok(None)) => {
                self.failed = Some(HELD.to_string());
                return true;
            }
            Some(Ok(Some(reservation))) => Some(reservation),
            Some(Err(error)) => {
                self.failed = Some(format!("{UNRESERVED}: {error}"));
                return true;
            }
            None => None,
        };
        let mut command = self.command.clone();
        let started_at = SystemTime::now();
        let transcript = self
            .transcripts
            .as_deref()
            .map(|projects| Transcript::new(projects, &id).since(started_at));
        command.extend(["--resume".to_string(), id]);
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
                entry.reservation = reservation;
                entry.on_editor = false;
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
        let held = if self.sessions.iter().any(Entry::ended) {
            self.held_ids()
        } else {
            HashSet::new()
        };
        for entry in &mut self.sessions {
            if entry.ended() {
                entry.why = if !entry.directory.is_dir() {
                    DIRECTORY_GONE
                } else if entry
                    .remembered
                    .as_ref()
                    .is_some_and(|id| held.contains(id))
                {
                    ELSEWHERE
                } else {
                    STOPPED
                };
            }
        }
        if self
            .sessions
            .get(self.selected)
            .is_some_and(|entry| entry.why != ELSEWHERE)
            && self.failed.as_deref() == Some(HELD)
        {
            self.failed = None;
        }
        let mut index = 0;
        while index < self.sessions.len() {
            index = self.keep(index) + 1;
        }
    }

    fn keep(&mut self, mut index: usize) -> usize {
        let Some(memory) = &self.memory else {
            return index;
        };
        let Some(wanted) = wanted(&self.sessions[index], index) else {
            return index;
        };
        let entry = &self.sessions[index];
        if entry.reservation.is_none()
            && entry.remembered.as_deref() == Some(wanted.session_id.as_str())
            && !memory.reserved().contains(&wanted.session_id)
        {
            self.sessions[index].reservation = memory.reserve(&wanted.session_id).ok().flatten();
        }
        let entry = &self.sessions[index];
        if entry.kept.as_ref() == Some(&wanted) && memory.holds(&wanted.session_id) {
            return index;
        }
        let old = entry
            .remembered
            .clone()
            .filter(|old| *old != wanted.session_id);
        if entry.remembered.as_deref() != Some(wanted.session_id.as_str()) {
            let other = self
                .sessions
                .iter()
                .position(|other| other.remembered.as_deref() == Some(wanted.session_id.as_str()));
            if let Some(other) = other {
                if self.sessions[other].running() {
                    leave(memory, &mut self.sessions[index]);
                    return index;
                }
                self.sessions.remove(other);
                if other < index {
                    index -= 1;
                }
                if other == self.selected {
                    self.selected = index;
                } else if other < self.selected {
                    self.selected -= 1;
                }
            }
        }
        let reservation = (self.sessions[index].remembered.as_deref()
            != Some(wanted.session_id.as_str()))
        .then(|| memory.reserve(&wanted.session_id).ok().flatten());
        let kept = match &old {
            Some(old) => memory.replace(old, &wanted),
            None => memory.keep(&wanted),
        };
        if kept.is_ok() {
            let entry = &mut self.sessions[index];
            if let Some(reservation) = reservation {
                entry.reservation = reservation;
            }
            entry.remembered = Some(wanted.session_id.clone());
            entry.kept = Some(wanted);
        }
        index
    }

    pub(super) fn forget(&self, entry: &Entry) {
        let Some((memory, id)) = self.memory.as_ref().zip(entry.remembered.as_ref()) else {
            return;
        };
        let shared = self
            .sessions
            .iter()
            .any(|other| other.remembered.as_ref() == Some(id));
        if !shared && !self.held_ids().contains(id) {
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

    fn held_ids(&self) -> HashSet<String> {
        let mut held = self
            .registry
            .as_deref()
            .map(registry::open_ids)
            .unwrap_or_default();
        if let Some(memory) = &self.memory {
            held.extend(memory.reserved());
        }
        held
    }
}

fn leave(memory: &Memory, entry: &mut Entry) {
    entry.reservation = None;
    entry.kept = None;
    if let Some(old) = entry.remembered.take() {
        memory.forget(&old);
    }
}

fn wanted(entry: &Entry, position: usize) -> Option<Remembered> {
    entry.session.as_ref()?;
    entry.exited_at.is_none().then_some(())?;
    let transcript = entry.transcript.as_ref().filter(|found| found.is_found())?;
    Some(Remembered {
        session_id: transcript.session_id().to_string(),
        directory: entry.directory.clone(),
        title: transcript
            .facts()
            .title
            .clone()
            .unwrap_or_else(|| entry.name.clone()),
        position,
    })
}
