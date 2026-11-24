mod ages;
mod asked_keys;
mod bindings;
mod cards;
mod changelog;
mod cursor;
mod directory;
mod editor;
mod editor_ending;
mod editor_exits;
mod editor_input;
mod ending;
mod exits;
mod finding;
mod finding_states;
mod focus;
mod following;
mod frames;
mod handing;
mod handoff;
mod handoff_socket;
mod help;
mod input;
mod jumping;
mod keeping;
mod layout;
mod logging;
mod logging_edits;
mod logging_lives;
mod marks;
mod moving;
mod navigating;
mod nearest;
mod neovim;
mod picker;
mod prefix;
mod real_neovim;
mod reconciling;
mod refusing;
mod remembering;
mod removed;
mod reordering;
mod reserving;
mod selection;
mod settling;
mod side;
mod side_border;
mod side_jump;
mod side_mouse;
mod side_states;
mod side_swap;
mod status;
mod stopping;
mod switching;
mod transcripts;
mod usage;
mod usage_bars;
mod usage_states;
mod waits;
mod width;
mod width_keys;
mod zoom;
mod zoom_border;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use harbour::app::{App, FOCUSED_BORDER, UNFOCUSED_BORDER};
use harbour::editor::Program;
use harbour::picker::Picker;
use harbour::tui::Frames;
use ratatui::backend::TestBackend;
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Rect};
use ratatui::symbols::border;
use ratatui::widgets::BorderType;
use ratatui::{Terminal, TerminalOptions, Viewport};
use tempfile::TempDir;

#[path = "../support/borders.rs"]
mod borders;
#[path = "../support/timing.rs"]
mod timing;
use borders::BORDERS;
use timing::{deadline, scaled};

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

    fn prefixed(&mut self, c: char) {
        self.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
        self.press(c);
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

    fn wait_for_focused(&mut self, title: &str) -> String {
        self.wait_until(&format!("focus on {title}"), |shown| {
            focused_title(shown).starts_with(title)
        })
    }

    fn wait_for_unfocused(&mut self, title: &str) -> String {
        self.wait_until(&format!("{title} without focus"), |shown| {
            unfocused_title(shown).starts_with(title)
        })
    }

    fn wait_for_stable(&mut self) -> String {
        let patience = 5 * deadline();
        let until = Instant::now() + patience;
        let mut last = self.draw();
        let mut since = Instant::now();
        loop {
            sleep(Duration::from_millis(20));
            let shown = self.draw();
            if shown != last {
                last = shown;
                since = Instant::now();
            } else if since.elapsed() >= Duration::from_millis(1000) {
                return shown;
            }
            assert!(
                Instant::now() < until,
                "waited {patience:?} for the screen to stop changing, last screen:\n{last}"
            );
        }
    }

    fn wait_until(&mut self, what: &str, mut done: impl FnMut(&str) -> bool) -> String {
        let patience = 5 * deadline();
        let until = Instant::now() + patience;
        loop {
            let shown = self.draw();
            if done(&shown) {
                return shown;
            }
            assert!(
                Instant::now() < until,
                "waited {patience:?} for {what}, last screen:\n{shown}"
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
    let column = top.chars().skip(1).position(top_left).unwrap();
    u16::try_from(column + 1).unwrap()
}

fn below(shown: &str, text: &str) -> String {
    let mut lines = shown.lines();
    lines.find(|line| line.contains(text));
    inside(lines.next().unwrap_or_default())
        .trim_end()
        .to_string()
}

fn inside(line: &str) -> String {
    line.chars()
        .skip(1)
        .skip_while(|c| ['▌', ' '].contains(c))
        .take_while(|c| !vertical(*c))
        .collect()
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
    let inside = top.trim_start_matches(top_left);
    let end = inside.find(horizontal).unwrap_or(inside.len());
    inside[..end].to_string()
}

fn parts(set: &border::Set<'static>) -> [&'static str; 8] {
    [
        set.top_left,
        set.top_right,
        set.bottom_left,
        set.bottom_right,
        set.vertical_left,
        set.vertical_right,
        set.horizontal_top,
        set.horizontal_bottom,
    ]
}

fn drawn_by(c: char, part: impl Fn(&border::Set<'static>) -> &'static str) -> bool {
    let mut buffer = [0; 4];
    let symbol: &str = c.encode_utf8(&mut buffer);
    BORDERS.iter().any(|set| part(set) == symbol)
}

fn drawn(c: char) -> bool {
    BORDERS
        .iter()
        .any(|set| parts(set).iter().any(|part| part.starts_with(c)))
}

fn top_left(c: char) -> bool {
    drawn_by(c, |set| set.top_left)
}

fn vertical(c: char) -> bool {
    drawn_by(c, |set| set.vertical_left)
}

fn horizontal(c: char) -> bool {
    drawn_by(c, |set| set.horizontal_top)
}

fn focused(symbol: &str) -> bool {
    drawn_only_by(symbol, FOCUSED_BORDER, UNFOCUSED_BORDER)
}

fn starts_focused(line: &str) -> bool {
    line.chars().next().is_some_and(focused_char)
}

fn focused_char(c: char) -> bool {
    focused(c.encode_utf8(&mut [0; 4]))
}

fn unfocused(symbol: &str) -> bool {
    drawn_only_by(symbol, UNFOCUSED_BORDER, FOCUSED_BORDER)
}

fn drawn_only_by(symbol: &str, style: BorderType, other: BorderType) -> bool {
    parts(&style.to_border_set()).contains(&symbol)
        && !parts(&other.to_border_set()).contains(&symbol)
}

fn unfocused_char(c: char) -> bool {
    unfocused(c.encode_utf8(&mut [0; 4]))
}

fn column_of(shown: &str, row: usize, text: &str) -> u16 {
    let line = shown.lines().nth(row).unwrap();
    let at = line.find(text).unwrap();
    u16::try_from(line[..at].chars().count()).unwrap()
}

fn in_order(text: &str, parts: &[&str]) -> bool {
    let mut rest = text;
    parts.iter().all(|part| match rest.find(part) {
        Some(at) => {
            rest = &rest[at + part.len()..];
            true
        }
        None => false,
    })
}

fn focused_title(shown: &str) -> String {
    title(shown, focused_char)
}

fn unfocused_title(shown: &str) -> String {
    title(shown, unfocused_char)
}

fn title(shown: &str, style: impl Fn(char) -> bool) -> String {
    let top = shown.lines().next().unwrap_or_default();
    let Some(at) = top.find(|c: char| top_left(c) && style(c)) else {
        return String::new();
    };
    let inside = top[at..].trim_start_matches(top_left);
    let end = inside.find(horizontal).unwrap_or(inside.len());
    inside[..end].trim().to_string()
}

fn shows_titled(shown: &str, text: &str) -> bool {
    shown.lines().any(|line| {
        line.match_indices(top_left)
            .any(|(at, _)| title(&line[at..], |_| true) == text)
    })
}

fn frames() -> Frames {
    Frames::new(TerminalOptions {
        viewport: Viewport::Fixed(Rect::new(0, 0, 120, 20)),
    })
    .unwrap()
}

fn settled(screen: &mut Screen, mut happened: impl FnMut(&str) -> bool) -> String {
    let until = Instant::now() + deadline() * 3 / 4;
    loop {
        let shown = screen.draw();
        if happened(&shown) || Instant::now() > until {
            return shown;
        }
        sleep(Duration::from_millis(20));
    }
}

fn recording_hangups(scratch: &Path, name: &str) -> String {
    let scratch = scratch.display();
    format!(
        "echo $$ > \"{scratch}/alive-{name}\"; \
         hung() {{ seen=0; all=0; for f in '{scratch}'/alive-*; do read -r p < \"$f\"; \
         all=$((all + 1)); ps -o stat= -p \"$p\" | grep -qv Z && seen=$((seen + 1)); done; \
         echo \"{name} saw $seen alive\" >> '{scratch}/hangups'; \
         tries=0; while [ $(($(wc -l < '{scratch}/hangups'))) -lt $all ] && [ $tries -lt {patience} ]; \
         do sleep 0.01; tries=$((tries + 1)); done; exit 0; }}; trap hung HUP; ",
        patience = (5 * deadline()).as_millis() / 10,
    )
}

fn staying_after_hangups() -> &'static str {
    "while :; do sleep 600 & wait $!; done"
}

fn hangups(scratch: &Path) -> Vec<String> {
    let mut hangups: Vec<String> = std::fs::read_to_string(scratch.join("hangups"))
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect();
    hangups.sort();
    hangups
}
