use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use portable_pty::ExitStatus;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Clear, Paragraph, Wrap};

use crate::card::{Card, Ended, Sidebar};
use crate::demo;
use crate::help;
use crate::keys;
use crate::memory::{Memory, Remembered};
use crate::mouse::{self, Gesture, Target};
use crate::picker::{Pick, Picked, Picker};
use crate::registry::{self, Record};
use crate::repo::{self, Place};
use crate::session::{self, Session};
use crate::state;
use crate::summary::Summary;
use crate::text;
use crate::transcript::Transcript;

mod remember;

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
    cards: Sidebar,
    pane_focused: bool,
    gesture: Gesture,
    failed: Option<String>,
    asking: Option<Question>,
    helping: bool,
    quitting: bool,
    registry: Option<PathBuf>,
    transcripts: Option<PathBuf>,
    read_at: Option<Instant>,
    summary: Option<Summary>,
    memory: Option<Memory>,
    recalled: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Question {
    End,
    Quit,
}

struct Entry {
    id: usize,
    name: String,
    session: Option<Session>,
    started_at: SystemTime,
    record: Option<Record>,
    directory: PathBuf,
    place: Place,
    exited_at: Option<Instant>,
    transcript: Option<Transcript>,
    remembered: Option<String>,
    kept: Option<Remembered>,
    why: &'static str,
}

impl Entry {
    fn elapsed(&self) -> Option<Duration> {
        match self.exited_at {
            Some(at) => Some(at.elapsed()),
            None => self
                .record
                .as_ref()
                .and_then(|record| record.status_since)
                .map(|since| SystemTime::now().duration_since(since).unwrap_or_default()),
        }
    }
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
            cards: Sidebar::default(),
            pane_focused: false,
            gesture: Gesture::Idle,
            failed: None,
            asking: None,
            helping: false,
            quitting: false,
            registry: None,
            transcripts: None,
            read_at: None,
            summary: None,
            memory: None,
            recalled: false,
        }
    }

    pub fn with_home(mut self, home: PathBuf) -> App {
        self.home = home;
        self
    }

    pub fn with_picker(mut self, picker: Picker) -> App {
        self.picker = Some(picker);
        self
    }

    pub fn with_registry(mut self, dir: PathBuf) -> App {
        self.registry = Some(dir);
        self
    }

    pub fn with_transcripts(mut self, projects: PathBuf) -> App {
        self.transcripts = Some(projects);
        self
    }

    pub fn with_summary(mut self, summary: Summary) -> App {
        self.summary = Some(summary);
        self
    }

    pub fn demo(registry: PathBuf) -> App {
        let mut app = App::new(demo::fresh())
            .with_home(demo::checkout(&registry, "acme-api", "main"))
            .with_registry(registry.clone())
            .with_transcripts(registry.join("projects"))
            .with_memory(demo::stopped(&registry))
            .with_picker(demo::picker());
        app.start_with(&demo::working(&registry));
        app.start_with(&demo::blocked(&registry));
        app.start_with(&demo::idle(&registry));
        app.start_in(
            &demo::failed(),
            &demo::checkout(&registry, "acme-web", "main"),
        );
        app.start_in(
            &demo::unknown(),
            &demo::checkout(&registry, "acme-infra", "tidy-dns"),
        );
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
            _ if is_ctrl(key, 'l') => self.focus_or_resume(),
            KeyCode::Enter => self.focus_or_resume(),
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
                self.pane_focused = self.has_process();
            }
        }
        let shown = self
            .sessions
            .get(self.selected)
            .and_then(|entry| entry.session.as_ref().map(|session| (entry.id, session)));
        let target = shown.map(|(id, session)| {
            let (mode, encoding) = session.mouse_protocol();
            Target {
                session: id,
                mode,
                encoding,
            }
        });
        let (gesture, bytes) = mouse::route(self.gesture, event, self.shown_pane, target);
        self.gesture = gesture;
        if let (Some((_, session)), Some(bytes)) = (shown, bytes) {
            session.send(&bytes);
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
            .and_then(|entry| entry.session.as_ref())
    }

    fn focus_or_resume(&mut self) {
        if !self.resume() {
            self.pane_focused = !self.sessions.is_empty();
        }
    }

    fn end_or_remove(&mut self) {
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return;
        };
        if entry
            .session
            .as_mut()
            .is_none_or(|session| session.exit().is_some())
        {
            let removed = self.sessions.remove(self.selected);
            self.forget(&removed);
            self.selected = self.selected.min(self.sessions.len().saturating_sub(1));
        } else {
            self.asking = Some(Question::End);
        }
    }

    fn quit(&mut self) {
        if self
            .sessions
            .iter_mut()
            .filter_map(|entry| entry.session.as_mut())
            .all(|session| session.exit().is_some())
        {
            self.quitting = true;
        } else {
            self.asking = Some(Question::Quit);
        }
    }

    fn answer_yes(&mut self, question: Question) {
        match question {
            Question::End => {
                if let Some(session) = self
                    .sessions
                    .get_mut(self.selected)
                    .and_then(|entry| entry.session.as_mut())
                {
                    session.end();
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
                    session: Some(session),
                    started_at,
                    record: None,
                    directory: directory.to_path_buf(),
                    place: repo::place(directory),
                    exited_at: None,
                    transcript: None,
                    remembered: None,
                    kept: None,
                    why: remember::STOPPED,
                });
                self.selected = self.sessions.len() - 1;
            }
            Err(error) => {
                self.failed = Some(refusal(command.first().map_or("", String::as_str), &error))
            }
        }
    }

    fn read_files(&mut self) {
        if self.read_at.is_some_and(|at| at.elapsed() < READ_EVERY) {
            return;
        }
        self.read_at = Some(Instant::now());
        for entry in &mut self.sessions {
            let last = entry.record.take();
            let Some(session) = entry.session.as_mut() else {
                continue;
            };
            if session.exit().is_some() {
                continue;
            }
            entry.record = self.registry.as_deref().and_then(|dir| {
                let pid = session.pid()?;
                registry::read(dir, pid, entry.started_at, last)
            });
            let id = entry
                .record
                .as_ref()
                .and_then(|record| record.session_id.as_deref());
            if let (Some(id), Some(projects)) = (id, &self.transcripts)
                && entry
                    .transcript
                    .as_ref()
                    .is_none_or(|transcript| transcript.session_id() != id)
            {
                entry.transcript = Some(Transcript::new(projects, id).since(entry.started_at));
            }
            if let Some(transcript) = &mut entry.transcript {
                transcript.read();
            }
            let directory = entry
                .transcript
                .as_ref()
                .and_then(|transcript| transcript.facts().directory.as_deref())
                .unwrap_or(&entry.directory);
            entry.place = repo::place(directory);
        }
        self.follow();
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        self.finish_pick();
        self.read_files();
        self.screen = frame.area();
        let (sidebar, pane) = self.areas(frame.area());
        self.sidebar = sidebar;
        let notes: Vec<Option<Ended>> = self
            .sessions
            .iter_mut()
            .map(|entry| {
                let ended = match &mut entry.session {
                    None => Some(Ended::Stopped(entry.why)),
                    Some(session) => session
                        .exit()
                        .map(|status| Ended::Exited(exit_note(status))),
                };
                if matches!(ended, Some(Ended::Exited(_))) {
                    entry.exited_at.get_or_insert_with(Instant::now);
                }
                ended
            })
            .collect();
        let counts = state::counts(
            self.sessions
                .iter()
                .zip(&notes)
                .filter(|(entry, _)| entry.session.is_some())
                .map(|(entry, note)| (note.is_some(), entry.record.as_ref())),
        );
        if let Some(summary) = &mut self.summary {
            summary.publish(counts);
        }
        let title = state::title(counts, usize::from(sidebar.width.saturating_sub(2)));
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
        for session in self
            .sessions
            .iter_mut()
            .filter_map(|entry| entry.session.as_mut())
        {
            session.resize(self.pane.height, self.pane.width);
        }
        let cards: Vec<Card> = self
            .sessions
            .iter()
            .zip(notes)
            .map(|(entry, note)| {
                Card::new(
                    &entry.name,
                    note,
                    entry.record.as_ref(),
                    &entry.place,
                    entry.elapsed(),
                    entry.transcript.as_ref().map(Transcript::facts),
                )
            })
            .collect();
        let exited = self
            .sessions
            .get_mut(self.selected)
            .and_then(|entry| entry.session.as_mut())
            .is_some_and(|session| session.exit().is_some());
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
                frame.render_widget(sidebar_block, sidebar);
                self.cards
                    .draw(&cards, self.selected, below_rows, frame.buffer_mut());
                self.note_quit(below_rows, frame.buffer_mut());
                if !pane.is_empty() {
                    match &shown.session {
                        Some(session) => {
                            frame.render_widget(pane_block, pane);
                            frame.render_widget(session, self.pane);
                        }
                        None => frame.render_widget(
                            Paragraph::new(remember::RESUME)
                                .wrap(Wrap { trim: false })
                                .block(pane_block),
                            pane,
                        ),
                    }
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
        session::end_all(
            self.sessions
                .iter_mut()
                .filter_map(|entry| entry.session.as_mut()),
        );
    }
}

fn draw_failure(frame: &mut Frame, failed: &str, area: Rect) {
    let lines: Vec<Line> = text::wrap(failed, usize::from(area.width))
        .into_iter()
        .map(Line::from)
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
