use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;
use harbour::summary::Summary;

use crate::remembering::{ID, OTHER_ID, Scratch, TITLE};
use crate::{Screen, below, focused, reading_agent, script, sidebar_title, starts_focused};

const COME_BACK: &str = "Conversations come back stopped";
const CJK_TITLE: &str = "修复登录测试的不稳定问题";

#[test]
fn q_with_a_running_session_says_its_sessions_come_back_stopped() {
    let scratch = Scratch::new();
    let mut screen = scratch.watching(reading_agent());
    screen.press('n');
    let ready = screen.wait_for("acme agent ready");
    assert!(ready.contains("acme agent ready"), "{ready}");

    screen.prefixed('q');

    let shown = screen.draw();
    assert!(shown.contains(COME_BACK), "{shown}");
    assert!(shown.contains("Quit? y/n"), "{shown}");
}

#[test]
fn a_signal_stops_harbour_with_a_running_session_without_asking() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    let ready = screen.wait_for("acme agent ready");
    assert!(ready.contains("acme agent ready"), "{ready}");

    screen.app.stop("signal name=TERM");

    assert!(!screen.app.running());
}

#[test]
fn a_signal_while_the_quit_question_shows_stops_harbour() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    let ready = screen.wait_for("acme agent ready");
    assert!(ready.contains("acme agent ready"), "{ready}");
    screen.prefixed('q');

    screen.app.stop("signal name=TERM");

    assert!(!screen.app.running());
}

#[test]
fn q_in_a_harbour_that_remembers_nothing_does_not_say_sessions_come_back() {
    let mut screen = Screen::new(reading_agent());
    screen.press('n');
    let ready = screen.wait_for("acme agent ready");
    assert!(ready.contains("acme agent ready"), "{ready}");

    screen.prefixed('q');

    let shown = screen.draw();
    assert!(!shown.contains(COME_BACK), "{shown}");
}

#[test]
fn q_with_only_stopped_cards_quits_without_asking() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(script("exec sleep 600"));
    let before = screen.wait_for(TITLE);
    assert!(before.contains(TITLE), "{before}");

    screen.prefixed('q');

    assert!(!screen.app.running());
}

#[test]
fn a_stopped_card_is_counted_nowhere_in_the_title() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(script("exec sleep 600"));

    let shown = screen.wait_for(TITLE);

    assert_eq!(sidebar_title(&shown), " sessions ", "{shown}");
}

#[test]
fn a_stopped_card_is_not_published_to_harbour_status() {
    let scratch = Scratch::new();
    let state = tempfile::tempdir().unwrap();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = Screen::of(
        App::new(script("exec sleep 600"))
            .with_registry(scratch.sessions())
            .with_transcripts(scratch.dir.path().join("claude/projects"))
            .with_memory(scratch.memory())
            .with_summary(Summary::new(state.path())),
    );
    let shown = screen.wait_for(TITLE);
    assert!(shown.contains(TITLE), "{shown}");

    let published = published(state.path());

    assert_eq!(published, "0 0 0 0\n");
}

#[test]
fn a_click_in_the_pane_of_a_stopped_card_does_not_focus_the_pane() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(script("exec sleep 600"));
    let before = screen.wait_for(TITLE);
    assert!(before.contains(TITLE), "{before}");

    screen.mouse(MouseEventKind::Down(MouseButton::Left), 60, 5);

    let shown = screen.draw();
    assert!(starts_focused(&shown), "{shown}");
}

#[test]
fn enter_on_a_card_that_cannot_resume_leaves_the_sidebar_focused() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    fs::remove_dir_all(scratch.checkout()).unwrap();
    let mut screen = scratch.watching(script("exec sleep 600"));
    let before = screen.wait_for("directory gone");
    assert!(before.contains("directory gone"), "{before}");

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.draw();
    assert!(starts_focused(&shown), "{shown}");
}

#[test]
fn the_narrowest_sidebar_keeps_a_cjk_title_inside_its_border() {
    let scratch = Scratch::new();
    scratch.remember(ID, CJK_TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(script("exec sleep 600"));
    let before = screen.wait_for("stopped");
    assert!(before.contains("stopped"), "{before}");

    screen.resize(60, 20);

    let shown = screen.draw();
    let row = shown.lines().position(|line| line.contains('修')).unwrap();
    assert!(focused(
        screen.cell(19, u16::try_from(row).unwrap()).symbol()
    ));
    assert!(shown.contains('…'), "{shown}");
}

#[test]
fn the_narrowest_sidebar_shows_why_a_card_is_stopped() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.remember(OTHER_ID, "Gone", 1);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    scratch.transcript(OTHER_ID, r#"{"type":"user","message":{}}"#);
    let gone = scratch.dir.path().join("acme-gone");
    fs::create_dir(&gone).unwrap();
    scratch
        .memory()
        .keep(&harbour::memory::Remembered {
            session_id: OTHER_ID.to_string(),
            directory: gone.clone(),
            title: "Gone".to_string(),
            position: 1,
        })
        .unwrap();
    fs::remove_dir(&gone).unwrap();
    let mut screen = scratch.watching(script("exec sleep 600"));
    screen.resize(60, 20);

    let shown = screen.wait_for("directory gone");

    assert_eq!(below(&shown, "Gone"), "directory gone");
    assert_eq!(below(&shown, "Fix the flaky"), "stopped");
}

#[test]
fn a_narrow_terminal_shows_a_stopped_card() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(script("exec sleep 600"));
    screen.resize(40, 20);

    let shown = screen.wait_for("stopped");

    assert_eq!(below(&shown, "Fix the flaky"), "stopped");
}

fn published(state: &Path) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let found = fs::read_dir(state)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| path.extension().is_some_and(|ext| ext == "counts"));
        if let Some(path) = found {
            return fs::read_to_string(path).unwrap();
        }
        assert!(Instant::now() < deadline, "no counts published");
        sleep(Duration::from_millis(20));
    }
}
