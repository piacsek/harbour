use std::fs::{self, File};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn with_no_harbour_ever_run_status_prints_nothing() {
    let state = tempfile::tempdir().unwrap();

    let output = status(state.path());

    assert!(output.stdout.is_empty(), "{}", stdout(&output));
}

#[test]
fn a_bad_config_file_is_never_read() {
    let state = tempfile::tempdir().unwrap();
    let config = state.path().join(".config/harbour");
    fs::create_dir_all(&config).unwrap();
    fs::write(config.join("config.toml"), "retention_days = 7\n").unwrap();

    let output = status(state.path());

    assert!(output.stderr.is_empty());
    assert!(output.status.success());
}

#[test]
fn with_no_harbour_ever_run_status_exits_zero() {
    let state = tempfile::tempdir().unwrap();

    assert!(status(state.path()).status.success());
}

#[test]
fn status_writes_nothing_on_stderr() {
    let state = tempfile::tempdir().unwrap();
    let _harbour = running(state.path(), 101, "1 1 3 0");

    assert!(status(state.path()).stderr.is_empty());
}

#[test]
fn with_no_harbour_running_status_prints_nothing() {
    let state = tempfile::tempdir().unwrap();
    fs::create_dir_all(summaries(state.path())).unwrap();

    assert_eq!(stdout(&status(state.path())), "");
}

#[test]
fn a_harbour_with_no_session_prints_nothing() {
    let state = tempfile::tempdir().unwrap();
    let _harbour = running(state.path(), 101, "0 0 0 0");

    assert_eq!(stdout(&status(state.path())), "");
}

#[test]
fn one_harbour_prints_its_counts_blocked_working_then_idle() {
    let state = tempfile::tempdir().unwrap();
    let _harbour = running(state.path(), 101, "1 1 3 0");

    assert_eq!(
        stdout(&status(state.path())),
        "#[fg=red]◉ 1#[default] #[fg=yellow]● 1#[default] #[dim]○ 3#[default]\n"
    );
}

#[test]
fn unknown_sessions_come_last_in_bright_black() {
    let state = tempfile::tempdir().unwrap();
    let _harbour = running(state.path(), 101, "0 2 0 4");

    assert_eq!(
        stdout(&status(state.path())),
        "#[fg=yellow]● 2#[default] #[fg=brightblack]· 4#[default]\n"
    );
}

#[test]
fn one_harbour_with_sessions_exits_zero() {
    let state = tempfile::tempdir().unwrap();
    let _harbour = running(state.path(), 101, "1 1 3 0");

    assert!(status(state.path()).status.success());
}

#[test]
fn two_harbours_print_the_sum_of_both() {
    let state = tempfile::tempdir().unwrap();
    let _first = running(state.path(), 101, "1 0 3 0");
    let _second = running(state.path(), 202, "1 2 0 1");

    assert_eq!(
        stdout(&status(state.path())),
        "#[fg=red]◉ 2#[default] #[fg=yellow]● 2#[default] #[dim]○ 3#[default] #[fg=brightblack]· 1#[default]\n"
    );
}

#[test]
fn a_summary_left_by_a_harbour_that_crashed_prints_nothing() {
    let state = tempfile::tempdir().unwrap();
    crashed(state.path(), 101, "1 1 3 0");

    assert_eq!(stdout(&status(state.path())), "");
}

#[test]
fn a_summary_left_by_a_harbour_that_crashed_is_left_out_of_the_sum() {
    let state = tempfile::tempdir().unwrap();
    crashed(state.path(), 101, "5 5 5 5");
    let _harbour = running(state.path(), 202, "0 1 0 0");

    assert_eq!(stdout(&status(state.path())), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn a_summary_that_cannot_be_parsed_is_left_out_of_the_sum() {
    let state = tempfile::tempdir().unwrap();
    let _broken = running(state.path(), 101, "acme");
    let _harbour = running(state.path(), 202, "0 1 0 0");

    assert_eq!(stdout(&status(state.path())), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn a_summary_with_too_few_counts_is_left_out_of_the_sum() {
    let state = tempfile::tempdir().unwrap();
    let _broken = running(state.path(), 101, "1 1 3");
    let _harbour = running(state.path(), 202, "0 1 0 0");

    assert_eq!(stdout(&status(state.path())), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn a_summary_that_cannot_be_read_is_left_out_of_the_sum() {
    let state = tempfile::tempdir().unwrap();
    let _harbour = running(state.path(), 202, "0 1 0 0");
    let _broken = running(state.path(), 101, "0 0 0 0");
    let counts = summaries(state.path()).join(format!("{}.counts", name(101)));
    fs::remove_file(&counts).unwrap();
    fs::create_dir(&counts).unwrap();

    assert_eq!(stdout(&status(state.path())), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn a_summary_with_no_lock_beside_it_prints_nothing() {
    let state = tempfile::tempdir().unwrap();
    crashed(state.path(), 101, "1 1 3 0");
    fs::remove_file(summaries(state.path()).join(format!("{}.lock", name(101)))).unwrap();

    assert_eq!(stdout(&status(state.path())), "");
}

#[test]
fn a_file_that_is_not_a_summary_is_ignored() {
    let state = tempfile::tempdir().unwrap();
    let _harbour = running(state.path(), 101, "0 1 0 0");
    fs::write(
        summaries(state.path()).join(format!("{}.tmp", name(101))),
        "9 9 9 9",
    )
    .unwrap();

    assert_eq!(stdout(&status(state.path())), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn a_summary_not_named_like_a_publication_is_left_out_of_the_sum() {
    let state = tempfile::tempdir().unwrap();
    let _harbour = running(state.path(), 101, "0 1 0 0");
    let dir = summaries(state.path());
    fs::write(dir.join("acme.counts"), "1 0 0 0\n").unwrap();
    let other = File::create(dir.join("acme.lock")).unwrap();
    other.lock().unwrap();

    assert_eq!(stdout(&status(state.path())), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn a_symlink_to_a_live_summary_is_not_counted_again() {
    let state = tempfile::tempdir().unwrap();
    let _harbour = running(state.path(), 101, "0 1 0 0");
    let dir = summaries(state.path());
    symlink(
        dir.join(format!("{}.counts", name(101))),
        dir.join(format!("{}.counts", name(202))),
    )
    .unwrap();
    symlink(
        dir.join(format!("{}.lock", name(101))),
        dir.join(format!("{}.lock", name(202))),
    )
    .unwrap();

    assert_eq!(stdout(&status(state.path())), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn without_xdg_state_home_status_reads_the_state_folder_under_home() {
    let home = tempfile::tempdir().unwrap();
    let state = home.path().join(".local/state");
    let _harbour = running(&state, 101, "0 1 0 0");

    let output = launch(home.path(), None);

    assert_eq!(stdout(&output), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn an_empty_xdg_state_home_counts_as_unset() {
    let home = tempfile::tempdir().unwrap();
    let state = home.path().join(".local/state");
    let _harbour = running(&state, 101, "0 1 0 0");

    let output = launch(home.path(), Some(""));

    assert_eq!(stdout(&output), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn a_relative_xdg_state_home_counts_as_unset() {
    let home = tempfile::tempdir().unwrap();
    let state = home.path().join(".local/state");
    let _harbour = running(&state, 101, "0 1 0 0");

    let output = launch(home.path(), Some("acme-state"));

    assert_eq!(stdout(&output), "#[fg=yellow]● 1#[default]\n");
}

#[test]
fn without_home_or_xdg_state_home_status_prints_nothing() {
    let output = Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("status")
        .env_remove("XDG_STATE_HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HOME")
        .output()
        .unwrap();

    assert_eq!(stdout(&output), "");
}

#[test]
fn a_relative_home_without_xdg_state_home_prints_nothing() {
    let cwd = tempfile::tempdir().unwrap();
    let _harbour = running(&cwd.path().join("acme-home/.local/state"), 101, "0 1 0 0");

    let output = Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("status")
        .env_remove("XDG_STATE_HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env("HOME", "acme-home")
        .current_dir(cwd.path())
        .output()
        .unwrap();

    assert_eq!(stdout(&output), "");
}

#[test]
fn an_empty_home_without_xdg_state_home_prints_nothing() {
    let cwd = tempfile::tempdir().unwrap();
    let _harbour = running(&cwd.path().join(".local/state"), 101, "0 1 0 0");

    let output = Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("status")
        .env_remove("XDG_STATE_HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env("HOME", "")
        .current_dir(cwd.path())
        .output()
        .unwrap();

    assert_eq!(stdout(&output), "");
}

fn status(state: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg("status")
        .env("XDG_STATE_HOME", state)
        .env_remove("XDG_CONFIG_HOME")
        .env("HOME", state)
        .output()
        .unwrap()
}

fn launch(home: &Path, state_home: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_harbour"));
    command
        .arg("status")
        .env("HOME", home)
        .env_remove("XDG_CONFIG_HOME");
    match state_home {
        Some(value) => command.env("XDG_STATE_HOME", value),
        None => command.env_remove("XDG_STATE_HOME"),
    };
    command.current_dir(home).output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn summaries(state: &Path) -> PathBuf {
    state.join("harbour/status")
}

fn running(state: &Path, pid: u32, counts: &str) -> File {
    let lock = crashed(state, pid, counts);
    lock.lock().unwrap();
    lock
}

fn crashed(state: &Path, pid: u32, counts: &str) -> File {
    let dir = summaries(state);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("{}.counts", name(pid))),
        format!("{counts}\n"),
    )
    .unwrap();
    File::create(dir.join(format!("{}.lock", name(pid)))).unwrap()
}

fn name(pid: u32) -> String {
    format!("{pid}-1700000000000000000-0")
}
