use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use portable_pty::ExitStatus;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{
    Block, BorderType, Clear, HighlightSpacing, List, ListState, Paragraph, Wrap,
};

use crate::help;
use crate::keys;
use crate::session::Session;

const NARROW: u16 = 60;
const HINT: &str = "No session yet: press n to start one.";

pub struct App {
    command: Vec<String>,
    directory: String,
    sessions: Vec<Entry>,
    selected: usize,
    started: usize,
    pane: Rect,
    shown_pane: Rect,
    sidebar: Rect,
    pane_focused: bool,
    failed: Option<String>,
    asking: Option<Question>,
    helping: bool,
    quitting: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Question {
    End,
    Quit,
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
            shown_pane: Rect::default(),
            sidebar: Rect::default(),
            pane_focused: false,
            failed: None,
            asking: None,
            helping: false,
            quitting: false,
        }
    }

    pub fn running(&self) -> bool {
        !self.quitting
    }

    pub fn key(&mut self, key: KeyEvent) {
        if self.pane_focused && is_ctrl(key, 'h') {
            self.pane_focused = false;
            return;
        }
        if let Some(session) = self.focused() {
            session.send(&keys::encode(key, session.application_cursor()));
            return;
        }
        if self.helping {
            self.helping = false;
            return;
        }
        if let Some(question) = self.asking.take() {
            if key.code == KeyCode::Char('y') {
                self.answer_yes(question);
            }
            return;
        }
        match key.code {
            _ if is_ctrl(key, 'l') => self.pane_focused = !self.sessions.is_empty(),
            KeyCode::Enter => self.pane_focused = !self.sessions.is_empty(),
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('n') => self.start(),
            KeyCode::Char('x') => self.end_or_remove(),
            KeyCode::Char('?') => self.helping = true,
            KeyCode::Char('j') => {
                self.selected = (self.selected + 1).min(self.sessions.len().saturating_sub(1))
            }
            KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
            _ => {}
        }
    }

    pub fn paste(&mut self, text: &str) {
        if let Some(session) = self.focused() {
            session.paste(text);
        }
    }

    pub fn mouse(&mut self, event: MouseEvent) {
        let at = Position::new(event.column, event.row);
        let pane = self.shown_pane;
        if matches!(event.kind, MouseEventKind::Down(_)) {
            if self.sidebar.contains(at) {
                self.pane_focused = false;
            } else if pane.contains(at) {
                self.pane_focused = true;
            }
        }
        if pane.contains(at)
            && let Some(entry) = self.sessions.get(self.selected)
        {
            entry
                .session
                .mouse(event.kind, event.column - pane.x, event.row - pane.y);
        }
    }

    fn focused(&self) -> Option<&Session> {
        self.sessions
            .get(self.selected)
            .filter(|_| self.pane_focused)
            .map(|entry| &entry.session)
    }

    fn end_or_remove(&mut self) {
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return;
        };
        if entry.session.exit().is_some() {
            self.sessions.remove(self.selected);
            self.selected = self.selected.min(self.sessions.len().saturating_sub(1));
        } else {
            self.asking = Some(Question::End);
        }
    }

    fn quit(&mut self) {
        if self
            .sessions
            .iter_mut()
            .all(|entry| entry.session.exit().is_some())
        {
            self.quitting = true;
        } else {
            self.asking = Some(Question::Quit);
        }
    }

    fn answer_yes(&mut self, question: Question) {
        match question {
            Question::End => {
                if let Some(entry) = self.sessions.get_mut(self.selected) {
                    entry.session.end();
                }
            }
            Question::Quit => self.quitting = true,
        }
    }

    fn start(&mut self) {
        match Session::start(&self.command, self.pane.height, self.pane.width) {
            Ok(session) => {
                self.started += 1;
                self.sessions.push(Entry {
                    name: format!("{} {}", self.directory, self.started),
                    session,
                });
                self.selected = self.sessions.len() - 1;
            }
            Err(_) => {
                self.failed = Some(format!(
                    "Could not start {}: is it installed and on PATH?",
                    self.command.first().map_or("", String::as_str)
                ))
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let (sidebar, pane) = self.areas(frame.area());
        self.sidebar = sidebar;
        let mut sidebar_block = Block::bordered()
            .border_type(border(!self.pane_focused))
            .title(" sessions ");
        match self.asking {
            Some(Question::End) => sidebar_block = sidebar_block.title_bottom(" End? y/n "),
            Some(Question::Quit) => sidebar_block = sidebar_block.title_bottom(" Quit? y/n "),
            None => {}
        }
        let pane_block = Block::bordered().border_type(border(self.pane_focused));
        self.pane = pane_block.inner(if pane.is_empty() { frame.area() } else { pane });
        self.shown_pane = if pane.is_empty() {
            Rect::default()
        } else {
            self.pane
        };
        for entry in &mut self.sessions {
            entry.session.resize(self.pane.height, self.pane.width);
        }
        let rows: Vec<String> = self
            .sessions
            .iter_mut()
            .map(|entry| match entry.session.exit() {
                None => format!("● {}", entry.name),
                Some(status) => format!("○ {} exited{}", entry.name, exit_note(status)),
            })
            .collect();
        let exited = self
            .sessions
            .get_mut(self.selected)
            .is_some_and(|entry| entry.session.exit().is_some());
        match self.sessions.get(self.selected) {
            None => {
                let message = self.failed.as_deref().unwrap_or(HINT);
                frame.render_widget(Paragraph::new(HINT).block(sidebar_block), sidebar);
                frame.render_widget(Paragraph::new(message).block(pane_block), pane);
            }
            Some(shown) => {
                let list = List::new(rows)
                    .block(sidebar_block)
                    .highlight_symbol("› ")
                    .highlight_spacing(HighlightSpacing::Always)
                    .highlight_style(Modifier::BOLD);
                let mut state = ListState::default().with_selected(Some(self.selected));
                frame.render_stateful_widget(list, sidebar, &mut state);
                if !pane.is_empty() {
                    frame.render_widget(pane_block, pane);
                    frame.render_widget(&shown.session, self.pane);
                    if exited {
                        frame.buffer_mut().set_style(self.pane, Modifier::DIM);
                    }
                }
            }
        }
        if self.helping {
            self.draw_help(frame);
        }
    }

    fn draw_help(&self, frame: &mut Frame) {
        let keys = help::KEYS
            .iter()
            .map(|(key, what)| Line::from(format!("{key:<10} {what}")));
        let tmux = help::TMUX.iter().map(|line| Line::from(*line));
        let lines: Vec<Line> = keys.chain([Line::default()]).chain(tmux).collect();
        frame.render_widget(Clear, frame.area());
        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .block(Block::bordered().title(" help ")),
            frame.area(),
        );
    }

    fn areas(&self, area: Rect) -> (Rect, Rect) {
        if area.width >= NARROW {
            let [sidebar, pane] =
                Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)]).areas(area);
            (sidebar, pane)
        } else if self.pane_focused {
            (Rect::default(), area)
        } else {
            (area, Rect::default())
        }
    }
}

fn exit_note(status: &ExitStatus) -> String {
    match (status.signal(), status.exit_code()) {
        (Some(signal), _) => format!(" ({signal})"),
        (None, 0) => String::new(),
        (None, code) => format!(" {code}"),
    }
}

fn border(focused: bool) -> BorderType {
    if focused {
        BorderType::Thick
    } else {
        BorderType::Plain
    }
}

fn is_ctrl(key: KeyEvent, c: char) -> bool {
    key.code == KeyCode::Char(c) && key.modifiers == KeyModifiers::CONTROL
}
