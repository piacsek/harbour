use std::cell::Cell;
use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use portable_pty::{Child, CommandBuilder, ExitStatus, MasterPty, PtySize, native_pty_system};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::widgets::Widget;
use tui_term::widget::{Cursor, PseudoTerminal};

use crate::keys::{Asked, Keyboard};

pub const HANGUP_GRACE: Duration = Duration::from_millis(200);
const HANGUP_POLL: Duration = Duration::from_millis(5);
const HIDDEN_GRACE: Duration = Duration::from_millis(60);
const UPDATE_PATIENCE: Duration = Duration::from_millis(100);
const SYNCHRONIZED_OUTPUT: u16 = 2026;

type Parser = vt100::Parser<Replies>;

thread_local! {
    static CONTAINING: Cell<bool> = const { Cell::new(false) };
}

pub struct Session {
    parser: Arc<Mutex<Parser>>,
    writer: Sender<Vec<u8>>,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    exit: Option<ExitStatus>,
    drained: Arc<AtomicBool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Agent,
    Picker,
    Editor,
}

impl Session {
    pub fn start<S: AsRef<OsStr>>(
        command: &[S],
        directory: &Path,
        rows: u16,
        cols: u16,
    ) -> io::Result<Session> {
        Session::spawn(command, directory, rows, cols, Role::Agent)
    }

    pub fn start_as<S: AsRef<OsStr>>(
        role: Role,
        command: &[S],
        directory: &Path,
        rows: u16,
        cols: u16,
    ) -> io::Result<Session> {
        Session::spawn(command, directory, rows, cols, role)
    }

    fn spawn<S: AsRef<OsStr>>(
        command: &[S],
        directory: &Path,
        rows: u16,
        cols: u16,
        role: Role,
    ) -> io::Result<Session> {
        if !directory.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                format!("Not a directory: {}", directory.display()),
            ));
        }
        let (rows, cols) = safe_size(rows, cols);
        let pty = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| io::Error::other(format!("could not open a terminal: {error}")))?;
        let mut builder = CommandBuilder::new(command[0].as_ref());
        builder.args(&command[1..]);
        builder.cwd(directory);
        builder.env("PWD", directory);
        if role != Role::Editor {
            builder.env_remove("TMUX");
            builder.env_remove("TMUX_PANE");
        }
        builder.env("TERM", "xterm-256color");
        builder.env("COLORTERM", "truecolor");
        let child = pty
            .slave
            .spawn_command(builder)
            .map_err(|error| io::Error::other(spawn_reason(&error.to_string())))?;
        drop(pty.slave);
        let mut reader = pty.master.try_clone_reader().map_err(io::Error::other)?;
        let writer = spawn_writer(pty.master.take_writer().map_err(io::Error::other)?);
        let parser = Arc::new(Mutex::new(Parser::new_with_callbacks(
            rows,
            cols,
            0,
            Replies {
                asked: (role != Role::Agent).then(Asked::default),
                ..Replies::default()
            },
        )));
        let shared = Arc::clone(&parser);
        let replier = writer.clone();
        let drained = Arc::new(AtomicBool::new(false));
        let read_to_end = Arc::clone(&drained);
        thread::spawn(move || {
            let mut buffer = [0; 65536];
            let mut escaped = false;
            while let Ok(read @ 1..) = reader.read(&mut buffer) {
                let replies = {
                    let mut parser = shared.lock().unwrap();
                    let (rows, cols) = parser.screen().size();
                    for (segment, reset) in segments(&buffer[..read], escaped) {
                        contained(&mut parser, rows, cols, |parser| parser.process(segment));
                        if reset {
                            parser.callbacks_mut().forget_asked();
                        }
                    }
                    escaped = buffer[read - 1] == 0x1b;
                    let screen = parser.screen();
                    let (hidden, at) = (screen.hide_cursor(), screen.cursor_position());
                    parser.callbacks_mut().track(hidden, at, Instant::now());
                    std::mem::take(&mut parser.callbacks_mut().bytes)
                };
                if !replies.is_empty() {
                    let _ = replier.send(replies);
                }
            }
            shared.lock().unwrap().callbacks_mut().forget_asked();
            read_to_end.store(true, Ordering::Release);
        });
        Ok(Session {
            parser,
            writer,
            master: pty.master,
            child,
            exit: None,
            drained,
        })
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        if rows == 0 || cols == 0 || self.exit().is_some() {
            return;
        }
        let (rows, cols) = safe_size(rows, cols);
        let mut parser = self.parser.lock().unwrap();
        let repaint = std::mem::take(&mut parser.callbacks_mut().repaint);
        if parser.screen().size() == (rows, cols) && !repaint {
            return;
        }
        contained(&mut parser, rows, cols, |parser| {
            parser.screen_mut().set_size(rows, cols);
        });
        if repaint {
            self.resize_pty(rows, cols.saturating_sub(1).max(1));
        }
        self.resize_pty(rows, cols);
    }

    fn resize_pty(&self, rows: u16, cols: u16) {
        let _ = self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
    }

    pub fn exit(&mut self) -> Option<&ExitStatus> {
        if self.exit.is_none() {
            self.exit = self.child.try_wait().ok().flatten();
        }
        self.exit.as_ref()
    }

    pub fn pid(&self) -> Option<u32> {
        self.child.process_id()
    }

    pub fn end(&mut self) {
        end_all([self], HANGUP_GRACE);
    }

    fn group(&mut self) -> Option<Pid> {
        if self.exit().is_some() {
            return None;
        }
        self.child
            .process_id()
            .and_then(|pid| i32::try_from(pid).ok())
            .map(Pid::from_raw)
    }

    pub fn send(&self, bytes: &[u8]) {
        let _ = self.writer.send(bytes.to_vec());
    }

    pub fn paste(&self, text: &str) -> usize {
        let bytes = if self.parser.lock().unwrap().screen().bracketed_paste() {
            [b"\x1b[200~", text.as_bytes(), b"\x1b[201~"].concat()
        } else {
            text.as_bytes().to_vec()
        };
        self.send(&bytes);
        bytes.len()
    }

    pub fn mouse_protocol(&self) -> (vt100::MouseProtocolMode, vt100::MouseProtocolEncoding) {
        let parser = self.parser.lock().unwrap();
        let screen = parser.screen();
        (
            screen.mouse_protocol_mode(),
            screen.mouse_protocol_encoding(),
        )
    }

    pub fn application_cursor(&self) -> bool {
        self.parser.lock().unwrap().screen().application_cursor()
    }

    pub fn keyboard(&self) -> Keyboard {
        self.parser
            .lock()
            .unwrap()
            .callbacks()
            .asked
            .as_ref()
            .map_or(Keyboard::Legacy, Asked::keyboard)
    }

    pub fn draw(&self, area: Rect, buffer: &mut Buffer, now: Instant) -> (Option<Position>, u8) {
        let mut parser = self.parser.lock().unwrap();
        let (rows, cols) = parser.screen().size();
        contained(&mut parser, rows, cols, |parser| {
            PseudoTerminal::new(shown(parser, now))
                .cursor(Cursor::default().visibility(false))
                .render(area, buffer);
        });
        (cursor(&parser, area, now), parser.callbacks().shape)
    }

    pub fn drained(&self) -> bool {
        self.drained.load(Ordering::Acquire)
    }

    pub fn printed(&self) -> bool {
        !self.text().trim().is_empty()
    }

    pub fn text(&self) -> String {
        self.parser.lock().unwrap().screen().contents()
    }

    pub fn take_panics(&self) -> usize {
        std::mem::take(&mut self.parser.lock().unwrap().callbacks_mut().panics)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.end();
    }
}

pub fn end_all<'a>(sessions: impl IntoIterator<Item = &'a mut Session>, grace: Duration) {
    let mut running: Vec<(&mut Session, Pid)> = sessions
        .into_iter()
        .filter_map(|session| session.group().map(|group| (session, group)))
        .collect();
    if running.is_empty() {
        return;
    }
    for (_, group) in &running {
        let _ = killpg(*group, Signal::SIGHUP);
    }
    let until = Instant::now() + grace;
    loop {
        running.retain_mut(|(session, group)| {
            session.exit().is_none() || killpg(*group, None).is_ok()
        });
        let left = until.saturating_duration_since(Instant::now());
        if running.is_empty() || left.is_zero() {
            break;
        }
        thread::sleep(HANGUP_POLL.min(left));
    }
    for (session, group) in &mut running {
        let _ = killpg(*group, Signal::SIGKILL);
        if session.exit.is_none() {
            session.exit = session.child.wait().ok();
        }
    }
}

fn spawn_reason(text: &str) -> String {
    if text.contains("No viable candidates") && !text.contains("exists but") {
        return "is it installed and on PATH?".to_string();
    }
    text.lines()
        .filter(|line| !line.starts_with("No viable candidates"))
        .map(|line| {
            line.rsplit(" because ")
                .next()
                .unwrap_or(line)
                .trim_end_matches([':', '.'])
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn containing() -> bool {
    CONTAINING.get()
}

fn contained(parser: &mut Parser, rows: u16, cols: u16, work: impl FnOnce(&mut Parser)) {
    let modes = modes(parser.screen());
    CONTAINING.set(true);
    let failed = panic::catch_unwind(AssertUnwindSafe(|| work(parser))).is_err();
    CONTAINING.set(false);
    if failed {
        let panics = parser.callbacks_mut().panics + 1;
        let asked = parser.callbacks_mut().asked.clone();
        let shape = parser.callbacks().shape;
        *parser = Parser::new_with_callbacks(
            rows,
            cols,
            0,
            Replies {
                bytes: Vec::new(),
                repaint: true,
                panics,
                asked,
                shape,
                ..Replies::default()
            },
        );
        parser.process(&modes);
    }
}

fn modes(screen: &vt100::Screen) -> Vec<u8> {
    let mut modes = screen.input_mode_formatted();
    if screen.hide_cursor() {
        modes.extend_from_slice(b"\x1b[?25l");
    }
    modes
}

fn segments(bytes: &[u8], escaped: bool) -> Vec<(&[u8], bool)> {
    let mut found = Vec::new();
    let mut start = 0;
    for (index, &byte) in bytes.iter().enumerate() {
        let after_escape = match index {
            0 => escaped,
            _ => bytes[index - 1] == 0x1b,
        };
        if byte == b'c' && after_escape {
            found.push((&bytes[start..=index], true));
            start = index + 1;
        }
    }
    found.push((&bytes[start..], false));
    found
}

fn safe_size(rows: u16, cols: u16) -> (u16, u16) {
    (rows.max(2), cols.max(2))
}

fn spawn_writer(mut pty: Box<dyn Write + Send>) -> Sender<Vec<u8>> {
    let (writer, pending) = mpsc::channel::<Vec<u8>>();
    thread::spawn(move || {
        for bytes in pending {
            if pty.write_all(&bytes).and_then(|()| pty.flush()).is_err() {
                break;
            }
        }
    });
    writer
}

impl Widget for &Session {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        self.draw(area, buffer, Instant::now());
    }
}

fn shown(parser: &Parser, now: Instant) -> &vt100::Screen {
    parser
        .callbacks()
        .whole_frame(now)
        .unwrap_or_else(|| parser.screen())
}

fn cursor(parser: &Parser, area: Rect, now: Instant) -> Option<Position> {
    let screen = shown(parser, now);
    let (row, col) = if screen.hide_cursor() {
        parser.callbacks().lingering(now)?
    } else {
        screen.cursor_position()
    };
    let col = col.min(screen.size().1.saturating_sub(1));
    let at = Position::new(area.x.checked_add(col)?, area.y.checked_add(row)?);
    area.contains(at).then_some(at)
}

#[derive(Default)]
struct Replies {
    bytes: Vec<u8>,
    repaint: bool,
    panics: usize,
    asked: Option<Asked>,
    shape: u8,
    seen: Option<(u16, u16)>,
    hidden_at: Option<Instant>,
    whole: Option<(vt100::Screen, Instant)>,
}

impl Replies {
    fn track(&mut self, hidden: bool, at: (u16, u16), now: Instant) {
        if hidden {
            self.hidden_at.get_or_insert(now);
        } else {
            self.seen = Some(at);
            self.hidden_at = None;
        }
    }

    fn lingering(&self, now: Instant) -> Option<(u16, u16)> {
        let hidden = self.hidden_at?;
        (now.saturating_duration_since(hidden) < HIDDEN_GRACE).then_some(self.seen?)
    }

    fn whole_frame(&self, now: Instant) -> Option<&vt100::Screen> {
        let (screen, since) = self.whole.as_ref()?;
        (now.saturating_duration_since(*since) < UPDATE_PATIENCE).then_some(screen)
    }

    fn begin_update(&mut self, screen: &vt100::Screen) {
        let now = Instant::now();
        if self.whole_frame(now).is_none() {
            self.whole = Some((screen.clone(), now));
        }
    }

    fn forget_asked(&mut self) {
        if let Some(asked) = &mut self.asked {
            *asked = Asked::default();
        }
    }
}

impl vt100::Callbacks for Replies {
    fn unhandled_csi(
        &mut self,
        screen: &mut vt100::Screen,
        first: Option<u8>,
        second: Option<u8>,
        parameters: &[&[u16]],
        last: char,
    ) {
        let synchronized = parameters.contains(&[SYNCHRONIZED_OUTPUT].as_slice());
        match (first, last) {
            (Some(b'?'), 'h') if synchronized => self.begin_update(screen),
            (Some(b'?'), 'l') if synchronized => self.whole = None,
            (Some(b'?'), 'p') if synchronized && second == Some(b'$') => {
                let state = if self.whole.is_some() { 1 } else { 2 };
                self.bytes
                    .extend(format!("\x1b[?{SYNCHRONIZED_OUTPUT};{state}$y").as_bytes());
            }
            _ => {}
        }
        if let Some(reply) = self
            .asked
            .as_mut()
            .and_then(|asked| asked.csi(first, parameters, last))
        {
            self.bytes.extend(reply);
            return;
        }
        let parameter = parameters.first().and_then(|values| values.first());
        match (first, last) {
            (None | Some(b'?'), 'c') => self.bytes.extend(b"\x1b[?62;22c"),
            (Some(b'>'), 'c') => self.bytes.extend(b"\x1b[>0;10;1c"),
            (None, 'n') if parameter == Some(&6) => {
                let (row, col) = screen.cursor_position();
                self.bytes
                    .extend(format!("\x1b[{};{}R", row + 1, col + 1).as_bytes());
            }
            (None, 'n') if parameter == Some(&5) => self.bytes.extend(b"\x1b[0n"),
            (Some(b' '), 'q') => {
                if let Ok(shape @ 0..=6) = u8::try_from(parameter.copied().unwrap_or(0)) {
                    self.shape = shape;
                }
            }
            _ => {}
        }
    }

    fn unhandled_osc(&mut self, _: &mut vt100::Screen, parameters: &[&[u8]]) {
        match parameters {
            [b"10", b"?"] => self.bytes.extend(b"\x1b]10;rgb:d0d0/d0d0/d0d0\x1b\\"),
            [b"11", b"?"] => self.bytes.extend(b"\x1b]11;rgb:1c1c/1c1c/1c1c\x1b\\"),
            _ => {}
        }
    }
}
