use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{Duration, Instant};

use harbour::tmux::{Direction, Tmux, located};

const LONG: Duration = Duration::from_secs(5);
const BOUND: &str = "list-keys) echo 'bind-key    -T root C-j select-pane -D'; echo 'bind-key -r -T root C-k select-pane -U' ;;";

#[test]
fn both_variables_give_the_pane() {
    assert_eq!(
        Tmux::pane(Some("/tmp/tmux-1/default,1,0".into()), Some("%7".into())),
        Some("%7".into())
    );
}

#[test]
fn without_tmux_there_is_none() {
    assert!(Tmux::pane(None, Some("%7".into())).is_none());
}

#[test]
fn without_a_pane_there_is_none() {
    assert!(Tmux::pane(Some("/tmp/tmux-1/default,1,0".into()), None).is_none());
}

#[test]
fn an_empty_tmux_counts_as_unset() {
    assert!(Tmux::pane(Some("".into()), Some("%7".into())).is_none());
}

#[test]
fn an_empty_pane_counts_as_unset() {
    assert!(Tmux::pane(Some("/tmp/tmux-1/default,1,0".into()), Some("".into())).is_none());
}

#[test]
fn tmux_on_a_relative_path_entry_is_found_from_where_harbour_was_opened() {
    let scratch = tempfile::tempdir().unwrap();
    let opened = scratch.path().join("opened");
    executable(&opened.join("bin/tmux"));

    let found = located("tmux", "bin".as_ref(), &opened);

    assert_eq!(found, Some(opened.join("bin/tmux")));
}

#[test]
fn tmux_on_an_absolute_path_entry_is_found_there() {
    let scratch = tempfile::tempdir().unwrap();
    executable(&scratch.path().join("tools/tmux"));
    let path =
        std::env::join_paths([scratch.path().join("empty"), scratch.path().join("tools")]).unwrap();

    let found = located("tmux", &path, Path::new("/"));

    assert_eq!(found, Some(scratch.path().join("tools/tmux")));
}

#[test]
fn tmux_on_no_path_entry_is_not_found() {
    let scratch = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(scratch.path().join("bin")).unwrap();
    std::fs::write(scratch.path().join("bin/tmux"), "").unwrap();

    let found = located("tmux", "bin".as_ref(), scratch.path());

    assert_eq!(found, None);
}

#[test]
fn below_asks_whether_the_pane_is_at_the_bottom_then_selects_the_pane_below() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo '110 /dev/acme-tty'");

    let moved = tmux.select(Direction::Below);

    assert!(moved);
    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_bottom} #{pane_tty}\nselect-pane -D -t %7\n"
    );
}

#[test]
fn above_asks_whether_the_pane_is_at_the_top_then_selects_the_pane_above() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo '110 /dev/acme-tty'");

    let moved = tmux.select(Direction::Above);

    assert!(moved);
    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_top} #{pane_tty}\nselect-pane -U -t %7\n"
    );
}

#[test]
fn at_the_edge_nothing_is_selected() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo '111 /dev/acme-tty'");

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_bottom} #{pane_tty}\n"
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
        "list-keys -T root\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_bottom} #{pane_tty}\n"
    );
}

#[test]
fn a_check_that_fails_selects_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo '110 /dev/acme-tty'; exit 1");

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_bottom} #{pane_tty}\n"
    );
}

#[test]
fn a_select_that_fails_is_no_move() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = Tmux::with_program(
        program(
            scratch.path(),
            &format!(
                "case $1 in {BOUND} display-message) echo '110 /dev/acme-tty' ;; select-pane) exit 1 ;; esac"
            ),
        ),
        "%7".into(),
        "/dev/acme-tty".into(),
        LONG,
    );

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
}

#[test]
fn a_tmux_that_cannot_start_is_no_move() {
    let tmux = Tmux::with_program(
        vec!["/nonexistent/acme-tmux".into()],
        "%7".into(),
        "/dev/acme-tty".into(),
        LONG,
    );

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
}

#[test]
fn a_tmux_that_never_answers_is_given_up_on_well_within_a_second() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = Tmux::with_program(
        program(
            scratch.path(),
            &format!(
                "case $1 in {BOUND} display-message) sleep 5; echo '110 /dev/acme-tty' ;; esac"
            ),
        ),
        "%7".into(),
        "/dev/acme-tty".into(),
        Duration::from_millis(100),
    );
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
    let tmux = answering(scratch.path(), "echo '110 /dev/acme-tty'; sleep 5 &");
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
            "case $1 in list-keys) exit 1 ;; display-message) echo '110 /dev/acme-tty' ;; esac",
        ),
        "%7".into(),
        "/dev/acme-tty".into(),
        LONG,
    );

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
    assert_eq!(calls(scratch.path()), "list-keys -T root\n");
}

#[test]
fn a_key_missing_from_the_root_table_tmux_lists_moves_nothing_and_asks_nothing_more() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = Tmux::with_program(
        program(
            scratch.path(),
            "case $1 in list-keys) echo 'bind-key -T root C-k select-pane -U'; echo 'bind-key -T root M-j send-keys C-j' ;; display-message) echo '110 /dev/acme-tty' ;; esac",
        ),
        "%7".into(),
        "/dev/acme-tty".into(),
        LONG,
    );

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
    assert_eq!(calls(scratch.path()), "list-keys -T root\n");
}

#[test]
fn a_key_listed_after_a_long_root_table_is_bound() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = Tmux::with_program(
        program(
            scratch.path(),
            "case $1 in list-keys) i=0; while [ $i -lt 200 ]; do echo \"bind-key -T root M-$i display-message acme-$i\"; i=$((i+1)); done; echo 'bind-key -T root C-j select-pane -D' ;; display-message) echo '110 /dev/acme-tty' ;; esac",
        ),
        "%7".into(),
        "/dev/acme-tty".into(),
        LONG,
    );

    let moved = tmux.select(Direction::Below);

    assert!(moved);
}

#[test]
fn a_key_listed_after_a_root_table_larger_than_a_pipe_holds_is_bound() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = Tmux::with_program(
        program(
            scratch.path(),
            "case $1 in list-keys) i=0; while [ $i -lt 4000 ]; do echo \"bind-key -T root M-$i display-message acme-binding-number-$i\"; i=$((i+1)); done; echo 'bind-key -T root C-j select-pane -D' ;; display-message) echo '110 /dev/acme-tty' ;; esac",
        ),
        "%7".into(),
        "/dev/acme-tty".into(),
        Duration::from_secs(2),
    );

    let moved = tmux.select(Direction::Below);

    assert!(moved);
}

#[test]
fn a_pane_that_is_not_the_active_one_moves_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo '010 /dev/acme-tty'");

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
}

#[test]
fn a_window_that_is_not_the_active_one_moves_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo '100 /dev/acme-tty'");

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
}

#[test]
fn a_pane_on_another_terminal_than_harbour_moves_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let tmux = answering(scratch.path(), "echo '110 /dev/acme-other'");

    let moved = tmux.select(Direction::Below);

    assert!(!moved);
    assert_eq!(
        calls(scratch.path()),
        "list-keys -T root\ndisplay-message -p -t %7 #{pane_active}#{window_active}#{pane_at_bottom} #{pane_tty}\n"
    );
}

#[test]
fn a_select_that_outlives_the_deadline_counts_as_a_move() {
    let scratch = tempfile::tempdir().unwrap();
    let deadline = Duration::from_secs(2);
    let tmux = Tmux::with_program(
        program(
            scratch.path(),
            &format!(
                "case $1 in {BOUND} display-message) echo '110 /dev/acme-tty' ;; select-pane) exec sleep 30 ;; esac"
            ),
        ),
        "%7".into(),
        "/dev/acme-tty".into(),
        deadline,
    );
    let began = Instant::now();

    let moved = tmux.select(Direction::Below);

    let elapsed = began.elapsed();
    assert!(moved);
    assert!(elapsed >= deadline, "{elapsed:?}");
    assert!(elapsed < deadline + Duration::from_secs(3), "{elapsed:?}");
}

fn answering(scratch: &Path, answer: &str) -> Tmux {
    Tmux::with_program(
        program(
            scratch,
            &format!("case $1 in {BOUND} display-message) {answer} ;; esac"),
        ),
        "%7".into(),
        "/dev/acme-tty".into(),
        LONG,
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

fn executable(path: &Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, "#!/bin/sh\n").unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}
