use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::text::Line;
use ratatui::widgets::Block;

use super::App;
use crate::card::age;
use crate::input::ESCAPED;
use crate::repo::Place;
use crate::state::{self, Counts, State};
use crate::text::clean;
use crate::transcript::Transcript;
use crate::zoom::{Headline, guaranteed, info};

impl App {
    pub(super) fn zoom_by_key(&mut self, key: KeyEvent) -> bool {
        if !is_zoom(key) {
            return false;
        }
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return true;
        };
        if self.pane_focused && entry.showing_editor().is_some() {
            return false;
        }
        if self.zoomed {
            self.zoomed = false;
        } else if !entry.ended() {
            entry.on_editor &= self.pane_focused;
            self.zoomed = true;
            self.side_by_side = false;
            self.pane_focused = true;
            self.failed = None;
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

    pub(super) fn zoom_border(
        &self,
        block: Block<'static>,
        width: u16,
        counts: Counts,
    ) -> Block<'static> {
        let room = usize::from(width.saturating_sub(2));
        let counted = state::title(counts, room);
        let left = room.saturating_sub(counted.width());
        let headline = self.headline();
        let kept = headline
            .as_ref()
            .map_or(0, |headline| guaranteed(headline).min(left));
        let names = self.tabs(left - kept).unwrap_or_default();
        let mut spans = headline
            .map(|headline| info(&headline, left.saturating_sub(names.width())))
            .unwrap_or_default();
        spans.extend(names.spans);
        block
            .title(Line::from(spans))
            .title(counted.right_aligned())
    }

    fn headline(&self) -> Option<Headline> {
        let entry = self.sessions.get(self.selected)?;
        let facts = entry.transcript.as_ref().map(Transcript::facts);
        let title = facts
            .and_then(|facts| facts.title.as_deref())
            .map(clean)
            .filter(|title| !title.is_empty())
            .unwrap_or_else(|| clean(&entry.name));
        let place = Place {
            repository: entry.place.repository.clone(),
            branch: facts
                .and_then(|facts| facts.branch.clone())
                .or_else(|| entry.place.branch.clone()),
        };
        Some(Headline {
            state: State::of(false, entry.record.as_ref()),
            title,
            place: clean(&place.to_string()),
            age: entry.elapsed().map(age),
        })
    }
}

pub(super) fn is_zoom(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('z' | 'Z'))
        && key.modifiers == KeyModifiers::ALT
        && key.kind == KeyEventKind::Press
        && !key.state.contains(ESCAPED)
}
