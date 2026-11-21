use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::App;

impl App {
    pub(super) fn jump(&mut self, index: usize) {
        if index >= self.sessions.len() {
            return;
        }
        let side = self.side_by_side;
        self.select(index);
        self.focus_or_resume();
        if side {
            self.side_by_side = false;
            self.enter_side();
        }
    }

    pub(super) fn jump_by_key(&mut self, key: KeyEvent) -> bool {
        let Some(position) = position(key) else {
            return false;
        };
        let index = match position {
            9 => self.sessions.len().saturating_sub(1),
            position => position - 1,
        };
        self.jump(index);
        true
    }
}

pub(super) fn position(key: KeyEvent) -> Option<usize> {
    match key.code {
        KeyCode::Char(c @ '1'..='9')
            if key.modifiers - KeyModifiers::SHIFT == KeyModifiers::CONTROL =>
        {
            c.to_digit(10).and_then(|digit| usize::try_from(digit).ok())
        }
        _ => None,
    }
}
