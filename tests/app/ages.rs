use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use harbour::app::App;

use crate::status::{showing, watching, written};
use crate::{directory, script};

#[test]
fn a_working_session_shows_how_long_it_has_been_in_its_state_at_the_right_edge() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), since(registry.path(), "busy", 180));
    screen.press('n');

    let shown = screen.wait_for(" 3m");

    let line = first_line(&shown);
    assert!(line.starts_with(&format!("● {} 1", directory())), "{shown}");
    assert!(line.ends_with(" 3m"), "{shown}");
    assert_eq!(right_edge(&shown, 1), "3m");
}

#[test]
fn hours_and_days_show_in_whole_units() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        since(registry.path(), "idle", 2 * 86400 + 60),
    );
    screen.press('n');

    let shown = screen.wait_for(" 2d");

    assert_eq!(right_edge(&shown, 1), "2d");
}

#[test]
fn a_session_without_a_registry_file_shows_no_age() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        script("echo 'acme shell ready'; exec sleep 600"),
    );
    screen.press('n');
    screen.wait_for("acme shell ready");
    sleep(Duration::from_millis(600));

    let shown = screen.draw();

    assert_eq!(first_line(&shown), format!("· {} 1", directory()));
}

#[test]
fn an_exited_session_shows_the_time_since_harbour_saw_it_exit() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), script("exit 3"));
    screen.press('n');

    let shown = screen.wait_for("exited 3");

    assert_eq!(right_edge(&shown, 1), "<1m");
}

#[test]
fn the_narrowest_sidebar_keeps_the_age_and_cuts_the_title() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = showing(
        App::new(since(registry.path(), "busy", 180)).with_registry(registry.path().into()),
    );
    screen.resize(60, 20);
    screen.press('n');

    let shown = screen.wait_for(" 3m");

    let line = first_line(&shown);
    assert!(line.contains('…'), "{shown}");
    assert!(line.ends_with(" 3m"), "{shown}");
}

fn since(registry: &Path, status: &str, seconds: u64) -> Vec<String> {
    let json = format!(
        r#"{{"status":"{status}","statusUpdatedAt":'$(( ($(date +%s) - {seconds}) * 1000 ))'}}"#
    );
    written(registry, &json)
}

fn first_line(shown: &str) -> String {
    shown
        .lines()
        .nth(1)
        .unwrap()
        .chars()
        .skip(1)
        .take_while(|c| !matches!(c, '│' | '┃'))
        .collect::<String>()
        .trim_start_matches(['▌', ' '])
        .trim_end()
        .to_string()
}

fn right_edge(shown: &str, row: usize) -> String {
    let inside: String = shown
        .lines()
        .nth(row)
        .unwrap()
        .chars()
        .skip(1)
        .take_while(|c| !matches!(c, '│' | '┃'))
        .collect();
    inside.rsplit(' ').next().unwrap().to_string()
}
