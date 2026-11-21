use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use super::read::{self, Reading};
use super::{App, refusal, user_home};
use crate::editor::{self, Editor, Program, Unopened};
use crate::handoff::{Claim, Listener, Reply, Verb};

const WAITING: &str = "harbour is waiting for an answer; try again after it";
const BUSY: &str = "harbour was busy and did not get to it in time";
const SILENT: &str = "Neovim did not answer";
const CLOSED: &str = "Neovim did not start in time; harbour closed it";
const EXITED: &str = "Neovim exited before it took the file";
const OPEN_PATIENCE: Duration = Duration::from_secs(1);
const REPLYING: Duration = Duration::from_millis(250);
const SERVED_PER_FRAME: usize = 4;

impl App {
    pub fn hand(&mut self, verb: Verb, file: &Path, deadline: Instant) -> Reply {
        if self.picking.is_some() || self.asking.is_some() || self.usage_open() {
            return match verb {
                Verb::Ask => Reply::None,
                Verb::Open => Reply::Refused(WAITING.to_string()),
            };
        }
        let neovim = self.editor.as_ref().is_none_or(Program::is_neovim);
        if verb == Verb::Ask && !neovim {
            return Reply::None;
        }
        if verb == Verb::Open {
            self.refresh_live();
        }
        match (verb, self.holder(file)) {
            (Verb::Ask, Some((holder, root))) => Reply::Holds(Claim {
                depth: canonical(&root).components().count(),
                selected: holder == self.selected,
                used: self.sessions[holder].used_at,
            }),
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
            let verb = request.verb;
            let file = request.file.clone();
            if Instant::now() < request.deadline {
                let asked = Instant::now();
                let reply = self.hand(verb, &file, request.deadline);
                self.log_served(verb, &file, &reply, asked);
                request.answer(reply);
            } else {
                self.log_expired(verb, &file);
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
        if let Err(Reply::Refused(refused)) = self.show_editor(self.selected, &root, None) {
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
        (0..self.sessions.len())
            .filter(holds)
            .max_by_key(|index| {
                (
                    canonical(&root(*index)).components().count(),
                    *index == self.selected,
                    self.sessions[*index].used_at,
                )
            })
            .map(|index| (index, root(index)))
    }

    fn open_in(&mut self, holder: usize, root: &Path, file: &Path, deadline: Instant) -> Reply {
        if let Some(program) = self.editor.as_ref().filter(|program| !program.is_neovim()) {
            return Reply::Refused(format!(
                "a file can only be handed to Neovim, and the editor is {}",
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
            Err(reply) => reply,
        }
    }

    fn show_editor(
        &mut self,
        index: usize,
        directory: &Path,
        file: Option<(&Path, Instant)>,
    ) -> Result<(), Reply> {
        let Some(program) = &self.editor else {
            return Ok(());
        };
        let file = match file {
            None => None,
            Some((file, deadline)) => match deadline
                .checked_sub(REPLYING)
                .filter(|last| Instant::now() < *last)
            {
                Some(last) => Some((file, last)),
                None => return Err(Reply::Refused(BUSY.to_string())),
            },
        };
        let log = self.logging.log().clone();
        let entry = &mut self.sessions[index];
        let shown = match entry
            .editors
            .iter()
            .position(|editor| editor.directory == directory)
        {
            Some(running) => {
                if let Some((file, last)) = file {
                    let patient = Instant::now() + OPEN_PATIENCE;
                    let busy = if last < patient { BUSY } else { SILENT };
                    entry.editors[running]
                        .open(file, last.min(patient))
                        .map_err(|unopened| refused(unopened, busy))?;
                }
                running
            }
            None => {
                self.started += 1;
                entry.editors.push(
                    started(
                        program,
                        directory,
                        &self.temporary,
                        self.started,
                        if self.beside.is_empty() {
                            self.pane
                        } else {
                            self.beside
                        },
                        file.map(|(file, _)| file),
                    )
                    .map_err(Reply::Refused)?
                    .with_log(log),
                );
                let newest = entry.editors.len() - 1;
                if let Some((file, last)) = file
                    && let Err(unopened) = entry.editors[newest].open(file, last)
                {
                    if let Unopened::Silent = unopened {
                        entry.editors.pop();
                        return Err(Reply::Untaken(CLOSED.to_string()));
                    }
                    return Err(refused(unopened, SILENT));
                }
                self.failed = None;
                newest
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

fn refused(unopened: Unopened, silent: &str) -> Reply {
    match unopened {
        Unopened::Refused(message) => Reply::Refused(message),
        Unopened::Silent => Reply::Refused(silent.to_string()),
        Unopened::Exited => Reply::Untaken(EXITED.to_string()),
    }
}

fn canonical(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
