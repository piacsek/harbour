use std::fs;
use std::path::Path;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::memory::Remembered;
use harbour::picker::Picker;

use crate::directory::{cwd, folder, recording_agent, repository, write_transcript};
use crate::keeping::{registered, remembered_soon, titled};
use crate::remembering::{ID, OTHER_ID, Scratch, TITLE};
use crate::{Screen, in_order, numbered_agent, region, script};

const OTHER_TITLE: &str = "Add the acme export";
const THIRD_ID: &str = "5c4d3e2f-1a0b-4c9d-8e7f-6a5b4c3d2e1f";
const THIRD_TITLE: &str = "Rename the acme jobs";

#[test]
fn a_session_started_where_a_card_works_joins_that_cards_group_and_stays_selected() {
    let scratch = tempfile::tempdir().unwrap();

    let mut screen = api_web_api(scratch.path());

    let sidebar = sidebar(&screen.draw());
    assert!(
        in_order(
            &sidebar,
            &["acme-api ─", "acme-api 1", "acme-api 3", "acme-web 2"]
        ),
        "{sidebar}"
    );
    assert!(sidebar.contains("▌ · acme-api 3"), "{sidebar}");
}

#[test]
fn j_selects_the_next_card_on_screen() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = api_web_api(scratch.path());

    screen.press('j');

    let sidebar = sidebar(&screen.draw());
    assert!(sidebar.contains("▌ · acme-web 2"), "{sidebar}");
}

#[test]
fn the_prefix_then_2_jumps_to_the_second_card_on_screen() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = api_web_api(scratch.path());
    screen.press('k');
    screen.wait_for("acme agent 1");

    screen.prefixed('2');

    let shown = screen.wait_for("acme agent 3");
    assert!(!shown.contains("acme agent 2"), "{shown}");
}

#[test]
fn stopped_cards_come_back_grouped_and_their_positions_follow() {
    let scratch = Scratch::new();
    remember(&scratch, ID, TITLE, "acme-api", 0);
    remember(&scratch, OTHER_ID, OTHER_TITLE, "acme-web", 1);
    remember(&scratch, THIRD_ID, THIRD_TITLE, "acme-api", 2);

    let mut screen = stopped(&scratch);

    let sidebar = sidebar(&screen.draw());
    assert!(
        in_order(&sidebar, &[TITLE, THIRD_TITLE, OTHER_TITLE]),
        "{sidebar}"
    );
    let recalled = scratch.memory().recall();
    assert_eq!(recalled[0].session_id, ID);
    assert_eq!(recalled[1].session_id, THIRD_ID);
    assert_eq!(recalled[2].session_id, OTHER_ID);
}

#[test]
fn shift_j_on_a_groups_first_card_moves_the_whole_group() {
    let scratch = Scratch::new();
    let mut screen = api_api_web(&scratch);

    screen.key(KeyCode::Char('J'), KeyModifiers::SHIFT);

    let recalled = scratch.memory().recall();
    assert_eq!(recalled[0].session_id, OTHER_ID);
    assert_eq!(recalled[1].session_id, ID);
    assert_eq!(recalled[2].session_id, THIRD_ID);
}

#[test]
fn shift_j_on_a_groups_last_card_does_nothing() {
    let scratch = Scratch::new();
    let mut screen = api_api_web(&scratch);
    screen.press('j');

    screen.key(KeyCode::Char('J'), KeyModifiers::SHIFT);

    let recalled = scratch.memory().recall();
    assert_eq!(recalled[0].session_id, ID);
    assert_eq!(recalled[1].session_id, THIRD_ID);
    assert_eq!(recalled[2].session_id, OTHER_ID);
}

#[test]
fn shift_k_on_a_lone_card_moves_it_above_the_whole_group() {
    let scratch = Scratch::new();
    let mut screen = api_api_web(&scratch);
    screen.press('j');
    screen.press('j');

    screen.key(KeyCode::Char('K'), KeyModifiers::SHIFT);

    let recalled = scratch.memory().recall();
    assert_eq!(recalled[0].session_id, OTHER_ID);
    assert_eq!(recalled[1].session_id, ID);
    assert_eq!(recalled[2].session_id, THIRD_ID);
    let sidebar = sidebar(&screen.draw());
    assert!(sidebar.contains(&format!("▌ ○ {OTHER_TITLE}")), "{sidebar}");
}

#[test]
fn a_session_that_moves_to_another_repository_joins_that_repositorys_group() {
    let scratch = tempfile::tempdir().unwrap();
    let one = repository(scratch.path(), "acme-one");
    let two = repository(scratch.path(), "acme-two");
    let three = repository(scratch.path(), "acme-three");
    let mut screen = Screen::of(
        App::new(recording_agent(scratch.path()))
            .with_picker(cycling(scratch.path(), &[&one, &two, &three]))
            .with_transcripts(scratch.path().join("projects"))
            .naming_sessions(),
    );
    let args = scratch.path().join("args");
    for n in 1..=3 {
        let _ = fs::remove_file(&args);
        screen.press('n');
        screen.wait_for(&format!("acme-{} {n}", ["one", "two", "three"][n - 1]));
        screen.wait_until("the agent's arguments", |_| args.exists());
        screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    }

    write_transcript(scratch.path(), &[&cwd(&one)]);

    let shown = screen.wait_until("acme-three 3 to join acme-one", |shown| {
        in_order(
            &sidebar(shown),
            &["acme-one ─", "acme-one 1", "acme-three 3", "acme-two 2"],
        )
    });
    let sidebar = sidebar(&shown);
    assert!(sidebar.contains("▌ · acme-three 3"), "{sidebar}");
}

#[test]
fn a_session_that_moves_to_another_repository_just_before_quit_is_saved_in_its_group() {
    let scratch = Scratch::new();
    remember(&scratch, ID, TITLE, "acme-api", 0);
    remember(&scratch, OTHER_ID, OTHER_TITLE, "acme-web", 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(THIRD_ID, &titled(THIRD_TITLE));
    let mut screen = scratch.watching(registered(&scratch.sessions(), THIRD_ID));
    screen.wait_for_all(&[TITLE, OTHER_TITLE]);
    screen.press('n');
    remembered_soon(&mut screen, &scratch.memory(), THIRD_ID, THIRD_TITLE);

    scratch.transcript(THIRD_ID, &cwd(&scratch.checkout()));
    screen.app.stop("acme test");

    let recalled = scratch.memory().recall();
    assert_eq!(recalled[0].session_id, ID);
    assert_eq!(recalled[1].session_id, THIRD_ID);
    assert_eq!(recalled[2].session_id, OTHER_ID);
}

fn api_web_api(scratch: &Path) -> Screen {
    let api = repository(scratch, "acme-api");
    let web = folder(scratch, "acme-web");
    let mut screen = Screen::of(
        App::new(numbered_agent(scratch)).with_picker(cycling(scratch, &[&api, &web, &api])),
    );
    for n in 1..=3 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {n}"));
        screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    }
    screen
}

fn api_api_web(scratch: &Scratch) -> Screen {
    remember(scratch, ID, TITLE, "acme-api", 0);
    remember(scratch, THIRD_ID, THIRD_TITLE, "acme-api", 1);
    remember(scratch, OTHER_ID, OTHER_TITLE, "acme-web", 2);
    stopped(scratch)
}

fn stopped(scratch: &Scratch) -> Screen {
    let mut screen = Screen::of(App::new(script("exec sleep 600")).with_memory(scratch.memory()));
    screen.wait_for_all(&[TITLE, OTHER_TITLE, THIRD_TITLE]);
    screen
}

pub(crate) fn remember(scratch: &Scratch, id: &str, title: &str, checkout: &str, position: usize) {
    scratch
        .memory()
        .keep(&Remembered {
            session_id: id.to_string(),
            directory: folder(scratch.dir.path(), checkout),
            title: title.to_string(),
            given_title: None,
            position,
        })
        .unwrap();
}

pub(crate) fn cycling(scratch: &Path, directories: &[&Path]) -> Picker {
    let list = scratch.join("directories");
    let lines: Vec<String> = directories
        .iter()
        .map(|directory| directory.display().to_string())
        .collect();
    fs::write(&list, lines.join("\n") + "\n").unwrap();
    let count = scratch.join("picked");
    Picker::Command(script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         sed -n \"${{n}}p\" '{list}'",
        count = count.display(),
        list = list.display()
    )))
}

fn sidebar(shown: &str) -> String {
    region(shown, 1, 38)
}
