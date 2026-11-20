use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::memory::{Memory, Remembered};

use crate::remembering::{ID, OTHER_ID, Scratch, TITLE};
use crate::{Screen, home, script};

#[test]
fn a_running_session_is_remembered_once_its_transcript_exists() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(registered(&scratch.sessions(), ID));
    screen.press('n');

    let kept = remembered_soon(&mut screen, &scratch.memory(), ID, TITLE);

    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].session_id, ID);
    assert_eq!(kept[0].title, TITLE);
    assert_eq!(kept[0].directory, home());
    assert_eq!(kept[0].position, 0);
}

#[test]
fn a_running_session_without_a_transcript_is_not_remembered() {
    let scratch = Scratch::new();
    let mut screen = scratch.watching(registered(&scratch.sessions(), ID));
    screen.press('n');
    let shown = screen.wait_for("acme agent registered");
    assert!(shown.contains("acme agent registered"), "{shown}");

    let kept = remembered_soon(&mut screen, &scratch.memory(), ID, TITLE);

    assert_eq!(kept, Vec::new());
}

#[test]
fn a_session_without_a_title_is_remembered_under_its_name() {
    let scratch = Scratch::new();
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let mut screen = scratch.watching(registered(&scratch.sessions(), ID));
    screen.press('n');

    let kept = remembered_soon(&mut screen, &scratch.memory(), ID, "acme-home 1");

    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].title, "acme-home 1");
}

#[test]
fn a_new_title_is_remembered() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled("A first guess"));
    let mut screen = scratch.watching(registered(&scratch.sessions(), ID));
    screen.press('n');
    let first = remembered_soon(&mut screen, &scratch.memory(), ID, "A first guess");
    assert_eq!(first.len(), 1);

    scratch.transcript(ID, &titled("A better title"));

    let kept = remembered_soon(&mut screen, &scratch.memory(), ID, "A better title");
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].title, "A better title");
}

#[test]
fn a_new_session_id_replaces_the_remembered_one() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    scratch.transcript(OTHER_ID, &titled("The second one"));
    let pid_file = scratch.dir.path().join("pid");
    let mut screen = scratch.watching(registered_with_pid(&scratch.sessions(), ID, &pid_file));
    screen.press('n');
    let first = remembered_soon(&mut screen, &scratch.memory(), ID, TITLE);
    assert_eq!(first.len(), 1);

    let pid = std::fs::read_to_string(&pid_file).unwrap();
    std::fs::write(
        scratch.sessions().join(format!("{}.json", pid.trim())),
        format!(r#"{{"status":"idle","sessionId":"{OTHER_ID}"}}"#),
    )
    .unwrap();

    let kept = remembered_soon(&mut screen, &scratch.memory(), OTHER_ID, "The second one");
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].session_id, OTHER_ID);
    assert!(!scratch.memory().holds(ID));
}

#[test]
fn a_remembered_session_that_exits_resumes_with_enter() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(script(&format!(
        "printf '%s' '{{\"status\":\"busy\",\"sessionId\":\"{ID}\"}}' > '{}'/$$.json; \
         echo \"acme agent resumed:$1\"; sleep 1; exit 0",
        scratch.sessions().display()
    )));
    screen.press('n');
    let kept = remembered_soon(&mut screen, &scratch.memory(), ID, TITLE);
    assert_eq!(kept.len(), 1);
    let exited = screen.wait_for("exited");
    assert!(exited.contains("exited"), "{exited}");

    screen.key(KeyCode::Enter, KeyModifiers::NONE);

    let shown = screen.wait_for(&format!("acme agent resumed:{ID}"));
    assert!(
        shown.contains(&format!("acme agent resumed:{ID}")),
        "{shown}"
    );
}

#[test]
fn x_on_an_exited_remembered_session_forgets_it() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(script(&format!(
        "printf '%s' '{{\"status\":\"busy\",\"sessionId\":\"{ID}\"}}' > '{}'/$$.json; \
         echo 'acme agent'; sleep 1; exit 0",
        scratch.sessions().display()
    )));
    screen.press('n');
    let kept = remembered_soon(&mut screen, &scratch.memory(), ID, TITLE);
    assert_eq!(kept.len(), 1);
    let exited = screen.wait_for("exited");
    assert!(exited.contains("exited"), "{exited}");

    screen.press('x');

    assert_eq!(scratch.memory().recall(), Vec::new());
}

pub(crate) fn titled(title: &str) -> String {
    format!(r#"{{"type":"ai-title","aiTitle":"{title}"}}"#)
}

pub(crate) fn registered(sessions: &Path, id: &str) -> Vec<String> {
    script(&format!(
        "printf '%s' '{{\"status\":\"busy\",\"sessionId\":\"{id}\"}}' > '{}'/$$.json; \
         echo 'acme agent registered'; exec sleep 600",
        sessions.display()
    ))
}

fn registered_with_pid(sessions: &Path, id: &str, pid_file: &Path) -> Vec<String> {
    script(&format!(
        "echo $$ > '{}'; \
         printf '%s' '{{\"status\":\"busy\",\"sessionId\":\"{id}\"}}' > '{}'/$$.json; \
         echo 'acme agent registered'; exec sleep 600",
        pid_file.display(),
        sessions.display()
    ))
}

pub(crate) fn remembered_soon(
    screen: &mut Screen,
    memory: &Memory,
    id: &str,
    title: &str,
) -> Vec<Remembered> {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        screen.draw();
        let kept = memory.recall();
        let done = kept
            .iter()
            .any(|remembered| remembered.session_id == id && remembered.title == title);
        if done || Instant::now() > deadline {
            return kept;
        }
        sleep(Duration::from_millis(20));
    }
}
