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
    let mut terminal = ratatui::init();
    let result = execute!(io::stdout(), EnableBracketedPaste, EnableMouseCapture)
        .and_then(|()| drive(&mut terminal));
    let _ = execute!(io::stdout(), DisableMouseCapture, DisableBracketedPaste);
    ratatui::restore();
    result
}

fn drive(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let mut app = App::new(session_command());
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
