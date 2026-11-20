use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use portable_pty::ExitStatus;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Clear, HighlightSpacing, List, ListState, Paragraph, Wrap,
};

use crate::demo;
use crate::help;
use crate::keys;
use crate::mouse::{self, Gesture, Target};
use crate::picker::{Pick, Picked, Picker};
use crate::registry::{self, Record};
use crate::session::{self, Session};
use crate::state;

const NARROW: u16 = 60;
const HINT: &str = "No session yet: press n to start one.";
const READ_EVERY: Duration = Duration::from_millis(500);

pub struct App {
    command: Vec<String>,
    home: PathBuf,
    picker: Option<Picker>,
    picking: Option<Pick>,
    picking_area: Rect,
    screen: Rect,
    sessions: Vec<Entry>,
    selected: usize,
    started: usize,
    pane: Rect,
    shown_pane: Rect,
    sidebar: Rect,
    pane_focused: bool,
    gesture: Gesture,
    failed: Option<String>,
    asking: Option<Question>,
    helping: bool,
    quitting: bool,
    registry: Option<PathBuf>,
    read_at: Option<Instant>,
}

#[derive(Clone, Copy, PartialEq)]
enum Question {
    End,
    Quit,
}

struct Entry {
    id: usize,
    name: String,
    session: Session,
    started_at: SystemTime,
    record: Option<Record>,
}

impl App {
    pub fn new(command: Vec<String>) -> App {
        let home = std::env::current_dir()
            .ok()
            .or_else(|| std::env::var_os("PWD").map(PathBuf::from))
            .unwrap_or_default();
        App {
            command,
            home,
            picker: None,
            picking: None,
            picking_area: Rect::default(),
            screen: Rect::new(0, 0, 80, 24),
            sessions: Vec::new(),
            selected: 0,
            started: 0,
            pane: Rect::new(0, 0, 80, 24),
            shown_pane: Rect::default(),
            sidebar: Rect::default(),
            pane_focused: false,
            gesture: Gesture::Idle,
            failed: None,
            asking: None,
            helping: false,
            quitting: false,
            registry: None,
            read_at: None,
        }
    }

    pub fn with_picker(mut self, picker: Picker) -> App {
        self.picker = Some(picker);
        self
    }

    pub fn with_registry(mut self, dir: PathBuf) -> App {
        self.registry = Some(dir);
        self
    }

    pub fn demo(registry: PathBuf) -> App {
        let mut app = App::new(demo::working(&registry))
            .with_registry(registry.clone())
            .with_picker(demo::picker());
        app.start();
        app.start_with(&demo::blocked(&registry));
        app.start_with(&demo::idle(&registry));
        app.start_with(&demo::failed());
        app.start_with(&demo::unknown());
        app.selected = 0;
        app
    }

    pub fn running(&self) -> bool {
        !self.quitting
    }

    pub fn key(&mut self, key: KeyEvent) {
        if let Some(pick) = &self.picking {
            let session = pick.session();
            session.send(&keys::encode(key, session.application_cursor()));
            return;
        }
        if self.dismiss(is_yes(key)) {
            return;
        }
        if self.pane_focused && is_ctrl(key, 'h') {
            self.pane_focused = false;
            return;
        }
        if let Some(session) = self.focused() {
            session.send(&keys::encode(key, session.application_cursor()));
            return;
        }
        match key.code {
            _ if is_ctrl(key, 'l') => self.pane_focused = !self.sessions.is_empty(),
            KeyCode::Enter => self.pane_focused = !self.sessions.is_empty(),
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('n') => self.pick_or_start(),
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
        if let Some(pick) = &self.picking {
            pick.session().paste(text);
            return;
        }
        if self.dismiss(false) {
            return;
        }
        if let Some(session) = self.focused() {
            session.paste(text);
        }
    }

    pub fn mouse(&mut self, event: MouseEvent) {
        if let Some(pick) = &self.picking {
            let (mode, encoding) = pick.session().mouse_protocol();
            let target = Target {
                session: 0,
                mode,
                encoding,
            };
            let (gesture, bytes) =
                mouse::route(self.gesture, event, self.picking_area, Some(target));
            self.gesture = gesture;
            if let Some(bytes) = bytes {
                pick.session().send(&bytes);
            }
            return;
        }
        if self.helping || self.asking.is_some() {
            self.gesture = Gesture::Idle;
            if matches!(event.kind, MouseEventKind::Down(_)) {
                self.dismiss(false);
            }
            return;
        }
        let at = Position::new(event.column, event.row);
        if matches!(event.kind, MouseEventKind::Down(_)) {
            if self.sidebar.contains(at) {
                self.pane_focused = false;
            } else if self.shown_pane.contains(at) {
                self.pane_focused = !self.sessions.is_empty();
            }
        }
        let entry = self.sessions.get(self.selected);
        let target = entry.map(|entry| {
            let (mode, encoding) = entry.session.mouse_protocol();
            Target {
                session: entry.id,
                mode,
                encoding,
            }
        });
        let (gesture, bytes) = mouse::route(self.gesture, event, self.shown_pane, target);
        self.gesture = gesture;
        if let (Some(entry), Some(bytes)) = (entry, bytes) {
            entry.session.send(&bytes);
        }
    }

    fn dismiss(&mut self, yes: bool) -> bool {
        if self.helping {
            self.helping = false;
            return true;
        }
        let Some(question) = self.asking.take() else {
            return false;
        };
        if yes {
            self.answer_yes(question);
        }
        true
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

    fn pick_or_start(&mut self) {
        let Some(picker) = &self.picker else {
            self.start();
            return;
        };
        let popup = popup(self.screen);
        match Pick::open(picker, &self.home, popup.height, popup.width) {
            Ok(Some(pick)) => {
                self.gesture = Gesture::Idle;
                self.picking = Some(pick);
            }
            Ok(None) => self.start(),
            Err(error) => self.failed = Some(refusal(picker.program(), &error)),
        }
    }

    fn finish_pick(&mut self) {
        let Some(picked) = self.picking.as_mut().and_then(Pick::picked) else {
            return;
        };
        self.picking = None;
        self.gesture = Gesture::Idle;
        match picked {
            Picked::Cancelled => {}
            Picked::Refused(refusal) => self.failed = Some(refusal),
            Picked::Chosen(directory) => {
                let before = self.started;
                self.start_in(&self.command.clone(), &directory);
                if self.started > before {
                    self.pane_focused = true;
                }
            }
        }
    }

    fn start(&mut self) {
        self.start_with(&self.command.clone());
    }

    fn start_with(&mut self, command: &[String]) {
        self.start_in(command, &self.home.clone());
    }

    fn start_in(&mut self, command: &[String], directory: &Path) {
        let started_at = SystemTime::now();
        match Session::start(command, directory, self.pane.height, self.pane.width) {
            Ok(session) => {
                self.failed = None;
                self.started += 1;
                self.sessions.push(Entry {
                    id: self.started,
                    name: format!("{} {}", folder_name(directory), self.started),
                    session,
                    started_at,
                    record: None,
                });
                self.selected = self.sessions.len() - 1;
            }
            Err(error) => {
                self.failed = Some(refusal(command.first().map_or("", String::as_str), &error))
            }
        }
    }

    fn read_registry(&mut self) {
        let Some(dir) = &self.registry else {
            return;
        };
        if self.read_at.is_some_and(|at| at.elapsed() < READ_EVERY) {
            return;
        }
        self.read_at = Some(Instant::now());
        for entry in &mut self.sessions {
            let last = entry.record.take();
            entry.record = match entry.session.exit() {
                Some(_) => None,
                None => entry
                    .session
                    .pid()
                    .and_then(|pid| registry::read(dir, pid, entry.started_at, last)),
            };
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        self.finish_pick();
        self.read_registry();
        self.screen = frame.area();
        let (sidebar, pane) = self.areas(frame.area());
        self.sidebar = sidebar;
        let notes: Vec<Option<String>> = self
            .sessions
            .iter_mut()
            .map(|entry| entry.session.exit().map(exit_note))
            .collect();
        let title = state::title(
            self.sessions
                .iter()
                .zip(&notes)
                .map(|(entry, note)| (note.is_some(), entry.record.as_ref())),
            usize::from(sidebar.width.saturating_sub(2)),
        );
        let mut sidebar_block = bordered(!self.pane_focused).title(title);
        match self.asking {
            Some(Question::End) => sidebar_block = sidebar_block.title_bottom(" End? y/n "),
            Some(Question::Quit) => sidebar_block = sidebar_block.title_bottom(" Quit? y/n "),
            None => {}
        }
        let below_rows = sidebar_block.inner(sidebar);
        let pane_block = bordered(self.pane_focused);
        self.pane = pane_block.inner(if pane.is_empty() { frame.area() } else { pane });
        self.shown_pane = if pane.is_empty() {
            Rect::default()
        } else {
            self.pane
        };
        for entry in &mut self.sessions {
            entry.session.resize(self.pane.height, self.pane.width);
        }
        let rows: Vec<Line> = self
            .sessions
            .iter()
            .zip(notes)
            .map(|(entry, note)| {
                state::row(
                    &entry.name,
                    note,
                    entry.record.as_ref(),
                    usize::from(below_rows.width.saturating_sub(2)),
                )
            })
            .collect();
        let exited = self
            .sessions
            .get_mut(self.selected)
            .is_some_and(|entry| entry.session.exit().is_some());
        match self.sessions.get(self.selected) {
            None => {
                let message = self.failed.as_deref().unwrap_or(HINT);
                frame.render_widget(
                    Paragraph::new(HINT)
                        .wrap(Wrap { trim: false })
                        .block(sidebar_block),
                    sidebar,
                );
                frame.render_widget(
                    Paragraph::new(message)
                        .wrap(Wrap { trim: false })
                        .block(pane_block),
                    pane,
                );
            }
            Some(shown) => {
                let list = List::new(rows)
                    .block(sidebar_block)
                    .highlight_symbol(Line::from(vec![
                        Span::from("▌").fg(Color::Cyan),
                        Span::from(" "),
                    ]))
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
        if let Some(failed) = &self.failed {
            draw_failure(frame, failed, below_rows);
        }
        if self.helping {
            self.draw_help(frame);
        }
        if let Some(pick) = &mut self.picking {
            let area = popup(frame.area());
            let block = bordered(true).title(" directory ");
            let inner = block.inner(area);
            self.picking_area = inner;
            pick.resize(inner.height, inner.width);
            frame.render_widget(Clear, area);
            frame.render_widget(block, area);
            frame.render_widget(pick.session(), inner);
        }
    }

    fn draw_help(&self, frame: &mut Frame) {
        let keys = help::KEYS
            .iter()
            .map(|(key, what)| Line::from(format!("{key:<10} {what}")));
        let notes = help::NOTES.iter().map(|line| Line::from(*line));
        let lines: Vec<Line> = keys
            .chain([Line::default()])
            .chain(state::legend())
            .chain([Line::default()])
            .chain(notes)
            .collect();
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

impl Drop for App {
    fn drop(&mut self) {
        session::end_all(self.sessions.iter_mut().map(|entry| &mut entry.session));
    }
}

fn draw_failure(frame: &mut Frame, failed: &str, area: Rect) {
    let width = usize::from(area.width.max(1));
    let chars: Vec<char> = failed.chars().collect();
    let lines: Vec<Line> = chars
        .chunks(width)
        .map(|chunk| Line::from(chunk.iter().collect::<String>()))
        .collect();
    let height = u16::try_from(lines.len())
        .unwrap_or(u16::MAX)
        .min(area.height);
    let bottom = Rect::new(area.x, area.bottom() - height, area.width, height);
    frame.render_widget(Clear, bottom);
    frame.render_widget(Paragraph::new(lines), bottom);
}

fn exit_note(status: &ExitStatus) -> String {
    match (status.signal(), status.exit_code()) {
        (Some(signal), _) => format!(" ({signal})"),
        (None, 0) => String::new(),
        (None, code) => format!(" {code}"),
    }
}

fn bordered(focused: bool) -> Block<'static> {
    if focused {
        Block::bordered()
            .border_type(BorderType::Thick)
            .border_style(Color::Cyan)
    } else {
        Block::bordered().border_type(BorderType::Plain)
    }
}

fn is_ctrl(key: KeyEvent, c: char) -> bool {
    key.code == KeyCode::Char(c) && key.modifiers == KeyModifiers::CONTROL
}

fn is_yes(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('y' | 'Y')) && KeyModifiers::SHIFT.contains(key.modifiers)
}

fn refusal(program: &str, error: &io::Error) -> String {
    match error.kind() {
        io::ErrorKind::NotADirectory => error.to_string(),
        _ => format!("Could not start {program}: {error}"),
    }
}

fn folder_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

fn popup(area: Rect) -> Rect {
    area.centered(Constraint::Percentage(80), Constraint::Percentage(80))
}
