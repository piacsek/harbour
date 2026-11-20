use std::sync::mpsc::{self, Receiver};

use harbour::app::App;
use harbour::log::Log;

use crate::{Screen, script};

#[test]
fn the_first_frame_logs_the_start_with_the_version() {
    let (app, lines) = logged(App::new(script("exec sleep 600")));

    Screen::of(app);

    let start = only(&read(&lines), " start ");
    assert!(
        start.contains(&format!("version={}", env!("CARGO_PKG_VERSION"))),
        "{start}"
    );
}

#[test]
fn the_start_names_the_size_of_the_terminal() {
    let (app, lines) = logged(App::new(script("exec sleep 600")));

    Screen::of(app);

    assert!(only(&read(&lines), " start ").contains(" size=120x20"));
}

#[test]
fn the_start_names_the_session_command_picker_and_editor() {
    let (app, lines) = logged(App::new(script("exec sleep 600")));

    Screen::of(app);

    let start = only(&read(&lines), " start ");
    assert!(start.contains(r#" command="/bin/sh""#), "{start}");
    assert!(start.contains(" picker=none"), "{start}");
    assert!(start.contains(" editor=none"), "{start}");
}

#[test]
fn the_start_is_logged_once() {
    let (app, lines) = logged(App::new(script("exec sleep 600")));
    let mut screen = Screen::of(app);

    screen.draw();
    screen.draw();

    assert_eq!(count(&read(&lines), " start "), 1);
}

fn logged(app: App) -> (App, Receiver<String>) {
    let (sender, lines) = mpsc::sync_channel(4096);
    (app.with_log(Log::sending(sender)), lines)
}

fn read(lines: &Receiver<String>) -> Vec<String> {
    lines.try_iter().collect()
}

fn count(lines: &[String], event: &str) -> usize {
    lines.iter().filter(|line| line.contains(event)).count()
}

fn only(lines: &[String], event: &str) -> String {
    let found: Vec<&String> = lines.iter().filter(|line| line.contains(event)).collect();
    assert_eq!(found.len(), 1, "{event:?} in {lines:#?}");
    found[0].clone()
}
