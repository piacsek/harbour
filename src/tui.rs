use std::io;
use std::panic::PanicHookInfo;
use std::time::Duration;

use crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event, KeyEventKind,
};
use crossterm::execute;

use crate::app::App;
use crate::demo::Registry;
use crate::editor::Program;
use crate::picker::Picker;
use crate::registry;
use crate::session;
use crate::summary::{self, Summary};

type PanicHook = dyn Fn(&PanicHookInfo<'_>) + Send + Sync + 'static;

pub const SESSION_COMMAND: &str = "HARBOUR_SESSION_COMMAND";
pub const DIR_PICKER: &str = "HARBOUR_DIR_PICKER";
pub const EDITOR: &str = "HARBOUR_EDITOR";

pub fn run() -> io::Result<()> {
    let mut app = App::new(session_command())
        .with_picker(dir_picker())
        .with_editor(editor());
    if let Some(dir) = claude_sessions() {
        app = app.with_registry(dir);
    }
    if let Some(dir) = claude_projects() {
        app = app.with_transcripts(dir);
    }
    if std::env::var_os(SESSION_COMMAND).is_none() {
        app = app.naming_sessions();
    }
    if let Some(summary) = published() {
        app = app.with_summary(summary);
    }
    show(app)
}

pub fn demo() -> io::Result<()> {
    let registry = Registry::create_in(&std::env::temp_dir())?;
    show(App::demo(registry.path().to_path_buf()))
}

fn show(app: App) -> io::Result<()> {
    let app = app.with_mover(Box::new(|directory| std::env::set_current_dir(directory)));
    let mut terminal = ratatui::init();
    std::panic::set_hook(panic_hook(std::panic::take_hook()));
    let result = execute!(io::stdout(), EnableBracketedPaste, EnableMouseCapture)
        .and_then(|()| drive(&mut terminal, app));
    let _ = leave_terminal(&mut io::stdout());
    ratatui::restore();
    result
}

pub fn panic_hook(restore_terminal: Box<PanicHook>) -> Box<PanicHook> {
    Box::new(move |info| {
        if session::containing() {
            return;
        }
        let _ = leave_terminal(&mut io::stdout());
        restore_terminal(info);
    })
}

pub fn leave_terminal(out: &mut impl io::Write) -> io::Result<()> {
    execute!(out, DisableMouseCapture, DisableBracketedPaste)
}

fn drive(terminal: &mut ratatui::DefaultTerminal, mut app: App) -> io::Result<()> {
    while app.running() {
        terminal.draw(|frame| app.draw(frame))?;
        if event::poll(Duration::from_millis(30))? {
            match event::read()? {
                Event::Key(key) if key.kind != KeyEventKind::Release => app.key(key),
                Event::Paste(text) => app.paste(&text),
                Event::Mouse(mouse) => app.mouse(mouse),
                _ => {}
            }
        }
    }
    Ok(())
}

fn dir_picker() -> Picker {
    match std::env::var(DIR_PICKER) {
        Ok(program) if !program.is_empty() => Picker::Command(vec![program]),
        _ => Picker::Fuzzy {
            fzf: vec!["fzf".to_string()],
            list: ["zoxide", "query", "--list"].map(String::from).to_vec(),
            here_first: true,
        },
    }
}

fn editor() -> Program {
    match std::env::var_os(EDITOR) {
        Some(program) if !program.is_empty() => Program::named(program),
        _ => Program::named("nvim".into()),
    }
}

fn session_command() -> Vec<String> {
    vec![std::env::var(SESSION_COMMAND).unwrap_or_else(|_| "claude".to_string())]
}

fn claude_sessions() -> Option<std::path::PathBuf> {
    registry::sessions_dir(
        std::env::var_os("CLAUDE_CONFIG_DIR").map(Into::into),
        std::env::var_os("HOME").map(Into::into),
    )
}

fn claude_projects() -> Option<std::path::PathBuf> {
    registry::projects_dir(
        std::env::var_os("CLAUDE_CONFIG_DIR").map(Into::into),
        std::env::var_os("HOME").map(Into::into),
    )
}

fn published() -> Option<Summary> {
    summary::directory_from_env().map(|dir| Summary::new(&dir))
}
