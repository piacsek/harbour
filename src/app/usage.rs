use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::widgets::Clear;

use super::logging::Input;
use super::{App, draw, refusal};
use crate::mouse::{self, Gesture, Target};
use crate::session::{Role, Session};

const ROWS: u16 = 200;

pub(super) struct Usage {
    command: Result<Vec<String>, String>,
    open: Option<Session>,
    area: Rect,
}

impl Usage {
    pub(super) fn new(session_command: &[String]) -> Usage {
        Usage {
            command: derived(session_command),
            open: None,
            area: Rect::default(),
        }
    }
}

impl App {
    pub fn with_usage(mut self, command: Vec<String>) -> App {
        self.usage.command = Ok(command);
        self
    }

    pub(super) fn open_usage(&mut self) {
        let command = match &self.usage.command {
            Ok(command) => command.clone(),
            Err(why) => {
                self.failed = Some(why.clone());
                return;
            }
        };
        let width = draw::bordered(true, self.accent)
            .inner(area(self.screen))
            .width;
        let directory = self.usage_directory();
        match Session::start_as(Role::Picker, &command, &directory, ROWS, width) {
            Ok(session) => {
                self.failed = None;
                self.gesture = Gesture::Idle;
                self.usage.open = Some(session);
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
            || (key.code == KeyCode::Char('q')
                && key.modifiers - KeyModifiers::SHIFT == KeyModifiers::NONE)
        {
            self.close_usage();
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
        let block = draw::bordered(true, self.accent).title(" usage ");
        let inner = block.inner(area);
        self.usage.area = inner;
        session.resize(ROWS, inner.width);
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
        frame.render_widget(&*session, inner);
        true
    }

    fn close_usage(&mut self) {
        self.usage.open = None;
        self.gesture = Gesture::Idle;
    }

    fn usage_directory(&self) -> PathBuf {
        self.sessions
            .get(self.selected)
            .map(|entry| entry.seen.clone())
            .filter(|seen| seen.is_dir())
            .unwrap_or_else(|| self.home.clone())
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
            "/usage".to_string(),
        ])
    } else {
        Err(format!(
            "Usage needs Claude Code; the session command runs {program}"
        ))
    }
}
