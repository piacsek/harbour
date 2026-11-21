use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::Frame;
use ratatui::widgets::{Block, Clear, Paragraph};

use super::App;
use crate::changelog::shown;

#[derive(Default)]
pub(super) struct Reading {
    scroll: usize,
    rows: usize,
    length: usize,
}

impl App {
    pub fn with_changelog(mut self, changelog: &str) -> App {
        self.changelog = changelog.to_string();
        self
    }

    pub(super) fn changelog_key(&mut self, key: KeyEvent) -> bool {
        let Some(reading) = &mut self.reading else {
            return false;
        };
        let half = (reading.rows / 2).max(1);
        let page = reading.rows.max(1);
        match (key.code, key.modifiers) {
            (KeyCode::Char('j') | KeyCode::Down, KeyModifiers::NONE) => reading.down(1),
            (KeyCode::Char('k') | KeyCode::Up, KeyModifiers::NONE) => reading.up(1),
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => reading.down(half),
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => reading.up(half),
            (KeyCode::PageDown, KeyModifiers::NONE) => reading.down(page),
            (KeyCode::PageUp, KeyModifiers::NONE) => reading.up(page),
            _ => self.close_changelog(),
        }
        self.log_changelog_key(false);
        true
    }

    pub(super) fn changelog_mouse(&mut self, event: MouseEvent) -> bool {
        let Some(reading) = &mut self.reading else {
            return false;
        };
        match event.kind {
            MouseEventKind::ScrollDown => reading.down(1),
            MouseEventKind::ScrollUp => reading.up(1),
            MouseEventKind::Down(_) => self.close_changelog(),
            _ => {}
        }
        true
    }

    pub(super) fn close_changelog(&mut self) {
        if self.reading.take().is_some() {
            self.log_changelog_closed();
        }
    }

    pub(super) fn show_changelog(&mut self) {
        self.reading = Some(Reading::default());
        self.log_changelog_key(true);
    }

    pub(super) fn draw_changelog(&mut self, frame: &mut Frame) -> bool {
        let Some(reading) = &mut self.reading else {
            return false;
        };
        let area = frame.area();
        let block = Block::bordered().title(" changelog ");
        let inner = block.inner(area);
        let lines = shown(&self.changelog, usize::from(inner.width));
        reading.rows = usize::from(inner.height);
        reading.length = lines.len();
        reading.scroll = reading.scroll.min(reading.last());
        let lines: Vec<_> = lines.into_iter().skip(reading.scroll).collect();
        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new(lines).block(block), area);
        true
    }
}

impl Reading {
    fn last(&self) -> usize {
        self.length.saturating_sub(self.rows)
    }

    fn down(&mut self, by: usize) {
        self.scroll = (self.scroll + by).min(self.last());
    }

    fn up(&mut self, by: usize) {
        self.scroll = self.scroll.saturating_sub(by);
    }
}
