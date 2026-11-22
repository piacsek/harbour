use std::ffi::OsString;
use std::path::Path;
use std::time::{Duration, Instant};

use harbour::tmux::{Direction, Tmux};

#[test]
fn both_variables_make_a_tmux() {
    assert!(Tmux::from_vars(Some("/tmp/tmux-1/default,1,0".into()), Some("%7".into())).is_some());
}

#[test]
fn without_tmux_there_is_none() {
    assert!(Tmux::from_vars(None, Some("%7".into())).is_none());
}

#[test]
fn without_a_pane_there_is_none() {
    assert!(Tmux::from_vars(Some("/tmp/tmux-1/default,1,0".into()), None).is_none());
}

#[test]
fn an_empty_tmux_counts_as_unset() {
    assert!(Tmux::from_vars(Some("".into()), Some("%7".into())).is_none());
}

#[test]
fn an_empty_pane_counts_as_unset() {
    assert!(Tmux::from_vars(Some("/tmp/tmux-1/default,1,0".into()), Some("".into())).is_none());
}

#[test]
fn below_asks_whether_the_pane_is_at_the_bottom_then_selects_the_pane_below() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo 110");

    let moved = tmux.select(Direction::Below);

    assert!(moved);
    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root C-j\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_bottom}\nselect-pane -D -t %7\n"
    );
}

#[test]
fn above_asks_whether_the_pane_is_at_the_top_then_selects_the_pane_above() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo 110");

    let moved = tmux.select(Direction::Above);

    assert!(moved);
    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root C-k\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_top}\nselect-pane -U -t %7\n"
    );
}

#[test]
fn at_the_edge_nothing_is_selected() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo 111");

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root C-j\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_bottom}\n"
    );
}

#[test]
fn an_empty_answer_selects_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo");

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root C-j\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_bottom}\n"
    );
}

#[test]
fn a_check_that_fails_selects_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo 110; exit 1");

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root C-j\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_bottom}\n"
    );
}

#[test]
fn a_select_that_fails_is_no_move() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = Tmux::with_program(
        program(
            scratch.path(),
            "case $1 in display-message) echo 110 ;; select-pane) exit 1 ;; esac",
        ),
        "%7".into(),
    );

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
}

#[test]
fn a_tmux_that_cannot_start_is_no_move() {
    let tmux = Tmux::with_program(vec!["/nonexistent/acme-tmux".into()], "%7".into());

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
}

#[test]
fn a_tmux_that_never_answers_is_given_up_on_well_within_a_second() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "sleep 5; echo 110");
    let began = Instant::now();

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
    assert!(
        began.elapsed() < Duration::from_millis(500),
        "{:?}",
        began.elapsed()
    );
}

#[test]
fn an_answer_whose_output_stays_open_after_it_exits_is_read_at_once() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo 110; sleep 5 &");
    let began = Instant::now();

    let moved = tmux.select(Direction::Below);

    assert!(moved);
    assert!(
        began.elapsed() < Duration::from_millis(500),
        "{:?}",
        began.elapsed()
    );
}

#[test]
fn a_key_tmux_does_not_bind_moves_nothing_and_asks_nothing_more() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = Tmux::with_program(
        program(
            scratch.path(),
            "case $1 in list-keys) exit 1 ;; display-message) echo 110 ;; esac",
        ),
        "%7".into(),
    );

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
    assert_eq!(calls(scratch.path()), "list-keys -T root C-j\n");
}

#[test]
fn a_pane_that_is_not_the_active_one_moves_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo 010");

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
}

#[test]
fn a_window_that_is_not_the_active_one_moves_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo 100");

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
}

fn answering(scratch: &Path, answer: &str) -> Tmux {
    Tmux::with_program(
        program(
            scratch,
            &format!("case $1 in display-message) {answer} ;; esac"),
        ),
        "%7".into(),
    )
}

fn program(scratch: &Path, body: &str) -> Vec<OsString> {
    let calls = scratch.join("calls");
    vec![
        "/bin/sh".into(),
        "-c".into(),
        format!("echo \"$*\" >> '{}'; {body}", calls.display()).into(),
        "tmux".into(),
    ]
}

fn calls(scratch: &Path) -> String {
    std::fs::read_to_string(scratch.join("calls")).unwrap_or_default()
}
