use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{Block, Paragraph};

const HINT: &str = "No session yet: press n to start one.";

#[derive(Default)]
pub struct App {
    quitting: bool,
}

impl App {
    pub fn running(&self) -> bool {
        !self.quitting
    }

    pub fn key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Char('q') {
            self.quitting = true;
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let [sidebar, pane] =
            Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
                .areas(frame.area());
        frame.render_widget(
            Paragraph::new(HINT).block(Block::bordered().title(" sessions ")),
            sidebar,
        );
        frame.render_widget(Paragraph::new(HINT).block(Block::bordered()), pane);
    }
}
