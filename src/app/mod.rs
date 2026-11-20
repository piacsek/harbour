use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};

mod draw;

use draw::exit_note;

use crate::demo;
use crate::editor::{self, Editor, Program};
use crate::keys;
use crate::mouse::{self, Gesture, Target};
use crate::picker::{Pick, Picked, Picker};
use crate::registry::{self, Record};
use crate::session::{self, Session};
use crate::summary::Summary;
use crate::transcript::Transcript;

const NARROW: u16 = 60;
const HINT: &str = "No session yet: press n to start one.";
const READ_EVERY: Duration = Duration::from_millis(500);

pub struct App {
    command: Vec<String>,
    home: PathBuf,
    picker: Option<Picker>,
    editor: Option<Program>,
    transcripts: Option<PathBuf>,
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
    summary: Option<Summary>,
}

#[derive(Clone, Copy, PartialEq)]
enum Question {
    End { unsaved: bool },
    Quit { unsaved: bool },
}

struct Entry {
    id: usize,
    name: String,
    directory: PathBuf,
    session: Session,
    started_at: SystemTime,
    record: Option<Record>,
    transcript: Option<Transcript>,
    editors: Vec<Editor>,
    editor: Option<usize>,
    on_editor: bool,
}

impl Entry {
    fn editor(&self) -> Option<&Editor> {
        self.editor.and_then(|index| self.editors.get(index))
    }

    fn showing_editor(&self) -> Option<&Editor> {
        self.editor().filter(|_| self.on_editor)
    }

    fn shown_id(&self) -> usize {
        self.showing_editor().map_or(self.id, |editor| editor.id)
    }

    fn running(&mut self) -> bool {
        self.sessions().any(|session| session.exit().is_none())
    }

    fn sessions(&mut self) -> impl Iterator<Item = &mut Session> {
        std::iter::once(&mut self.session)
            .chain(self.editors.iter_mut().map(|editor| &mut editor.session))
    }

    fn shown(&self) -> &Session {
        match self.showing_editor() {
            Some(editor) => &editor.session,
            None => &self.session,
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
            editor: None,
            transcripts: None,
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
            summary: None,
        }
    }

    pub fn with_picker(mut self, picker: Picker) -> App {
        self.picker = Some(picker);
        self
    }

    pub fn with_editor(mut self, editor: Program) -> App {
        self.editor = Some(editor);
        self
    }

    pub fn with_transcripts(mut self, projects: PathBuf) -> App {
        self.transcripts = Some(projects);
        self
    }

    pub fn with_registry(mut self, dir: PathBuf) -> App {
        self.registry = Some(dir);
        self
    }

    pub fn with_summary(mut self, summary: Summary) -> App {
        self.summary = Some(summary);
        self
    }

    pub fn demo(registry: PathBuf) -> App {
        let mut app = App::new(demo::working(&registry))
            .with_registry(registry.clone())
            .with_picker(demo::picker())
            .with_editor(demo::editor());
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
            match self.sessions.get_mut(self.selected) {
                Some(entry) if entry.on_editor => {
                    if entry.showing_editor().is_some_and(Editor::wants_ctrl_h) {
                        entry.shown().send(&keys::encode(key, false));
                    } else {
                        entry.on_editor = false;
                    }
                }
                _ => self.pane_focused = false,
            }
            return;
        }
        if self.pane_focused
            && is_ctrl(key, 'l')
            && self
                .sessions
                .get(self.selected)
                .is_some_and(|entry| !entry.on_editor)
        {
            self.open_editor();
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
            let (mode, encoding) = entry.shown().mouse_protocol();
            Target {
                session: entry.shown_id(),
                mode,
                encoding,
            }
        });
        let (gesture, bytes) = mouse::route(self.gesture, event, self.shown_pane, target);
        self.gesture = gesture;
        if let (Some(entry), Some(bytes)) = (entry, bytes) {
            entry.shown().send(&bytes);
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
            .map(Entry::shown)
    }

    fn open_editor(&mut self) {
        let Some(program) = &self.editor else {
            return;
        };
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return;
        };
        let at = entry
            .transcript
            .as_mut()
            .and_then(Transcript::directory)
            .unwrap_or_else(|| entry.directory.clone());
        let directory = editor::root(&at);
        if let Some(index) = entry
            .editors
            .iter()
            .position(|editor| editor.directory == directory)
        {
            entry.editor = Some(index);
            entry.on_editor = true;
            return;
        }
        self.started += 1;
        match Editor::start(
            program,
            &directory,
            self.started,
            self.pane.height,
            self.pane.width,
        ) {
            Ok(editor) => {
                self.failed = None;
                entry.editors.push(editor);
                entry.editor = Some(entry.editors.len() - 1);
                entry.on_editor = true;
            }
            Err(error) => self.failed = Some(refusal(program.name(), &error)),
        }
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

    fn end_or_remove(&mut self) {
        let Some(entry) = self.sessions.get_mut(self.selected) else {
            return;
        };
        if entry.running() {
            let unsaved = entry.editors.iter().any(Editor::unsaved);
            self.asking = Some(Question::End { unsaved });
        } else {
            self.remove_selected();
        }
    }

    fn quit(&mut self) {
        if self.sessions.iter_mut().any(Entry::running) {
            let unsaved = self
                .sessions
                .iter()
                .flat_map(|entry| &entry.editors)
                .any(Editor::unsaved);
            self.asking = Some(Question::Quit { unsaved });
        } else {
            self.quitting = true;
        }
    }

    fn answer_yes(&mut self, question: Question) {
        match question {
            Question::End { .. } => {
                let Some(entry) = self.sessions.get_mut(self.selected) else {
                    return;
                };
                let exited = entry.session.exit().is_some();
                session::end_all(entry.sessions());
                entry.editors.clear();
                entry.editor = None;
                entry.on_editor = false;
                if exited {
                    self.remove_selected();
                }
            }
            Question::Quit { .. } => self.quitting = true,
        }
    }

    fn remove_selected(&mut self) {
        self.sessions.remove(self.selected);
        self.selected = self.selected.min(self.sessions.len().saturating_sub(1));
    }

    fn pick_or_start(&mut self) {
        let Some(picker) = &self.picker else {
            self.start();
            return;
        };
        let popup = draw::popup(self.screen);
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
        let transcript = self
            .transcripts
            .as_deref()
            .and_then(|projects| Transcript::new(projects).ok());
        let mut command = command.to_vec();
        if let Some(transcript) = &transcript {
            command.extend(["--session-id".to_string(), transcript.id().to_string()]);
        }
        match Session::start(&command, directory, self.pane.height, self.pane.width) {
            Ok(session) => {
                self.failed = None;
                self.started += 1;
                self.sessions.push(Entry {
                    id: self.started,
                    name: format!("{} {}", folder_name(directory), self.started),
                    directory: directory.to_path_buf(),
                    session,
                    started_at,
                    record: None,
                    transcript,
                    editors: Vec::new(),
                    editor: None,
                    on_editor: false,
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
}

impl Drop for App {
    fn drop(&mut self) {
        session::end_all(self.sessions.iter_mut().flat_map(Entry::sessions));
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
