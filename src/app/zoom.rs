use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use super::App;
use crate::input::ESCAPED;

impl App {
    pub(super) fn zoom_by_key(&mut self, key: KeyEvent) -> bool {
        if !is_zoom(key) {
            return false;
        }
        let Some(entry) = self.sessions.get(self.selected) else {
            return true;
        };
        if self.pane_focused && entry.showing_editor().is_some() {
            return false;
        }
        if self.zoomed {
            self.zoomed = false;
        } else if !entry.ended() {
            self.zoomed = true;
            self.pane_focused = true;
        }
        true
    }

    pub(super) fn settle_zoom(&mut self) {
        self.zoomed &= self.pane_focused
            && self
                .sessions
                .get(self.selected)
                .is_some_and(|entry| !entry.ended());
    }
}

pub(super) fn is_zoom(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('z')
        && key.modifiers == KeyModifiers::ALT
        && key.kind == KeyEventKind::Press
        && !key.state.contains(ESCAPED)
}
