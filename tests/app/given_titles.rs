use harbour::app::App;
use harbour::memory::{Memory, Remembered};

use crate::keeping::{registered, remembered_soon, titled};
use crate::refusing::Elsewhere;
use crate::remembering::{ID, Scratch, TITLE};
use crate::renaming::rename;
use crate::{Screen, script};

const GIVEN: &str = "acme login fix";

#[test]
fn a_title_given_to_a_stopped_card_is_saved_at_once() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    let mut screen = stopped(&scratch);

    rename(&mut screen, GIVEN);

    assert_eq!(saved(&scratch.memory()).given_title.as_deref(), Some(GIVEN));
}

#[test]
fn a_given_title_comes_back_after_a_restart() {
    let scratch = Scratch::new();
    remember_given(&scratch);

    let mut screen = stopped_as(&scratch, GIVEN);

    let shown = screen.draw();
    assert!(!shown.contains(TITLE), "{shown}");
}

#[test]
fn an_empty_title_takes_the_saved_one_away() {
    let scratch = Scratch::new();
    remember_given(&scratch);
    let mut screen = stopped_as(&scratch, GIVEN);

    rename(&mut screen, "");

    assert_eq!(saved(&scratch.memory()).given_title, None);
    screen.wait_for(TITLE);
}

#[test]
fn a_title_given_to_a_running_session_is_saved() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    let mut screen = scratch.watching(registered(&scratch.sessions(), ID));
    screen.press('n');
    remembered_soon(&mut screen, &scratch.memory(), ID, TITLE);

    rename(&mut screen, GIVEN);
    given_soon(&mut screen, &scratch.memory());

    screen.wait_for_stable();
    assert_eq!(saved(&scratch.memory()).given_title.as_deref(), Some(GIVEN));
}

#[test]
fn a_stopped_card_merged_into_the_session_that_resumed_it_gives_it_its_title() {
    let scratch = Scratch::new();
    scratch.transcript(ID, &titled(TITLE));
    remember_given(&scratch);
    let mut screen = scratch.watching(registered(&scratch.sessions(), ID));
    screen.wait_for(GIVEN);

    screen.press('n');

    screen.wait_until("the stopped card merged", |shown| {
        !shown.contains("stopped")
    });
    let shown = screen.wait_for_stable();
    assert!(shown.contains(GIVEN), "{shown}");
    assert!(!shown.contains(TITLE), "{shown}");
    assert_eq!(saved(&scratch.memory()).given_title.as_deref(), Some(GIVEN));
}

#[test]
fn a_card_open_elsewhere_is_not_renamed_and_says_why() {
    let scratch = Scratch::new();
    scratch.remember(ID, TITLE, 0);
    scratch.transcript(ID, r#"{"type":"user","message":{}}"#);
    let _other = Elsewhere::open(&scratch.sessions(), ID);
    let mut screen = scratch.watching(script("exec sleep 600"));
    screen.wait_for("open elsewhere");

    screen.prefixed('r');

    let shown = screen.draw();
    assert!(
        shown.contains("This conversation is open elsewhere"),
        "{shown}"
    );
    assert!(!shown.contains("Title:"), "{shown}");
}

fn stopped(scratch: &Scratch) -> Screen {
    stopped_as(scratch, TITLE)
}

fn stopped_as(scratch: &Scratch, title: &str) -> Screen {
    let mut screen = Screen::of(App::new(script("exec sleep 600")).with_memory(scratch.memory()));
    screen.wait_for(title);
    screen
}

fn remember_given(scratch: &Scratch) {
    scratch
        .memory()
        .keep(&Remembered {
            session_id: ID.to_string(),
            directory: scratch.checkout(),
            title: TITLE.to_string(),
            given_title: Some(GIVEN.to_string()),
            position: 0,
        })
        .unwrap();
}

fn saved(memory: &Memory) -> Remembered {
    memory.recall().into_iter().next().unwrap()
}

fn given_soon(screen: &mut Screen, memory: &Memory) {
    screen.wait_until(&format!("{GIVEN:?} saved"), |_| {
        memory
            .recall()
            .iter()
            .any(|remembered| remembered.given_title.as_deref() == Some(GIVEN))
    });
}
