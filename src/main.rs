use std::io::IsTerminal;
use std::process::ExitCode;

use harbour::changelog::CHANGELOG;
use harbour::{handoff, log, state, summary};

const USAGE: &str = "harbour: Harbour: where shipping begins

usage: harbour [--demo | --help | --version | changelog | status | edit [--] <file>]

With no argument on a terminal, harbour opens a sidebar of Claude Code sessions
beside a live pane. --demo opens the same with made-up sessions: no claude needed.
status prints how many sessions of every running harbour are blocked, working,
idle or unknown. For tmux, put #(harbour status) in your status-right, for example
  set -g status-right '#(harbour status) %H:%M'
and raise status-right-length if the counts are cut off.
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
        [Some("edit"), Some("--"), file] if file != Some("") => edit(&args[2]),
        [Some("edit"), file] if !matches!(file, Some("--" | "")) => edit(&args[1]),
        [] if on_a_terminal() => shown(harbour::tui::run()),
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
