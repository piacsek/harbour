use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Position, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use unicode_segmentation::UnicodeSegmentation;

use super::App;
use crate::rename;
use crate::text::{clean, columns};

pub(super) struct Renaming {
    text: String,
    before: String,
}

impl App {
    pub(super) fn open_rename(&mut self) {
        let Some(title) = self.title_of(self.selected) else {
            return;
        };
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
                renaming.text.push(c);
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
        true
    }

    pub(super) fn renaming(&self) -> bool {
        self.renaming.is_some()
    }

    pub(super) fn rename_line(&self, width: u16) -> Option<(Line<'static>, u16)> {
        let renaming = self.renaming.as_ref()?;
        let (label, text) = rename::line(&renaming.text, usize::from(width));
        let at = u16::try_from(columns(label) + columns(&text)).unwrap_or(width);
        Some((
            Line::from(vec![Span::from(label).bold(), Span::from(text)]),
            at,
        ))
    }

    pub(super) fn rename_cursor(&self, area: Rect) -> Option<Position> {
        let (_, at) = self.rename_line(area.width)?;
        Some(Position::new(area.x + at, area.bottom().checked_sub(1)?))
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
