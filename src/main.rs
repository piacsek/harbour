use std::io::IsTerminal;
use std::process::ExitCode;

use harbour::changelog::CHANGELOG;
use harbour::config::Config;
use harbour::{handoff, log, state, summary};

const USAGE: &str = "harbour: Harbour: where shipping begins

usage: harbour [--demo | --help | --version | changelog | status | log | config | edit [--] <file>]

With no argument on a terminal, harbour opens a sidebar of Claude Code sessions
beside a live pane. --demo opens the same with made-up sessions: no claude needed.
C-/ is harbour's prefix: press it, then a key, from the sidebar or any pane.
C-/ ? shows the keys and what the dots mean, C-/ c the changelog, and C-/ C-/
sends C-/ to the program. Most terminals send C-/ as C-_, the same key.
Settings come from $XDG_CONFIG_HOME/harbour/config.toml, else
~/.config/harbour/config.toml, read at start: editor, picker and
session_command (each a list: the program, then its arguments), sidebar_width
(columns), accent (a colour name or #rrggbb), nerd_font (true draws Nerd
Font icons) and claude_icon (the glyph before claude with nerd_font on:
\"sparkle\", the default, or \"clawd\", U+100000, which only a font of your own
draws; in Ghostty, map it with font-codepoint-map = U+100000=<font family>).
HARBOUR_EDITOR, HARBOUR_DIR_PICKER, HARBOUR_SESSION_COMMAND and HARBOUR_PREFIX
outrank the file's editor, picker, session_command and prefix; the other
settings are read from the file only. prefix names another prefix (C-] or
C-Space, for one), and a [keys] table moves an action to another key, or to
none, as zoom = \"f\" or usage = \"none\". A [colors] table sets the colour
of a card's repository, branch and pull_request (each a colour name or
#rrggbb; blue, magenta and cyan by default). config prints each setting and
where it comes from.
n runs fzf over the directory harbour was opened in and zoxide's list;
HARBOUR_DIR_PICKER names another picker.
C-/ s lists every session in fzf and Enter jumps to the one picked; with a
picker other than fzf it says so.
C-l opens nvim at Claude Code's repository root; HARBOUR_EDITOR names another.
C-Backspace acts as C-h. In Neovim, C-h goes back only from the leftmost window,
in normal mode.
With vim-tmux-navigator, add harbour to the processes it sends C-h, C-j, C-k
and C-l to. When tmux binds C-j and C-k itself, as that plugin does, C-j and
C-k from the sidebar and Claude Code move to the pane below or above harbour,
when there is one.
Under tmux, a newline is Shift-Enter with extended-keys on, else Alt-Enter.
C-/ 1 to C-/ 8 jump to that card, C-/ 9 to the last.
A terminal tmux does not know (Ghostty for one) also needs this line in
tmux.conf for Shift-Enter, with its TERM outside tmux before
the colon; then tmux source-file ~/.tmux.conf, detach and attach again:
  set -as terminal-features 'xterm-ghostty:extkeys'
C-/ z zooms the selected session to the whole screen, and back.
C-/ - and C-/ = make the sidebar 2 columns narrower or wider, from 15 columns to
half the terminal, until harbour quits.
C-/ \\ shows Claude Code on the left and its editor on the right, each in half the
terminal, and back; C-l and C-h move between the halves. It needs 120 columns;
narrower, the focused half fills the screen until the terminal widens. C-/ {
and C-/ } put Claude Code in the left or the right half. Swapped, Neovim gives
up C-l only from its rightmost window.
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
