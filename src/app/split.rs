use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};

use super::App;
use crate::input::ESCAPED;

const NARROW: u16 = 60;
const NARROWEST: u16 = 15;

impl App {
    pub(super) fn areas(&self, area: Rect) -> (Rect, Rect) {
        if self.zoomed {
            (Rect::default(), area)
        } else if area.width >= NARROW {
            two_columns(area, self.sidebar_width)
        } else {
            self.one_side(area)
        }
    }

    pub(super) fn resize_by_key(&mut self, key: KeyEvent) -> bool {
        let Some(wider) = widens(key) else {
            return false;
        };
        if self.pane_focused
            && self
                .sessions
                .get(self.selected)
                .is_some_and(|entry| entry.showing_editor().is_some())
        {
            return false;
        }
        if self.sidebar.is_empty() || self.shown_pane.is_empty() {
            return true;
        }
        let drawn = self.sidebar.width;
        let widest = self.screen.width / 2;
        let width = match wider {
            true if drawn < widest => (drawn + 2).min(widest),
            false if drawn > NARROWEST => (drawn - 2).max(NARROWEST),
            _ => return true,
        };
        self.sidebar_width = Some(width);
        true
    }

    fn one_side(&self, area: Rect) -> (Rect, Rect) {
        if self.pane_focused {
            (Rect::default(), area)
        } else {
            (area, Rect::default())
        }
    }
}

pub(super) fn widens(key: KeyEvent) -> Option<bool> {
    if key.modifiers - KeyModifiers::SHIFT != KeyModifiers::ALT
        || key.kind != KeyEventKind::Press
        || key.state.contains(ESCAPED)
    {
        return None;
    }
    match key.code {
        KeyCode::Char('-') => Some(false),
        KeyCode::Char('=') => Some(true),
        _ => None,
    }
}

fn two_columns(area: Rect, width: Option<u16>) -> (Rect, Rect) {
    let sidebar = match width {
        Some(width) => Constraint::Length(width.min(area.width / 2)),
        None => Constraint::Ratio(1, 3),
    };
    let [sidebar, pane] = Layout::horizontal([sidebar, Constraint::Fill(1)]).areas(area);
    (sidebar, pane)
}
