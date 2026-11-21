use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::logging::Input;
use super::{App, Entry, Question, draw, refusal};
use crate::editor::{Changes, Editor};
use crate::keys::{self, Keyboard};
use crate::mouse::Gesture;
use crate::picker::Pick;
use crate::session::Session;

impl App {
    pub fn key(&mut self, key: KeyEvent) {
        self.key_handled(key);
        self.log_key(key);
    }

    fn key_handled(&mut self, key: KeyEvent) {
        let key = ctrl_backspace_as_ctrl_h(key);
        let acted = ctrl_shift_as_ctrl(key);
        if let Some(pick) = &self.picking {
            let session = pick.session();
            let bytes = beside(session, key);
            session.send(&bytes);
            self.log_input(Input::Picker, bytes.len());
            return;
        }
        if self.dismiss(is_yes(key)) {
            return;
        }
        if self.sidebar.is_empty() && !self.sessions.is_empty() {
            self.failed = None;
        }
        if self.jump_by_key(key) || self.zoom_by_key(key) || self.resize_by_key(key) {
            return;
        }
        if self.pane_focused && is_ctrl(acted, 'h') {
            match self.sessions.get_mut(self.selected) {
                Some(entry) if entry.on_editor => {
                    match entry
                        .showing_editor()
                        .filter(|editor| editor.wants_ctrl_h())
                    {
                        Some(editor) => {
                            let bytes = beside(&editor.session, key);
                            editor.session.send(&bytes);
                            self.log_input(Input::Shown, bytes.len());
                        }
                        None => entry.on_editor = false,
                    }
                }
                _ => self.pane_focused = false,
            }
            return;
        }
        if self.pane_focused
            && is_ctrl(acted, 'l')
            && self
                .sessions
                .get(self.selected)
                .is_some_and(|entry| !entry.on_editor)
        {
            self.open_editor();
            self.touch();
            return;
        }
        let on_editor = self
            .sessions
            .get(self.selected)
            .is_some_and(|entry| entry.showing_editor().is_some());
        if let Some(session) = self.focused() {
            let bytes = if on_editor {
                beside(session, key)
            } else {
                keys::encode(key, session.application_cursor())
            };
            session.send(&bytes);
            self.log_input(Input::Shown, bytes.len());
            self.touch();
            return;
        }
        match acted.code {
            _ if is_ctrl(acted, 'l') => self.focus_or_resume(),
            _ if is_ctrl(acted, 'm') => self.focus_or_resume(),
            KeyCode::Enter => self.focus_or_resume(),
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('n') => self.pick_or_start(),
            KeyCode::Char('x') => self.end_or_remove(),
            KeyCode::Char('e') => self.next_editor(),
            KeyCode::Char('?') => self.helping = true,
            KeyCode::Char('j') => {
                self.select((self.selected + 1).min(self.sessions.len().saturating_sub(1)))
            }
            KeyCode::Char('k') => self.select(self.selected.saturating_sub(1)),
            _ => {}
        }
    }

    pub(super) fn focus_or_resume(&mut self) {
        if !self.resume() {
            self.pane_focused = !self.sessions.is_empty();
        } else if self.failed.is_some() {
            self.pane_focused = false;
        }
    }

    fn next_editor(&mut self) {
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return;
        };
        if entry.editors.is_empty() {
            return;
        }
        let next = entry
            .editor
            .map_or(0, |shown| (shown + 1) % entry.editors.len());
        entry.editor = Some(next);
        entry.on_editor = true;
        self.pane_focused = true;
    }

    fn end_or_remove(&mut self) {
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return;
        };
        if entry.running() {
            let changes = changes(&entry.editors);
            self.asking = Some(Question::End(changes));
        } else {
            self.remove_selected();
        }
    }

    fn quit(&mut self) {
        if self.sessions.iter_mut().any(Entry::running) {
            let changes = changes(self.sessions.iter().flat_map(|entry| &entry.editors));
            self.asking = Some(Question::Quit(changes));
        } else {
            self.settle_all();
            self.quitting = true;
        }
    }

    fn pick_or_start(&mut self) {
        let Some(picker) = &self.picker else {
            self.start();
            return;
        };
        let popup = draw::popup(self.screen);
        match Pick::open(
            picker,
            &self.home,
            &self.temporary,
            popup.height,
            popup.width,
        ) {
            Ok(Some(pick)) => {
                self.gesture = Gesture::Idle;
                self.picking = Some(pick);
            }
            Ok(None) => self.start(),
            Err(error) => self.failed = Some(refusal(picker.program(), &error)),
        }
    }
}

fn changes<'a>(editors: impl IntoIterator<Item = &'a Editor>) -> Changes {
    editors
        .into_iter()
        .map(Editor::changes)
        .max()
        .unwrap_or(Changes::Saved)
}

fn ctrl_backspace_as_ctrl_h(key: KeyEvent) -> KeyEvent {
    if key.code == KeyCode::Backspace && key.modifiers == KeyModifiers::CONTROL {
        KeyEvent::new(KeyCode::Char('h'), KeyModifiers::CONTROL)
    } else {
        key
    }
}

fn ctrl_shift_as_ctrl(key: KeyEvent) -> KeyEvent {
    match key.code {
        KeyCode::Char(c @ ('h' | 'H' | 'l' | 'L'))
            if key.modifiers - KeyModifiers::SHIFT == KeyModifiers::CONTROL =>
        {
            KeyEvent::new(KeyCode::Char(c.to_ascii_lowercase()), KeyModifiers::CONTROL)
        }
        _ => key,
    }
}

fn beside(session: &Session, key: KeyEvent) -> Vec<u8> {
    match session.keyboard() {
        Keyboard::Legacy => keys::encode(plain_enter(key), session.application_cursor()),
        keyboard => keys::encode_as(key, session.application_cursor(), keyboard),
    }
}

fn plain_enter(key: KeyEvent) -> KeyEvent {
    if key.code == KeyCode::Enter {
        KeyEvent::new(
            KeyCode::Enter,
            key.modifiers - KeyModifiers::SHIFT - KeyModifiers::CONTROL,
        )
    } else {
        key
    }
}

fn is_ctrl(key: KeyEvent, c: char) -> bool {
    key.code == KeyCode::Char(c) && key.modifiers == KeyModifiers::CONTROL
}

fn is_yes(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('y' | 'Y')) && KeyModifiers::SHIFT.contains(key.modifiers)
}
