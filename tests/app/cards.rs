use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use harbour::picker::Picker;
use ratatui::style::{Color, Modifier};

use crate::{Screen, below, directory, numbered_agent, script};

#[test]
fn the_selected_card_has_the_cyan_bar_on_all_three_lines() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::new(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');
    screen.wait_for("acme agent 2");

    screen.press('k');
    screen.wait_for("acme agent 1");

    assert_eq!(screen.cell(1, 1).symbol(), "▌");
    assert_eq!(screen.cell(1, 2).symbol(), "▌");
    assert_eq!(screen.cell(1, 3).symbol(), "▌");
    assert_eq!(screen.cell(1, 3).fg, Color::Cyan);
    assert_eq!(screen.cell(1, 4).symbol(), " ");
    assert_eq!(screen.cell(1, 5).symbol(), " ");
}

#[test]
fn a_blank_line_separates_one_card_from_the_next() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::wide(numbered_agent(scratch.path()));
    screen.press('n');
    screen.wait_for("acme agent 1");
    screen.press('n');

    let shown = screen.wait_for(&format!("· {} 2", directory()));

    let lines: Vec<&str> = shown.lines().collect();
    assert!(
        lines[1].contains(&format!("· {} 1", directory())),
        "{shown}"
    );
    assert!(
        lines[5].contains(&format!("· {} 2", directory())),
        "{shown}"
    );
    assert_eq!(below(&shown, &format!("· {} 1", directory())), "");
}

#[test]
fn the_selected_card_is_bold_on_every_line() {
    let mut screen = Screen::wide(script("exit 3"));
    screen.press('n');

    screen.wait_for("exited 3");

    assert!(screen.cell(5, 1).modifier.contains(Modifier::BOLD));
    assert!(screen.cell(5, 2).modifier.contains(Modifier::BOLD));
}

#[test]
fn an_exited_session_says_so_on_the_second_line() {
    let mut screen = Screen::wide(script("exit 3"));
    screen.press('n');

    let shown = screen.wait_for("exited 3");

    let name = format!("○ {} 1", directory());
    assert!(shown.contains(&name), "{shown}");
    assert_eq!(below(&shown, &name), "exited 3");
}

#[test]
fn more_cards_than_fit_scroll_to_keep_the_selected_one_whole() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::wide(numbered_agent(scratch.path()));
    for started in 1..=6 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {started}"));
    }

    let shown = screen.draw();

    let last = format!("· {} 6", directory());
    let line = shown.lines().position(|line| line.contains(&last));
    assert!(line.is_some_and(|line| line + 2 <= 18), "{shown}");
    assert!(!shown.contains(&format!("· {} 1", directory())), "{shown}");
}

#[test]
fn the_sidebar_scrolls_back_when_the_selection_moves_up() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::wide(numbered_agent(scratch.path()));
    for started in 1..=6 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {started}"));
    }

    for _ in 0..5 {
        screen.press('k');
    }

    let shown = screen.wait_for("acme agent 1");
    let lines: Vec<&str> = shown.lines().collect();
    assert!(
        lines[1].contains(&format!("▌ · {} 1", directory())),
        "{shown}"
    );
}

#[test]
fn the_sidebar_moves_only_when_the_selection_would_leave_it() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::wide(numbered_agent(scratch.path()));
    for started in 1..=6 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {started}"));
    }
    let before = screen.draw();

    screen.press('k');

    let after = screen.wait_for("acme agent 5");
    assert_eq!(top_card(&after), top_card(&before));
}

#[test]
fn a_taller_terminal_shows_the_cards_scrolled_away_again() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::wide(numbered_agent(scratch.path()));
    for started in 1..=6 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {started}"));
    }
    let width = screen.terminal.backend().buffer().area.width;

    screen.resize(width, 30);

    let shown = screen.draw();
    assert!(
        top_card(&shown).contains(&format!("· {} 1", directory())),
        "{shown}"
    );
}

#[test]
fn a_terminal_too_short_for_one_card_shows_the_top_of_the_selected_one() {
    let mut screen = Screen::wide(script("exit 3"));
    screen.press('n');
    screen.press('n');
    screen.wait_for(&format!("○ {} 2", directory()));

    screen.resize(120, 4);
    let shown = screen.draw();

    assert!(shown.contains(&format!("▌ ○ {} 2", directory())), "{shown}");
    assert!(shown.contains("exited 3"), "{shown}");
    assert!(!shown.contains(&format!("○ {} 1", directory())), "{shown}");
}

#[test]
fn the_narrow_layout_shows_the_same_cards_across_the_whole_width() {
    let mut screen = Screen::new(script("exit 3"));
    screen.press('n');
    screen.wait_for("exited 3");

    screen.resize(50, 20);
    let shown = screen.draw();

    let name = format!("○ {}", directory());
    assert!(shown.contains(&name), "{shown}");
    assert_eq!(below(&shown, &name), "exited 3");
}

#[test]
fn the_third_line_shows_the_repository_and_branch_the_session_started_in() {
    let scratch = tempfile::tempdir().unwrap();
    let repository = repository(scratch.path(), "acme-api", "main");
    let mut screen = picked_in(&repository, "sleep 600");

    let shown = screen.wait_for("acme-api · main");

    assert_eq!(third_line(&shown, "acme-api 1"), "acme-api · main");
}

#[test]
fn the_third_line_follows_a_branch_switch() {
    let scratch = tempfile::tempdir().unwrap();
    let repository = repository(scratch.path(), "acme-api", "main");
    let mut screen = picked_in(&repository, "sleep 600");
    screen.wait_for("acme-api · main");

    fs::write(repository.join(".git/HEAD"), "ref: refs/heads/fix-login\n").unwrap();

    let shown = screen.wait_for("acme-api · fix-login");
    assert_eq!(third_line(&shown, "acme-api 1"), "acme-api · fix-login");
}

#[test]
fn an_exited_session_keeps_the_place_it_last_ran_in() {
    let scratch = tempfile::tempdir().unwrap();
    let repository = repository(scratch.path(), "acme-api", "main");
    let mut screen = picked_in(&repository, "exit 3");
    screen.wait_for("exited 3");
    screen.wait_for("acme-api · main");

    fs::write(repository.join(".git/HEAD"), "ref: refs/heads/fix-login\n").unwrap();
    sleep(Duration::from_millis(700));

    let shown = screen.draw();
    assert_eq!(third_line(&shown, "acme-api 1"), "acme-api · main");
}

fn picked_in(directory: &Path, agent: &str) -> Screen {
    let picker = script(&format!("printf '%s\\n' '{}'", directory.display()));
    let mut screen = Screen::picking(script(agent), Picker::Command(picker));
    screen.press('n');
    screen
}

fn repository(parent: &Path, name: &str, branch: &str) -> PathBuf {
    let repository = parent.join(name);
    fs::create_dir_all(repository.join(".git")).unwrap();
    fs::write(
        repository.join(".git/HEAD"),
        format!("ref: refs/heads/{branch}\n"),
    )
    .unwrap();
    repository
}

fn third_line(shown: &str, title: &str) -> String {
    let mut lines = shown.lines();
    lines.find(|line| line.contains(title));
    lines.next();
    lines
        .next()
        .unwrap_or_default()
        .chars()
        .skip(1)
        .take_while(|c| !matches!(c, '│' | '┃'))
        .collect::<String>()
        .trim_start_matches(['▌', ' '])
        .trim_end()
        .to_string()
}

fn top_card(shown: &str) -> String {
    shown
        .lines()
        .nth(1)
        .unwrap()
        .chars()
        .skip(3)
        .take_while(|c| !matches!(c, '│' | '┃'))
        .collect()
}
