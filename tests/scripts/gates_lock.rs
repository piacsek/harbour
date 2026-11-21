use std::fs::{self, File};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;

use crate::helpers::{calls, git, link_script, recorder, repository, stderr, which};

#[test]
fn gates_hold_the_lock_while_they_run_and_drop_it_once_they_pass() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let held = dir.path().join("held");

    let first = holding(dir.path(), &work, &held, &[]);
    let pid = first.id();
    appeared(&held);

    assert_eq!(holder(&work), format!("{pid} {}", canonical(&work)));
    let output = first.finish();
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(!locked(&work));
}

#[test]
fn gates_drop_the_lock_when_a_gate_fails() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let held = dir.path().join("held");

    let first = holding(dir.path(), &work, &held, &[("GATE_STATUS", "1")]);
    appeared(&held);

    assert!(locked(&work));
    let output = first.finish();
    assert!(!output.status.success());
    assert!(!locked(&work));
}

#[test]
fn gates_drop_the_lock_when_they_are_terminated() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let held = dir.path().join("held");

    let first = holding(dir.path(), &work, &held, &[]);
    appeared(&held);
    terminate(first.id());

    let output = first.finish();
    assert!(!output.status.success());
    assert!(!locked(&work));
}

#[test]
fn gates_wait_for_the_lock_and_run_once_the_holder_finishes() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let held = dir.path().join("held");
    let first = holding(dir.path(), &work, &held, &[]);
    let pid = first.id();
    appeared(&held);
    let second_root = dir.path().join("second");
    fs::create_dir_all(&second_root).unwrap();

    let mut second = waiting(dir.path(), &work, &second_root);
    let said = said(
        &second_root.join("stderr"),
        &format!(
            "==> waiting for the gates of pid {pid} in {} to finish\n",
            canonical(&work)
        ),
    );

    assert!(second.running(), "{said}");
    assert!(calls(&second_root).is_empty());
    first.finish();
    assert!(second.finish().status.success());
    assert_eq!(calls(&second_root)[0], "scrub-check.sh ");
}

#[test]
fn gates_that_find_the_lock_held_say_by_whom() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    hold_by_hand(&work, std::process::id());

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(
        stderr(&output).contains(&format!(
            "==> waiting for the gates of pid {} in /acme/worktree to finish\n",
            std::process::id()
        )),
        "{}",
        stderr(&output)
    );
}

#[test]
fn gates_give_up_once_the_wait_runs_out_and_run_no_gate() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    hold_by_hand(&work, std::process::id());

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(&format!(
            "gates: gave up after 0 s waiting for the gates of pid {} in /acme/worktree\n",
            std::process::id()
        )),
        "{}",
        stderr(&output)
    );
    assert!(calls(dir.path()).is_empty());
}

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
            "==> taking over the gates lock of pid {gone} in /acme/worktree, which is no longer running\n"
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
    symlink(format!("{} /acme/other", std::process::id()), guard(&work)).unwrap();

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
    symlink(format!("{gone} /acme/other"), guard(&work)).unwrap();

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(!locked(&work));
    assert!(fs::symlink_metadata(guard(&work)).is_err());
}

#[test]
fn gates_give_up_on_a_lock_they_cannot_read_instead_of_spinning() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    fs::write(lock(&work), "").unwrap();

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(&format!(
            "gates: gave up after 0 s waiting for {}\n",
            canonical(&lock(&work))
        )),
        "{}",
        stderr(&output)
    );
}

#[test]
fn gates_in_two_worktrees_of_one_repository_share_the_lock() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    git(&work, &["worktree", "add", "-q", "-b", "other", "../other"]);
    let other = dir.path().join("other");
    let held = dir.path().join("held");
    let first = holding(dir.path(), &other, &held, &[]);
    let pid = first.id();
    appeared(&held);

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(
        stderr(&output).contains(&format!(
            "==> waiting for the gates of pid {pid} in {} to finish\n",
            canonical(&other)
        )),
        "{}",
        stderr(&output)
    );
    assert!(!output.status.success());
    first.finish();
}

#[test]
fn gates_in_ci_take_no_lock() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    hold_by_hand(&work, std::process::id());

    let output = gates(
        dir.path(),
        &work,
        &[("HARBOUR_GATES_WAIT", "0"), ("CI", "true")],
    );

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        holder(&work),
        format!("{} /acme/worktree", std::process::id())
    );
}

#[test]
fn gates_outside_a_repository_take_no_lock_and_say_so() {
    let dir = tempfile::tempdir().unwrap();
    sandbox_bin(dir.path());
    link_script(dir.path(), "gates.sh");
    link_script(dir.path(), "script-tests-skippable.sh");
    link_script(dir.path(), "docs-only.sh");
    symlink(recorder(), dir.path().join("scripts/scrub-check.sh")).unwrap();

    let output = gates(dir.path(), dir.path(), &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("==> no gates lock: not in a git repository\n"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn gates_that_cannot_write_the_shared_git_directory_take_no_lock_and_say_so() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let common = work.join(".git");
    fs::set_permissions(&common, fs::Permissions::from_mode(0o555)).unwrap();

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    fs::set_permissions(&common, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    assert!(
        stderr(&output).contains(&format!(
            "==> no gates lock: {} cannot be written\n",
            canonical(&common)
        )),
        "{}",
        stderr(&output)
    );
}

#[test]
#[should_panic(expected = "never appeared")]
fn waiting_for_a_file_that_never_appears_fails() {
    let dir = tempfile::tempdir().unwrap();

    appeared_within(&dir.path().join("never"), Duration::from_millis(50));
}

#[test]
#[should_panic(expected = "never said")]
fn waiting_for_words_never_said_fails() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("stderr");
    fs::write(&file, "something else\n").unwrap();

    said_within(&file, "the words", Duration::from_millis(50));
}

const DEADLINE: Duration = Duration::from_secs(20);

fn locking_repository(root: &Path) -> PathBuf {
    sandbox_bin(root);
    let work = repository(root);
    link_script(&work, "gates.sh");
    link_script(&work, "script-tests-skippable.sh");
    link_script(&work, "docs-only.sh");
    symlink(held_gate(), work.join("scripts/scrub-check.sh")).unwrap();
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-qm", "the gates"]);
    git(&work, &["update-ref", "refs/remotes/origin/main", "HEAD"]);
    work
}

fn held_gate() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/held-gate.sh")
}

fn sandbox_bin(root: &Path) {
    let bin = root.join("bin");
    fs::create_dir_all(&bin).unwrap();
    for program in [
        "bash", "dirname", "git", "grep", "ln", "readlink", "rm", "sed", "sleep",
    ] {
        symlink(which(program), bin.join(program)).unwrap();
    }
    symlink(recorder(), bin.join("cargo")).unwrap();
}

fn gating(root: &Path, work: &Path, calls_root: &Path, env: &[(&str, &str)]) -> Command {
    let mut command = Command::new(work.join("scripts/gates.sh"));
    command
        .env_clear()
        .env("PATH", root.join("bin"))
        .env("CALLS_LOG", calls_root.join("calls.log"))
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CEILING_DIRECTORIES", root)
        .env("HARBOUR_GATES_WAIT", "10")
        .process_group(0);
    for (name, value) in env {
        command.env(name, value);
    }
    command
}

fn gates(root: &Path, work: &Path, env: &[(&str, &str)]) -> Output {
    Gates::start(
        gating(root, work, root, env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
        None,
    )
    .finish()
}

fn holding(root: &Path, work: &Path, held: &Path, env: &[(&str, &str)]) -> Gates {
    Gates::start(
        gating(root, work, root, env)
            .env("GATE_HELD", held)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
        Some(held),
    )
}

fn waiting(root: &Path, work: &Path, calls_root: &Path) -> Gates {
    Gates::start(
        gating(root, work, calls_root, &[])
            .stdout(Stdio::null())
            .stderr(File::create(calls_root.join("stderr")).unwrap()),
        None,
    )
}

struct Gates {
    child: Option<Child>,
    held: Option<PathBuf>,
}

impl Gates {
    fn start(command: &mut Command, held: Option<&Path>) -> Self {
        Self {
            child: Some(command.spawn().unwrap()),
            held: held.map(Path::to_path_buf),
        }
    }

    fn id(&self) -> u32 {
        self.child.as_ref().unwrap().id()
    }

    fn running(&mut self) -> bool {
        self.child.as_mut().unwrap().try_wait().unwrap().is_none()
    }

    fn let_go(&self) {
        if let Some(held) = &self.held {
            let _ = fs::write(format!("{}.release", held.display()), "");
        }
    }

    fn finish(mut self) -> Output {
        self.let_go();
        let start = Instant::now();
        while self.running() {
            assert!(start.elapsed() < DEADLINE, "the gates never finished");
            std::thread::sleep(Duration::from_millis(20));
        }
        self.child.take().unwrap().wait_with_output().unwrap()
    }
}

impl Drop for Gates {
    fn drop(&mut self) {
        let Some(mut child) = self.child.take() else {
            return;
        };
        self.let_go();
        let group = Pid::from_raw(i32::try_from(child.id()).unwrap());
        let _ = killpg(group, Signal::SIGKILL);
        let _ = child.wait();
        let start = Instant::now();
        while killpg(group, None).is_ok() {
            if start.elapsed() > DEADLINE {
                assert!(
                    std::thread::panicking(),
                    "the gates left process group {group} running"
                );
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

fn terminate(pid: u32) {
    let status = Command::new(which("kill"))
        .args(["-TERM", &pid.to_string()])
        .status()
        .unwrap();
    assert!(status.success());
}

fn hold_by_hand(work: &Path, pid: u32) {
    symlink(format!("{pid} /acme/worktree"), lock(work)).unwrap();
}

fn gone_pid() -> u32 {
    let mut child = Command::new(which("true")).spawn().unwrap();
    let pid = child.id();
    child.wait().unwrap();
    pid
}

fn lock(work: &Path) -> PathBuf {
    work.join(".git/harbour-gates.lock")
}

fn guard(work: &Path) -> PathBuf {
    work.join(".git/harbour-gates.lock.takeover")
}

fn locked(work: &Path) -> bool {
    fs::symlink_metadata(lock(work)).is_ok()
}

fn holder(work: &Path) -> String {
    fs::read_link(lock(work))
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn canonical(path: &Path) -> String {
    fs::canonicalize(path).unwrap().display().to_string()
}

fn appeared(file: &Path) {
    appeared_within(file, DEADLINE);
}

fn appeared_within(file: &Path, deadline: Duration) {
    let start = Instant::now();
    while !file.exists() {
        assert!(
            start.elapsed() < deadline,
            "{} never appeared",
            file.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn said(file: &Path, words: &str) -> String {
    said_within(file, words, DEADLINE)
}

fn said_within(file: &Path, words: &str, deadline: Duration) -> String {
    let start = Instant::now();
    loop {
        let text = fs::read_to_string(file).unwrap_or_default();
        if text.contains(words) {
            return text;
        }
        assert!(
            start.elapsed() < deadline,
            "{} never said {words:?}, last: {text:?}",
            file.display()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}
