use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::picker::Picker;
use ratatui::style::Modifier;

use crate::{
    Screen, drawn_by, focused_char, focused_title, reading_agent, script, starts_focused, top_left,
};

#[test]
fn the_popup_shows_the_loading_animation_until_the_report_has_finished() {
    let mut screen = Screen::of(app().with_usage(script("exec sleep 600")));

    screen.prefixed('u');

    let shown = screen.draw();
    assert!(shown.contains(" usage "), "{shown}");
    assert!(shown.contains("reading usage…"), "{shown}");
    assert_eq!(cells(&bar(&shown)), 24);
    assert!(!shown.contains("loading…"), "{shown}");
}

#[test]
fn the_loading_popup_says_how_to_show_the_text_and_close_it() {
    let mut screen = Screen::of(app().with_usage(script("exec sleep 600")));

    screen.prefixed('u');

    let shown = screen.draw();
    assert!(shown.contains("  t text · Esc close"), "{shown}");
    assert!(!shown.contains("t raw text"), "{shown}");
}

#[test]
fn t_before_anything_printed_says_so_and_how_to_go_back() {
    let mut screen = Screen::of(app().with_usage(script("exec sleep 600")));
    screen.prefixed('u');

    screen.press('t');

    let shown = screen.draw();
    assert!(shown.contains("nothing printed yet"), "{shown}");
    assert!(shown.contains("  t animation · Esc close"), "{shown}");
    assert!(!shown.contains("reading usage…"), "{shown}");
}

#[test]
fn nothing_printed_yet_is_dim() {
    let mut screen = Screen::of(app().with_usage(script("exec sleep 600")));
    screen.prefixed('u');

    screen.press('t');

    let shown = screen.draw();
    let (x, y) = at(&shown, "nothing printed yet");
    assert!(screen.cell(x, y).modifier.contains(Modifier::DIM));
}

#[test]
fn the_text_view_while_loading_drops_its_note_once_something_printed() {
    let mut screen =
        Screen::of(app().with_usage(script("echo 'acme usage begins'; exec sleep 600")));
    screen.prefixed('u');

    screen.press('t');

    let shown = screen.wait_for("acme usage begins");
    assert!(!shown.contains("nothing printed yet"), "{shown}");
}

#[test]
fn the_loading_animation_moves_with_the_clock_while_nothing_else_changes() {
    let mut screen = Screen::of(app().with_usage(script("exec sleep 600")));
    screen.prefixed('u');
    let first = bar(&screen.draw());

    let shown = screen.wait_until("the patch to move", |shown| bar(shown) != first);

    assert!(shown.contains("reading usage…"), "{shown}");
}

#[test]
fn a_report_that_prints_nothing_says_so() {
    let mut screen = Screen::of(app().with_usage(script("exit 0")));

    screen.prefixed('u');

    let shown = screen.wait_for("printed nothing");
    assert!(!shown.contains("reading usage…"), "{shown}");
}

#[test]
fn a_report_that_fails_keeps_what_it_printed_and_names_its_exit_code() {
    let failing = script("echo 'acme usage: not logged in'; exit 3");
    let mut screen = Screen::of(app().with_usage(failing));

    screen.prefixed('u');

    let shown = screen.wait_for(" exit 3 ");
    assert!(shown.contains("acme usage: not logged in"), "{shown}");
}

#[test]
fn a_report_that_succeeds_names_no_exit_code() {
    let mut screen = Screen::of(app().with_usage(placed_report()));
    screen.prefixed('u');
    screen.wait_for("acme usage in");

    std::thread::sleep(Duration::from_millis(300));

    let shown = screen.draw();
    assert!(!shown.contains(" exit "), "{shown}");
}

#[test]
fn the_report_runs_in_harbours_private_folder_without_a_session() {
    let mut screen = Screen::of(app().with_usage(placed_report()));

    screen.prefixed('u');

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

    screen.prefixed('u');

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

    screen.prefixed('u');

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

    screen.prefixed('u');

    screen.wait_for("acme usage in usage");
    assert_eq!(mode(&place), 0o700);
}

#[test]
fn without_a_state_folder_u_opens_no_popup_and_says_why() {
    let app = App::new(reading_agent())
        .with_usage_place(None)
        .with_usage(placed_report());
    let mut screen = Screen::of(app);

    screen.prefixed('u');

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

    screen.prefixed('u');

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
        screen.prefixed('u');
        screen.wait_until("the quick report", |shown| {
            nothing += usize::from(shown.contains("printed nothing"));
            shown.contains("acme quick report")
        });
        screen.press('q');
    }

    assert_eq!(nothing, 0);
}

#[test]
fn a_finished_report_comes_back_whole_after_the_terminal_shrinks_and_widens() {
    let line = "acme weekly limit: 41% spent, resets on the first day of the month";
    let mut screen = Screen::of(app().with_usage(script(&format!("echo '{line}'"))));
    screen.prefixed('u');
    screen.wait_for(line);
    std::thread::sleep(Duration::from_millis(300));
    screen.draw();

    screen.resize(40, 26);
    screen.draw();
    screen.resize(160, 26);

    let shown = screen.draw();
    assert!(shown.contains(line), "{shown}");
}

#[test]
fn a_report_whose_child_holds_the_terminal_is_decided_once_it_exits() {
    let held = script("(trap '' HUP; exec sleep 8) & exit 3");
    let mut screen = Screen::of(app().with_usage(held));

    screen.prefixed('u');

    let shown = screen.wait_for(" exit 3 ");
    assert!(shown.contains("printed nothing"), "{shown}");
}

#[test]
fn a_report_longer_than_the_popup_shows_its_start() {
    let long = script("for n in $(seq 1 40); do echo \"acme line $n.\"; done");
    let mut screen = Screen::of(app().with_usage(long));

    screen.prefixed('u');

    let shown = screen.wait_for("acme line 1.");
    assert!(!shown.contains("acme line 40."), "{shown}");
}

#[test]
fn the_popup_takes_a_terminal_smaller_than_it() {
    let mut screen = Screen::of(app().with_usage(placed_report()));
    screen.resize(40, 12);

    screen.prefixed('u');

    let shown = screen.wait_for("acme usage in");
    let first = shown.lines().next().unwrap();
    assert_eq!(focused_title(&shown), "usage", "{shown}");
    assert!(first.ends_with(focused_char), "{shown}");
    let last = shown.lines().last().unwrap();
    assert!(starts_focused(last), "{shown}");
}

#[test]
fn the_popup_is_80_by_30_on_a_larger_terminal() {
    let mut screen = Screen::of(app().with_usage(placed_report()));
    screen.resize(120, 40);

    screen.prefixed('u');

    let shown = screen.wait_for("acme usage in");
    let top = shown
        .lines()
        .position(|line| line.contains(" usage "))
        .unwrap();
    let bottom = top
        + shown
            .lines()
            .skip(top)
            .position(|line| line.chars().any(|c| drawn_by(c, |set| set.bottom_left)))
            .unwrap();
    let line = shown.lines().nth(top).unwrap();
    let left = line.chars().position(top_left).unwrap();
    let right = line
        .chars()
        .position(|c| drawn_by(c, |set| set.top_right))
        .unwrap();
    assert_eq!(bottom - top + 1, 30);
    assert_eq!(right - left + 1, 80);
}

fn cells(bar: &str) -> usize {
    bar.chars().filter(|c| "░▒▓█".contains(*c)).count()
}

fn at(shown: &str, text: &str) -> (u16, u16) {
    let (y, line) = shown
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains(text))
        .unwrap();
    let x = line[..line.find(text).unwrap()].chars().count();
    (u16::try_from(x).unwrap(), u16::try_from(y).unwrap())
}

fn bar(shown: &str) -> String {
    shown
        .lines()
        .find(|line| line.contains('█'))
        .unwrap_or_default()
        .to_string()
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
