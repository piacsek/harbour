use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::App;
use crate::log::Log;

const BOUND: [char; 9] = ['q', 'n', 'x', 'e', '?', 'j', 'k', 'y', 'Y'];

#[derive(Default)]
pub(super) struct Logging {
    log: Log,
    started: bool,
    forwarded: bool,
    selected: Option<usize>,
    focus: Option<Focus>,
    failed: Option<String>,
    quit: bool,
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
    Shown,
}

impl App {
    pub fn with_log(mut self, log: Log) -> App {
        self.logging.log = log;
        self
    }

    pub(super) fn log_start(&mut self) {
        if std::mem::replace(&mut self.logging.started, true) {
            return;
        }
        self.logging.log.write(|| {
            format!(
                "start version={} size={}x{} command={:?} picker={} editor={}",
                env!("CARGO_PKG_VERSION"),
                self.screen.width,
                self.screen.height,
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
                .write(|| format!("key name={}", named(key)));
        }
        self.log_changes();
    }

    pub(super) fn log_changes(&mut self) {
        let selected = self.sessions.get(self.selected).map(|entry| entry.id);
        if selected != self.logging.selected {
            self.logging.selected = selected;
            let entry = self.sessions.get(self.selected);
            self.logging.log.write(|| match entry {
                Some(entry) => format!(
                    "selected session={} directory={:?}",
                    entry.id, entry.directory
                ),
                None => "selected none".to_string(),
            });
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
}

fn named(key: KeyEvent) -> String {
    let name = match key.code {
        KeyCode::Char(c) if key.modifiers == KeyModifiers::CONTROL && matches!(c, 'h' | 'l') => {
            format!("C-{c}")
        }
        KeyCode::Char(c) if BOUND.contains(&c) && KeyModifiers::SHIFT.contains(key.modifiers) => {
            c.to_string()
        }
        KeyCode::Enter if key.modifiers.is_empty() => "Enter".to_string(),
        _ => return "other".to_string(),
    };
    format!("{name:?}")
}
