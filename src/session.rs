use std::cell::Cell;
use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use portable_pty::{Child, CommandBuilder, ExitStatus, MasterPty, PtySize, native_pty_system};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use tui_term::widget::PseudoTerminal;

const HANGUP_GRACE: Duration = Duration::from_millis(200);

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
}

impl Session {
    pub fn start<S: AsRef<OsStr>>(
        command: &[S],
        directory: &Path,
        rows: u16,
        cols: u16,
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
        builder.env_remove("TMUX");
        builder.env_remove("TMUX_PANE");
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
            Replies::default(),
        )));
        let shared = Arc::clone(&parser);
        let replier = writer.clone();
        thread::spawn(move || {
            let mut buffer = [0; 65536];
            while let Ok(read @ 1..) = reader.read(&mut buffer) {
                let replies = {
                    let mut parser = shared.lock().unwrap();
                    let (rows, cols) = parser.screen().size();
                    contained(&mut parser, rows, cols, |parser| {
                        parser.process(&buffer[..read]);
                    });
                    std::mem::take(&mut parser.callbacks_mut().bytes)
                };
                if !replies.is_empty() {
                    let _ = replier.send(replies);
                }
            }
        });
        Ok(Session {
            parser,
            writer,
            master: pty.master,
            child,
            exit: None,
        })
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        if rows == 0 || cols == 0 {
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
        end_all([self]);
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

    pub fn paste(&self, text: &str) {
        if self.parser.lock().unwrap().screen().bracketed_paste() {
            self.send(&[b"\x1b[200~", text.as_bytes(), b"\x1b[201~"].concat());
        } else {
            self.send(text.as_bytes());
        }
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

    pub fn take_panics(&self) -> usize {
        std::mem::take(&mut self.parser.lock().unwrap().callbacks_mut().panics)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.end();
    }
}

pub fn end_all<'a>(sessions: impl IntoIterator<Item = &'a mut Session>) {
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
    thread::sleep(HANGUP_GRACE);
    for (session, group) in &mut running {
        let _ = killpg(*group, Signal::SIGKILL);
        session.exit = session.child.wait().ok();
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
        *parser = Parser::new_with_callbacks(
            rows,
            cols,
            0,
            Replies {
                bytes: Vec::new(),
                repaint: true,
                panics,
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
        let mut parser = self.parser.lock().unwrap();
        let (rows, cols) = parser.screen().size();
        contained(&mut parser, rows, cols, |parser| {
            PseudoTerminal::new(parser.screen()).render(area, buffer);
        });
    }
}

#[derive(Default)]
struct Replies {
    bytes: Vec<u8>,
    repaint: bool,
    panics: usize,
}

impl vt100::Callbacks for Replies {
    fn unhandled_csi(
        &mut self,
        screen: &mut vt100::Screen,
        first: Option<u8>,
        _: Option<u8>,
        parameters: &[&[u16]],
        last: char,
    ) {
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
