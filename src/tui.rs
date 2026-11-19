use std::io;

use crossterm::event::{self, Event, KeyEventKind};

use crate::app::App;

pub fn run() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = drive(&mut terminal);
    ratatui::restore();
    result
}

fn drive(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    let mut app = App::default();
    while app.running() {
        terminal.draw(|frame| app.draw(frame))?;
        if let Event::Key(key) = event::read()?
            && key.kind != KeyEventKind::Release
        {
            app.key(key);
        }
    }
    Ok(())
}
