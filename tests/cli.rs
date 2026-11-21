use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::process::{Command, Output};

#[test]
fn version_prints_the_crate_version_on_stdout() {
    assert_eq!(stdout(&launch(&["--version"])), version_line());
}

#[test]
fn version_exits_zero() {
    assert!(launch(&["--version"]).status.success());
}

#[test]
fn the_short_version_flag_prints_the_crate_version() {
    assert_eq!(stdout(&launch(&["-V"])), version_line());
}

#[test]
fn the_short_version_flag_exits_zero() {
    assert!(launch(&["-V"]).status.success());
}

#[test]
fn the_version_word_prints_the_crate_version() {
    assert_eq!(stdout(&launch(&["version"])), version_line());
}

#[test]
fn the_version_word_exits_zero() {
    assert!(launch(&["version"]).status.success());
}

#[test]
fn version_writes_nothing_on_stderr() {
    assert!(launch(&["--version"]).stderr.is_empty());
}

#[test]
fn help_prints_the_usage_on_stdout() {
    assert_eq!(stdout(&launch(&["--help"])), USAGE);
}

#[test]
fn help_exits_zero() {
    assert!(launch(&["--help"]).status.success());
}

#[test]
fn the_short_help_flag_prints_the_usage() {
    assert_eq!(stdout(&launch(&["-h"])), USAGE);
}

#[test]
fn the_short_help_flag_exits_zero() {
    assert!(launch(&["-h"]).status.success());
}

#[test]
fn the_help_word_prints_the_usage() {
    assert_eq!(stdout(&launch(&["help"])), USAGE);
}

#[test]
fn the_help_word_exits_zero() {
    assert!(launch(&["help"]).status.success());
}

#[test]
fn help_writes_nothing_on_stderr() {
    assert!(launch(&["--help"]).stderr.is_empty());
}

#[test]
fn an_unknown_argument_prints_the_usage_on_stderr() {
    assert_eq!(stderr(&launch(&["bogus"])), USAGE);
}

#[test]
fn an_unknown_argument_exits_one() {
    assert_eq!(launch(&["bogus"]).status.code(), Some(1));
}

#[test]
fn an_unknown_argument_writes_nothing_on_stdout() {
    assert!(launch(&["bogus"]).stdout.is_empty());
}

#[test]
fn no_argument_prints_the_usage_on_stderr() {
    assert_eq!(stderr(&launch(&[])), USAGE);
}

#[test]
fn no_argument_exits_one() {
    assert_eq!(launch(&[]).status.code(), Some(1));
}

#[test]
fn a_second_argument_prints_the_usage_on_stderr() {
    assert_eq!(stderr(&launch(&["--version", "--help"])), USAGE);
}

#[test]
fn a_second_argument_exits_one() {
    assert_eq!(launch(&["--version", "--help"]).status.code(), Some(1));
}

#[test]
fn a_non_utf8_argument_exits_one() {
    assert_eq!(launch_with_bytes(b"\xff").status.code(), Some(1));
}

#[test]
fn a_non_utf8_argument_prints_the_usage_on_stderr() {
    assert_eq!(stderr(&launch_with_bytes(b"\xff")), USAGE);
}

#[test]
fn edit_without_a_file_prints_the_usage_on_stderr() {
    assert_eq!(stderr(&launch(&["edit"])), USAGE);
}

#[test]
fn edit_without_a_file_exits_one() {
    assert_eq!(launch(&["edit"]).status.code(), Some(1));
}

#[test]
fn edit_with_two_files_prints_the_usage_on_stderr() {
    assert_eq!(stderr(&launch(&["edit", "a.rs", "b.rs"])), USAGE);
}

#[test]
fn edit_with_two_files_exits_one() {
    assert_eq!(launch(&["edit", "a.rs", "b.rs"]).status.code(), Some(1));
}

#[test]
fn edit_takes_the_file_after_a_leading_double_dash() {
    let scratch = tempfile::tempdir().unwrap();

    let output = launch_at_home(scratch.path(), &["edit", "--", "acme-missing/main.rs"]);

    assert_eq!(
        stderr(&output),
        "harbour: acme-missing/main.rs: No such file or directory\n"
    );
}

#[test]
fn edit_takes_a_file_named_like_a_flag_after_a_double_dash() {
    let scratch = tempfile::tempdir().unwrap();

    let output = launch_at_home(scratch.path(), &["edit", "--", "--help/main.rs"]);

    assert_eq!(
        stderr(&output),
        "harbour: --help/main.rs: No such file or directory\n"
    );
}

#[test]
fn edit_with_an_empty_file_prints_the_usage_on_stderr() {
    assert_eq!(stderr(&launch(&["edit", ""])), USAGE);
}

#[test]
fn edit_with_an_empty_file_exits_one() {
    assert_eq!(launch(&["edit", ""]).status.code(), Some(1));
}

#[test]
fn edit_with_an_empty_file_after_a_double_dash_prints_the_usage_on_stderr() {
    assert_eq!(stderr(&launch(&["edit", "--", ""])), USAGE);
}

#[test]
fn edit_help_prints_the_usage_on_stdout() {
    assert_eq!(stdout(&launch(&["edit", "--help"])), USAGE);
}

#[test]
fn edit_help_exits_zero() {
    assert_eq!(launch(&["edit", "--help"]).status.code(), Some(0));
}

#[test]
fn edit_with_the_short_help_flag_prints_the_usage_on_stdout() {
    assert_eq!(stdout(&launch(&["edit", "-h"])), USAGE);
}

#[test]
fn edit_with_the_short_help_flag_exits_zero() {
    assert_eq!(launch(&["edit", "-h"]).status.code(), Some(0));
}

#[test]
fn edit_with_a_missing_folder_exits_two() {
    let scratch = tempfile::tempdir().unwrap();

    let output = launch_at_home(scratch.path(), &["edit", "acme-missing/main.rs"]);

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn edit_with_no_harbour_running_exits_three() {
    let scratch = tempfile::tempdir().unwrap();
    std::fs::write(scratch.path().join("main.rs"), "").unwrap();

    let output = launch_at_home(scratch.path(), &["edit", "main.rs"]);

    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn edit_with_only_a_double_dash_prints_the_usage_on_stderr() {
    let scratch = tempfile::tempdir().unwrap();

    assert_eq!(
        stderr(&launch_at_home(scratch.path(), &["edit", "--"])),
        USAGE
    );
}

#[test]
fn edit_with_a_double_dash_and_two_files_prints_the_usage_on_stderr() {
    let scratch = tempfile::tempdir().unwrap();

    let output = launch_at_home(scratch.path(), &["edit", "--", "a.rs", "b.rs"]);

    assert_eq!(stderr(&output), USAGE);
}

#[test]
fn changelog_prints_the_bundled_changelog_on_stdout() {
    assert_eq!(
        stdout(&launch(&["changelog"])),
        include_str!("../CHANGELOG.md")
    );
}

#[test]
fn changelog_exits_zero() {
    assert!(launch(&["changelog"]).status.success());
}

#[test]
fn changelog_writes_nothing_on_stderr() {
    assert!(launch(&["changelog"]).stderr.is_empty());
}

#[test]
fn log_prints_the_path_of_the_newest_log() {
    let scratch = tempfile::tempdir().unwrap();
    let logs = log_folder(scratch.path());
    write_aged(&logs.join("4100.log"), 60);
    write_aged(&logs.join("4200.log"), 10);
    write_aged(&logs.join("4300.log"), 30);

    let output = launch_at_home(scratch.path(), &["log"]);

    assert_eq!(
        stdout(&output),
        format!("{}\n", logs.join("4200.log").display())
    );
}

#[test]
fn log_with_a_log_exits_zero() {
    let scratch = tempfile::tempdir().unwrap();
    write_aged(&log_folder(scratch.path()).join("4100.log"), 10);

    assert!(launch_at_home(scratch.path(), &["log"]).status.success());
}

#[test]
fn log_leaves_out_a_newer_file_not_named_like_a_log() {
    let scratch = tempfile::tempdir().unwrap();
    let logs = log_folder(scratch.path());
    write_aged(&logs.join("4100.log"), 60);
    write_aged(&logs.join("notes.log"), 10);
    write_aged(&logs.join("4200.txt"), 10);

    let output = launch_at_home(scratch.path(), &["log"]);

    assert_eq!(
        stdout(&output),
        format!("{}\n", logs.join("4100.log").display())
    );
}

#[test]
fn log_leaves_out_a_folder_named_like_a_log() {
    let scratch = tempfile::tempdir().unwrap();
    let logs = log_folder(scratch.path());
    write_aged(&logs.join("4100.log"), 60);
    std::fs::create_dir(logs.join("4200.log")).unwrap();

    let output = launch_at_home(scratch.path(), &["log"]);

    assert_eq!(
        stdout(&output),
        format!("{}\n", logs.join("4100.log").display())
    );
}

#[test]
fn log_without_a_log_prints_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    log_folder(scratch.path());

    assert!(launch_at_home(scratch.path(), &["log"]).stdout.is_empty());
}

#[test]
fn log_without_a_log_exits_one() {
    let scratch = tempfile::tempdir().unwrap();
    log_folder(scratch.path());

    assert_eq!(
        launch_at_home(scratch.path(), &["log"]).status.code(),
        Some(1)
    );
}

#[test]
fn log_without_its_folder_exits_one() {
    let scratch = tempfile::tempdir().unwrap();

    assert_eq!(
        launch_at_home(scratch.path(), &["log"]).status.code(),
        Some(1)
    );
}

#[test]
fn log_without_a_log_writes_nothing_on_stderr() {
    let scratch = tempfile::tempdir().unwrap();

    assert!(launch_at_home(scratch.path(), &["log"]).stderr.is_empty());
}

const USAGE: &str = "harbour: Harbour: where shipping begins

usage: harbour [--demo | --help | --version | changelog | status | log | config | edit [--] <file>]

With no argument on a terminal, harbour opens a sidebar of Claude Code sessions
beside a live pane. --demo opens the same with made-up sessions: no claude needed.
In harbour, ? shows the keys and what the dots mean.
Settings come from $XDG_CONFIG_HOME/harbour/config.toml, else
~/.config/harbour/config.toml, read at start: editor, picker and
session_command (each a list: the program, then its arguments), sidebar_width
(columns), accent (a colour name or #rrggbb) and nerd_font (true draws Nerd
Font icons). HARBOUR_EDITOR, HARBOUR_DIR_PICKER and HARBOUR_SESSION_COMMAND
outrank the file's editor, picker and session_command; sidebar_width, accent
and nerd_font are read from the file only. config prints each setting and
where it comes from.
n runs fzf over the directory harbour was opened in and zoxide's list;
HARBOUR_DIR_PICKER names another picker.
M-s lists every session in fzf and Enter jumps to the one picked; with a picker
other than fzf it says so. It works only where the terminal sends Alt-s as an
extended key (under tmux, extended-keys on), and never from an editor. Under
tmux, Esc then s typed fast in Claude Code arrives as Alt-s and opens the list.
C-l opens nvim at Claude Code's repository root; HARBOUR_EDITOR names another.
C-Backspace acts as C-h. In Neovim, C-h goes back only from the leftmost window,
in normal mode.
With vim-tmux-navigator, add harbour to the processes it sends C-h and C-l to.
Under tmux, a newline is Shift-Enter with extended-keys on, else Alt-Enter.
C-1 to C-8 jump to that card, C-9 to the last; under tmux they need
extended-keys on. On macOS, Mission Control's Switch to Desktop N shortcuts can
take them first.
A terminal tmux does not know (Ghostty for one) also needs this line in
tmux.conf for any extended key, with its TERM outside tmux before the colon;
it applies after detaching and attaching again:
  set -as terminal-features 'xterm-ghostty:extkeys'
M-z zooms the selected session to the whole screen, and back. It works only
where the terminal sends Alt-z as an extended key (under tmux, extended-keys
on), and never from an editor: C-h first (from Neovim, Esc then C-h). Under
tmux, Esc then z typed fast in Claude Code arrives as Alt-z and zooms.
M-- and M-= make the sidebar 2 columns narrower or wider, from 15 columns to
half the terminal, until harbour quits; like M-z, only as extended keys and
never from an editor. Under tmux, Esc then - or = typed fast in Claude Code
arrives as the Alt key and changes the width.
M-\\ shows Claude Code on the left and its editor on the right, each in half the
terminal, and back; C-l and C-h move between the halves. It needs 120 columns;
narrower, the focused half fills the screen until the terminal widens. Like M-z,
only as an extended key and never from the editor half: C-h first.
status prints how many sessions of every running harbour are blocked, working,
idle or unknown. For tmux, put #(harbour status) in your status-right, for example
  set -g status-right '#(harbour status) %H:%M'
and raise status-right-length if the counts are cut off.
With HARBOUR_LOG set to anything but empty, harbour keeps a private log of what
it saw and decided, never what was typed, pasted or printed: one file a run
under its state folder, removed after 7 days. log prints the newest one's path:
  tail -f \"$(harbour log)\"
edit <file> opens the file in the Neovim editor of the harbour session whose
repository or worktree holds it, selects that session and shows the editor.
It exits 0 when harbour took the file, 3 when no harbour runs or answers, no
session holds the file or the Neovim harbour started did not answer in time or
exited (open it yourself), 2 when harbour or Neovim refused it or harbour went
silent once it had the file (it may still open it), and 1 on a usage error.
In tmux, splits and popups opened at #{pane_current_path} follow the selected
session when harbour is typed at a shell prompt, or when a pane command or a
wrapper script ends with exec harbour.
";

fn version_line() -> String {
    format!("harbour {}\n", env!("CARGO_PKG_VERSION"))
}

fn launch(args: &[&str]) -> Output {
    let home = tempfile::tempdir().unwrap();
    Command::new(env!("CARGO_BIN_EXE_harbour"))
        .args(args)
        .env("HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .unwrap()
}

fn launch_at_home(home: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_harbour"))
        .args(args)
        .env("HOME", home)
        .env_remove("XDG_CONFIG_HOME")
        .env("XDG_STATE_HOME", home.join("state"))
        .current_dir(home)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn launch_with_bytes(arg: &[u8]) -> Output {
    let home = tempfile::tempdir().unwrap();
    Command::new(env!("CARGO_BIN_EXE_harbour"))
        .arg(OsStr::from_bytes(arg))
        .env("HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .unwrap()
}

fn log_folder(home: &std::path::Path) -> std::path::PathBuf {
    let logs = home.join("state/harbour/log");
    std::fs::create_dir_all(&logs).unwrap();
    logs
}

fn write_aged(path: &std::path::Path, seconds: u64) {
    let file = std::fs::File::create(path).unwrap();
    let then = std::time::SystemTime::now() - std::time::Duration::from_secs(seconds);
    file.set_modified(then).unwrap();
}
