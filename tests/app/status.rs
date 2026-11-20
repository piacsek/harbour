use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

use harbour::app::App;
use harbour::summary::Summary;
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
fn a_session_without_a_registry_file_shows_a_dark_grey_middle_dot() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        script("echo 'acme shell ready'; exec sleep 600"),
    );
    screen.press('n');

    screen.wait_for("acme shell ready");
    let dot = after_a_read(&mut screen);

    assert_eq!(dot.symbol(), "·");
    assert_eq!(dot.fg, Color::DarkGray);
}

#[test]
fn an_unknown_status_shows_a_dark_grey_middle_dot() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "meditating"));
    screen.press('n');

    screen.wait_for("acme agent registered");
    let dot = after_a_read(&mut screen);

    assert_eq!(dot.symbol(), "·");
    assert_eq!(dot.fg, Color::DarkGray);
}

#[test]
fn a_malformed_registry_file_shows_a_dark_grey_middle_dot() {
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

    let dot = wait_for_dot(&mut screen, |cell| cell.symbol() == "·");
    assert_eq!(dot.symbol(), "·");
    assert_eq!(dot.fg, Color::DarkGray);
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
    let dir = registry.path().display();
    let mut screen = watching(
        registry.path(),
        script(&format!(
            "printf '{{\"status\":\"busy\"}}' > '{dir}'/$$.json; sleep 1; exit 3"
        )),
    );
    screen.press('n');

    let row = format!("○ {} 1 exited 3", directory());
    let shown = screen.wait_for(&row);

    assert!(shown.contains(&row), "{shown}");
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
    screen.wait_for(&format!("○ {} 1 exited 3", directory()));
    let pid = std::fs::read_to_string(scratch.path().join("pid")).unwrap();
    let fifo = registry.path().join(format!("{}.json", pid.trim()));
    let made = Command::new("mkfifo").arg(&fifo).status().unwrap();
    assert!(made.success());
    unblock_after(Duration::from_secs(3), fifo);

    let start = Instant::now();
    after_a_read(&mut screen);

    assert!(
        start.elapsed() < Duration::from_secs(2),
        "{:?}",
        start.elapsed()
    );
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
        wait_for_files(registry.path(), started + 1);
    }

    let shown = screen.wait_for("●10 ◉1 ○1 ·0");
    let title = shown.lines().next().unwrap();
    assert!(title.contains(" ●10 ◉1 ○1 ·0 "), "{shown}");
}

#[test]
fn a_wide_sidebar_names_the_sessions_in_the_title() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "busy"));
    screen.resize(120, 20);

    let shown = screen.draw();

    assert!(shown.contains(" sessions ● 0 ◉ 0 ○ 0 · 0 "), "{shown}");
}

#[test]
fn the_narrowest_sidebar_shortens_the_name_so_the_waiting_reason_shows() {
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

    let row = wait_for_sidebar_row(&mut screen, " perm");

    assert!(row.contains("◉ "), "{row}");
    assert!(row.contains(" perm"), "{row}");
}

#[test]
fn a_waiting_reason_that_fits_beside_the_name_is_not_shortened() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(
        registry.path(),
        written(
            registry.path(),
            r#"{"status":"waiting","waitingFor":"input"}"#,
        ),
    );
    screen.press('n');

    let row = format!("◉ {} 1 input", directory());
    let shown = screen.wait_for(&row);

    assert!(shown.contains(&row), "{shown}");
}

#[test]
fn with_no_session_the_title_counts_zero_of_each() {
    let registry = tempfile::tempdir().unwrap();
    let mut screen = watching(registry.path(), registered(registry.path(), "busy"));

    let shown = screen.draw();

    assert!(shown.contains(" sessions ● 0 ◉ 0 ○ 0 · 0 "), "{shown}");
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
fn a_harbour_that_quits_removes_its_counts() {
    let state = tempfile::tempdir().unwrap();
    let screen =
        showing(App::new(script("exec sleep 600")).with_summary(Summary::new(state.path())));
    counts_file(state.path());

    drop(screen);

    assert_eq!(std::fs::read_dir(state.path()).unwrap().count(), 0);
}

fn watching(registry: &Path, command: Vec<String>) -> Screen {
    showing(App::new(command).with_registry(registry.to_path_buf()))
}

fn showing(app: App) -> Screen {
    let mut screen = Screen {
        app,
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

fn chosen(registry: &Path, scratch: &Path) -> Vec<String> {
    script(&format!(
        "s=$(cat '{}/status'); \
         [ \"$s\" = none ] || printf '{{\"status\":\"%s\"}}' \"$s\" > '{}'/$$.json; \
         echo \"acme agent $s\"; exec sleep 600",
        scratch.display(),
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

fn counts_file(state: &Path) -> PathBuf {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let found = std::fs::read_dir(state)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().is_some_and(|ext| ext == "counts"));
        if let Some(path) = found {
            return path;
        }
        assert!(std::time::Instant::now() < deadline, "no counts published");
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
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let shown = screen.draw();
        let row: String = shown.lines().nth(1).unwrap().chars().take(20).collect();
        if row.contains(text) || Instant::now() > deadline {
            return row;
        }
        sleep(Duration::from_millis(20));
    }
}

fn wait_for_files(registry: &Path, count: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while std::fs::read_dir(registry).unwrap().count() < count && Instant::now() < deadline {
        sleep(Duration::from_millis(20));
    }
}
