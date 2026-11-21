use std::path::Path;

use crossterm::event::{KeyCode, KeyModifiers};
use harbour::app::App;
use harbour::picker::Picker;

use crate::{Screen, script};

pub(crate) const FINDER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/finder/fzf");

#[test]
fn alt_s_lists_the_sessions_in_fzf_and_enter_jumps_to_the_chosen_one() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");
    screen.type_text("home 2\r");
    screen.wait_until("the popup to close", |shown| !shown.contains("acme fzf"));
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 2 got:");
    assert!(shown.contains("acme agent 2 got: hi"), "{shown}");
}

#[test]
fn the_popup_says_sessions_and_hands_fzf_one_line_per_card_in_sidebar_order() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());

    alt_s(&mut screen);

    let shown = screen.wait_for("acme fzf ready");
    assert!(shown.contains("┏ sessions ━"), "{shown}");
    assert!(shown.contains("0|·|acme-home 1 |acme-home |"), "{shown}");
    assert!(shown.contains("1|·|acme-home 2 |acme-home |"), "{shown}");
    assert!(shown.contains("2|·|acme-home 3 |acme-home |"), "{shown}");
}

#[test]
fn fzf_is_given_the_options_that_hide_and_skip_the_number() {
    let scratch = tempfile::tempdir().unwrap();
    let picker = Picker::Listed {
        fzf: vec!["fzf".to_string()],
        lines: Vec::new(),
    };

    let command = picker.command(scratch.path(), &scratch.path().join("picked"));

    let fzf = command
        .iter()
        .position(|argument| argument == "fzf")
        .unwrap();
    assert_eq!(command[fzf + 1], "--no-expect");
    assert_eq!(command[fzf + 2], "--no-print-query");
    assert_eq!(command[fzf + 3], "--no-read0");
    assert_eq!(command[fzf + 4], "--no-print0");
    assert_eq!(command[fzf + 5], "--delimiter=\t");
    assert_eq!(command[fzf + 6], "--with-nth=2..");
    assert_eq!(command[fzf + 7], "--nth=2,3");
    assert_eq!(command[fzf + 8], "--tabstop=1");
    assert_eq!(command[fzf + 9], "--no-sort");
    assert_eq!(command[fzf + 10], "--layout=reverse");
    assert_eq!(command[fzf + 11], "--bind=enter:accept");
    assert_eq!(command[fzf + 12], "--no-preview");
    assert_eq!(command[fzf + 13], "--header-lines=0");
    assert_eq!(command.len(), fzf + 14);
}

#[test]
fn esc_closes_the_popup_and_changes_neither_the_selection_nor_the_focus() {
    let scratch = tempfile::tempdir().unwrap();
    let mut screen = three_sessions(scratch.path());
    alt_s(&mut screen);
    screen.wait_for("acme fzf ready");

    screen.key(KeyCode::Esc, KeyModifiers::NONE);
    screen.type_text("\r");
    screen.wait_until("the popup to close", |shown| !shown.contains("acme fzf"));
    screen.press('k');
    screen.key(KeyCode::Enter, KeyModifiers::NONE);
    screen.type_text("hi\r");

    let shown = screen.wait_for("acme agent 2 got:");
    assert!(shown.contains("acme agent 2 got: hi"), "{shown}");
}

fn alt_s(screen: &mut Screen) {
    screen.key(KeyCode::Char('s'), KeyModifiers::ALT);
}

fn three_sessions(scratch: &Path) -> Screen {
    let app = App::new(lined_agent(scratch)).with_picker(Picker::Command(vec![FINDER.into()]));
    let mut screen = Screen::of(app);
    for n in 1..=3 {
        screen.press('n');
        screen.wait_for(&format!("acme agent {n}"));
        screen.key(KeyCode::Char('h'), KeyModifiers::CONTROL);
    }
    screen
}

fn lined_agent(scratch: &Path) -> Vec<String> {
    let count = scratch.join("count");
    script(&format!(
        "n=$(($(cat '{count}' 2>/dev/null || echo 0) + 1)); echo $n > '{count}'; \
         echo \"acme agent $n\"; \
         while read -r line; do echo \"acme agent $n got: $line\"; done; exec sleep 600",
        count = count.display()
    ))
}
