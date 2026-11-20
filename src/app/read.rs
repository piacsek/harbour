use std::path::Path;
use std::time::Instant;

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
                false,
            );
        }
        self.remember_all();
    }
}

pub(super) fn read_entry(
    entry: &mut Entry,
    registry: Option<&Path>,
    transcripts: Option<&Path>,
    searching: bool,
) {
    let last = entry.record.take();
    let Some(session) = entry.session.as_mut() else {
        return;
    };
    if session.exit().is_some() {
        return;
    }
    let pid = session.pid();
    entry.record = registry.and_then(|dir| registry::read(dir, pid?, entry.started_at, last));
    follow(entry, searching);
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
        follow(entry, searching);
    }
    let cwd = entry.record.as_ref().and_then(|record| record.cwd.clone());
    if let Some(cwd) = cwd
        && !entry.followed
    {
        entry.seen = cwd;
    }
    entry.place = repo::place(&entry.seen, user_home().as_deref());
}

fn follow(entry: &mut Entry, searching: bool) {
    let Some(transcript) = &mut entry.transcript else {
        return;
    };
    if searching {
        transcript.read_now();
    } else {
        transcript.read();
    }
    if let Some(directory) = transcript.facts().directory.clone() {
        entry.seen = directory;
        entry.followed = true;
    }
}
