use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::widgets::{Block, Clear, Paragraph};

use super::App;
use crate::changelog::shown;

#[derive(Default)]
pub(super) struct Reading;

impl App {
    pub fn with_changelog(mut self, changelog: &str) -> App {
        self.changelog = changelog.to_string();
        self
    }

    pub(super) fn changelog_key(&mut self, _key: KeyEvent) -> bool {
        self.reading.take().is_some()
    }

    pub(super) fn draw_changelog(&mut self, frame: &mut Frame) -> bool {
        if self.reading.is_none() {
            return false;
        }
        let area = frame.area();
        let block = Block::bordered().title(" changelog ");
        let lines = shown(&self.changelog, usize::from(block.inner(area).width));
        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new(lines).block(block), area);
        true
    }
}
