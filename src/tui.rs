use std::io;
use std::time::Duration;

use crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event, KeyEventKind,
};
use crossterm::execute;

use crate::app::App;

pub const SESSION_COMMAND: &str = "HARBOUR_SESSION_COMMAND";

pub fn run() -> io::Result<()> {
    show(App::new(session_command()))
}

pub fn demo() -> io::Result<()> {
    show(App::demo())
}

fn show(app: App) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let restore_terminal = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = leave_terminal(&mut io::stdout());
        restore_terminal(info);
    }));
    let result = execute!(io::stdout(), EnableBracketedPaste, EnableMouseCapture)
        .and_then(|()| drive(&mut terminal, app));
    let _ = leave_terminal(&mut io::stdout());
    ratatui::restore();
    result
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

fn session_command() -> Vec<String> {
    vec![std::env::var(SESSION_COMMAND).unwrap_or_else(|_| "claude".to_string())]
}
