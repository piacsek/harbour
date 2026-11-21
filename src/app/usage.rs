use std::ffi::OsStr;
use std::fs::{self, DirBuilder, Permissions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use nix::unistd::Uid;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::{Clear, Paragraph};

use super::logging::Input;
use super::{App, draw, refusal};
use crate::mouse::{self, Gesture, Target};
use crate::session::{Role, Session};

const ROWS: u16 = 200;
const HELD: Duration = Duration::from_millis(500);
const NO_PLACE: &str = "Usage needs a state folder; set XDG_STATE_HOME or HOME";

pub(super) struct Usage {
    command: Result<Vec<String>, String>,
    place: Option<PathBuf>,
    open: Option<Session>,
    exited: Option<Instant>,
    area: Rect,
    bars: bool,
    raw: bool,
}

impl Usage {
    pub(super) fn new(session_command: &[String]) -> Usage {
        Usage {
            command: derived(session_command),
            place: None,
            open: None,
            exited: None,
            area: Rect::default(),
            bars: false,
            raw: false,
        }
    }
}

impl App {
    pub fn with_usage(mut self, command: Vec<String>) -> App {
        self.usage.command = Ok(command);
        self
    }

    pub fn with_usage_place(mut self, place: Option<PathBuf>) -> App {
        self.usage.place = place;
        self
    }

    pub(super) fn usage_open(&self) -> bool {
        self.usage.open.is_some()
    }

    pub(super) fn open_usage(&mut self) {
        let command = match &self.usage.command {
            Ok(command) => command.clone(),
            Err(why) => {
                self.failed = Some(why.clone());
                return;
            }
        };
        let Some(directory) = self.usage.place.clone() else {
            self.failed = Some(NO_PLACE.to_string());
            return;
        };
        if let Err(error) = private(&directory) {
            self.failed = Some(format!("Could not make {}: {error}", directory.display()));
            return;
        }
        let width = draw::bordered(true, self.accent)
            .inner(area(self.screen))
            .width;
        match Session::start_as(Role::Picker, &command, &directory, ROWS, width) {
            Ok(session) => {
                self.failed = None;
                self.gesture = Gesture::Idle;
                self.usage.open = Some(session);
                self.usage.exited = None;
                self.usage.bars = false;
                self.usage.raw = false;
            }
            Err(error) => {
                self.failed = Some(refusal(command.first().map_or("", String::as_str), &error));
            }
        }
    }

    pub(super) fn usage_key(&mut self, key: KeyEvent) -> bool {
        let Some(session) = &self.usage.open else {
            return false;
        };
        if key.code == KeyCode::Esc
            || (matches!(key.code, KeyCode::Char('q' | 'Q'))
                && key.modifiers - KeyModifiers::SHIFT == KeyModifiers::NONE)
        {
            self.close_usage();
            self.log_usage_closed();
            return true;
        }
        if self.usage.bars
            && matches!(key.code, KeyCode::Char('t' | 'T'))
            && key.modifiers - KeyModifiers::SHIFT == KeyModifiers::NONE
        {
            self.usage.raw = !self.usage.raw;
            return true;
        }
        let bytes = super::key::beside(session, key);
        session.send(&bytes);
        self.log_input(Input::Usage, bytes.len());
        true
    }

    pub(super) fn usage_paste(&mut self, text: &str) -> bool {
        let Some(session) = &self.usage.open else {
            return false;
        };
        let bytes = session.paste(text);
        self.log_input(Input::Usage, bytes);
        true
    }

    pub(super) fn usage_mouse(&mut self, event: MouseEvent) -> bool {
        let Some(session) = &self.usage.open else {
            return false;
        };
        let (mode, encoding) = session.mouse_protocol();
        let target = Target {
            session: 0,
            mode,
            encoding,
        };
        let (gesture, bytes) = mouse::route(self.gesture, event, self.usage.area, Some(target));
        self.gesture = gesture;
        if let Some(bytes) = bytes {
            session.send(&bytes);
        }
        true
    }

    pub(super) fn draw_usage(&mut self, frame: &mut Frame) -> bool {
        let Some(session) = &mut self.usage.open else {
            return false;
        };
        let area = area(frame.area());
        let mut block = draw::bordered(true, self.accent).title(" usage ");
        let drained = session.drained();
        let exit = session.exit().cloned();
        if exit.is_some() && self.usage.exited.is_none() {
            self.usage.exited = Some(Instant::now());
        }
        let held = self.usage.exited.is_some_and(|at| at.elapsed() >= HELD);
        let finished = exit.filter(|_| drained || held);
        if let Some(status) = finished.as_ref().filter(|status| !status.success()) {
            block = block.title(format!(" exit{} ", draw::exit_note(status)));
        }
        let report = finished
            .as_ref()
            .and_then(|_| crate::usage::parse(&session.text()));
        self.usage.bars = report.is_some();
        let report = report.filter(|_| !self.usage.raw);
        if self.usage.bars && self.usage.raw {
            block = block.title_bottom(" t bars ");
        }
        let inner = block.inner(area);
        self.usage.area = inner;
        let printed = session.printed();
        if !printed {
            session.resize(ROWS, inner.width);
        }
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
        if let Some(report) = report {
            let lines = crate::usage::lines(&report, inner.width, inner.height, self.accent);
            frame.render_widget(Paragraph::new(lines), inner);
        } else if printed {
            frame.render_widget(&*session, inner);
        } else {
            let note = if finished.is_some() {
                "printed nothing"
            } else {
                "loading…"
            };
            frame.render_widget(Paragraph::new(note), inner);
        }
        true
    }

    fn close_usage(&mut self) {
        self.usage.open = None;
        self.usage.exited = None;
        self.gesture = Gesture::Idle;
    }
}

fn area(screen: Rect) -> Rect {
    screen.centered(
        Constraint::Length(screen.width.min(80)),
        Constraint::Length(screen.height.min(30)),
    )
}

fn derived(session_command: &[String]) -> Result<Vec<String>, String> {
    let program = session_command.first().map_or("", String::as_str);
    if Path::new(program).file_name() == Some(OsStr::new("claude")) {
        Ok(vec![
            program.to_string(),
            "-p".to_string(),
            "--no-session-persistence".to_string(),
            "--setting-sources".to_string(),
            "user".to_string(),
            "/usage".to_string(),
        ])
    } else {
        Err(format!(
            "Usage needs Claude Code; the session command runs {program}"
        ))
    }
}

fn private(directory: &Path) -> io::Result<()> {
    DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(directory)?;
    let meta = fs::symlink_metadata(directory)?;
    if !meta.is_dir() || meta.uid() != Uid::current().as_raw() {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    fs::set_permissions(directory, Permissions::from_mode(0o700))
}
