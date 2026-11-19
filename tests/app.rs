use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use harbour::app::App;
use ratatui::Terminal;
use ratatui::backend::TestBackend;

#[test]
fn k_selects_the_previous_session_and_the_pane_follows() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");

    screen.press('k');

    let shown = screen.wait_for("acme agent 1");
    assert!(shown.contains("acme agent 1"), "{shown}");
    assert!(!shown.contains("acme agent 2"), "{shown}");
}

#[test]
fn j_selects_the_next_session_and_the_pane_follows() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");
    screen.press('k');
    screen.wait_for("acme agent 1");

    screen.press('j');

    let shown = screen.wait_for("acme agent 2");
    assert!(shown.contains("acme agent 2"), "{shown}");
    assert!(!shown.contains("acme agent 1"), "{shown}");
}

#[test]
fn j_on_the_last_session_keeps_it_selected() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");

    screen.press('j');

    let shown = screen.wait_for("acme agent 1");
    assert!(shown.contains("acme agent 1"), "{shown}");
}

#[test]
fn the_sidebar_marks_the_selected_session() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");

    screen.press('k');

    let shown = screen.wait_for("acme agent 1");
    let first = format!("› ● {} 1", directory());
    let second = format!("› ● {} 2", directory());
    assert!(shown.contains(&first), "{shown}");
    assert!(!shown.contains(&second), "{shown}");
}

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
        self.app
            .key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
        self.draw();
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
