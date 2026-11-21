use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::picker::Picker;

use crate::{Screen, reading_agent, script};

#[test]
fn the_popup_says_it_is_loading_until_the_report_prints() {
    let mut screen = Screen::of(app().with_usage(script("exec sleep 600")));

    screen.press('u');

    let shown = screen.draw();
    assert!(shown.contains(" usage "), "{shown}");
    assert!(shown.contains("loading…"), "{shown}");
}

#[test]
fn a_report_that_prints_nothing_says_so() {
    let mut screen = Screen::of(app().with_usage(script("exit 0")));

    screen.press('u');

    let shown = screen.wait_for("printed nothing");
    assert!(!shown.contains("loading…"), "{shown}");
}

#[test]
fn a_report_that_fails_keeps_what_it_printed_and_names_its_exit_code() {
    let failing = script("echo 'acme usage: not logged in'; exit 3");
    let mut screen = Screen::of(app().with_usage(failing));

    screen.press('u');

    let shown = screen.wait_for(" exit 3 ");
    assert!(shown.contains("acme usage: not logged in"), "{shown}");
}

#[test]
fn a_report_that_succeeds_names_no_exit_code() {
    let mut screen = Screen::of(app().with_usage(placed_report()));
    screen.press('u');
    screen.wait_for("acme usage in");

    std::thread::sleep(Duration::from_millis(300));

    let shown = screen.draw();
    assert!(!shown.contains(" exit "), "{shown}");
}

#[test]
fn the_report_runs_in_harbours_private_folder_without_a_session() {
    let mut screen = Screen::of(app().with_usage(placed_report()));

    screen.press('u');

    let shown = screen.wait_for("acme usage in");
    assert!(shown.contains("acme usage in usage"), "{shown}");
}

#[test]
fn the_report_runs_in_harbours_private_folder_and_not_in_the_selected_sessions_directory() {
    let scratch = tempfile::tempdir().unwrap();
    let web = directory_in(scratch.path(), "acme-web");
    std::fs::create_dir(web.join(".claude")).unwrap();
    std::fs::write(web.join(".claude/settings.json"), "{}").unwrap();
    let mut screen = placed(&web);

    screen.press('u');

    let shown = screen.wait_for("acme usage in");
    assert!(shown.contains("acme usage in usage"), "{shown}");
    assert!(!shown.contains("acme project settings seen"), "{shown}");
}

#[test]
fn the_private_folder_is_made_for_its_owner_only() {
    let scratch = tempfile::tempdir().unwrap();
    let place = scratch.path().join("state/harbour/usage");
    let app = App::new(reading_agent())
        .with_usage_place(Some(place.clone()))
        .with_usage(placed_report());
    let mut screen = Screen::of(app);

    screen.press('u');

    screen.wait_for("acme usage in usage");
    assert_eq!(mode(&place), 0o700);
}

#[test]
fn a_private_folder_left_open_to_others_is_closed_again() {
    let scratch = tempfile::tempdir().unwrap();
    let place = scratch.path().join("usage");
    std::fs::create_dir(&place).unwrap();
    std::fs::set_permissions(&place, Permissions::from_mode(0o755)).unwrap();
    let app = App::new(reading_agent())
        .with_usage_place(Some(place.clone()))
        .with_usage(placed_report());
    let mut screen = Screen::of(app);

    screen.press('u');

    screen.wait_for("acme usage in usage");
    assert_eq!(mode(&place), 0o700);
}

#[test]
fn without_a_state_folder_u_opens_no_popup_and_says_why() {
    let app = App::new(reading_agent())
        .with_usage_place(None)
        .with_usage(placed_report());
    let mut screen = Screen::of(app);

    screen.press('u');

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
    assert!(
        shown.contains("Usage needs a state folder; set XDG_STATE_HOME or HOME"),
        "{shown}"
    );
}

#[test]
fn a_private_folder_that_cannot_be_made_opens_no_popup_and_says_so() {
    let scratch = tempfile::tempdir().unwrap();
    let blocked = scratch.path().join("acme-file");
    std::fs::write(&blocked, "").unwrap();
    let app = App::new(reading_agent())
        .with_usage_place(Some(blocked.join("usage")))
        .with_usage(placed_report());
    let mut screen = Screen::of(app);

    screen.press('u');

    let shown = screen.draw();
    assert!(!shown.contains(" usage "), "{shown}");
    assert!(shown.contains("Could not make "), "{shown}");
}

#[test]
fn a_report_that_prints_and_exits_at_once_never_flashes_printed_nothing() {
    let quick = script("echo 'acme quick report'");
    let mut screen = Screen::of(app().with_usage(quick));
    let mut nothing = 0;

    for _ in 0..10 {
        screen.press('u');
        screen.wait_until("the quick report", |shown| {
            nothing += usize::from(shown.contains("printed nothing"));
            shown.contains("acme quick report")
        });
        screen.press('q');
    }

    assert_eq!(nothing, 0);
}

#[test]
fn a_report_longer_than_the_popup_shows_its_start() {
    let long = script("for n in $(seq 1 40); do echo \"acme line $n.\"; done");
    let mut screen = Screen::of(app().with_usage(long));

    screen.press('u');

    let shown = screen.wait_for("acme line 1.");
    assert!(!shown.contains("acme line 40."), "{shown}");
}

#[test]
fn the_popup_takes_a_terminal_smaller_than_it() {
    let mut screen = Screen::of(app().with_usage(placed_report()));
    screen.resize(40, 12);

    screen.press('u');

    let shown = screen.wait_for("acme usage in");
    let first = shown.lines().next().unwrap();
    assert!(first.starts_with("┏ usage "), "{shown}");
    assert!(first.ends_with('┓'), "{shown}");
    let last = shown.lines().last().unwrap();
    assert!(last.starts_with('┗'), "{shown}");
}

#[test]
fn the_popup_is_80_by_30_on_a_larger_terminal() {
    let mut screen = Screen::of(app().with_usage(placed_report()));
    screen.resize(120, 40);

    screen.press('u');

    let shown = screen.wait_for("acme usage in");
    let top = shown
        .lines()
        .position(|line| line.contains("┏ usage "))
        .unwrap();
    let bottom = shown.lines().position(|line| line.contains('┗')).unwrap();
    let line = shown.lines().nth(top).unwrap();
    let left = line.chars().position(|c| c == '┏').unwrap();
    let right = line.chars().position(|c| c == '┓').unwrap();
    assert_eq!(bottom - top + 1, 30);
    assert_eq!(right - left + 1, 80);
}

fn placed(directory: &Path) -> Screen {
    let picker = Picker::Command(script(&format!("printf '%s\\n' '{}'", directory.display())));
    let app = app().with_picker(picker).with_usage(placed_report());
    let mut screen = Screen::of(app);
    screen.press('n');
    screen.wait_for("acme agent ready");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen
}

fn placed_report() -> Vec<String> {
    script(
        "echo \"acme usage in ${PWD##*/}\"; \
         if [ -e .claude/settings.json ]; then echo 'acme project settings seen'; fi",
    )
}

pub(crate) fn app() -> App {
    App::new(reading_agent()).with_usage_place(Some(place()))
}

pub(crate) fn place() -> PathBuf {
    crate::home().with_file_name("state").join("harbour/usage")
}

fn mode(path: &Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

fn directory_in(scratch: &Path, name: &str) -> PathBuf {
    let directory = scratch.join(name);
    std::fs::create_dir(&directory).unwrap();
    directory
}
