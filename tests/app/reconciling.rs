use crate::keeping::{registered, remembered_soon, titled};
use crate::remembering::{ID, Scratch, TITLE};
use crate::script;

#[test]
fn a_title_written_just_before_the_session_exits_is_remembered() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(script(&format!(
        "printf '%s' '{{\"status\":\"busy\",\"sessionId\":\"{ID}\"}}' > '{}'/$$.json; \
         sleep 1; printf '%s\\n' '{}' >> '{}'; exit 0",
        scratch.sessions().display(),
        titled("The last word"),
        scratch.transcript_file(ID).display()
    )));
    screen.press('n');
    let first = remembered_soon(&mut screen, &scratch.memory(), ID, TITLE);
    assert_eq!(first.len(), 1);
    let exited = screen.wait_for("sessions ○ 1");
    assert!(exited.contains("sessions ○ 1"), "{exited}");

    let kept = remembered_soon(&mut screen, &scratch.memory(), ID, "The last word");

    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].title, "The last word");
}

#[test]
fn a_title_written_just_before_quitting_is_remembered() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(registered(&scratch.sessions(), ID));
    screen.press('n');
    let first = remembered_soon(&mut screen, &scratch.memory(), ID, TITLE);
    assert_eq!(first.len(), 1);
    scratch.transcript(ID, &titled("The last word"));

    screen.press('q');
    screen.press('y');

    let kept = scratch.memory().recall();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].title, "The last word");
}

#[test]
fn a_remembered_file_deleted_from_outside_is_written_again() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(registered(&scratch.sessions(), ID));
    screen.press('n');
    let first = remembered_soon(&mut screen, &scratch.memory(), ID, TITLE);
    assert_eq!(first.len(), 1);

    scratch.memory().forget(ID);

    let kept = remembered_soon(&mut screen, &scratch.memory(), ID, TITLE);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].session_id, ID);
}

#[test]
fn a_new_session_that_resumes_a_stopped_card_merges_the_two() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(registered(&scratch.sessions(), ID));
    let before = screen.wait_for("Stopped. Enter resumes it");
    assert!(before.contains("Stopped. Enter resumes it"), "{before}");

    screen.press('n');

    let shown = screen.wait_until("the stopped card merged", |shown| {
        !shown.contains("stopped")
    });
    assert!(!shown.contains("stopped"), "{shown}");
    assert_eq!(shown.matches(TITLE).count(), 1, "{shown}");
    assert!(shown.contains("acme agent registered"), "{shown}");
    assert_eq!(scratch.memory().recall().len(), 1);
}
