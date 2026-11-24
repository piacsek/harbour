use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::editor::Program;
use harbour::picker::Picker;
use harbour::summary::Summary;
use ratatui::buffer::Cell;
use ratatui::style::{Color, Modifier};

use crate::{Screen, below, directory, failure, script, sidebar_title};

use crate::deadline;

#[test]
fn a_wait_for_a_dot_that_never_shows_fails_showing_the_last_dot() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "busy"));
    screen.press('n');
    screen.wait_for(&format!("● {} 1", directory()));

    let failure = failure(|| {
        wait_for_dot(&mut screen, "an acme dot", deadline(), |cell| {
            cell.symbol() == "acme"
        });
    });

    assert!(failure.contains("for an acme dot"), "{failure}");
    assert!(failure.contains("Yellow"), "{failure}");
}

#[test]
fn a_wait_for_a_sidebar_row_that_never_shows_fails_naming_it_and_the_row() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        written(
            registry.path(),
            r#"{"status":"waiting","waitingFor":"permission prompt"}"#,
        ),
    );
    screen.resize(60, 20);
    screen.press('n');
    wait_for_sidebar_row(&mut screen, "perm");

    let failure = failure(|| {
        wait_for_sidebar_row(&mut screen, "acme never shown");
    });

    assert!(failure.contains("acme never shown"), "{failure}");
    assert!(failure.contains("perm"), "{failure}");
}

#[test]
fn a_wait_for_more_registry_files_than_come_fails_naming_the_count() {
    let registry = tempfile::tempdir().unwrap();

    let failure = failure(|| {
        wait_for_files(registry.path(), 3, deadline());
    });

    assert!(failure.contains("3 registry files"), "{failure}");
    assert!(failure.contains("found 0"), "{failure}");
}

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
fn the_accent_leaves_the_dots_their_colours() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = showing(
        App::new(registered(registry.path(), "busy"))
            .with_registry(registry.path().to_path_buf())
            .with_accent(Color::Magenta),
    );
    screen.press('n');

    screen.wait_for(&format!("● {} 1", directory()));

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

    let dot = wait_for_dot(&mut screen, "a dim dot", 5 * deadline(), |cell| {
        cell.modifier.contains(Modifier::DIM)
    });

    assert_eq!(dot.symbol(), "○");
    assert!(dot.modifier.contains(Modifier::DIM));
    assert_eq!(dot.fg, Color::Reset);
}

#[test]
fn a_session_without_a_registry_file_shows_a_middle_dot_in_the_default_foreground() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        script("echo 'acme shell ready'; exec sleep 600"),
    );
    screen.press('n');

    screen.wait_for("acme shell ready");
    let dot = after_a_read(&mut screen);

    assert_eq!(dot.symbol(), "·");
    assert_eq!(dot.fg, Color::Reset);
}

#[test]
fn an_unknown_status_shows_a_middle_dot_in_the_default_foreground() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "meditating"));
    screen.press('n');

    screen.wait_for("acme agent registered");
    let dot = after_a_read(&mut screen);

    assert_eq!(dot.symbol(), "·");
    assert_eq!(dot.fg, Color::Reset);
}

#[test]
fn a_malformed_registry_file_shows_a_middle_dot_in_the_default_foreground() {
    let registry = tempfile::tempdir().unwrap();
    let dir = registry.path().display();
    let mut screen = watching(
        registry.path(),
        script(&format!(
            "printf '{{not json' > '{dir}'/$$.json; echo 'acme agent registered'; exec sleep 600"
        )),
    );
    screen.press('n');

    screen.wait_for("acme agent registered");
    let dot = after_a_read(&mut screen);

    assert_eq!(dot.symbol(), "·");
    assert_eq!(dot.fg, Color::Reset);
}

#[test]
fn the_dot_follows_a_change_of_status() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "busy"));
    screen.press('n');
    screen.wait_for(&format!("● {} 1", directory()));

    std::fs::write(only_file(registry.path()), r#"{"status":"idle"}"#).unwrap();

    let dot = wait_for_dot(&mut screen, "a dim dot", 5 * deadline(), |cell| {
        cell.modifier.contains(Modifier::DIM)
    });
    assert_eq!(dot.symbol(), "○");
    assert!(dot.modifier.contains(Modifier::DIM));
}

#[test]
fn a_file_caught_half_written_keeps_the_dot_it_had() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "busy"));
    screen.press('n');
    screen.wait_for(&format!("● {} 1", directory()));

    std::fs::write(only_file(registry.path()), r#"{"status":"id"#).unwrap();
    let dot = after_a_read(&mut screen);

    assert_eq!(dot.symbol(), "●");
    assert_eq!(dot.fg, Color::Yellow);
}

#[test]
fn a_file_that_is_removed_turns_the_dot_unknown() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "busy"));
    screen.press('n');
    screen.wait_for(&format!("● {} 1", directory()));

    std::fs::remove_file(only_file(registry.path())).unwrap();

    let dot = wait_for_dot(&mut screen, "an unknown dot", 5 * deadline(), |cell| {
        cell.symbol() == "·"
    });
    assert_eq!(dot.symbol(), "·");
    assert_eq!(dot.fg, Color::Reset);
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

    let shown = screen.wait_for("permission prompt");

    let name = format!("◉ {} 1", directory());
    assert!(shown.contains(&name), "{shown}");
    assert_eq!(below(&shown, &name), "permission prompt");
    assert_eq!(screen.cell(3, 1).fg, Color::Red);
    assert_eq!(
        screen.cell(last_column(&shown, "prompt", 2), 2).fg,
        Color::Red
    );
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

    assert_eq!(below(&shown, &row), "");
    assert_eq!(screen.cell(3, 1).fg, Color::Red);
}

#[test]
fn an_exited_session_shows_a_dim_hollow_dot_whatever_its_file_says() {
    let registry = tempfile::tempdir().unwrap();
    let dir = registry.path().display();
    let mut screen = watching(
        registry.path(),
        script(&format!(
            "printf '{{\"status\":\"busy\"}}' > '{dir}'/$$.json; sleep 1; exit 3"
        )),
    );
    screen.press('n');

    let shown = screen.wait_for("exited 3");

    let name = format!("○ {} 1", directory());
    assert_eq!(below(&shown, &name), "exited 3");
    assert!(screen.cell(3, 1).modifier.contains(Modifier::DIM));
    assert_eq!(screen.cell(3, 1).fg, Color::Reset);
}

#[test]
fn the_file_of_an_exited_session_is_never_read() {
    let registry = tempfile::tempdir().unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        script(&format!(
            "echo $$ > '{}/pid'; exit 3",
            scratch.path().display()
        )),
    );
    screen.press('n');
    screen.wait_for("exited 3");
    let pid = std::fs::read_to_string(scratch.path().join("pid")).unwrap();
    let fifo = registry.path().join(format!("{}.json", pid.trim()));
    let made = Command::new("mkfifo").arg(&fifo).status().unwrap();
    assert!(made.success());
    unblock_after(5 * deadline(), fifo);

    let start = Instant::now();
    after_a_read(&mut screen);

    let elapsed = start.elapsed();
    assert!(elapsed < deadline(), "{elapsed:?}");
}

#[test]
fn the_sidebar_title_counts_the_sessions_by_dot() {
    let registry = tempfile::tempdir().unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), chosen(registry.path(), scratch.path()));

    for status in ["busy", "waiting", "idle", "none"] {
        std::fs::write(scratch.path().join("status"), status).unwrap();
        screen.press('n');
        screen.wait_for(&format!("acme agent {status}"));
    }

    let shown = screen.wait_for(" sessions ● 1 ◉ 1 ○ 1 · 1 ");
    assert!(shown.contains(" sessions ● 1 ◉ 1 ○ 1 · 1 "), "{shown}");
    assert_eq!(screen.cell(11, 0).fg, Color::Yellow);
    assert_eq!(screen.cell(15, 0).fg, Color::Red);
}

#[test]
fn the_narrowest_sidebar_keeps_every_count_in_the_title_even_past_nine() {
    let registry = tempfile::tempdir().unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), chosen(registry.path(), scratch.path()));
    screen.resize(60, 20);

    for (started, status) in ["busy"; 10]
        .into_iter()
        .chain(["waiting", "idle"])
        .enumerate()
    {
        std::fs::write(scratch.path().join("status"), status).unwrap();
        screen.press('n');
        wait_for_files(registry.path(), started + 1, 5 * deadline());
    }

    let shown = screen.wait_for("●10 ◉1 ○1 ");
    assert_eq!(sidebar_title(&shown), " ●10 ◉1 ○1 ", "{shown}");
}

#[test]
fn a_wide_sidebar_names_the_sessions_in_the_title() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "busy"));
    screen.resize(120, 20);

    let shown = screen.draw();

    assert_eq!(sidebar_title(&shown), " sessions ", "{shown}");
}

#[test]
fn the_narrowest_sidebar_cuts_the_waiting_reason_with_an_ellipsis() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        written(
            registry.path(),
            r#"{"status":"waiting","waitingFor":"permission prompt"}"#,
        ),
    );
    screen.resize(60, 20);
    screen.press('n');

    let row = wait_for_sidebar_row(&mut screen, "perm");

    assert_eq!(
        row.trim_start_matches(|c: char| crate::vertical(c) || c == '▌' || c == ' '),
        "permission pr…"
    );
}

#[test]
fn a_short_waiting_reason_shows_whole() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        written(
            registry.path(),
            r#"{"status":"waiting","waitingFor":"input"}"#,
        ),
    );
    screen.press('n');

    let shown = screen.wait_for("input");

    assert_eq!(below(&shown, &format!("◉ {} 1", directory())), "input");
}

#[test]
fn with_no_session_the_title_names_the_sessions_alone() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "busy"));

    let shown = screen.draw();

    assert_eq!(sidebar_title(&shown), " sessions ", "{shown}");
}

#[test]
fn a_running_harbour_publishes_the_counts_of_the_title() {
    let registry = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut screen = showing(
        App::new(registered(registry.path(), "busy"))
            .with_registry(registry.path().to_path_buf())
            .with_summary(Summary::new(state.path())),
    );
    screen.press('n');

    screen.wait_for(&format!("● {} 1", directory()));

    assert_eq!(
        std::fs::read_to_string(counts_file(state.path())).unwrap(),
        "0 1 0 0\n"
    );
}

#[test]
fn a_session_started_through_the_picker_is_published_like_any_other() {
    let registry = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let web = scratch.path().join("acme-web");
    std::fs::create_dir(&web).unwrap();
    let mut screen = showing(
        App::new(registered(registry.path(), "busy"))
            .with_picker(Picker::Command(script(
                "echo 'acme picker' >&2; read -r line; printf '%s\\n' \"$line\"",
            )))
            .with_registry(registry.path().to_path_buf())
            .with_summary(Summary::new(state.path())),
    );
    screen.press('n');
    screen.wait_for("acme picker");
    screen.type_text(&format!("{}\r", web.display()));

    screen.wait_for("● acme-web 1");

    assert_eq!(
        std::fs::read_to_string(counts_file(state.path())).unwrap(),
        "0 1 0 0\n"
    );
}

#[test]
fn a_session_with_its_editor_open_is_published_once() {
    let registry = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut screen = showing(
        App::new(registered(registry.path(), "busy"))
            .with_editor(Program::new(
                "acme-edit",
                script("echo 'acme editor ready'; exec sleep 600"),
            ))
            .with_registry(registry.path().to_path_buf())
            .with_summary(Summary::new(state.path())),
    );
    screen.press('n');
    screen.wait_for(&format!("● {} 1", directory()));
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);

    screen.wait_for("acme editor ready");

    assert_eq!(
        std::fs::read_to_string(counts_file(state.path())).unwrap(),
        "0 1 0 0\n"
    );
}

#[test]
fn a_harbour_that_quits_removes_its_counts() {
    let state = tempfile::tempdir().unwrap();
    let screen =
        showing(App::new(script("exec sleep 600")).with_summary(Summary::new(state.path())));
    counts_file(state.path());

    drop(screen);

    assert_eq!(std::fs::read_dir(state.path()).unwrap().count(), 0);
}

pub(crate) fn watching(registry: &Path, command: Vec<String>) -> Screen {
    showing(App::new(command).with_registry(registry.to_path_buf()))
}

pub(crate) fn showing(app: App) -> Screen {
    Screen::of(app)
}

fn registered(registry: &Path, status: &str) -> Vec<String> {
    written(registry, &format!(r#"{{"status":"{status}"}}"#))
}

pub(crate) fn written(registry: &Path, json: &str) -> Vec<String> {
    script(&format!(
        "printf '%s' '{json}' > '{}'/$$.json; echo 'acme agent registered'; exec sleep 600",
        registry.display()
    ))
}

fn chosen(registry: &Path, scratch: &Path) -> Vec<String> {
    script(&format!(
        "s=$(cat '{}/status'); \
         [ \"$s\" = none ] || printf '{{\"status\":\"%s\"}}' \"$s\" > '{}'/$$.json; \
         echo \"acme agent $s\"; exec sleep 600",
        scratch.display(),
        registry.display()
    ))
}

fn last_column(shown: &str, text: &str, row: usize) -> u16 {
    let line = shown.lines().nth(row).unwrap();
    let start = line[..line.find(text).unwrap()].chars().count();
    u16::try_from(start + text.chars().count() - 1).unwrap()
}

fn wait_for_dot(
    screen: &mut Screen,
    what: &str,
    patience: Duration,
    done: impl Fn(&Cell) -> bool,
) -> Cell {
    let until = Instant::now() + patience;
    loop {
        screen.draw();
        let dot = screen.cell(3, 1);
        if done(&dot) {
            return dot;
        }
        assert!(
            Instant::now() < until,
            "waited {patience:?} for {what} on the first card, last dot: {dot:?}"
        );
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

fn counts_file(state: &Path) -> PathBuf {
    let until = Instant::now() + 5 * deadline();
    loop {
        let found = std::fs::read_dir(state)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().is_some_and(|ext| ext == "counts"));
        if let Some(path) = found {
            return path;
        }
        assert!(Instant::now() < until, "no counts published");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn unblock_after(delay: Duration, fifo: PathBuf) {
    std::thread::spawn(move || {
        sleep(delay);
        while std::fs::OpenOptions::new().write(true).open(&fifo).is_ok() {}
    });
}

fn wait_for_sidebar_row(screen: &mut Screen, text: &str) -> String {
    let patience = 5 * deadline();
    let until = Instant::now() + patience;
    loop {
        let shown = screen.draw();
        let row: String = shown.lines().nth(2).unwrap().chars().take(19).collect();
        if row.contains(text) {
            return row;
        }
        assert!(
            Instant::now() < until,
            "waited {patience:?} for {text:?} in the sidebar row, last row: {row:?}"
        );
        sleep(Duration::from_millis(20));
    }
}

fn wait_for_files(registry: &Path, count: usize, patience: Duration) {
    let until = Instant::now() + patience;
    loop {
        let found = std::fs::read_dir(registry).unwrap().count();
        if found >= count {
            return;
        }
        assert!(
            Instant::now() < until,
            "waited {patience:?} for {count} registry files, found {found}"
        );
        sleep(Duration::from_millis(20));
    }
}
