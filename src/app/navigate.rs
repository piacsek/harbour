use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::App;
use crate::tmux::Direction;

impl App {
    pub(super) fn navigate_by_key(&mut self, key: KeyEvent) -> bool {
        let Some(direction) = direction(key) else {
            return false;
        };
        let on_editor = self.pane_focused
            && self
                .sessions
                .get(self.selected)
                .is_some_and(|entry| entry.showing_editor().is_some());
        if on_editor
            || !self
                .tmux
                .as_ref()
                .is_some_and(|tmux| tmux.select(direction))
        {
            return false;
        }
        self.log_tmux(direction);
        true
    }
}

fn direction(key: KeyEvent) -> Option<Direction> {
    if key.modifiers - KeyModifiers::SHIFT != KeyModifiers::CONTROL {
        return None;
    }
    match key.code {
        KeyCode::Char('j' | 'J') => Some(Direction::Below),
        KeyCode::Char('k' | 'K') => Some(Direction::Above),
        _ => None,
    }
}
