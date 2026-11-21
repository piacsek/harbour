use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

mod draw;
mod handoff;
mod jump;
mod key;
mod logging;
mod read;
mod remember;
mod split;
mod usage;

use draw::exit_note;
use logging::Input;

use crate::card::Sidebar;
use crate::demo;
use crate::editor::{Changes, Editor, Program};
use crate::follow::{self, Follow, Mover};
use crate::handoff::Listener;
use crate::memory::{Memory, Remembered, Reservation};
use crate::mouse::{self, Gesture, Target};
use crate::picker::{Pick, Picked, Picker};
use crate::registry::Record;
use crate::repo::{self, Place};
use crate::session::{self, Session};
use crate::summary::Summary;
use crate::transcript::{self, Transcript};

const HINT: &str = "No session yet: press n to start one.";
const READ_EVERY: Duration = Duration::from_millis(500);

pub struct App {
    command: Vec<String>,
    home: PathBuf,
    picker: Option<Picker>,
    editor: Option<Program>,
    picking: Option<Pick>,
    picking_area: Rect,
    usage: usage::Usage,
    screen: Rect,
    sessions: Vec<Entry>,
    selected: usize,
    started: usize,
    named: usize,
    pane: Rect,
    shown_pane: Rect,
    sidebar: Rect,
    cards: Sidebar,
    accent: Color,
    nerd_font: bool,
    sidebar_width: Option<u16>,
    pane_focused: bool,
    gesture: Gesture,
    failed: Option<String>,
    asking: Option<Question>,
    helping: bool,
    quitting: bool,
    shape: u8,
    cursor: Option<Position>,
    registry: Option<PathBuf>,
    transcripts: Option<PathBuf>,
    naming: bool,
    read_at: Option<Instant>,
    summary: Option<Summary>,
    memory: Option<Memory>,
    recalled: bool,
    follow: Option<Follow>,
    handoff: Option<Listener>,
    temporary: PathBuf,
    logging: logging::Logging,
}

#[derive(Clone, Copy, PartialEq)]
enum Question {
    End(Changes),
    Quit(Changes),
}

struct Entry {
    id: usize,
    name: String,
    session: Option<Session>,
    started_at: SystemTime,
    used_at: SystemTime,
    record: Option<Record>,
    place: Place,
    exited_at: Option<Instant>,
    transcript: Option<Transcript>,
    editors: Vec<Editor>,
    editor: Option<usize>,
    on_editor: bool,
    seen: PathBuf,
    followed: bool,
    directory: PathBuf,
    remembered: Option<String>,
    kept: Option<Remembered>,
    why: &'static str,
    reservation: Option<Reservation>,
}

impl Entry {
    fn editor(&self) -> Option<&Editor> {
        self.editor.and_then(|index| self.editors.get(index))
    }

    fn showing_editor(&self) -> Option<&Editor> {
        self.editor().filter(|_| self.on_editor)
    }

    fn ended(&self) -> bool {
        self.session.is_none() || self.exited_at.is_some()
    }

    fn shown_id(&self) -> usize {
        self.showing_editor().map_or(self.id, |editor| editor.id)
    }

    fn running(&mut self) -> bool {
        self.sessions().any(|session| session.exit().is_none())
    }

    fn sessions(&mut self) -> impl Iterator<Item = &mut Session> {
        self.session
            .iter_mut()
            .chain(self.editors.iter_mut().map(|editor| &mut editor.session))
    }

    fn shown(&self) -> Option<&Session> {
        match self.showing_editor() {
            Some(editor) => Some(&editor.session),
            None => self.session.as_ref(),
        }
    }

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
        let usage = usage::Usage::new(&command);
        App {
            command,
            home,
            picker: None,
            editor: None,
            picking: None,
            picking_area: Rect::default(),
            usage,
            screen: Rect::new(0, 0, 80, 24),
            sessions: Vec::new(),
            selected: 0,
            started: 0,
            named: 0,
            pane: Rect::new(0, 0, 80, 24),
            shown_pane: Rect::default(),
            sidebar: Rect::default(),
            cards: Sidebar::default(),
            accent: Color::Cyan,
            nerd_font: false,
            sidebar_width: None,
            pane_focused: false,
            gesture: Gesture::Idle,
            failed: None,
            asking: None,
            helping: false,
            quitting: false,
            shape: 0,
            cursor: None,
            registry: None,
            transcripts: None,
            naming: false,
            read_at: None,
            summary: None,
            memory: None,
            recalled: false,
            follow: None,
            handoff: None,
            temporary: follow::temporary(),
            logging: logging::Logging::default(),
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

    pub fn with_accent(mut self, accent: Color) -> App {
        self.accent = accent;
        self
    }

    pub fn with_nerd_font(mut self) -> App {
        self.nerd_font = true;
        self
    }

    pub fn with_sidebar_width(mut self, width: u16) -> App {
        self.sidebar_width = Some(width);
        self
    }

    pub fn with_editor(mut self, editor: Program) -> App {
        self.editor = Some(editor);
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

    pub fn naming_sessions(mut self) -> App {
        self.naming = true;
        self
    }

    pub fn with_summary(mut self, summary: Summary) -> App {
        self.summary = Some(summary);
        self
    }

    pub fn with_handoff(mut self, listener: Listener) -> App {
        self.handoff = Some(listener);
        self
    }

    pub fn with_mover(mut self, mover: Mover) -> App {
        self.follow = Some(Follow::new(mover));
        self
    }

    pub fn demo(registry: PathBuf) -> App {
        let mut app = App::new(demo::fresh())
            .with_home(demo::checkout(&registry, "acme-api", "main"))
            .with_registry(registry.clone())
            .with_transcripts(registry.join("projects"))
            .with_picker(demo::picker())
            .with_editor(demo::editor())
            .with_memory(demo::stopped(&registry));
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

    pub fn cursor_shape(&self) -> u8 {
        self.shape
    }

    pub fn cursor_position(&self) -> Option<Position> {
        self.cursor
    }

    pub fn running(&self) -> bool {
        !self.quitting
    }

    pub fn paste(&mut self, text: &str) {
        self.paste_handled(text);
        self.log_paste();
    }

    fn paste_handled(&mut self, text: &str) {
        if let Some(pick) = &self.picking {
            let bytes = pick.session().paste(text);
            self.log_input(Input::Picker, bytes);
            return;
        }
        if self.dismiss(false) {
            return;
        }
        if let Some(session) = self.focused() {
            let bytes = session.paste(text);
            self.log_input(Input::Shown, bytes);
            self.touch();
        }
    }

    pub fn mouse(&mut self, event: MouseEvent) {
        self.mouse_handled(event);
        self.log_changes();
    }

    fn mouse_handled(&mut self, event: MouseEvent) {
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
            .and_then(|entry| Some((entry.shown_id(), entry.shown()?)));
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
            self.touch();
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
            .and_then(Entry::shown)
    }

    fn finish_editors(&mut self) {
        let Some(program) = &self.editor else {
            return;
        };
        for entry in &mut self.sessions {
            let mut index = 0;
            while index < entry.editors.len() {
                let Some(status) = entry.editors[index].session.exit().cloned() else {
                    index += 1;
                    continue;
                };
                entry.editors.remove(index);
                entry.editor = match entry.editor {
                    Some(shown) if shown == index => None,
                    Some(shown) if shown > index => Some(shown - 1),
                    other => other,
                };
                entry.on_editor &= entry.editor.is_some();
                if !status.success() {
                    self.failed = Some(format!("{} exited{}", program.name(), exit_note(&status)));
                }
            }
        }
    }

    fn answer_yes(&mut self, question: Question) {
        match question {
            Question::End(_) => {
                let Some(entry) = self.sessions.get_mut(self.selected) else {
                    return;
                };
                let exited = entry
                    .session
                    .as_mut()
                    .is_none_or(|session| session.exit().is_some());
                session::end_all(entry.sessions());
                entry.editors.clear();
                entry.editor = None;
                entry.on_editor = false;
                if exited {
                    self.remove_selected();
                }
            }
            Question::Quit(_) => {
                self.settle_all();
                self.quitting = true;
            }
        }
    }

    fn remove_selected(&mut self) {
        let mut removed = self.sessions.remove(self.selected);
        removed.reservation = None;
        self.forget(&removed);
        self.selected = self.selected.min(self.sessions.len().saturating_sub(1));
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
        let transcript = self
            .transcripts
            .as_deref()
            .filter(|_| self.naming)
            .and_then(|projects| {
                let id = transcript::new_session_id().ok()?;
                Some(Transcript::new(projects, &id).since(started_at))
            });
        let mut command = command.to_vec();
        if let Some(transcript) = &transcript {
            command.extend([
                "--session-id".to_string(),
                transcript.session_id().to_string(),
            ]);
        }
        match Session::start(&command, directory, self.pane.height, self.pane.width) {
            Ok(session) => {
                self.failed = None;
                self.started += 1;
                self.named += 1;
                self.sessions.push(Entry {
                    id: self.started,
                    name: format!("{} {}", folder_name(directory), self.named),
                    session: Some(session),
                    started_at,
                    used_at: started_at,
                    record: None,
                    place: repo::place(directory, user_home().as_deref()),
                    exited_at: None,
                    transcript,
                    editors: Vec::new(),
                    editor: None,
                    on_editor: false,
                    seen: directory.to_path_buf(),
                    followed: false,
                    directory: directory.to_path_buf(),
                    remembered: None,
                    kept: None,
                    why: remember::STOPPED,
                    reservation: None,
                });
                self.selected = self.sessions.len() - 1;
            }
            Err(error) => {
                self.failed = Some(refusal(command.first().map_or("", String::as_str), &error))
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        session::end_all(self.sessions.iter_mut().flat_map(Entry::sessions));
    }
}

fn user_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
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
