use std::fs::{self, File};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::picker::Picker;

use crate::logging::{logged, only, read};
use crate::remembering::{ID, OTHER_ID, Scratch};
use crate::{Screen, deadline, directory, needs_a_terminal, script};

const HOUR: Duration = Duration::from_secs(60 * 60);
const THIRD_ID: &str = "5c4b3a29-1807-4f6e-9d8c-7b6a59483726";
const EVERY: Duration = Duration::from_millis(50);

#[test]
fn an_exited_card_is_removed_once_it_has_not_run_for_the_setting() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::of(tidying(exiting_agent(scratch.path(), 1), deadline()));
    screen.press('n');
    screen.wait_for("exited");
    screen.press('n');

    let shown = screen.wait_until("the exited card removed", |shown| {
        !shown.contains(&card(1)) && shown.contains("acme agent 2 ready")
    });

    assert!(shown.contains(&card(2)), "{shown}");
}

#[test]
fn the_selected_card_is_kept_until_the_selection_moves() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::of(tidying(
        exiting_agent(scratch.path(), 2),
        Duration::from_millis(100),
    ));
    screen.press('n');
    screen.wait_for("acme agent 1 ready");
    screen.press('n');
    screen.wait_for("exited");
    sleep(deadline());
    let kept = screen.draw();
    assert!(kept.contains(&card(2)), "{kept}");

    screen.press('k');

    let shown = screen.wait_until("the exited card removed", |shown| !shown.contains(&card(2)));
    assert!(shown.contains(&card(1)), "{shown}");
}

#[test]
fn no_card_is_removed_while_a_question_is_open() {
    kept_while(
        |screen| {
            screen.press('x');
            screen.wait_for("End? y/n");
        },
        |screen| screen.key(KeyCode::Esc, KeyModifiers::NONE),
    );
}

#[test]
fn no_card_is_removed_while_the_rename_popup_is_open() {
    kept_while(
        |screen| {
            screen.prefixed('r');
            screen.wait_for(" rename ");
        },
        |screen| screen.key(KeyCode::Esc, KeyModifiers::NONE),
    );
}

#[test]
fn no_card_is_removed_while_the_prefix_waits_for_its_key() {
    kept_while(
        |screen| {
            screen.key(KeyCode::Char('/'), KeyModifiers::CONTROL);
        },
        |screen| screen.key(KeyCode::Esc, KeyModifiers::NONE),
    );
}

#[test]
fn a_stopped_card_whose_conversation_has_not_changed_for_the_setting_is_removed_at_start() {
    let scratch = Scratch::new();
    remembered(&scratch, ID, "The first one", 0, HOUR / 2);
    remembered(&scratch, OTHER_ID, "The second one", 1, 2 * HOUR);

    let mut screen = Screen::of(scratch.app(idle_agent()).with_tidy_after(HOUR));

    let shown = screen.wait_for("The first one");
    assert!(!shown.contains("The second one"), "{shown}");
    let kept = scratch.memory().recall();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].session_id, ID);
}

#[test]
fn a_stopped_card_whose_conversation_changed_lately_is_kept() {
    let scratch = Scratch::new();
    remembered(&scratch, ID, "The first one", 0, HOUR / 2);
    remembered(&scratch, OTHER_ID, "The second one", 1, HOUR / 2);

    let mut screen = Screen::of(scratch.app(idle_agent()).with_tidy_after(HOUR));

    let shown = screen.wait_for("The first one");
    assert!(shown.contains("The second one"), "{shown}");
}

#[test]
fn removing_cards_logs_how_many_and_never_their_titles() {
    let scratch = Scratch::new();
    remembered(&scratch, ID, "The first one", 0, HOUR / 2);
    remembered(&scratch, OTHER_ID, "The second one", 1, 2 * HOUR);
    let (app, lines) = logged(scratch.app(idle_agent()).with_tidy_after(HOUR));

    let mut screen = Screen::of(app);
    screen.wait_for("The first one");

    let tidied = only(&read(&lines), "tidy ");
    assert!(tidied.ends_with(" tidy cards=1\n"), "{tidied}");
}

#[test]
fn no_card_is_removed_while_the_usage_popup_is_open() {
    kept_while_in(
        |app| {
            app.with_usage(script("echo 'acme usage loading'; exec sleep 600"))
                .with_usage_place(Some(crate::usage_states::place()))
        },
        |screen| {
            screen.prefixed('u');
            screen.wait_for(" usage ");
        },
        |screen| screen.key(KeyCode::Esc, KeyModifiers::NONE),
    );
}

#[test]
fn no_card_is_removed_while_the_picker_is_open() {
    needs_a_terminal();
    let picks = tempfile::tempdir().unwrap();
    let count = picks.path().join("count");
    let picker = script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         if [ \"$n\" -lt 3 ]; then echo '{home}'; exit 0; fi; \
         echo 'acme picker waiting' >&2; read -r line; sleep 0.5; exit 1",
        count = count.display(),
        home = crate::home().display()
    ));
    kept_while_in(
        |app| app.with_picker(Picker::Command(picker)),
        |screen| {
            screen.press('n');
            screen.wait_for("acme picker waiting");
        },
        |screen| screen.key(KeyCode::Enter, KeyModifiers::NONE),
    );
}

#[test]
fn an_exited_card_counts_its_age_by_the_wall_clock() {
    let scratch = tempfile::tempdir().unwrap();
    let ahead = Arc::new(AtomicU64::new(0));
    let clock = ahead.clone();
    let mut screen = Screen::of(
        tidying(exiting_agent(scratch.path(), 1), 6 * HOUR).with_tidy_clock(Box::new(move || {
            SystemTime::now() + Duration::from_secs(clock.load(Ordering::SeqCst))
        })),
    );
    screen.press('n');
    screen.wait_for("exited");
    screen.press('n');
    screen.wait_for("acme agent 2 ready");

    ahead.store(7 * HOUR.as_secs(), Ordering::SeqCst);

    let shown = screen.wait_until("the exited card removed", |shown| !shown.contains(&card(1)));
    assert!(shown.contains(&card(2)), "{shown}");
}

#[test]
fn a_removed_stopped_card_open_elsewhere_keeps_its_file() {
    let scratch = Scratch::new();
    remembered(&scratch, ID, "The first one", 0, HOUR / 2);
    remembered(&scratch, OTHER_ID, "The second one", 1, 2 * HOUR);
    fs::write(
        scratch
            .sessions()
            .join(format!("{}.json", std::process::id())),
        format!("{{\"sessionId\":\"{OTHER_ID}\"}}"),
    )
    .unwrap();

    let mut screen = Screen::of(scratch.app(idle_agent()).with_tidy_after(HOUR));

    let shown = screen.wait_for("The first one");
    assert!(!shown.contains("The second one"), "{shown}");
    assert_eq!(scratch.memory().recall().len(), 2);
}

#[test]
fn the_cards_left_after_a_tidy_keep_their_places_on_screen() {
    let scratch = Scratch::new();
    remembered(&scratch, ID, "The first one", 0, HOUR / 2);
    remembered(&scratch, OTHER_ID, "The second one", 1, 2 * HOUR);
    remembered(&scratch, THIRD_ID, "The third one", 2, HOUR / 2);

    let mut screen = Screen::of(scratch.app(idle_agent()).with_tidy_after(HOUR));
    screen.wait_for("The third one");

    let kept = scratch.memory().recall();
    assert_eq!(kept.len(), 2);
    assert_eq!(kept[1].session_id, THIRD_ID);
    assert_eq!(kept[1].position, 1);
}

fn kept_while(open: impl FnOnce(&mut Screen), close: impl FnOnce(&mut Screen)) {
    kept_while_in(|app| app, open, close);
}

fn kept_while_in(
    extra: impl FnOnce(App) -> App,
    open: impl FnOnce(&mut Screen),
    close: impl FnOnce(&mut Screen),
) {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = Screen::of(extra(tidying(exiting_agent(scratch.path(), 2), deadline())));
    screen.press('n');
    screen.wait_for("acme agent 1 ready");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('n');
    screen.wait_for("exited");
    screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    screen.press('k');
    open(&mut screen);
    sleep(2 * deadline());
    let kept = screen.draw();
    assert!(kept.contains("exited"), "{kept}");

    close(&mut screen);

    let shown = screen.wait_until("the exited card removed", |shown| {
        !shown.contains(&card(2)) && shown.contains(&card(1))
    });
    assert!(shown.contains(&card(1)), "{shown}");
}

fn tidying(command: Vec<String>, after: Duration) -> App {
    App::new(command)
        .with_tidy_after(after)
        .with_tidy_every(EVERY)
}

fn exiting_agent(scratch: &Path, exiting: usize) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         if [ \"$n\" = {exiting} ]; then echo \"acme agent $n done\"; exit 0; fi; \
         echo \"acme agent $n ready\"; exec sleep 600",
        count = count.display()
    ))
}

fn idle_agent() -> Vec<String> {
    script("exec sleep 600")
}

fn card(number: usize) -> String {
    format!("{} {number}", directory())
}

fn remembered(scratch: &Scratch, id: &str, title: &str, position: usize, age: Duration) {
    scratch.remember(id, title, position);
    scratch.transcript(id, "{\"type\":\"user\"}");
    File::options()
        .write(true)
        .open(scratch.transcript_file(id))
        .unwrap()
        .set_modified(SystemTime::now() - age)
        .unwrap();
}
