use std::io::IsTerminal;
use std::process::ExitCode;

use harbour::changelog::CHANGELOG;
use harbour::config::Config;
use harbour::{handoff, log, state, summary};

const USAGE: &str = "harbour: Harbour: where shipping begins

usage: harbour [--demo | --help | --version | changelog | status | log | config | edit [--] <file>]

With no argument on a terminal, harbour opens a sidebar of Claude Code sessions
beside a live pane. --demo opens the same with made-up sessions: no claude needed.
In harbour, ? shows the keys and what the dots mean.
Settings come from $XDG_CONFIG_HOME/harbour/config.toml, else
~/.config/harbour/config.toml, read at start: editor, picker and
session_command (each a list: the program, then its arguments), sidebar_width
(columns) and accent (a colour name or #rrggbb). A HARBOUR_ variable outranks
the file. config prints each setting and where it comes from.
n runs fzf over the directory harbour was opened in and zoxide's list;
HARBOUR_DIR_PICKER names another picker.
C-l opens nvim at Claude Code's repository root; HARBOUR_EDITOR names another.
C-Backspace acts as C-h. In Neovim, C-h goes back only from the leftmost window,
in normal mode.
With vim-tmux-navigator, add harbour to the processes it sends C-h and C-l to.
Under tmux, a newline is Shift-Enter with extended-keys on, else Alt-Enter.
C-1 to C-8 jump to that card, C-9 to the last; under tmux they need extended-keys
on. On macOS, Mission Control's Switch to Desktop N shortcuts can take them first.
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
wrapper script ends with exec harbour.";

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    match args.iter().map(|arg| arg.to_str()).collect::<Vec<_>>()[..] {
        [Some("--version" | "-V" | "version")] => {
            println!("harbour {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [Some("--help" | "-h" | "help")] | [Some("edit"), Some("--help" | "-h")] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        [Some("changelog")] => {
            print!("{CHANGELOG}");
            ExitCode::SUCCESS
        }
        [Some("status")] => {
            print_status();
            ExitCode::SUCCESS
        }
        [Some("log")] => print_log(),
        [Some("config")] => print_config(),
        [Some("edit"), Some("--"), file] if file != Some("") => edit(&args[2]),
        [Some("edit"), file] if !matches!(file, Some("--" | "")) => edit(&args[1]),
        [] if on_a_terminal() => match Config::from_env() {
            Ok(config) => shown(harbour::tui::run(config)),
            Err(refusal) => refused(&refusal),
        },
        [Some("--demo")] if on_a_terminal() => shown(harbour::tui::demo()),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}

fn print_status() {
    let Some(dir) = summary::directory_from_env() else {
        return;
    };
    let line = state::status_line(summary::total(&dir));
    if !line.is_empty() {
        println!("{line}");
    }
}

fn print_config() -> ExitCode {
    match Config::from_env() {
        Ok(config) => {
            for line in config.report().lines() {
                println!("{}", escaped(line));
            }
            ExitCode::SUCCESS
        }
        Err(refusal) => refused(&refusal),
    }
}

fn refused(refusal: &harbour::config::Refusal) -> ExitCode {
    eprintln!("harbour: {}", escaped(&refusal.to_string()));
    ExitCode::FAILURE
}

fn print_log() -> ExitCode {
    match log::directory_from_env().and_then(|dir| log::newest(&dir)) {
        Some(path) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        None => ExitCode::FAILURE,
    }
}

fn edit(file: &std::ffi::OsStr) -> ExitCode {
    match handoff::edit(file) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("harbour: {}", escaped(failure.message()));
            ExitCode::from(failure.status())
        }
    }
}

fn escaped(message: &str) -> String {
    message
        .chars()
        .map(|character| match character {
            control if control.is_control() => control.escape_default().to_string(),
            plain => plain.to_string(),
        })
        .collect()
}

fn on_a_terminal() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

fn shown(result: std::io::Result<()>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("harbour: {err}");
            ExitCode::FAILURE
        }
    }
}
