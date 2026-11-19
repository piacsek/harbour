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

#[test]
fn enter_focuses_the_pane_and_keys_typed_there_reach_the_session() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("hello\r");

    let shown = screen.wait_for("acme agent got: hello");
    assert!(shown.contains("acme agent got: hello"), "{shown}");
}

#[test]
fn keys_typed_while_the_sidebar_has_focus_do_not_reach_the_session() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.type_text("zz\r\r");

    let shown = screen.wait_for("acme agent got:");
    assert!(shown.contains("acme agent got:"), "{shown}");
    assert!(!shown.contains("got: zz"), "{shown}");
}

#[test]
fn ctrl_l_focuses_the_pane() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    screen.wait_for("acme agent ready");

    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.type_text("hello\r");

    let shown = screen.wait_for("acme agent got: hello");
    assert!(shown.contains("acme agent got: hello"), "{shown}");
}

#[test]
fn a_focused_pane_has_the_thick_border() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.draw();
    let top = shown.lines().next().unwrap();
    assert!(top.starts_with('┌'), "{shown}");
    assert!(top.contains('┏'), "{shown}");
}

#[test]
fn a_focused_sidebar_has_the_thick_border() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');

    let shown = screen.wait_for("acme agent 1");

    let top = shown.lines().next().unwrap();
    assert!(top.starts_with('┏'), "{shown}");
    assert!(top.contains('┌'), "{shown}");
}

#[test]
fn ctrl_h_in_the_pane_gives_the_keys_back_to_the_sidebar() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('n');

    let shown = screen.wait_for("acme agent 2");
    assert!(shown.contains("acme agent 2"), "{shown}");
}

#[test]
fn a_resize_reaches_a_session_that_is_not_shown() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(sized_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");

    screen.resize(90, 16);
    screen.press('k');

    let shown = screen.wait_for("size:14 58");
    assert!(shown.contains("acme agent 1"), "{shown}");
    assert!(shown.contains("size:14 58"), "{shown}");
}

#[test]
fn a_narrow_terminal_shows_the_sidebar_alone_while_it_has_focus() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");

    screen.resize(50, 16);

    let shown = screen.draw();
    let row = format!("● {} 1", directory());
    assert!(shown.contains(&row), "{shown}");
    assert!(!shown.contains("acme agent 1"), "{shown}");
}

#[test]
fn a_narrow_terminal_shows_the_pane_alone_while_it_has_focus() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.resize(50, 16);

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("acme agent 1");
    assert!(shown.contains("acme agent 1"), "{shown}");
    assert!(!shown.contains("sessions"), "{shown}");
}

#[test]
fn a_narrow_terminal_gives_a_session_the_whole_width() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(sized_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.resize(50, 16);

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("size:14 48");
    assert!(shown.contains("size:14 48"), "{shown}");
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
        self.key(KeyCode::Char(c), KeyModifiers::NONE);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        self.app.key(KeyEvent::new(code, modifiers));
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
