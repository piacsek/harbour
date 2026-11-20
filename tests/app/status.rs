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
    script(&format!(
        "printf '{{\"status\":\"{status}\"}}' > '{}'/$$.json; \
         echo 'acme agent registered'; exec sleep 600",
        registry.display()
    ))
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
