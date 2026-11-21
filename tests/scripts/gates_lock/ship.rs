use std::fs::{self, File};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use super::{
    Gates, appeared, canonical, gates, hold_by_hand, holder, locked, locking_repository, pretender,
    running_script, said,
};
use crate::helpers::{calls, git, link_script, recorder, stderr};

#[test]
fn a_ship_holds_the_lock_its_gates_run_under_without_taking_it_twice() {
    let dir = tempfile::tempdir().unwrap();
    let work = shipping_repository(dir.path());
    let held = dir.path().join("held");

    let first = Gates::start(
        shipping(dir.path(), &work, dir.path(), &[])
            .env("GATE_HELD", &held)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
        Some(&held),
    );
    let pid = first.id();
    appeared(&held);

    assert_eq!(holder(&work), format!("{pid} {}", canonical(&work)));
    let output = first.finish();
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(!locked(&work));
}

#[test]
fn a_second_ship_waits_while_the_first_runs_its_dev_install() {
    let dir = tempfile::tempdir().unwrap();
    let work = shipping_repository(dir.path());
    let other = second_worktree(dir.path(), &work);
    let installing = dir.path().join("installing");
    let first = installing_ship(dir.path(), &work, &installing);
    let pid = first.id();
    appeared(&installing);
    let second_root = dir.path().join("second");
    fs::create_dir_all(&second_root).unwrap();

    let mut second = Gates::start(
        shipping(
            dir.path(),
            &other,
            &second_root,
            &[("HARBOUR_GATES_WAIT", "120")],
        )
        .stdout(Stdio::null())
        .stderr(File::create(second_root.join("stderr")).unwrap()),
        None,
    );
    let said = said(
        &second_root.join("stderr"),
        &format!(
            "==> waiting for the gates of pid {pid} in {} to finish\n",
            canonical(&work)
        ),
    );

    assert!(second.running(), "{said}");
    assert_eq!(calls(&second_root).len(), 1);
    assert_eq!(calls(&second_root)[0], "cargo fmt");
    let output = first.finish();
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(second.finish().status.success());
    assert_eq!(calls(&second_root)[1], "scrub-check.sh ");
}

#[test]
fn bare_gates_wait_behind_a_ship_in_its_dev_install() {
    let dir = tempfile::tempdir().unwrap();
    let work = shipping_repository(dir.path());
    let other = second_worktree(dir.path(), &work);
    let installing = dir.path().join("installing");
    let first = installing_ship(dir.path(), &work, &installing);
    let pid = first.id();
    appeared(&installing);

    let output = gates(dir.path(), &other, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(&format!(
            "==> waiting for the gates of pid {pid} in {} to finish\n",
            canonical(&work)
        )),
        "{}",
        stderr(&output)
    );
    assert!(first.finish().status.success());
}

#[test]
fn a_ship_whose_gates_fail_drops_the_lock() {
    let dir = tempfile::tempdir().unwrap();
    let work = shipping_repository(dir.path());

    let output = Gates::start(
        shipping(dir.path(), &work, dir.path(), &[("GATE_STATUS", "1")])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
        None,
    )
    .finish();

    assert!(!output.status.success());
    assert!(!calls(dir.path()).contains(&"dev-install.sh ".to_string()));
    assert!(!locked(&work));
}

#[test]
fn gates_handed_a_lock_their_parent_does_not_hold_wait_for_it() {
    let dir = tempfile::tempdir().unwrap();
    let work = shipping_repository(dir.path());
    let other = pretender();
    hold_by_hand(&work, other.id());
    let named = format!("{} /acme/worktree", other.id());

    let output = gates(
        dir.path(),
        &work,
        &[("HARBOUR_GATES_WAIT", "0"), ("HARBOUR_GATES_HELD", &named)],
    );

    assert!(!output.status.success());
    assert!(calls(dir.path()).is_empty());
    assert_eq!(holder(&work), named);
}

fn shipping_repository(root: &Path) -> PathBuf {
    let work = locking_repository(root);
    link_script(&work, "ship.sh");
    symlink(held_install(), work.join("scripts/dev-install.sh")).unwrap();
    symlink(recorder(), root.join("bin/gh")).unwrap();
    git(root, &["init", "-q", "--bare", "origin.git"]);
    git(&work, &["remote", "add", "origin", "../origin.git"]);
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-qm", "the ship"]);
    work
}

fn second_worktree(root: &Path, work: &Path) -> PathBuf {
    git(work, &["worktree", "add", "-q", "-b", "other", "../other"]);
    let other = root.join("other");
    fs::write(other.join("note"), "other\n").unwrap();
    other
}

fn installing_ship(root: &Path, work: &Path, installing: &Path) -> Gates {
    fs::write(work.join("note"), "first\n").unwrap();
    Gates::start(
        shipping(root, work, root, &[])
            .env("INSTALL_HELD", installing)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
        Some(installing),
    )
}

fn shipping(
    root: &Path,
    work: &Path,
    calls_root: &Path,
    env: &[(&str, &str)],
) -> std::process::Command {
    let mut command = running_script(root, work, "ship.sh", calls_root, env);
    command.arg("fix: acme").env("HOME", root.join("home"));
    command
}

fn held_install() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/held-install.sh")
}
