use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEventKind};

use crate::app::App;

pub const SESSION_COMMAND: &str = "HARBOUR_SESSION_COMMAND";

pub fn run() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = drive(&mut terminal);
    ratatui::restore();
    result
}

fn drive(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let mut app = App::new(session_command());
    while app.running() {
        terminal.draw(|frame| app.draw(frame))?;
        if event::poll(Duration::from_millis(30))?
            && let Event::Key(key) = event::read()?
            && key.kind != KeyEventKind::Release
        {
            app.key(key);
        }
    }
    Ok(())
}

fn session_command() -> Vec<String> {
    vec![std::env::var(SESSION_COMMAND).unwrap_or_else(|_| "claude".to_string())]
}
