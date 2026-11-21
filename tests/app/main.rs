mod ages;
mod asked_keys;
mod cards;
mod cursor;
mod directory;
mod editor;
mod editor_ending;
mod editor_exits;
mod editor_input;
mod ending;
mod exits;
mod focus;
mod following;
mod handing;
mod handoff;
mod handoff_socket;
mod help;
mod input;
mod keeping;
mod layout;
mod logging;
mod logging_edits;
mod logging_lives;
mod moving;
mod nearest;
mod neovim;
mod picker;
mod real_neovim;
mod reconciling;
mod refusing;
mod remembering;
mod reserving;
mod selection;
mod status;
mod stopping;
mod switching;
mod transcripts;
mod waits;
mod width;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use harbour::app::App;
use harbour::editor::Program;
use harbour::picker::Picker;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Cell;
use ratatui::layout::Position;
use tempfile::TempDir;

const HOME: &str = "acme-home";

struct Screen {
    app: App,
    terminal: Terminal<TestBackend>,
}

impl Screen {
    fn of(app: App) -> Screen {
        Screen::at(app, home(), 120)
    }

    fn wide(app: App) -> Screen {
        Screen::at(app, home(), 200)
    }

    fn at(app: App, home: PathBuf, width: u16) -> Screen {
        let mut screen = Screen {
            app: app.with_home(home),
            terminal: Terminal::new(TestBackend::new(width, 20)).unwrap(),
        };
        screen.draw();
        screen
    }

    fn new(command: Vec<String>) -> Screen {
        Screen::of(App::new(command))
    }

    fn picking(command: Vec<String>, picker: Picker) -> Screen {
        Screen::of(App::new(command).with_picker(picker))
    }

    fn editing(command: Vec<String>, editor: Program) -> Screen {
        Screen::of(App::new(command).with_editor(editor))
    }

    fn press(&mut self, c: char) {
        self.key(KeyCode::Char(c), KeyModifiers::NONE);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        self.app.key(KeyEvent::new(code, modifiers));
        self.draw();
    }

    fn mouse(&mut self, kind: MouseEventKind, column: u16, row: u16) {
        self.mouse_with(kind, KeyModifiers::NONE, column, row);
    }

    fn mouse_with(&mut self, kind: MouseEventKind, modifiers: KeyModifiers, column: u16, row: u16) {
        self.app.mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers,
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

    fn cursor(&self) -> Option<Position> {
        self.app.cursor_position()
    }

    fn cell(&self, x: u16, y: u16) -> Cell {
        self.terminal.backend().buffer()[(x, y)].clone()
    }

    fn wait_for(&mut self, text: &str) -> String {
        self.wait_for_all(&[text])
    }

    fn wait_for_all(&mut self, texts: &[&str]) -> String {
        self.wait_until(&format!("{texts:?}"), |shown| {
            texts.iter().all(|text| shown.contains(text))
        })
    }

    fn wait_until(&mut self, what: &str, mut done: impl FnMut(&str) -> bool) -> String {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let shown = self.draw();
            if done(&shown) {
                return shown;
            }
            assert!(
                Instant::now() < deadline,
                "waited 5 s for {what}, last screen:\n{shown}"
            );
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
    HOME.to_string()
}

fn home() -> PathBuf {
    static SCRATCH: OnceLock<TempDir> = OnceLock::new();
    let home = SCRATCH
        .get_or_init(|| tempfile::tempdir().unwrap())
        .path()
        .join(HOME);
    std::fs::create_dir_all(&home).unwrap();
    home
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

fn region(shown: &str, from: usize, width: usize) -> String {
    shown
        .lines()
        .map(|line| {
            line.chars()
                .skip(from)
                .take(width)
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn pane_corner(shown: &str) -> u16 {
    let top = shown.lines().next().unwrap();
    let column = top
        .chars()
        .skip(1)
        .position(|c| c == '┌' || c == '┏')
        .unwrap();
    u16::try_from(column + 1).unwrap()
}

fn below(shown: &str, text: &str) -> String {
    let mut lines = shown.lines();
    lines.find(|line| line.contains(text));
    let next = lines.next().unwrap_or_default();
    next.chars()
        .skip(1)
        .take_while(|c| !matches!(c, '│' | '┃'))
        .collect::<String>()
        .trim_start_matches(['▌', ' '])
        .trim_end()
        .to_string()
}

fn failure(wait: impl FnOnce()) -> String {
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(wait))
        .expect_err("the wait returned without failing");
    unwound
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_default()
}

fn sidebar_title(shown: &str) -> String {
    let top = shown.lines().next().unwrap_or_default();
    let inside = top.trim_start_matches(['┌', '┏']);
    let end = inside.find(['─', '━']).unwrap_or(inside.len());
    inside[..end].to_string()
}
