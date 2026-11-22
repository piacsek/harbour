use std::cell::RefCell;
use std::collections::VecDeque;
use std::io;
use std::os::fd::AsFd;
use std::panic::PanicHookInfo;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event,
    KeyEventKind,
};
use crossterm::terminal::{BeginSynchronizedUpdate, EndSynchronizedUpdate};
use crossterm::{execute, queue};
use nix::errno::Errno;
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions};
use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::low_level::{emulate_default_handler, signal_name};

use crate::app::App;
use crate::config::{Config, Source};
use crate::cursor::Shape;
use crate::demo::Registry;
use crate::editor::Program;
use crate::handoff::{self, Listener};
use crate::input::Input;
use crate::log::{self, Log};
use crate::memory::Memory;
use crate::picker::Picker;
use crate::registry;
use crate::session;
use crate::summary::{self, Summary};

const EXTENDED_KEYS_ON: &[u8] = b"\x1b[>4;2m";
const EXTENDED_KEYS_OFF: &[u8] = b"\x1b[>4m";

type PanicHook = dyn Fn(&PanicHookInfo<'_>) + Send + Sync + 'static;

pub const SESSION_COMMAND: &str = "HARBOUR_SESSION_COMMAND";
pub const DIR_PICKER: &str = "HARBOUR_DIR_PICKER";
pub const EDITOR: &str = "HARBOUR_EDITOR";
pub const LOG: &str = "HARBOUR_LOG";

pub fn run(config: Config) -> io::Result<()> {
    let mut app = App::new(config.session_command().value.clone())
        .with_picker(dir_picker(&config))
        .with_editor(Program::command(config.editor().value.clone()))
        .with_accent(config.accent().value)
        .with_usage_place(summary::directory_from_env().map(|dir| dir.with_file_name("usage")));
    if let Some(width) = config.sidebar_width().value {
        app = app.with_sidebar_width(width);
    }
    if config.nerd_font().value {
        app = app.with_nerd_font();
    }
    if let Some(dir) = claude_sessions() {
        app = app.with_registry(dir);
    }
    if let Some(dir) = claude_projects() {
        app = app.with_transcripts(dir);
    }
    if config.names_sessions() {
        app = app.naming_sessions();
    }
    if let Some(summary) = published() {
        app = app.with_summary(summary);
    }
    if let Some(memory) = remembered() {
        app = app.with_memory(memory);
    }
    if let Some(dir) = handoff::directory_from_env() {
        app = app.with_handoff(Listener::new(&dir));
    }
    let log = logging();
    if let Some(log) = &log {
        app = app.with_log(log.clone());
    }
    let shown = show(app);
    if let Some(log) = log {
        log.finish(Duration::from_millis(500));
    }
    ended(shown)
}

pub fn demo() -> io::Result<()> {
    let registry = Registry::create_in(&std::env::temp_dir())?;
    let shown = show(App::demo(registry.path().to_path_buf()));
    drop(registry);
    ended(shown)
}

fn ended(shown: io::Result<Option<i32>>) -> io::Result<()> {
    if let Some(signal) = shown? {
        emulate_default_handler(signal)?;
    }
    Ok(())
}

fn show(app: App) -> io::Result<Option<i32>> {
    let caught = caught()?;
    let app = app.with_mover(Box::new(|directory| std::env::set_current_dir(directory)));
    std::mem::forget(ratatui::init());
    let shape = Shape::default();
    std::panic::set_hook(panic_hook(shape.clone(), std::panic::take_hook()));
    let result = enter_terminal(&mut io::stdout())
        .and_then(|()| Frames::new(TerminalOptions::default()))
        .and_then(|mut frames| drive(&mut frames, app, shape.clone(), &caught));
    let _ = leave_terminal(&mut io::stdout(), &shape);
    let _ = ratatui::try_restore();
    result.map(|ended| ended.or_else(|| signalled(&caught)))
}

fn caught() -> io::Result<Arc<AtomicUsize>> {
    let caught = Arc::new(AtomicUsize::new(0));
    let armed = Arc::new(AtomicBool::new(false));
    for signal in [SIGINT, SIGTERM] {
        signal_hook::flag::register_conditional_default(signal, Arc::clone(&armed))?;
        signal_hook::flag::register_usize(signal, Arc::clone(&caught), number(signal)?)?;
        signal_hook::flag::register(signal, Arc::clone(&armed))?;
    }
    signal_hook::flag::register_usize(SIGHUP, Arc::clone(&caught), number(SIGHUP)?)?;
    Ok(caught)
}

fn number(signal: i32) -> io::Result<usize> {
    usize::try_from(signal).map_err(io::Error::other)
}

pub fn hung_up(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::UnexpectedEof || error.raw_os_error() == Some(Errno::EIO as i32)
}

pub fn settled(
    app: &mut App,
    stepped: io::Result<()>,
    caught: &AtomicUsize,
) -> io::Result<Option<i32>> {
    if let Some(signal) = signalled(caught) {
        app.stop(&named(signal));
        return Ok(Some(signal));
    }
    match stepped {
        Ok(()) => Ok(None),
        Err(error) if hung_up(&error) => {
            app.stop("hangup");
            Ok(Some(SIGHUP))
        }
        Err(error) => {
            app.stop("error");
            Err(error)
        }
    }
}

fn signalled(caught: &AtomicUsize) -> Option<i32> {
    match caught.load(Ordering::Relaxed) {
        0 => None,
        signal => i32::try_from(signal).ok(),
    }
}

fn named(signal: i32) -> String {
    let name = signal_name(signal).unwrap_or("SIG?");
    format!("signal name={}", name.trim_start_matches("SIG"))
}

pub fn panic_hook(shape: Shape, restore_terminal: Box<PanicHook>) -> Box<PanicHook> {
    Box::new(move |info| {
        if session::containing() {
            return;
        }
        let _ = leave_terminal(&mut io::stdout(), &shape);
        restore_terminal(info);
    })
}

pub fn enter_terminal(out: &mut impl io::Write) -> io::Result<()> {
    execute!(out, EnableBracketedPaste, EnableMouseCapture)?;
    out.write_all(EXTENDED_KEYS_ON)?;
    out.flush()
}

pub fn leave_terminal(out: &mut impl io::Write, shape: &Shape) -> io::Result<()> {
    shape.leave(out)?;
    execute!(out, DisableMouseCapture, DisableBracketedPaste, Show)?;
    out.write_all(EXTENDED_KEYS_OFF)?;
    out.flush()
}

fn drive(
    frames: &mut Frames,
    mut app: App,
    mut shape: Shape,
    caught: &AtomicUsize,
) -> io::Result<Option<i32>> {
    let stdin = io::stdin();
    let mut input = Input::default();
    let mut pending = VecDeque::new();
    while app.running() {
        let stepped = step(
            frames,
            &mut app,
            &mut shape,
            &mut input,
            &mut pending,
            &stdin,
        );
        if let Some(signal) = settled(&mut app, stepped, caught)? {
            return Ok(Some(signal));
        }
    }
    Ok(None)
}

fn step(
    frames: &mut Frames,
    app: &mut App,
    shape: &mut Shape,
    input: &mut Input,
    pending: &mut VecDeque<Event>,
    stdin: &io::Stdin,
) -> io::Result<()> {
    frames.draw(app, shape, &mut io::stdout().lock())?;
    if pending.is_empty() {
        pending.extend(input.read(stdin.as_fd(), Duration::from_millis(30))?);
    }
    match pending.pop_front() {
        Some(Event::Key(key)) if key.kind != KeyEventKind::Release => app.key(key),
        Some(Event::Paste(text)) => app.paste(&text),
        Some(Event::Mouse(mouse)) => app.mouse(mouse),
        _ => {}
    }
    Ok(())
}

#[derive(Clone, Default)]
struct Held(Rc<RefCell<Vec<u8>>>);

impl io::Write for Held {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct Frames {
    held: Held,
    terminal: Terminal<CrosstermBackend<Held>>,
}

impl Frames {
    pub fn new(options: TerminalOptions) -> io::Result<Frames> {
        let held = Held::default();
        let terminal = Terminal::with_options(CrosstermBackend::new(held.clone()), options)?;
        Ok(Frames { held, terminal })
    }

    pub fn draw(
        &mut self,
        app: &mut App,
        shape: &mut Shape,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let mut held = self.held.clone();
        held.0.borrow_mut().clear();
        queue!(held, BeginSynchronizedUpdate, Hide)?;
        self.terminal.try_draw(|frame| {
            app.draw(frame);
            shape.send(&mut held, app.cursor_shape())
        })?;
        if let Some(at) = app.cursor_position() {
            queue!(held, MoveTo(at.x, at.y), Show)?;
        }
        queue!(held, EndSynchronizedUpdate)?;
        out.write_all(&held.0.take())?;
        out.flush()
    }
}

fn dir_picker(config: &Config) -> Picker {
    let picker = config.picker();
    let fzf = std::path::Path::new(&picker.value[0]).file_name() == Some("fzf".as_ref());
    match &picker.source {
        Source::Variable(_) => Picker::Command(picker.value.clone()),
        Source::File if !fzf => Picker::Command(picker.value.clone()),
        source => Picker::Fuzzy {
            fzf: picker.value.clone(),
            list: ["zoxide", "query", "--list"].map(String::from).to_vec(),
            here_first: true,
            required: *source == Source::File,
        },
    }
}

fn claude_sessions() -> Option<std::path::PathBuf> {
    registry::sessions_dir(
        std::env::var_os("CLAUDE_CONFIG_DIR").map(Into::into),
        std::env::var_os("HOME").map(Into::into),
    )
    .and_then(|dir| std::path::absolute(dir).ok())
}

fn claude_projects() -> Option<std::path::PathBuf> {
    registry::projects_dir(
        std::env::var_os("CLAUDE_CONFIG_DIR").map(Into::into),
        std::env::var_os("HOME").map(Into::into),
    )
    .and_then(|dir| std::path::absolute(dir).ok())
}

fn logging() -> Option<Log> {
    std::env::var_os(LOG).filter(|value| !value.is_empty())?;
    log::directory_from_env().map(|dir| Log::open(&dir))
}

fn remembered() -> Option<Memory> {
    let state = summary::directory_from_env()?;
    Some(Memory::of(
        &state.with_file_name("sessions"),
        &claude_projects()?,
    ))
}

fn published() -> Option<Summary> {
    summary::directory_from_env().map(|dir| Summary::new(&dir))
}
