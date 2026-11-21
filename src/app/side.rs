use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::widgets::Block;

use super::App;
use crate::input::ESCAPED;

const WIDEST: u16 = 120;

impl App {
    pub(super) fn side_by_key(&mut self, key: KeyEvent) -> bool {
        if !is_side(key) {
            return false;
        }
        let Some(entry) = self.sessions.get(self.selected) else {
            return true;
        };
        if self.pane_focused && entry.showing_editor().is_some() {
            return false;
        }
        if self.side_by_side {
            self.side_by_side = false;
            return true;
        }
        self.enter_side();
        true
    }

    pub(super) fn enter_side(&mut self) {
        let Some(entry) = self.sessions.get(self.selected) else {
            return;
        };
        if entry.ended() || self.screen.width < WIDEST {
            return;
        }
        let [_, right] = split(self.screen);
        self.beside = Block::bordered().inner(right);
        self.failed = None;
        self.open_editor();
        let entry = &mut self.sessions[self.selected];
        if self.failed.is_some() || entry.editor().is_none() {
            self.beside = Rect::default();
            return;
        }
        entry.on_editor = false;
        self.side_by_side = true;
        self.zoomed = false;
        self.pane_focused = true;
    }

    pub(super) fn halves(&self, area: Rect) -> Option<[Rect; 2]> {
        (self.side_by_side && area.width >= WIDEST).then(|| split(area))
    }

    pub(super) fn settle_side(&mut self) {
        let Some(entry) = self.sessions.get(self.selected) else {
            self.side_by_side = false;
            return;
        };
        if !self.side_by_side {
            return;
        }
        if !self.pane_focused || entry.ended() {
            self.side_by_side = false;
        } else if entry.editor().is_none() {
            self.side_by_side = false;
            self.zoomed = true;
        }
    }
}

fn is_side(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('\\')
        && key.modifiers == KeyModifiers::ALT
        && key.kind == KeyEventKind::Press
        && !key.state.contains(ESCAPED)
}

fn split(area: Rect) -> [Rect; 2] {
    Layout::horizontal([Constraint::Ratio(1, 2); 2]).areas(area)
}
