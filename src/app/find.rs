use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::{App, Entry, draw, refusal};
use crate::finder;
use crate::input::ESCAPED;
use crate::mouse::Gesture;
use crate::picker::{Pick, Picker};

const GONE: &str = "That session is gone: it was removed while the list was open.";
const UNREAD: &str =
    "The session finder printed a line harbour cannot read; check FZF_DEFAULT_OPTS.";

pub(super) struct Who {
    id: Option<usize>,
    remembered: Option<String>,
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
            self.failed = Some(UNREAD.to_string());
            return;
        };
        let session = who.id.and_then(|id| {
            self.sessions
                .iter()
                .position(|entry| entry.session.is_some() && entry.id == id)
        });
        let remembered = |stopped: bool| {
            let wanted = who.remembered.as_ref()?;
            self.sessions.iter().position(|entry| {
                (!stopped || entry.session.is_none()) && entry.remembered.as_ref() == Some(wanted)
            })
        };
        match session
            .or_else(|| remembered(true))
            .or_else(|| remembered(false))
        {
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
            lines: finder::lines(&headlines),
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
                self.finding = Some(self.sessions.iter().map(who).collect());
            }
            Ok(None) => {}
            Err(error) => self.failed = Some(refusal(picker.program(), &error)),
        }
    }
}

fn who(entry: &Entry) -> Who {
    Who {
        id: entry.session.is_some().then_some(entry.id),
        remembered: entry.remembered.clone(),
    }
}

pub(super) fn is_find(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('s' | 'S'))
        && key.modifiers == KeyModifiers::ALT
        && key.kind == KeyEventKind::Press
        && !key.state.contains(ESCAPED)
}
