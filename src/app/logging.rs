use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;

use super::{App, Entry};
use crate::handoff::{Reply, Verb};
use crate::keymap::{Action, Keymap};
use crate::log::Log;
use crate::session::Session;
use crate::tmux::Direction;

const BOUND: [char; 4] = ['j', 'k', 'y', 'Y'];
const SLOW_FRAME: Duration = Duration::from_millis(100);

#[derive(Default)]
pub(super) struct Logging {
    log: Log,
    slow: Option<Duration>,
    started: bool,
    forwarded: bool,
    selected: Option<Selected>,
    lives: Vec<(usize, bool)>,
    editors: Vec<(usize, usize)>,
    focus: Option<Focus>,
    failed: Option<String>,
    quit: bool,
}

#[derive(PartialEq)]
enum Selected {
    Session(usize),
    Stopped(Option<String>),
}

#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Sidebar,
    Session,
    Editor,
}

impl Focus {
    fn line(self) -> &'static str {
        match self {
            Focus::Sidebar => "focus to=sidebar",
            Focus::Session => "focus to=pane shown=session",
            Focus::Editor => "focus to=pane shown=editor",
        }
    }
}

pub(super) enum Input {
    Picker,
    Usage,
    Shown,
}

impl Logging {
    pub(super) fn log(&self) -> &Log {
        &self.log
    }
}

impl App {
    pub fn with_log(mut self, log: Log) -> App {
        self.logging.log = log;
        self
    }

    pub fn with_slow_frame(mut self, slow: Duration) -> App {
        self.logging.slow = Some(slow);
        self
    }

    pub(super) fn log_frame(&self, began: Instant) {
        let took = began.elapsed();
        if took >= self.logging.slow.unwrap_or(SLOW_FRAME) {
            self.logging
                .log
                .write(|| format!("slow frame took={}ms", took.as_millis()));
        }
    }

    pub(super) fn log_start(&mut self, size: Rect) {
        if std::mem::replace(&mut self.logging.started, true) {
            return;
        }
        self.logging.log.write(|| {
            format!(
                "start version={} size={}x{} command={:?} picker={} editor={}",
                env!("CARGO_PKG_VERSION"),
                size.width,
                size.height,
                self.command.first().map_or("", String::as_str),
                self.picker
                    .as_ref()
                    .map_or("none".to_string(), |picker| format!(
                        "{:?}",
                        picker.program()
                    )),
                self.editor
                    .as_ref()
                    .map_or("none".to_string(), |editor| format!("{:?}", editor.name())),
            )
        });
    }

    pub(super) fn log_input(&mut self, input: Input, bytes: usize) {
        self.logging.forwarded = true;
        let to = match input {
            Input::Picker => "to=picker".to_string(),
            Input::Usage => "to=usage".to_string(),
            Input::Shown => match self.sessions.get(self.selected) {
                Some(entry) => match entry.showing_editor() {
                    Some(editor) => format!("to=editor session={} editor={}", entry.id, editor.id),
                    None => format!("to=session session={}", entry.id),
                },
                None => return,
            },
        };
        self.logging
            .log
            .write(|| format!("input {to} bytes={bytes}"));
    }

    pub(super) fn log_key(&mut self, key: KeyEvent) {
        if !std::mem::take(&mut self.logging.forwarded) {
            self.logging
                .log
                .write(|| format!("key name={}", named(key, &self.keymap)));
        }
        self.log_changes();
    }

    pub(super) fn log_prefix(&mut self, what: &str) {
        self.logging.forwarded = true;
        let line = if what.is_empty() {
            "prefix".to_string()
        } else {
            format!("prefix {what}")
        };
        self.logging.log.write(|| line);
    }

    pub(super) fn fail_unlogged(&mut self, message: String) {
        self.logging.failed = Some(message.clone());
        self.failed = Some(message);
    }

    pub(super) fn log_changelog_key(&mut self, shown: bool) {
        self.logging.forwarded = true;
        if shown {
            self.logging.log.write(|| "changelog shown".to_string());
        }
    }

    pub(super) fn log_changelog_closed(&mut self) {
        self.logging.log.write(|| "changelog closed".to_string());
    }

    pub(super) fn log_usage_closed(&mut self) {
        self.logging.forwarded = true;
        self.logging.log.write(|| "usage closed".to_string());
    }

    pub(super) fn log_usage_view(&mut self, raw: bool) {
        self.logging.forwarded = true;
        let view = if raw { "raw" } else { "bars" };
        self.logging.log.write(|| format!("usage view={view}"));
    }

    pub(super) fn log_tmux(&mut self, direction: Direction) {
        self.logging.forwarded = true;
        let pane = match direction {
            Direction::Below => "below",
            Direction::Above => "above",
        };
        self.logging.log.write(|| format!("tmux pane={pane}"));
    }

    pub(super) fn log_stop(&self, cause: &str) {
        self.logging.log.write(|| cause.to_string());
    }

    pub(super) fn log_paste(&mut self) {
        self.logging.forwarded = false;
        self.log_changes();
    }

    pub(super) fn log_changes(&mut self) {
        self.log_lives();
        let entry = self.sessions.get(self.selected);
        let selected = entry.map(selection);
        if selected != self.logging.selected {
            let resumed = resumed(self.logging.selected.take(), entry);
            self.logging.selected = selected;
            if !resumed {
                self.logging.log.write(|| match entry {
                    Some(entry) => format!(
                        "selected {} directory={:?}",
                        named_entry(entry),
                        entry.directory
                    ),
                    None => "selected none".to_string(),
                });
            }
        }
        let focus = match self.sessions.get(self.selected) {
            Some(entry) if self.pane_focused && entry.showing_editor().is_some() => Focus::Editor,
            Some(_) if self.pane_focused => Focus::Session,
            _ => Focus::Sidebar,
        };
        if self
            .logging
            .focus
            .replace(focus)
            .is_some_and(|was| was != focus)
        {
            self.logging.log.write(|| focus.line().to_string());
        }
        if self.failed != self.logging.failed {
            self.logging.failed.clone_from(&self.failed);
            if let Some(message) = &self.failed {
                self.logging
                    .log
                    .write(|| format!("refused message={message:?}"));
            }
        }
        if self.quitting && !std::mem::replace(&mut self.logging.quit, true) {
            let sessions = self.sessions.len();
            self.logging
                .log
                .write(|| format!("quit sessions={sessions}"));
        }
    }

    pub(super) fn log_served(&self, verb: Verb, file: &Path, reply: &Reply, asked: Instant) {
        self.logging.log.write(|| {
            let reply = match reply {
                Reply::Holds(claim) => format!(
                    "holds depth={} selected={}",
                    claim.depth,
                    if claim.selected { "yes" } else { "no" }
                ),
                Reply::None => "none".to_string(),
                Reply::Opened => "opened".to_string(),
                Reply::Refused(message) => format!("refused message={message:?}"),
                Reply::Untaken(message) => format!("untaken message={message:?}"),
            };
            format!(
                "edit verb={} file={file:?} reply={reply} took={}ms",
                verb_name(verb),
                asked.elapsed().as_millis()
            )
        });
    }

    pub(super) fn log_expired(&self, verb: Verb, file: &Path) {
        self.logging
            .log
            .write(|| format!("edit verb={} file={file:?} expired", verb_name(verb)));
    }

    fn log_lives(&mut self) {
        let log = &self.logging.log;
        let program = self.command.first().map_or("", String::as_str);
        let editor_program = self.editor.as_ref().map_or("", |editor| editor.name());
        let mut lives = Vec::new();
        let mut editors = Vec::new();
        for entry in &mut self.sessions {
            if entry.session.is_none() {
                continue;
            }
            let known = self
                .logging
                .lives
                .iter()
                .find(|(id, _)| *id == entry.id)
                .copied();
            if known.is_none() {
                log.write(|| started(entry, program));
            }
            let exited = entry.exited_at.is_some();
            if exited && known.is_none_or(|(_, was)| !was) {
                let exit = entry.session.as_mut().and_then(Session::exit);
                let status = exit.map_or(String::new(), |status| match status.signal() {
                    Some(signal) => format!(" signal={signal:?}"),
                    None => format!(" code={}", status.exit_code()),
                });
                log.write(|| format!("exited session={}{status}", entry.id));
            }
            lives.push((entry.id, exited));
            if let Some(count) = entry.session.as_ref().map(Session::take_panics)
                && count > 0
            {
                log.write(|| format!("panic contained session={} count={count}", entry.id));
            }
            for editor in &entry.editors {
                let count = editor.session.take_panics();
                if count > 0 {
                    log.write(|| {
                        format!(
                            "panic contained session={} editor={} count={count}",
                            entry.id, editor.id
                        )
                    });
                }
                if !self.logging.editors.contains(&(entry.id, editor.id)) {
                    log.write(|| {
                        format!(
                            "editor started session={} editor={} program={editor_program:?} directory={:?}",
                            entry.id, editor.id, editor.directory
                        )
                    });
                }
                editors.push((entry.id, editor.id));
            }
        }
        for (session, editor) in &self.logging.editors {
            if !editors.contains(&(*session, *editor)) {
                log.write(|| format!("editor ended session={session} editor={editor}"));
            }
        }
        self.logging.lives = lives;
        self.logging.editors = editors;
    }
}

fn started(entry: &Entry, program: &str) -> String {
    match &entry.remembered {
        Some(id) => format!(
            "resumed session={} id={id:?} program={program:?} directory={:?}",
            entry.id, entry.directory
        ),
        None => {
            let id = entry
                .transcript
                .as_ref()
                .map_or(String::new(), |transcript| {
                    format!(" id={:?}", transcript.session_id())
                });
            format!(
                "started session={} program={program:?}{id} directory={:?}",
                entry.id, entry.directory
            )
        }
    }
}

fn verb_name(verb: Verb) -> &'static str {
    match verb {
        Verb::Ask => "ask",
        Verb::Open => "open",
    }
}

fn selection(entry: &Entry) -> Selected {
    match entry.session {
        Some(_) => Selected::Session(entry.id),
        None => Selected::Stopped(entry.remembered.clone()),
    }
}

fn resumed(was: Option<Selected>, entry: Option<&Entry>) -> bool {
    match (was, entry) {
        (Some(Selected::Stopped(Some(id))), Some(entry)) => {
            entry.session.is_some() && entry.remembered.as_ref() == Some(&id)
        }
        _ => false,
    }
}

fn named_entry(entry: &Entry) -> String {
    match (&entry.session, &entry.remembered) {
        (None, Some(id)) => format!("id={id:?}"),
        _ => format!("session={}", entry.id),
    }
}

fn named(key: KeyEvent, keymap: &Keymap) -> String {
    let name = match key.code {
        KeyCode::Char(c) if key.modifiers == KeyModifiers::CONTROL && matches!(c, 'h' | 'l') => {
            format!("C-{c}")
        }
        KeyCode::Char(c)
            if KeyModifiers::SHIFT.contains(key.modifiers)
                && (BOUND.contains(&c)
                    || matches!(keymap.action_on(c), Some(Action::New | Action::End))) =>
        {
            c.to_string()
        }
        KeyCode::Enter if key.modifiers.is_empty() => "Enter".to_string(),
        _ => return "other".to_string(),
    };
    format!("{name:?}")
}
