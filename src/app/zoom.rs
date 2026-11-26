use ratatui::text::{Line, Span};
use ratatui::widgets::Block;

use super::App;
use crate::card::age;
use crate::state::{self, Counts, State};
use crate::text::clean;
use crate::transcript::Transcript;
use crate::zoom::{Headline, guaranteed, info};

impl App {
    pub(super) fn zoom(&mut self) {
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return;
        };
        if self.zoomed {
            self.zoomed = false;
        } else if !entry.ended() {
            entry.on_editor &= self.pane_focused;
            self.zoomed = true;
            self.side_by_side = false;
            self.pane_focused = true;
            self.failed = None;
        }
    }

    pub(super) fn settle_zoom(&mut self) {
        self.zoomed &= self.pane_focused
            && self
                .sessions
                .get(self.selected)
                .is_some_and(|entry| !entry.ended());
    }

    pub(super) fn zoom_heading(
        &self,
        block: Block<'static>,
        room: u16,
        counts: Counts,
    ) -> Block<'static> {
        let room = usize::from(room);
        let mark = self
            .waiting_mark()
            .filter(|mark| self.zoomed && mark.width() < room);
        let marked = mark.as_ref().map_or(0, |mark| mark.width() + 1);
        let mut counted = state::title(counts, room - marked);
        if let Some(mark) = mark {
            counted.spans.splice(0..0, [Span::from(" "), mark]);
        }
        let left = room.saturating_sub(counted.width());
        let headline = self.headline_of(self.selected);
        let kept = headline
            .as_ref()
            .map_or(0, |headline| guaranteed(headline).min(left));
        let names = self.tabs(left - kept).unwrap_or_default();
        let mut spans = headline
            .map(|headline| {
                info(
                    &headline,
                    left.saturating_sub(names.width()),
                    &self.colors,
                    self.nerd_font,
                )
            })
            .unwrap_or_default();
        spans.extend(names.spans);
        block
            .title(Line::from(spans))
            .title(counted.right_aligned())
    }

    pub(super) fn headline_of(&self, index: usize) -> Option<Headline> {
        let entry = self.sessions.get(index)?;
        let facts = entry.transcript.as_ref().map(Transcript::facts);
        let title = self.title_of(index)?;
        let branch = facts
            .and_then(|facts| facts.branch.as_deref())
            .or(entry.place.branch.as_deref())
            .map(clean)
            .filter(|branch| !branch.is_empty());
        Some(Headline {
            state: State::of(entry.ended(), entry.record.as_ref()),
            title,
            repository: clean(&entry.place.repository),
            branch,
            age: entry.elapsed().map(age),
        })
    }
}
