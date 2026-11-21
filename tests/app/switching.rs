use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime};

use crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use harbour::app::App;
use harbour::editor::Program;

use crate::keeping::{remembered_soon, titled};
use crate::remembering::{ID, OTHER_ID, Scratch, TITLE};
use crate::{Screen, below, script};

const SECOND: &str = "The second one";

#[test]
fn merging_away_the_selected_last_card_selects_the_one_that_stays() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    scratch.transcript(OTHER_ID, &titled(SECOND));
    let mut screen = scratch.watching(stoppable(&scratch));
    let first = started(&mut screen, &scratch, ID, TITLE);
    let second = started(&mut screen, &scratch, OTHER_ID, SECOND);
    stop(&mut screen, &scratch, &second);

    register(&scratch, &first, OTHER_ID);

    let shown = screen.wait_until("the left conversation forgotten", |_| {
        !scratch.memory().holds(ID)
    });
    assert!(!shown.contains("No session yet"), "{shown}");
    assert!(shown.contains("acme agent registered"), "{shown}");
}

#[test]
fn a_session_that_switches_to_a_conversation_another_card_runs_forgets_the_one_it_left() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    scratch.transcript(OTHER_ID, &titled(SECOND));
    let mut screen = scratch.watching(stoppable(&scratch));
    let first = started(&mut screen, &scratch, ID, TITLE);
    started(&mut screen, &scratch, OTHER_ID, SECOND);

    register(&scratch, &first, OTHER_ID);

    screen.wait_until("the left conversation forgotten", |_| {
        !scratch.memory().holds(ID)
    });
    let kept = scratch.memory().recall();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].session_id, OTHER_ID);
}

#[test]
fn a_session_that_resumes_an_exited_card_takes_its_place() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    scratch.transcript(OTHER_ID, &titled(SECOND));
    let mut screen = scratch.watching(stoppable(&scratch));
    let first = started(&mut screen, &scratch, ID, TITLE);
    let second = started(&mut screen, &scratch, OTHER_ID, SECOND);
    stop(&mut screen, &scratch, &second);

    register(&scratch, &first, OTHER_ID);

    let shown = screen.wait_until("the exited card merged", |shown| !shown.contains("exited"));
    assert!(!shown.contains("exited"), "{shown}");
    assert_eq!(shown.matches(SECOND).count(), 1, "{shown}");
    let kept = scratch.memory().recall();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].session_id, OTHER_ID);
}

#[test]
fn a_conversation_left_and_taken_up_again_comes_back_after_quitting() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    scratch.transcript(OTHER_ID, &titled(SECOND));
    let mut screen = scratch.watching(stoppable(&scratch));
    let first = started(&mut screen, &scratch, ID, TITLE);
    let second = started(&mut screen, &scratch, OTHER_ID, SECOND);
    stop(&mut screen, &scratch, &second);
    register(&scratch, &first, OTHER_ID);
    screen.wait_until("the left conversation forgotten", |_| {
        !scratch.memory().holds(ID)
    });

    started(&mut screen, &scratch, ID, TITLE);
    screen.press('q');
    screen.press('y');

    let mut again = scratch.watching(stoppable(&scratch));
    let shown = again.wait_for_all(&[TITLE, SECOND]);
    assert!(shown.contains(TITLE), "{shown}");
    assert!(shown.contains(SECOND), "{shown}");
}

#[test]
fn a_session_that_switches_to_a_conversation_open_elsewhere_is_remembered_once() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    scratch.transcript(OTHER_ID, &titled(SECOND));
    fs::write(
        scratch
            .sessions()
            .join(format!("{}.json", std::process::id())),
        format!(r#"{{"status":"idle","sessionId":"{OTHER_ID}"}}"#),
    )
    .unwrap();
    let mut screen = scratch.watching(stoppable(&scratch));
    let first = started(&mut screen, &scratch, ID, TITLE);

    register(&scratch, &first, OTHER_ID);

    screen.wait_until("the left conversation forgotten", |_| {
        !scratch.memory().holds(ID)
    });
    let kept = scratch.memory().recall();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].session_id, OTHER_ID);
}

#[test]
fn an_exited_card_forgotten_elsewhere_stays_forgotten() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(stoppable(&scratch));
    let first = started(&mut screen, &scratch, ID, TITLE);
    stop(&mut screen, &scratch, &first);

    scratch.memory().forget(ID);

    settled(&mut screen, |_| scratch.memory().holds(ID));
    assert!(!scratch.memory().holds(ID));
}

#[test]
fn a_stopped_card_forgotten_elsewhere_does_not_come_back() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(stoppable(&scratch));
    let stopped = screen.wait_for("stopped");
    assert!(stopped.contains(TITLE), "{stopped}");

    scratch.memory().forget(ID);
    settled(&mut screen, |_| scratch.memory().holds(ID));
    screen.press('q');

    let mut again = scratch.watching(stoppable(&scratch));
    let shown = again.wait_for("No session yet");
    assert!(!shown.contains(TITLE), "{shown}");
}

#[test]
fn ending_a_running_session_keeps_its_conversation() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(stoppable(&scratch));
    started(&mut screen, &scratch, ID, TITLE);

    screen.press('x');
    screen.press('y');

    let exited = screen.wait_for("exited");
    assert!(exited.contains("exited"), "{exited}");
    let kept = scratch.memory().recall();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].session_id, ID);
}

#[test]
fn a_resumed_session_that_exits_releases_its_conversation() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(stoppable(&scratch));
    let stopped = screen.wait_for("stopped");
    assert!(stopped.contains(TITLE), "{stopped}");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let resumed = pids_soon(&mut screen, &scratch, 1);
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    stop(&mut screen, &scratch, &resumed[0]);

    screen.press('x');

    assert!(!scratch.memory().holds(ID));
}

#[test]
fn resuming_an_exited_card_shows_its_claude_code_pane() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = Screen::of(
        App::new(stoppable(&scratch))
            .with_registry(scratch.sessions())
            .with_transcripts(scratch.dir.path().join("claude/projects"))
            .with_memory(scratch.memory())
            .with_editor(Program::new(
                "acme-edit",
                script("echo 'acme editor up'; exec sleep 600"),
            )),
    );
    let first = started(&mut screen, &scratch, ID, TITLE);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let editing = screen.wait_for("acme editor up");
    assert!(editing.contains("acme editor up"), "{editing}");
    stop(&mut screen, &scratch, &first);
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 2, 2);
    fs::remove_file(stop_file(&scratch, &first)).unwrap();

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for(&format!("acme agent resumed {ID}"));
    assert!(
        shown.contains(&format!("acme agent resumed {ID}")),
        "{shown}"
    );
}

#[test]
fn enter_on_a_gone_conversation_keeps_the_card_while_its_editor_runs() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = Screen::of(
        App::new(stoppable(&scratch))
            .with_registry(scratch.sessions())
            .with_transcripts(scratch.dir.path().join("claude/projects"))
            .with_memory(scratch.memory())
            .with_editor(Program::new(
                "acme-edit",
                script("echo 'acme editor up'; exec sleep 600"),
            )),
    );
    let first = started(&mut screen, &scratch, ID, TITLE);
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    screen.wait_for("acme editor up");
    stop(&mut screen, &scratch, &first);
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 2, 2);
    fs::remove_file(scratch.transcript_file(ID)).unwrap();

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("This conversation is gone");
    assert!(shown.contains(TITLE), "{shown}");
    assert_eq!(scratch.memory().recall(), Vec::new());
    screen.key(KeyCode::Char('l'), KeyModifiers::CONTROL);
    let editing = screen.wait_for("acme editor up");
    assert!(editing.contains("acme editor up"), "{editing}");
}

#[test]
fn a_fresh_session_is_open_elsewhere_in_a_second_harbour_without_its_registry_file() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut first = scratch.watching(stoppable(&scratch));
    let pid = started(&mut first, &scratch, ID, TITLE);
    fs::remove_file(scratch.sessions().join(format!("{pid}.json"))).unwrap();
    let mut second = scratch.watching(stoppable(&scratch));
    let held = second.wait_for("open elsewhere");
    assert_eq!(below(&held, TITLE), "open elsewhere");

    second.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = second.wait_for("is open elsewhere");
    assert!(
        shown.contains("This conversation is open elsewhere"),
        "{shown}"
    );
    assert_eq!(pids(&scratch).len(), 1);
}

#[test]
fn a_session_whose_id_changes_holds_only_its_new_conversation() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    scratch.transcript(OTHER_ID, &titled(SECOND));
    let mut first = scratch.watching(stoppable(&scratch));
    let pid = started(&mut first, &scratch, ID, TITLE);
    register(&scratch, &pid, OTHER_ID);
    remembered_soon(&mut first, &scratch.memory(), OTHER_ID, SECOND);
    fs::remove_file(scratch.sessions().join(format!("{pid}.json"))).unwrap();

    let mut second = scratch.watching(stoppable(&scratch));

    let held = second.wait_for("open elsewhere");
    assert_eq!(below(&held, SECOND), "open elsewhere");
    assert!(!scratch.memory().reserved().contains(ID));
    assert!(scratch.memory().reserved().contains(OTHER_ID));
}

#[test]
fn enter_refuses_when_the_conversation_cannot_be_reserved() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(stoppable(&scratch));
    let stopped = screen.wait_for("stopped");
    assert!(stopped.contains(TITLE), "{stopped}");
    let folder = scratch.dir.path().join("state/harbour/sessions");
    let moved = scratch.dir.path().join("state/harbour/moved");
    fs::rename(&folder, &moved).unwrap();
    std::os::unix::fs::symlink(&moved, &folder).unwrap();

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for("Could not reserve");
    assert!(shown.contains("Could not reserve"), "{shown}");
    assert!(!shown.contains("acme agent"), "{shown}");
}

#[test]
fn a_resumed_card_counts_only_the_subagents_started_after_it() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, &titled(TITLE));
    subagent(
        &scratch,
        "agent-a1",
        SystemTime::now() - Duration::from_secs(3600),
    );
    let mut screen = scratch.watching(stoppable(&scratch));
    let stopped = screen.wait_for("stopped");
    assert!(stopped.contains(TITLE), "{stopped}");
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let resumed = screen.wait_for(&format!("acme agent resumed {ID}"));
    assert!(resumed.contains(TITLE), "{resumed}");

    let shown = settled(&mut screen, |shown| shown.contains("subagent"));

    assert!(!shown.contains("subagent"), "{shown}");
    subagent(&scratch, "agent-a2", SystemTime::now());
    let counted = screen.wait_for("1 subagent");
    assert!(counted.contains("1 subagent"), "{counted}");
}

#[test]
fn a_session_that_switched_to_a_conversation_held_elsewhere_takes_it_once_let_go() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    scratch.transcript(OTHER_ID, &titled(SECOND));
    let elsewhere = scratch.memory().reserve(OTHER_ID).unwrap().unwrap();
    let mut screen = scratch.watching(stoppable(&scratch));
    let pid = started(&mut screen, &scratch, ID, TITLE);
    register(&scratch, &pid, OTHER_ID);
    remembered_soon(&mut screen, &scratch.memory(), OTHER_ID, SECOND);
    settled(&mut screen, |_| false);

    drop(elsewhere);

    screen.wait_until("the let-go conversation reserved here", |_| {
        scratch.memory().reserved().contains(OTHER_ID)
    });
    assert!(scratch.memory().reserved().contains(OTHER_ID));
}

#[test]
fn the_held_message_of_an_exited_card_stays_while_it_is_held_elsewhere() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(stoppable(&scratch));
    let first = started(&mut screen, &scratch, ID, TITLE);
    stop(&mut screen, &scratch, &first);
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 2, 2);
    let _elsewhere = scratch.memory().reserve(ID).unwrap().unwrap();
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    let refused = screen.wait_for("is open elsewhere");
    assert!(
        refused.contains("This conversation is open elsewhere"),
        "{refused}"
    );

    let later = settled(&mut screen, |shown| !shown.contains("is open elsewhere"));

    assert!(
        later.contains("This conversation is open elsewhere"),
        "{later}"
    );
}

#[test]
fn enter_right_after_a_session_exits_resumes_it() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(stoppable(&scratch));
    let first = started(&mut screen, &scratch, ID, TITLE);
    screen.mouse(MouseEventKind::Down(MouseButton::Left), 2, 2);
    fs::write(stop_file(&scratch, &first), "").unwrap();
    sleep(Duration::from_millis(400));
    fs::remove_file(stop_file(&scratch, &first)).unwrap();

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for(&format!("acme agent resumed {ID}"));
    assert!(
        shown.contains(&format!("acme agent resumed {ID}")),
        "{shown}"
    );
    assert!(!shown.contains("is open elsewhere"), "{shown}");
}

#[test]
fn a_stopped_card_a_session_here_switches_to_never_says_open_elsewhere() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, &titled(TITLE));
    scratch.transcript(OTHER_ID, &titled(SECOND));
    let mut screen = scratch.watching(stoppable(&scratch));
    let stopped = screen.wait_for("stopped");
    assert!(stopped.contains(TITLE), "{stopped}");
    let pid = started(&mut screen, &scratch, OTHER_ID, SECOND);

    register(&scratch, &pid, ID);

    let mut frames = Vec::new();
    settled(&mut screen, |shown| {
        frames.push(shown.to_string());
        false
    });
    assert!(
        !frames.iter().any(|shown| shown.contains("open elsewhere")),
        "{frames:?}"
    );
}

fn stoppable(scratch: &Scratch) -> Vec<String> {
    let dir = scratch.dir.path().display();
    script(&format!(
        "echo $$ >> '{dir}/pids'; echo \"acme agent registered\"; echo \"acme agent resumed $1\"; \
         while [ ! -e '{dir}/stop.'$$ ]; do sleep 0.05; done; exit 0"
    ))
}

fn started(screen: &mut Screen, scratch: &Scratch, id: &str, title: &str) -> String {
    let before = pids(scratch).len();
    screen.press('n');
    let pid = pids_soon(screen, scratch, before + 1)[before].clone();
    register(scratch, &pid, id);
    let kept = remembered_soon(screen, &scratch.memory(), id, title);
    assert!(kept.iter().any(|remembered| remembered.session_id == id));
    pid
}

fn register(scratch: &Scratch, pid: &str, id: &str) {
    fs::write(
        scratch.sessions().join(format!("{pid}.json")),
        format!(r#"{{"status":"busy","sessionId":"{id}"}}"#),
    )
    .unwrap();
}

fn stop(screen: &mut Screen, scratch: &Scratch, pid: &str) {
    fs::write(stop_file(scratch, pid), "").unwrap();
    let exited = screen.wait_for("exited");
    assert!(exited.contains("exited"), "{exited}");
}

fn stop_file(scratch: &Scratch, pid: &str) -> PathBuf {
    scratch.dir.path().join(format!("stop.{pid}"))
}

fn pids(scratch: &Scratch) -> Vec<String> {
    fs::read_to_string(scratch.dir.path().join("pids"))
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect()
}

fn pids_soon(screen: &mut Screen, scratch: &Scratch, count: usize) -> Vec<String> {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        screen.draw();
        let pids = pids(scratch);
        if pids.len() >= count {
            return pids;
        }
        assert!(Instant::now() < deadline, "{pids:?}");
        sleep(Duration::from_millis(20));
    }
}

fn settled(screen: &mut Screen, mut happened: impl FnMut(&str) -> bool) -> String {
    let deadline = Instant::now() + Duration::from_millis(1500);
    loop {
        let shown = screen.draw();
        if happened(&shown) || Instant::now() > deadline {
            return shown;
        }
        sleep(Duration::from_millis(20));
    }
}

fn subagent(scratch: &Scratch, name: &str, modified: SystemTime) {
    let folder = scratch
        .dir
        .path()
        .join(format!("claude/projects/-home-acme/{ID}/subagents"));
    fs::create_dir_all(&folder).unwrap();
    let file = folder.join(format!("{name}.meta.json"));
    fs::write(&file, r#"{"agentType":"Explore"}"#).unwrap();
    fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(modified)
        .unwrap();
}
