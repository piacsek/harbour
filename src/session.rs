use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use crossterm::event::MouseEventKind;
use portable_pty::{Child, CommandBuilder, ExitStatus, MasterPty, PtySize, native_pty_system};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use tui_term::widget::PseudoTerminal;

use crate::keys;

type Parser = vt100::Parser<Replies>;

type Writer = Arc<Mutex<Box<dyn Write + Send>>>;

pub struct Session {
    parser: Arc<Mutex<Parser>>,
    writer: Writer,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    exit: Option<ExitStatus>,
}

impl Session {
    pub fn start(command: &[String], rows: u16, cols: u16) -> io::Result<Session> {
        let (rows, cols) = (rows.max(1), cols.max(1));
        let pty = native_pty_system()
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(io::Error::other)?;
        let mut builder = CommandBuilder::new(&command[0]);
        builder.args(&command[1..]);
        builder.cwd(std::env::current_dir()?);
        builder.env_remove("TMUX");
        builder.env_remove("TMUX_PANE");
        builder.env("TERM", "xterm-256color");
        builder.env("COLORTERM", "truecolor");
        let child = pty.slave.spawn_command(builder).map_err(io::Error::other)?;
        drop(pty.slave);
        let mut reader = pty.master.try_clone_reader().map_err(io::Error::other)?;
        let writer: Writer = Arc::new(Mutex::new(
            pty.master.take_writer().map_err(io::Error::other)?,
        ));
        let parser = Arc::new(Mutex::new(Parser::new_with_callbacks(
            rows,
            cols,
            0,
            Replies::default(),
        )));
        let shared = Arc::clone(&parser);
        let replier = Arc::clone(&writer);
        thread::spawn(move || {
            let mut buffer = [0; 65536];
            while let Ok(read @ 1..) = reader.read(&mut buffer) {
                let replies = {
                    let mut parser = shared.lock().unwrap();
                    parser.process(&buffer[..read]);
                    std::mem::take(&mut parser.callbacks_mut().bytes)
                };
                if !replies.is_empty() {
                    write(&replier, &replies);
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
        let mut parser = self.parser.lock().unwrap();
        if rows == 0 || cols == 0 || parser.screen().size() == (rows, cols) {
            return;
        }
        parser.screen_mut().set_size(rows, cols);
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

    pub fn end(&mut self) {
        let _ = self.child.kill();
    }

    pub fn send(&self, bytes: &[u8]) {
        write(&self.writer, bytes);
    }

    pub fn paste(&self, text: &str) {
        if self.parser.lock().unwrap().screen().bracketed_paste() {
            self.send(&[b"\x1b[200~", text.as_bytes(), b"\x1b[201~"].concat());
        } else {
            self.send(text.as_bytes());
        }
    }

    pub fn mouse(&self, kind: MouseEventKind, column: u16, row: u16) {
        let mode = self.parser.lock().unwrap().screen().mouse_protocol_mode();
        if mode != vt100::MouseProtocolMode::None
            && let Some(bytes) = keys::mouse(kind, column, row)
        {
            self.send(&bytes);
        }
    }

    pub fn application_cursor(&self) -> bool {
        self.parser.lock().unwrap().screen().application_cursor()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.exit().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

fn write(writer: &Writer, bytes: &[u8]) {
    let mut writer = writer.lock().unwrap();
    let _ = writer.write_all(bytes).and_then(|()| writer.flush());
}

impl Widget for &Session {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        PseudoTerminal::new(self.parser.lock().unwrap().screen()).render(area, buffer);
    }
}

#[derive(Default)]
struct Replies {
    bytes: Vec<u8>,
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
