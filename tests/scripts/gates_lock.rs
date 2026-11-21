use std::fs::{self, File};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;

use crate::helpers::{calls, git, link_script, recorder, repository, sandbox_bin, stderr, which};

mod ship;
mod takeover;

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
fn gates_terminated_alone_keep_the_lock_until_the_running_gate_ends_then_drop_it() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let held = dir.path().join("held");

    let first = holding(dir.path(), &work, &held, &[]);
    appeared(&held);
    terminate(first.id());

    assert!(locked(&work));
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
    let other = pretender();
    hold_by_hand(&work, other.id());

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(
        stderr(&output).contains(&format!(
            "==> waiting for the gates of pid {} in /acme/worktree to finish\n",
            other.id()
        )),
        "{}",
        stderr(&output)
    );
}

#[test]
fn gates_give_up_once_the_wait_runs_out_and_run_no_gate() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    let other = pretender();
    hold_by_hand(&work, other.id());

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(&format!(
            "gates: gave up after 0 s waiting for the gates of pid {} in /acme/worktree\n",
            other.id()
        )),
        "{}",
        stderr(&output)
    );
    assert!(calls(dir.path()).is_empty());
}

#[test]
fn gates_refuse_a_wait_that_is_not_a_number_of_seconds() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "soon")]);

    assert!(!output.status.success());
    assert!(
        stderr(&output)
            .contains("gates: HARBOUR_GATES_WAIT must be a whole number of seconds, not soon\n"),
        "{}",
        stderr(&output)
    );
    assert!(calls(dir.path()).is_empty());
}

#[test]
fn gates_wait_for_a_lock_they_cannot_read_say_so_and_give_up() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    fs::write(lock(&work), "").unwrap();

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(&format!(
            "==> waiting for whoever holds {} to finish\n",
            canonical(&lock(&work))
        )),
        "{}",
        stderr(&output)
    );
    assert!(
        stderr(&output).contains(&format!(
            "gates: gave up after 0 s waiting for whoever holds {0}; if no gates run, remove it with: rm {0}\n",
            canonical(&lock(&work))
        )),
        "{}",
        stderr(&output)
    );
}

#[test]
fn gates_refuse_a_wait_too_long_to_compare() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());

    let output = gates(
        dir.path(),
        &work,
        &[("HARBOUR_GATES_WAIT", "99999999999999999999")],
    );

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains(
            "gates: HARBOUR_GATES_WAIT must be a whole number of seconds, not 99999999999999999999\n"
        ),
        "{}",
        stderr(&output)
    );
    assert!(calls(dir.path()).is_empty());
}

#[test]
fn gates_that_find_the_lock_gone_after_a_failed_claim_claim_it_again() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    failing_once(dir.path(), "ln");

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(output.status.success(), "{}", stderr(&output));
    assert!(!stderr(&output).contains("waiting"), "{}", stderr(&output));
}

#[test]
fn gates_wait_for_a_directory_at_the_lock_instead_of_linking_inside_it() {
    let dir = tempfile::tempdir().unwrap();
    let work = locking_repository(dir.path());
    fs::create_dir(lock(&work)).unwrap();

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

    assert!(!output.status.success());
    assert!(calls(dir.path()).is_empty());
    assert_eq!(fs::read_dir(lock(&work)).unwrap().count(), 0);
    assert!(
        stderr(&output).contains(&format!(
            "gates: gave up after 0 s waiting for whoever holds {0}; if no gates run, remove it with: rmdir {0}\n",
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
    let other = pretender();
    hold_by_hand(&work, other.id());

    let output = gates(
        dir.path(),
        &work,
        &[("HARBOUR_GATES_WAIT", "0"), ("CI", "true")],
    );

    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(holder(&work), format!("{} /acme/worktree", other.id()));
}

#[test]
fn gates_outside_a_repository_take_no_lock_and_say_so() {
    let dir = tempfile::tempdir().unwrap();
    sandbox_bin(dir.path());
    link_script(dir.path(), "gates.sh");
    link_script(dir.path(), "gates-lock.sh");
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
    let _writable_again = Unwritable::make(&common);

    let output = gates(dir.path(), &work, &[("HARBOUR_GATES_WAIT", "0")]);

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
fn an_unwritable_directory_is_writable_again_after_a_panic() {
    let dir = tempfile::tempdir().unwrap();
    let common = dir.path().join("common");
    fs::create_dir(&common).unwrap();

    let panicked = std::panic::catch_unwind(|| {
        let _writable_again = Unwritable::make(&common);
        panic!("a failed assertion");
    });

    assert!(panicked.is_err());
    assert_eq!(
        fs::metadata(&common).unwrap().permissions().mode() & 0o777,
        0o755
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
    link_script(&work, "gates-lock.sh");
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

fn gating(root: &Path, work: &Path, calls_root: &Path, env: &[(&str, &str)]) -> Command {
    running_script(root, work, "gates.sh", calls_root, env)
}

fn running_script(
    root: &Path,
    work: &Path,
    script: &str,
    calls_root: &Path,
    env: &[(&str, &str)],
) -> Command {
    let mut command = Command::new(work.join("scripts").join(script));
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
        gating(root, work, calls_root, &[("HARBOUR_GATES_WAIT", "120")])
            .stdout(Stdio::null())
            .stderr(File::create(calls_root.join("stderr")).unwrap()),
        None,
    )
}

fn pretender() -> Gates {
    Gates::start(
        Command::new(which("sleep"))
            .arg0("scripts/gates.sh")
            .arg("60")
            .process_group(0),
        None,
    )
}

fn bystander() -> Gates {
    Gates::start(
        Command::new(which("sleep")).arg("60").process_group(0),
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

    fn finish(self) -> Output {
        self.finish_within(DEADLINE)
    }

    fn finish_within(mut self, deadline: Duration) -> Output {
        self.let_go();
        let start = Instant::now();
        while self.running() {
            assert!(start.elapsed() < deadline, "the gates never finished");
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

struct Unwritable(PathBuf);

impl Unwritable {
    fn make(dir: &Path) -> Self {
        fs::set_permissions(dir, fs::Permissions::from_mode(0o555)).unwrap();
        Self(dir.to_path_buf())
    }
}

impl Drop for Unwritable {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
    }
}

fn terminate(pid: u32) {
    let status = Command::new(which("kill"))
        .args(["-TERM", &pid.to_string()])
        .status()
        .unwrap();
    assert!(status.success());
}

fn failing_once(root: &Path, program: &str) {
    let link = root.join("bin").join(program);
    let real = fs::read_link(&link).unwrap();
    fs::remove_file(&link).unwrap();
    let marker = root.join(format!("{program}.failed"));
    fs::write(
        &link,
        format!(
            "#!/bin/sh\nif [ ! -e '{0}' ]; then : > '{0}'; exit 1; fi\nexec '{1}' \"$@\"\n",
            marker.display(),
            real.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&link, fs::Permissions::from_mode(0o755)).unwrap();
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

fn guard(work: &Path, pid: u32) -> PathBuf {
    work.join(format!(".git/harbour-gates.lock.takeover-{pid}"))
}

fn leftovers(work: &Path) -> Vec<String> {
    fs::read_dir(work.join(".git"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("harbour-gates"))
        .collect()
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
