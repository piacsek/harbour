use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Clear;
use unicode_segmentation::UnicodeSegmentation;

use super::key::typed;
use super::{App, draw};
use crate::rename;
use crate::text::{clean, columns};

const LONGEST: usize = 512;
const WIDEST: u16 = 60;

pub(super) struct Renaming {
    text: String,
    before: String,
}

impl App {
    pub(super) fn open_rename(&mut self) {
        let Some(title) = self.title_of(self.selected) else {
            return;
        };
        if !self.renamable() {
            return;
        }
        self.failed = None;
        self.renaming = Some(Renaming {
            text: title.clone(),
            before: title,
        });
    }

    pub(super) fn title_of(&self, index: usize) -> Option<String> {
        let entry = self.sessions.get(index)?;
        let transcribed = entry
            .transcript
            .as_ref()
            .and_then(|transcript| transcript.facts().title.as_deref());
        [entry.given.as_deref(), transcribed]
            .into_iter()
            .flatten()
            .map(clean)
            .find(|title| !title.is_empty())
            .or_else(|| Some(clean(&entry.name)))
    }

    pub(super) fn rename_key(&mut self, key: KeyEvent) -> bool {
        if self.renaming.is_none() {
            return false;
        }
        self.log_unforwarded();
        let Some(renaming) = &mut self.renaming else {
            return false;
        };
        match key.code {
            KeyCode::Esc => self.renaming = None,
            KeyCode::Enter => self.save_rename(),
            KeyCode::Backspace => {
                let kept = renaming
                    .text
                    .grapheme_indices(true)
                    .next_back()
                    .map_or(0, |(at, _)| at);
                renaming.text.truncate(kept);
            }
            KeyCode::Char(c) if (key.modifiers - KeyModifiers::SHIFT).is_empty() => {
                renaming.text.push(typed(key, c));
                kept_to_longest(&mut renaming.text);
            }
            _ => {}
        }
        true
    }

    pub(super) fn rename_paste(&mut self, text: &str) -> bool {
        let Some(renaming) = &mut self.renaming else {
            return false;
        };
        renaming.text.push_str(&clean(text));
        kept_to_longest(&mut renaming.text);
        true
    }

    pub(super) fn renaming(&self) -> bool {
        self.renaming.is_some()
    }

    pub(super) fn draw_rename(&self, frame: &mut Frame) -> Option<Position> {
        let renaming = self.renaming.as_ref()?;
        let screen = frame.area();
        let area = match boxed(screen) {
            Some(area) => {
                let block = draw::bordered(true, self.accent).title(" rename ");
                let inner = block.inner(area);
                frame.render_widget(Clear, area);
                frame.render_widget(block, area);
                inner
            }
            None => Rect {
                y: screen.bottom().checked_sub(1)?,
                height: 1,
                ..screen
            },
        };
        let (label, text) = rename::line(&renaming.text, usize::from(area.width));
        let at = u16::try_from(columns(label) + columns(&text)).unwrap_or(area.width);
        frame.render_widget(Clear, area);
        frame.render_widget(
            Line::from(vec![Span::from(label).bold(), Span::from(text)]),
            area,
        );
        Some(Position::new(area.x + at, area.y))
    }

    fn save_rename(&mut self) {
        let Some(renaming) = self.renaming.take() else {
            return;
        };
        let title = clean(&renaming.text);
        if title == clean(&renaming.before) {
            return;
        }
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return;
        };
        entry.given = (!title.is_empty()).then_some(title);
        let (Some(memory), Some(id)) = (&self.memory, &entry.remembered) else {
            return;
        };
        if memory.title(id, entry.given.as_deref()).is_ok()
            && let Some(kept) = &mut entry.kept
        {
            kept.given_title.clone_from(&entry.given);
        }
    }
}

fn kept_to_longest(text: &mut String) {
    if let Some((at, _)) = text.char_indices().nth(LONGEST) {
        text.truncate(at);
    }
}

fn boxed(screen: Rect) -> Option<Rect> {
    let width = WIDEST.min(screen.width.saturating_sub(4));
    (width >= 3 && screen.height >= 3)
        .then(|| screen.centered(Constraint::Length(width), Constraint::Length(3)))
}
