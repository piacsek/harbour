use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::widgets::{Block, HighlightSpacing, List, ListState, Paragraph};

use crate::session::Session;

const HINT: &str = "No session yet: press n to start one.";

pub struct App {
    command: Vec<String>,
    directory: String,
    sessions: Vec<Entry>,
    selected: usize,
    started: usize,
    pane: Rect,
    quitting: bool,
}

struct Entry {
    name: String,
    session: Session,
}

impl App {
    pub fn new(command: Vec<String>) -> App {
        let directory = std::env::current_dir()
            .ok()
            .and_then(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .unwrap_or_default();
        App {
            command,
            directory,
            sessions: Vec::new(),
            selected: 0,
            started: 0,
            pane: Rect::default(),
            quitting: false,
        }
    }

    pub fn running(&self) -> bool {
        !self.quitting
    }

    pub fn key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.quitting = true,
            KeyCode::Char('n') => self.start(),
            KeyCode::Char('j') => {
                self.selected = (self.selected + 1).min(self.sessions.len().saturating_sub(1))
            }
            KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
            _ => {}
        }
    }

    fn start(&mut self) {
        if let Ok(session) = Session::start(&self.command, self.pane.height, self.pane.width) {
            self.started += 1;
            self.sessions.push(Entry {
                name: format!("{} {}", self.directory, self.started),
                session,
            });
            self.selected = self.sessions.len() - 1;
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let [sidebar, pane] =
            Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
                .areas(frame.area());
        let sidebar_block = Block::bordered().title(" sessions ");
        let pane_block = Block::bordered();
        self.pane = pane_block.inner(pane);
        match self.sessions.get(self.selected) {
            None => {
                frame.render_widget(Paragraph::new(HINT).block(sidebar_block), sidebar);
                frame.render_widget(Paragraph::new(HINT).block(pane_block), pane);
            }
            Some(shown) => {
                let rows = self
                    .sessions
                    .iter()
                    .map(|entry| format!("● {}", entry.name));
                let list = List::new(rows)
                    .block(sidebar_block)
                    .highlight_symbol("› ")
                    .highlight_spacing(HighlightSpacing::Always)
                    .highlight_style(Modifier::BOLD);
                let mut state = ListState::default().with_selected(Some(self.selected));
                frame.render_stateful_widget(list, sidebar, &mut state);
                frame.render_widget(pane_block, pane);
                frame.render_widget(&shown.session, self.pane);
            }
        }
    }
}
