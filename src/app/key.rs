use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::logging::Input;
use super::{App, Entry, Question, draw, refusal};
use crate::editor::{Changes, Editor};
use crate::keymap::{Action, typed};
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
        if self.rename_key(key) {
            return;
        }
        if self.usage_key(key) {
            return;
        }
        if let Some(pick) = &self.picking {
            let session = pick.session();
            let bytes = beside(session, key);
            if !bytes.is_empty() {
                session.send(&bytes);
                self.log_input(Input::Picker, bytes.len());
            }
            return;
        }
        if self.prefix_closes_view(key) {
            return;
        }
        if self.changelog_key(key) {
            return;
        }
        let quit_with_nothing_unsaved = matches!(self.asking, Some(Question::Quit(Changes::Saved)));
        if self.dismiss(is_yes(key) || quit_with_nothing_unsaved && is_ctrl(key, 'c')) {
            return;
        }
        if self.sidebar.is_empty() && !self.sessions.is_empty() {
            self.failed = None;
        }
        if self.prefix_key(key) {
            return;
        }
        if self.navigate_by_key(acted) {
            return;
        }
        if self.pane_focused
            && self.side_by_side
            && self.swapped
            && (is_ctrl(acted, 'h') || is_ctrl(acted, 'l'))
            && self.swapped_ctrl(key, is_ctrl(acted, 'h'))
        {
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
            && !(self.side_by_side && self.swapped)
            && self
                .sessions
                .get(self.selected)
                .is_some_and(|entry| !entry.on_editor)
        {
            self.open_editor();
            self.touch();
            return;
        }
        if self.forward(key) {
            return;
        }
        match acted.code {
            _ if is_ctrl(acted, 'l') => self.focus_or_resume(),
            _ if is_ctrl(acted, 'm') => self.focus_or_resume(),
            _ if is_ctrl(key, 'c') => self.quit(),
            KeyCode::Enter => self.focus_or_resume(),
            KeyCode::Char(c) if self.keymap.action_on(c) == Some(Action::New) => {
                self.pick_or_start()
            }
            KeyCode::Char(c) if self.keymap.action_on(c) == Some(Action::End) => {
                self.end_or_remove()
            }
            KeyCode::Char(c) if self.keymap.action_on(c) == Some(Action::Close) => {
                self.close_or_remove()
            }
            KeyCode::Char(c) if typed(acted, c) == 'J' => self.move_card(true),
            KeyCode::Char(c) if typed(acted, c) == 'K' => self.move_card(false),
            KeyCode::Char('j') => {
                self.select((self.selected + 1).min(self.sessions.len().saturating_sub(1)))
            }
            KeyCode::Char('k') => self.select(self.selected.saturating_sub(1)),
            _ => {}
        }
    }

    pub(super) fn forward(&mut self, key: KeyEvent) -> bool {
        let on_editor = self
            .sessions
            .get(self.selected)
            .is_some_and(|entry| entry.showing_editor().is_some());
        let Some(session) = self.focused() else {
            return false;
        };
        let bytes = if on_editor {
            beside(session, key)
        } else {
            keys::encode(key, session.application_cursor())
        };
        if bytes.is_empty() {
            return true;
        }
        session.send(&bytes);
        self.log_input(Input::Shown, bytes.len());
        self.touch();
        true
    }

    pub(super) fn focus_or_resume(&mut self) {
        if !self.resume() {
            self.pane_focused = !self.sessions.is_empty();
        } else if self.failed.is_some() {
            self.pane_focused = false;
        }
    }

    pub(super) fn next_editor(&mut self) {
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

    pub(super) fn end_or_remove(&mut self) {
        self.ask_or_remove(false);
    }

    pub(super) fn close_or_remove(&mut self) {
        self.ask_or_remove(true);
    }

    fn ask_or_remove(&mut self, closing: bool) {
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return;
        };
        if entry.running() {
            let changes = changes(&entry.editors);
            self.closing = closing;
            self.asking = Some(Question::End(changes));
        } else {
            self.remove_selected();
        }
    }

    pub(super) fn quit(&mut self) {
        if self.sessions.iter_mut().any(Entry::running) {
            let changes = changes(self.sessions.iter().flat_map(|entry| &entry.editors));
            self.asking = Some(Question::Quit(changes));
        } else {
            self.settle_all();
            self.quitting = true;
        }
    }

    pub(super) fn pick_or_start(&mut self) {
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
            self.transport,
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
        KeyCode::Char(c @ ('h' | 'H' | 'j' | 'J' | 'k' | 'K' | 'l' | 'L'))
            if key.modifiers - KeyModifiers::SHIFT == KeyModifiers::CONTROL =>
        {
            KeyEvent::new(KeyCode::Char(c.to_ascii_lowercase()), KeyModifiers::CONTROL)
        }
        _ => key,
    }
}

pub(super) fn beside(session: &Session, key: KeyEvent) -> Vec<u8> {
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
