use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Block;

use super::App;
use super::key::beside;
use super::logging::Input;
use crate::input::ESCAPED;
use crate::mouse::{self, Gesture, Target};
use crate::state::{self, Counts};
use crate::tabs;
use crate::zoom::{guaranteed, info};

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

    pub(super) fn swap_by_key(&mut self, key: KeyEvent) -> bool {
        let Some(right) = swaps(key) else {
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
        if self.side_by_side && self.pane_focused {
            self.swapped = right;
        }
        true
    }

    pub(super) fn swapped_ctrl(&mut self, key: KeyEvent, ctrl_h: bool) -> bool {
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return false;
        };
        if !entry.on_editor {
            if ctrl_h {
                self.open_editor();
                self.touch();
            }
            return ctrl_h;
        }
        match entry.showing_editor().filter(|editor| {
            if ctrl_h {
                editor.wants_ctrl_h()
            } else {
                editor.wants_ctrl_l()
            }
        }) {
            Some(editor) => {
                let bytes = beside(&editor.session, key);
                editor.session.send(&bytes);
                self.log_input(Input::Shown, bytes.len());
            }
            None => {
                entry.on_editor = false;
                if ctrl_h {
                    self.pane_focused = false;
                }
            }
        }
        true
    }

    pub(super) fn enter_side(&mut self) {
        let Some(entry) = self.sessions.get(self.selected) else {
            return;
        };
        if entry.ended() || self.screen.width < WIDEST {
            return;
        }
        let [_, editor] = self.ordered(split(self.screen));
        self.beside = Block::bordered().inner(editor);
        self.failed = None;
        self.show_or_open_editor();
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

    pub(super) fn stay_side(&mut self) {
        self.side_by_side = false;
        let Some(entry) = self.sessions.get(self.selected) else {
            return;
        };
        if !self.pane_focused || entry.ended() {
            return;
        }
        let on_editor = entry.on_editor;
        self.failed = None;
        self.show_or_open_editor();
        let entry = &mut self.sessions[self.selected];
        if self.failed.is_some() || entry.editor().is_none() {
            entry.on_editor = false;
            self.zoomed = true;
            return;
        }
        entry.on_editor = on_editor;
        self.side_by_side = true;
        self.zoomed = false;
    }

    fn show_or_open_editor(&mut self) {
        if self
            .sessions
            .get(self.selected)
            .is_some_and(|entry| entry.editor().is_none())
        {
            self.open_editor();
        }
    }

    pub(super) fn side_mouse(&mut self, event: MouseEvent) -> bool {
        if self.beside.is_empty() {
            return false;
        }
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return false;
        };
        let Some(editor_id) = entry.editor().map(|editor| editor.id) else {
            return false;
        };
        let at = Position::new(event.column, event.row);
        let right = match (self.gesture, event.kind) {
            (Gesture::Pane(owner), MouseEventKind::Drag(_) | MouseEventKind::Up(_)) => {
                owner == editor_id
            }
            _ => self.beside.contains(at),
        };
        if matches!(event.kind, MouseEventKind::Down(_)) && (right || self.pane.contains(at)) {
            self.pane_focused = true;
            entry.on_editor = right;
        }
        let (area, id, session) = if right {
            (
                self.beside,
                editor_id,
                entry.editor().map(|editor| &editor.session),
            )
        } else {
            (self.pane, entry.id, entry.session.as_ref())
        };
        let target = session.map(|session| {
            let (mode, encoding) = session.mouse_protocol();
            Target {
                session: id,
                mode,
                encoding,
            }
        });
        let (gesture, bytes) = mouse::route(self.gesture, event, area, target);
        self.gesture = gesture;
        if let (Some(session), Some(bytes)) = (session, bytes) {
            session.send(&bytes);
            self.touch();
        }
        true
    }

    pub(super) fn side_borders(
        &self,
        [left, right]: [Block<'static>; 2],
        [left_width, right_width]: [u16; 2],
        counts: Counts,
    ) -> [Block<'static>; 2] {
        let on_editor = self
            .sessions
            .get(self.selected)
            .is_some_and(|entry| entry.on_editor);
        let (claude_style, editor_style) = if on_editor {
            (Modifier::DIM, Modifier::BOLD)
        } else {
            (Modifier::BOLD, Modifier::DIM)
        };
        let room = usize::from(left_width.saturating_sub(2));
        let headline = self.headline_of(self.selected);
        let kept = headline.as_ref().map_or(0, guaranteed).min(room);
        let claude = named(tabs::claude(self.nerd_font), claude_style);
        let claude = if claude.width() <= room - kept {
            claude
        } else {
            Line::default()
        };
        let mut spans = headline
            .map(|headline| info(&headline, room - claude.width()))
            .unwrap_or_default();
        spans.extend(claude.spans);
        let room = usize::from(right_width.saturating_sub(2));
        let counted = state::title(counts, room);
        let editor = self.editor.as_ref().map_or_else(String::new, |program| {
            tabs::editor_named(
                program.name(),
                self.nerd_font && program.is_neovim(),
                self.sessions
                    .get(self.selected)
                    .and_then(|entry| entry.editor())
                    .map(|editor| editor.label.as_str()),
                room.saturating_sub(counted.width() + 2),
            )
        });
        let editor = if editor.is_empty() {
            Line::default()
        } else {
            named(editor, editor_style)
        };
        [
            left.title(Line::from(spans)),
            right.title(editor).title(counted.right_aligned()),
        ]
    }

    pub(super) fn halves(&self, area: Rect) -> Option<[Rect; 2]> {
        (self.side_by_side && area.width >= WIDEST).then(|| self.ordered(split(area)))
    }

    fn ordered(&self, [left, right]: [Rect; 2]) -> [Rect; 2] {
        if self.swapped {
            [right, left]
        } else {
            [left, right]
        }
    }

    pub(super) fn settle_side(&mut self) {
        let Some(entry) = self.sessions.get(self.selected) else {
            self.side_by_side = false;
            return;
        };
        if !self.side_by_side {
            return;
        }
        if entry.ended() {
            self.side_by_side = false;
            self.sessions[self.selected].on_editor = false;
        } else if !self.pane_focused {
            self.side_by_side = false;
        } else if entry.editor().is_none() {
            self.side_by_side = false;
            self.zoomed = true;
        }
    }
}

pub(super) fn is_side(key: KeyEvent) -> bool {
    key.code == KeyCode::Char('\\')
        && key.modifiers == KeyModifiers::ALT
        && key.kind == KeyEventKind::Press
        && !key.state.contains(ESCAPED)
}

fn swaps(key: KeyEvent) -> Option<bool> {
    if key.modifiers - KeyModifiers::SHIFT != KeyModifiers::ALT
        || key.kind != KeyEventKind::Press
        || key.state.contains(ESCAPED)
    {
        return None;
    }
    match key.code {
        KeyCode::Char('{') => Some(false),
        KeyCode::Char('}') => Some(true),
        _ => None,
    }
}

fn named(name: String, style: Modifier) -> Line<'static> {
    Line::from(vec![
        Span::from(" "),
        Span::styled(name, style),
        Span::from(" "),
    ])
}

fn split(area: Rect) -> [Rect; 2] {
    Layout::horizontal([Constraint::Ratio(1, 2); 2]).areas(area)
}
