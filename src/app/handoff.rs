use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

use super::read::{self, Reading};
use super::{App, refusal, user_home};
use crate::editor::{self, Editor, Program};
use crate::handoff::{Listener, Reply, Verb};

const WAITING: &str = "harbour is waiting for an answer; try again after it";
const LATE: &str = "harbour took too long";
const SERVED_PER_FRAME: usize = 4;

impl App {
    pub fn hand(&mut self, verb: Verb, file: &Path, deadline: Instant) -> Reply {
        if self.picking.is_some() || self.asking.is_some() {
            return match verb {
                Verb::Ask => Reply::None,
                Verb::Open => Reply::Refused(WAITING.to_string()),
            };
        }
        if verb == Verb::Open {
            self.refresh_live();
        }
        match (verb, self.holder(file)) {
            (Verb::Ask, Some((holder, _))) => Reply::Holds(self.sessions[holder].used_at),
            (Verb::Ask, None) => Reply::None,
            (Verb::Open, Some((holder, root))) => self.open_in(holder, &root, file, deadline),
            (Verb::Open, None) => {
                Reply::Refused(format!("no harbour session holds {}", file.display()))
            }
        }
    }

    pub(super) fn serve(&mut self) {
        for _ in 0..SERVED_PER_FRAME {
            let Some(request) = self.handoff.as_mut().and_then(Listener::incoming) else {
                return;
            };
            if Instant::now() < request.deadline {
                let reply = self.hand(request.verb, &request.file, request.deadline);
                request.answer(reply);
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
        let root = self.fresh_root(self.selected);
        if let Err(refused) = self.show_editor(self.selected, &root, None) {
            self.failed = Some(refused);
        }
    }

    fn refresh_live(&mut self) {
        for index in 0..self.sessions.len() {
            if self.sessions[index].session.is_some() {
                self.fresh_root(index);
            }
        }
    }

    fn fresh_root(&mut self, index: usize) -> PathBuf {
        let entry = &mut self.sessions[index];
        read::read_entry(
            entry,
            self.registry.as_deref(),
            self.transcripts.as_deref(),
            Reading::Now,
        );
        editor::root(&entry.seen, user_home().as_deref())
    }

    fn holder(&self, file: &Path) -> Option<(usize, PathBuf)> {
        let home = user_home();
        let root = |index: usize| editor::root(&self.sessions[index].seen, home.as_deref());
        let holds = |index: &usize| {
            self.sessions[*index].session.is_some() && file.starts_with(canonical(&root(*index)))
        };
        let chosen = if self.selected < self.sessions.len() && holds(&self.selected) {
            Some(self.selected)
        } else {
            (0..self.sessions.len())
                .filter(holds)
                .max_by_key(|index| self.sessions[*index].used_at)
        };
        chosen.map(|index| (index, root(index)))
    }

    fn open_in(&mut self, holder: usize, root: &Path, file: &Path, deadline: Instant) -> Reply {
        if let Some(program) = self.editor.as_ref().filter(|program| !program.is_neovim()) {
            return Reply::Refused(format!(
                "a file can only be handed to Neovim, and HARBOUR_EDITOR is {}",
                program.name()
            ));
        }
        match self.show_editor(holder, root, Some((file, deadline))) {
            Ok(()) => {
                self.helping = false;
                self.selected = holder;
                self.pane_focused = true;
                self.failed = None;
                Reply::Opened
            }
            Err(refused) => Reply::Refused(refused),
        }
    }

    fn show_editor(
        &mut self,
        index: usize,
        directory: &Path,
        file: Option<(&Path, Instant)>,
    ) -> Result<(), String> {
        let Some(program) = &self.editor else {
            return Ok(());
        };
        if file.is_some_and(|(_, deadline)| Instant::now() >= deadline) {
            return Err(LATE.to_string());
        }
        let entry = &mut self.sessions[index];
        let shown = match entry
            .editors
            .iter()
            .position(|editor| editor.directory == directory)
        {
            Some(running) => {
                if let Some((file, deadline)) = file {
                    entry.editors[running].open(file, deadline)?;
                }
                running
            }
            None => {
                self.started += 1;
                entry.editors.push(started(
                    program,
                    directory,
                    &self.temporary,
                    self.started,
                    self.pane,
                    file.map(|(file, _)| file),
                )?);
                if let Some((file, deadline)) = file {
                    entry.editors[entry.editors.len() - 1].open(file, deadline)?;
                }
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
    file: Option<&Path>,
) -> Result<Editor, String> {
    Editor::start(
        program,
        directory,
        temporary,
        id,
        pane.height,
        pane.width,
        file,
    )
    .map_err(|error| refusal(program.name(), &error))
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
