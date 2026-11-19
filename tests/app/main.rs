mod ending;
mod exits;
mod focus;
mod help;
mod input;
mod layout;
mod selection;

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use harbour::app::App;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

struct Screen {
    app: App,
    terminal: Terminal<TestBackend>,
}

impl Screen {
    fn new(command: Vec<String>) -> Screen {
        let mut screen = Screen {
            app: App::new(command),
            terminal: Terminal::new(TestBackend::new(120, 20)).unwrap(),
        };
        screen.draw();
        screen
    }

    fn press(&mut self, c: char) {
        self.key(KeyCode::Char(c), KeyModifiers::NONE);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        self.app.key(KeyEvent::new(code, modifiers));
        self.draw();
    }

    fn mouse(&mut self, kind: MouseEventKind, column: u16, row: u16) {
        self.app.mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        });
        self.draw();
    }

    fn resize(&mut self, width: u16, height: u16) {
        self.terminal.backend_mut().resize(width, height);
        self.draw();
    }

    fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            match c {
                '\r' => self.key(KeyCode::Enter, KeyModifiers::NONE),
                c => self.press(c),
            }
        }
    }

    fn draw(&mut self) -> String {
        self.terminal.draw(|frame| self.app.draw(frame)).unwrap();
        let buffer = self.terminal.backend().buffer();
        (0..buffer.area.height)
            .map(|y| {
                (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn wait_for(&mut self, text: &str) -> String {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let shown = self.draw();
            if shown.contains(text) || Instant::now() > deadline {
                return shown;
            }
            sleep(Duration::from_millis(20));
        }
    }
}

fn script(text: &str) -> Vec<String> {
    vec!["/bin/sh".into(), "-c".into(), text.into()]
}

fn reading_agent() -> Vec<String> {
    let script = "echo 'acme agent ready'; \
                  while read -r line; do echo \"acme agent got: $line\"; done; exec sleep 600";
    vec!["/bin/sh".into(), "-c".into(), script.into()]
}

fn sized_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    let script = format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         trap 'echo \"size:$(stty size)\"' WINCH; \
         echo \"acme agent $n\"; while :; do sleep 0.1; done",
        count = count.display()
    );
    vec!["/bin/sh".into(), "-c".into(), script]
}

fn raw_agent(request: &str, length: usize) -> Vec<String> {
    let script = format!(
        "stty raw -echo; printf '{request}'; echo 'acme agent ready'; \
         r=$(dd bs=1 count={length} 2>/dev/null); \
         printf 'acme agent read:%s' \"$r\" | tr -d '\\033'; exec sleep 600"
    );
    vec!["/bin/sh".into(), "-c".into(), script]
}

fn directory() -> String {
    std::env::current_dir()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn numbered_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    let script = format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         echo \"acme agent $n\"; exec sleep 600",
        count = count.display()
    );
    vec!["/bin/sh".into(), "-c".into(), script]
}
