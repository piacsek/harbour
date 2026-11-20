use std::io::IsTerminal;
use std::process::ExitCode;

use harbour::changelog::CHANGELOG;
use harbour::{handoff, state, summary};

const USAGE: &str = "harbour: Harbour: where shipping begins

usage: harbour [--demo | --help | --version | changelog | status | edit <file>]

With no argument on a terminal, harbour opens a sidebar of Claude Code sessions
beside a live pane. --demo opens the same with made-up sessions: no claude needed.
status prints how many sessions of every running harbour are blocked, working,
idle or unknown. For tmux, put #(harbour status) in your status-right, for example
  set -g status-right '#(harbour status) %H:%M'
and raise status-right-length if the counts are cut off.
edit <file> opens the file in the Neovim editor of the harbour session whose
repository or worktree holds it, selects that session and shows the editor.
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
        [Some("--help" | "-h" | "help")] => {
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
        [Some("edit"), _] => edit(&args[1]),
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

fn edit(file: &std::ffi::OsStr) -> ExitCode {
    match handoff::edit(file) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("harbour: {message}");
            ExitCode::FAILURE
        }
    }
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
