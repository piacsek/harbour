use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::{App, Entry, draw, refusal};
use crate::finder;
use crate::input::ESCAPED;
use crate::mouse::Gesture;
use crate::picker::{Pick, Picker};

const GONE: &str = "That session is gone: it ended while the list was open.";

pub(super) enum Who {
    Live(usize),
    Ended(usize),
    Stopped(String),
}

impl App {
    pub(super) fn find_by_key(&mut self, key: KeyEvent) -> bool {
        if !is_find(key) {
            return false;
        }
        let on_editor = self
            .sessions
            .get(self.selected)
            .is_some_and(|entry| entry.showing_editor().is_some());
        if self.pane_focused && on_editor {
            return false;
        }
        if !self.sessions.is_empty() {
            self.open_finder();
        }
        true
    }

    pub(super) fn found(&mut self, finding: Option<Vec<Who>>, line: &[u8]) {
        let Some(who) = finder::chosen(line).and_then(|n| finding?.into_iter().nth(n)) else {
            return;
        };
        let index = self.sessions.iter_mut().position(|entry| match &who {
            Who::Live(id) => entry.id == *id && entry.session.is_some() && entry.running(),
            Who::Ended(id) => entry.id == *id && entry.session.is_some(),
            Who::Stopped(id) => entry.session.is_none() && entry.remembered.as_ref() == Some(id),
        });
        match index {
            Some(index) => self.jump(index),
            None => self.failed = Some(GONE.to_string()),
        }
    }

    fn open_finder(&mut self) {
        let fzf = match &self.picker {
            None => vec!["fzf".to_string()],
            Some(Picker::Command(fzf) | Picker::Fuzzy { fzf, .. } | Picker::Listed { fzf, .. }) => {
                fzf.clone()
            }
        };
        if Path::new(&fzf[0]).file_name() != Some("fzf".as_ref()) {
            self.failed = Some(format!(
                "Could not start the session finder: it needs fzf, not {}",
                fzf[0]
            ));
            return;
        }
        let popup = draw::popup(self.screen);
        let headlines: Vec<_> = (0..self.sessions.len())
            .filter_map(|index| self.headline_of(index))
            .collect();
        let picker = Picker::Listed {
            fzf,
            lines: finder::lines(&headlines, usize::from(popup.width.saturating_sub(5))),
        };
        match Pick::open(
            &picker,
            &self.home,
            &self.temporary,
            popup.height,
            popup.width,
        ) {
            Ok(Some(pick)) => {
                self.gesture = Gesture::Idle;
                self.picking = Some(pick);
                self.finding = Some(self.sessions.iter_mut().map(who).collect());
            }
            Ok(None) => {}
            Err(error) => self.failed = Some(refusal(picker.program(), &error)),
        }
    }
}

fn who(entry: &mut Entry) -> Who {
    if entry.session.is_none()
        && let Some(id) = &entry.remembered
    {
        return Who::Stopped(id.clone());
    }
    if entry.running() {
        Who::Live(entry.id)
    } else {
        Who::Ended(entry.id)
    }
}

fn is_find(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('s' | 'S'))
        && key.modifiers == KeyModifiers::ALT
        && key.kind == KeyEventKind::Press
        && !key.state.contains(ESCAPED)
}
