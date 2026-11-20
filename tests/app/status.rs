use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use harbour::app::App;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Cell;
use ratatui::style::{Color, Modifier};

use crate::{Screen, directory, script};

#[test]
fn a_busy_session_shows_a_yellow_filled_dot() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "busy"));
    screen.press('n');

    let shown = screen.wait_for(&format!("● {} 1", directory()));

    assert!(shown.contains(&format!("● {} 1", directory())), "{shown}");
    assert_eq!(screen.cell(3, 1).fg, Color::Yellow);
}

#[test]
fn a_session_running_a_shell_command_shows_as_working() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "shell"));
    screen.press('n');

    let shown = screen.wait_for(&format!("● {} 1", directory()));

    assert!(shown.contains(&format!("● {} 1", directory())), "{shown}");
    assert_eq!(screen.cell(3, 1).fg, Color::Yellow);
}

#[test]
fn an_idle_session_shows_a_dim_hollow_dot() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "idle"));
    screen.press('n');

    let dot = wait_for_dot(&mut screen, |cell| cell.modifier.contains(Modifier::DIM));

    assert_eq!(dot.symbol(), "○");
    assert!(dot.modifier.contains(Modifier::DIM));
    assert_eq!(dot.fg, Color::Reset);
}

#[test]
fn a_session_without_a_registry_file_shows_a_dark_grey_hollow_dot() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        script("echo 'acme shell ready'; exec sleep 600"),
    );
    screen.press('n');

    screen.wait_for("acme shell ready");
    let dot = after_a_read(&mut screen);

    assert_eq!(dot.symbol(), "○");
    assert_eq!(dot.fg, Color::DarkGray);
}

#[test]
fn an_unknown_status_shows_a_dark_grey_hollow_dot() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "meditating"));
    screen.press('n');

    screen.wait_for("acme agent registered");
    let dot = after_a_read(&mut screen);

    assert_eq!(dot.symbol(), "○");
    assert_eq!(dot.fg, Color::DarkGray);
}

#[test]
fn a_malformed_registry_file_shows_a_dark_grey_hollow_dot() {
    let registry = tempfile::tempdir().unwrap();
    let file = format!("{}/$$.json", registry.path().display());
    let mut screen = watching(
        registry.path(),
        script(&format!(
            "printf '{{not json' > '{file}'; echo 'acme agent registered'; exec sleep 600"
        )),
    );
    screen.press('n');

    screen.wait_for("acme agent registered");
    let dot = after_a_read(&mut screen);

    assert_eq!(dot.symbol(), "○");
    assert_eq!(dot.fg, Color::DarkGray);
}

#[test]
fn the_dot_follows_a_change_of_status() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "busy"));
    screen.press('n');
    screen.wait_for(&format!("● {} 1", directory()));

    std::fs::write(only_file(registry.path()), r#"{"status":"idle"}"#).unwrap();

    let dot = wait_for_dot(&mut screen, |cell| cell.modifier.contains(Modifier::DIM));
    assert_eq!(dot.symbol(), "○");
    assert!(dot.modifier.contains(Modifier::DIM));
}

#[test]
fn a_session_waiting_on_you_shows_a_red_target_and_what_it_waits_for() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        written(
            registry.path(),
            r#"{"status":"waiting","waitingFor":"permission prompt"}"#,
        ),
    );
    screen.press('n');

    let row = format!("◉ {} 1 permission prompt", directory());
    let shown = screen.wait_for(&row);

    assert!(shown.contains(&row), "{shown}");
    assert_eq!(screen.cell(3, 1).fg, Color::Red);
    assert_eq!(screen.cell(last_column(&shown, "prompt"), 1).fg, Color::Red);
    assert_eq!(screen.cell(5, 1).fg, Color::Reset);
}

#[test]
fn a_session_waiting_without_a_reason_shows_the_red_target_alone() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        written(registry.path(), r#"{"status":"waiting"}"#),
    );
    screen.press('n');

    let row = format!("◉ {} 1", directory());
    let shown = screen.wait_for(&row);

    assert!(shown.contains(&format!("{row} ")), "{shown}");
    assert_eq!(screen.cell(3, 1).fg, Color::Red);
}

#[test]
fn an_exited_session_shows_a_dim_hollow_dot_whatever_its_file_says() {
    let registry = tempfile::tempdir().unwrap();
    let file = format!("{}/$$.json", registry.path().display());
    let mut screen = watching(
        registry.path(),
        script(&format!(
            "printf '{{\"status\":\"busy\"}}' > '{file}'; sleep 1; exit 3"
        )),
    );
    screen.press('n');

    let row = format!("○ {} 1 exited 3", directory());
    let shown = screen.wait_for(&row);

    assert!(shown.contains(&row), "{shown}");
    assert!(screen.cell(3, 1).modifier.contains(Modifier::DIM));
    assert_eq!(screen.cell(3, 1).fg, Color::Reset);
}

fn watching(registry: &Path, command: Vec<String>) -> Screen {
    let mut screen = Screen {
        app: App::new(command).with_registry(registry.to_path_buf()),
        terminal: Terminal::new(TestBackend::new(120, 20)).unwrap(),
    };
    let width = 3 * (directory().chars().count() + 32);
    screen.resize(u16::try_from(width.max(120)).unwrap(), 20);
    screen
}

fn registered(registry: &Path, status: &str) -> Vec<String> {
    written(registry, &format!(r#"{{"status":"{status}"}}"#))
}

fn written(registry: &Path, json: &str) -> Vec<String> {
    script(&format!(
        "printf '%s' '{json}' > '{}'/$$.json; echo 'acme agent registered'; exec sleep 600",
        registry.display()
    ))
}

fn last_column(shown: &str, text: &str) -> u16 {
    let line = shown.lines().nth(1).unwrap();
    let start = line[..line.find(text).unwrap()].chars().count();
    u16::try_from(start + text.chars().count() - 1).unwrap()
}

fn wait_for_dot(screen: &mut Screen, done: impl Fn(&Cell) -> bool) -> Cell {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        screen.draw();
        let dot = screen.cell(3, 1);
        if done(&dot) || Instant::now() > deadline {
            return dot;
        }
        sleep(Duration::from_millis(20));
    }
}

fn after_a_read(screen: &mut Screen) -> Cell {
    screen.draw();
    sleep(Duration::from_millis(600));
    screen.draw();
    screen.cell(3, 1)
}

fn only_file(registry: &Path) -> PathBuf {
    std::fs::read_dir(registry)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path()
}
