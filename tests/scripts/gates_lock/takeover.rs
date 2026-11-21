use std::os::unix::fs::symlink;
use std::process::{Output, Stdio};
use std::time::Duration;

use super::{
    Gates, bystander, gates, gating, gone_pid, guard, hold_by_hand, holder, leftovers, locked,
    locking_repository, pretender,
};
use crate::helpers::{calls, stderr};

#[test]
fn gates_take_over_a_lock_whose_holder_is_gone_and_say_so() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let gone = gone_pid();
    hold_by_hand(&work, gone);

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stderr(&output).contains(&format!(
            "==> taking over the gates lock of pid {gone} in /acme/worktree, whose gates no longer run\n"
        )),
        "{}",
        stderr(&output)
    );
    assert!(!locked(&work));
}

#[test]
fn gates_take_over_a_lock_whose_pid_now_runs_something_else() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let other = bystander();
    hold_by_hand(&work, other.id());

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stderr(&output).contains(&format!(
            "==> taking over the gates lock of pid {} in /acme/worktree, whose gates no longer run\n",
            other.id()
        )),
        "{}",
        stderr(&output)
    );
    assert!(!locked(&work));
}

#[test]
fn gates_leave_a_dead_holder_alone_while_another_run_is_taking_it_over() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let gone = gone_pid();
    hold_by_hand(&work, gone);
    let other = pretender();
    symlink(format!("{} /acme/other", other.id()), guard(&work, gone)).unwrap();

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(!output.status.success());
    assert_eq!(holder(&work), format!("{gone} /acme/worktree"));
    assert!(calls(dir.path()).is_empty());
}

#[test]
fn gates_clear_a_takeover_left_by_a_run_that_died_during_it() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let gone = gone_pid();
    hold_by_hand(&work, gone);
    symlink(format!("{gone} /acme/other"), guard(&work, gone)).unwrap();

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(leftovers(&work).is_empty(), "{:?}", leftovers(&work));
}

#[test]
fn gates_racing_over_a_dead_lock_and_a_dead_takeover_run_one_at_a_time() {
    let first = race();
    let second = race();
    let third = race();

    assert_eq!(first.overlaps, 0, "first round overlaps: {}", first.said);
    assert_eq!(first.takeovers, 1, "first round takeovers: {}", first.said);
    assert_eq!(first.passed, RACERS, "first round passed: {}", first.said);
    assert_eq!(first.leftovers, 0, "first round leftovers: {}", first.said);
    assert_eq!(second.overlaps, 0, "second round overlaps: {}", second.said);
    assert_eq!(
        second.takeovers, 1,
        "second round takeovers: {}",
        second.said
    );
    assert_eq!(
        second.passed, RACERS,
        "second round passed: {}",
        second.said
    );
    assert_eq!(
        second.leftovers, 0,
        "second round leftovers: {}",
        second.said
    );
    assert_eq!(third.overlaps, 0, "third round overlaps: {}", third.said);
    assert_eq!(third.takeovers, 1, "third round takeovers: {}", third.said);
    assert_eq!(third.passed, RACERS, "third round passed: {}", third.said);
    assert_eq!(third.leftovers, 0, "third round leftovers: {}", third.said);
}

const RACERS: usize = 8;
const ROUND: Duration = Duration::from_secs(180);

struct Round {
    overlaps: usize,
    takeovers: usize,
    passed: usize,
    leftovers: usize,
    said: String,
}

fn race() -> Round {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let gone = gone_pid();
    hold_by_hand(&work, gone);
    symlink(format!("{gone} /acme/other"), guard(&work, gone)).unwrap();
    let alone = dir.path().join("alone");
    let alone_text = alone.display().to_string();
    let racers: Vec<Gates> = (0..RACERS)
        .map(|_| {
            Gates::start(
                gating(
                    dir.path(),
                    &work,
                    dir.path(),
                    &[("GATE_ALONE", &alone_text), ("HARBOUR_GATES_WAIT", "180")],
                )
                .stdout(Stdio::null())
                .stderr(Stdio::piped()),
                None,
            )
        })
        .collect();
    let outputs: Vec<Output> = racers
        .into_iter()
        .map(|racer| racer.finish_within(ROUND))
        .collect();
    Round {
        overlaps: std::fs::read_to_string(dir.path().join("alone.overlaps"))
            .unwrap_or_default()
            .lines()
            .count(),
        takeovers: outputs
            .iter()
            .filter(|output| stderr(output).contains("==> taking over"))
            .count(),
        passed: outputs
            .iter()
            .filter(|output| output.status.success())
            .count(),
        leftovers: leftovers(&work).len(),
        said: outputs.iter().map(stderr).collect::<Vec<_>>().join("\n"),
    }
}
