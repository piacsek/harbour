use std::path::Path;
use std::time::SystemTime;

use super::{App, Entry, folder_name, refusal, remember, user_home};
use crate::repo;
use crate::session::{Role, Session};
use crate::transcript::{self, Transcript};

impl App {
    pub(super) fn start(&mut self) {
        self.start_with(&self.command.clone());
    }

    pub(super) fn start_with(&mut self, command: &[String]) {
        self.start_in(command, &self.home.clone());
    }

    pub(super) fn start_in(&mut self, command: &[String], directory: &Path) {
        let started_at = SystemTime::now();
        let transcript = self
            .transcripts
            .as_deref()
            .filter(|_| self.naming)
            .and_then(|projects| {
                let id = transcript::new_session_id().ok()?;
                Some(Transcript::new(projects, &id).since(started_at))
            });
        let mut command = command.to_vec();
        if let Some(transcript) = &transcript {
            command.extend([
                "--session-id".to_string(),
                transcript.session_id().to_string(),
            ]);
        }
        match Session::start(
            self.transport,
            Role::Agent,
            &command,
            directory,
            self.pane.height,
            self.pane.width,
        ) {
            Ok(session) => {
                self.failed = None;
                self.started += 1;
                self.named += 1;
                self.sessions.push(Entry {
                    id: self.started,
                    name: format!("{} {}", folder_name(directory), self.named),
                    given: None,
                    session: Some(session),
                    started_at,
                    used_at: started_at,
                    record: None,
                    place: repo::place(directory, user_home().as_deref()),
                    exited_at: None,
                    exited_wall: None,
                    transcript,
                    editors: Vec::new(),
                    editor: None,
                    on_editor: false,
                    seen: directory.to_path_buf(),
                    followed: false,
                    directory: directory.to_path_buf(),
                    remembered: None,
                    kept: None,
                    why: remember::STOPPED,
                    reservation: None,
                });
                self.selected = self.sessions.len() - 1;
                self.regroup();
            }
            Err(error) => {
                self.failed = Some(refusal(command.first().map_or("", String::as_str), &error))
            }
        }
    }
}
