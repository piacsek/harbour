use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::{App, read_entry, refusal, user_home};
use crate::editor::{self, Editor, Program};
use crate::handoff::{Reply, Verb};

const WAITING: &str = "harbour is waiting for an answer; try again after it";

impl App {
    pub fn hand(&mut self, verb: Verb, file: &Path) -> Reply {
        if verb == Verb::Open && (self.picking.is_some() || self.asking.is_some()) {
            return Reply::Refused(WAITING.to_string());
        }
        match (verb, self.holder(file)) {
            (Verb::Ask, Some(holder)) => Reply::Holds(self.sessions[holder].used_at),
            (Verb::Ask, None) => Reply::None,
            (Verb::Open, Some(holder)) => self.open_in(holder),
            (Verb::Open, None) => {
                Reply::Refused(format!("no harbour session holds {}", file.display()))
            }
        }
    }

    pub(super) fn touch(&mut self) {
        if let Some(entry) = self.sessions.get_mut(self.selected) {
            entry.used_at = SystemTime::now();
        }
    }

    pub(super) fn open_editor(&mut self) {
        if self.editor.is_none() || self.sessions.get(self.selected).is_none() {
            return;
        }
        if let Err(refused) = self.show_editor(self.selected) {
            self.failed = Some(refused);
        }
    }

    fn holder(&self, file: &Path) -> Option<usize> {
        let home = user_home();
        let holds = |index: &usize| {
            let root = editor::root(&self.sessions[*index].seen, home.as_deref());
            file.starts_with(canonical(&root))
        };
        if self.selected < self.sessions.len() && holds(&self.selected) {
            return Some(self.selected);
        }
        (0..self.sessions.len())
            .filter(holds)
            .max_by_key(|index| self.sessions[*index].used_at)
    }

    fn open_in(&mut self, holder: usize) -> Reply {
        if let Some(program) = self.editor.as_ref().filter(|program| !program.is_neovim()) {
            return Reply::Refused(format!(
                "a file can only be handed to Neovim, and HARBOUR_EDITOR is {}",
                program.name()
            ));
        }
        self.helping = false;
        match self.show_editor(holder) {
            Ok(()) => {
                self.selected = holder;
                self.pane_focused = true;
                self.failed = None;
                Reply::Opened
            }
            Err(refused) => {
                self.failed = Some(refused.clone());
                Reply::Refused(refused)
            }
        }
    }

    fn show_editor(&mut self, index: usize) -> Result<(), String> {
        let Some(program) = &self.editor else {
            return Ok(());
        };
        let entry = &mut self.sessions[index];
        read_entry(
            entry,
            self.registry.as_deref(),
            self.transcripts.as_deref(),
            true,
        );
        let directory = editor::root(&entry.seen, user_home().as_deref());
        let shown = match entry
            .editors
            .iter()
            .position(|editor| editor.directory == directory)
        {
            Some(running) => running,
            None => {
                self.started += 1;
                entry.editors.push(started(
                    program,
                    &directory,
                    &self.temporary,
                    self.started,
                    self.pane,
                )?);
                self.failed = None;
                entry.editors.len() - 1
            }
        };
        entry.editor = Some(shown);
        entry.on_editor = true;
        Ok(())
    }
}

fn started(
    program: &Program,
    directory: &Path,
    temporary: &Path,
    id: usize,
    pane: ratatui::layout::Rect,
) -> Result<Editor, String> {
    Editor::start(program, directory, temporary, id, pane.height, pane.width)
        .map_err(|error| refusal(program.name(), &error))
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
