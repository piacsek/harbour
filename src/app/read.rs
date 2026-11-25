use std::path::Path;
use std::time::{Instant, SystemTime};

use super::{App, Entry, READ_EVERY, user_home};
use crate::registry;
use crate::repo;
use crate::transcript::Transcript;

impl App {
    pub(super) fn read_files(&mut self) {
        if self.read_at.is_some_and(|at| at.elapsed() < READ_EVERY) {
            return;
        }
        self.read_at = Some(Instant::now());
        for entry in &mut self.sessions {
            read_entry(
                entry,
                self.registry.as_deref(),
                self.transcripts.as_deref(),
                Reading::Timed,
            );
        }
        self.regroup();
        self.remember_all();
    }

    pub(super) fn settle_exited(&mut self) {
        let mut settling = Vec::new();
        for entry in &mut self.sessions {
            let exited = entry.exited_at.is_none()
                && entry
                    .session
                    .as_mut()
                    .is_some_and(|session| session.exit().is_some());
            if exited {
                read_entry(
                    entry,
                    self.registry.as_deref(),
                    self.transcripts.as_deref(),
                    Reading::Last,
                );
                settling.push(entry.id);
            }
        }
        if settling.is_empty() {
            return;
        }
        self.remember_all();
        for entry in &mut self.sessions {
            if entry.session.is_some() && settling.contains(&entry.id) {
                entry.exited_at = Some(Instant::now());
                entry.exited_wall = Some(SystemTime::now());
                entry.reservation = None;
            }
        }
    }

    pub(super) fn settle_all(&mut self) {
        for entry in &mut self.sessions {
            if entry.exited_at.is_none() {
                read_entry(
                    entry,
                    self.registry.as_deref(),
                    self.transcripts.as_deref(),
                    Reading::Last,
                );
            }
        }
        self.regroup();
        self.remember_all();
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Reading {
    Timed,
    Now,
    Last,
}

pub(super) fn read_entry(
    entry: &mut Entry,
    registry: Option<&Path>,
    transcripts: Option<&Path>,
    reading: Reading,
) {
    let Some(session) = entry.session.as_mut() else {
        return;
    };
    if reading != Reading::Last && session.exit().is_some() {
        return;
    }
    let last = entry.record.take();
    let pid = session.pid();
    entry.record = registry.and_then(|dir| registry::read(dir, pid?, entry.started_at, last));
    follow(entry, reading);
    let id = entry
        .record
        .as_ref()
        .and_then(|record| record.session_id.clone());
    if let (Some(id), Some(projects)) = (id, transcripts)
        && entry
            .transcript
            .as_ref()
            .is_none_or(|transcript| transcript.session_id() != id)
    {
        entry.transcript = Some(Transcript::new(projects, &id).since(entry.started_at));
        follow(entry, reading);
    }
    let cwd = entry.record.as_ref().and_then(|record| record.cwd.clone());
    if let Some(cwd) = cwd
        && !entry.followed
    {
        entry.seen = cwd;
    }
    entry.place = repo::place(&entry.seen, user_home().as_deref());
}

fn follow(entry: &mut Entry, reading: Reading) {
    let Some(transcript) = &mut entry.transcript else {
        return;
    };
    if reading != Reading::Timed {
        transcript.read_now();
    } else {
        transcript.read();
    }
    if let Some(directory) = transcript.facts().directory.clone() {
        entry.seen = directory;
        entry.followed = true;
    }
}
