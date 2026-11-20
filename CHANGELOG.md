# Changelog

## v0.5.1 (2026-11-20)
- The Nix flake builds with a toolchain that leaves out clippy and rustfmt, which the build never runs.

## v0.5.0 (2026-11-20)
- `harbour status` prints how many sessions of every running harbour are blocked, working, idle
  or unknown, as tmux status-line text: put `#(harbour status)` in your tmux `status-right`, for
  example `set -g status-right '#(harbour status) %H:%M'`, and raise `status-right-length` if
  the counts are cut off. It prints nothing when no harbour runs a session.

## v0.4.0 (2026-11-20)
- `n` opens a directory picker before it starts a session: fzf, fed with the directory harbour was opened from (first and selected) and the directories zoxide ranks. Enter starts the session there, typing finds another one, Esc starts nothing. Each session is named after its own directory.
- Once the picker confirms a directory, the new session's pane takes the focus, so you can type to it at once.
- `HARBOUR_DIR_PICKER` names another picker: a program that draws on the terminal and prints one directory. Without fzf and without `HARBOUR_DIR_PICKER`, `n` starts the session where harbour was opened, as before.

## v0.3.0 (2026-11-20)
- Each session's dot in the sidebar follows Claude Code's own status: a yellow `●` while it works, a red `◉` with what it waits for while it is blocked on you, a dim `○` while it is idle or once it has exited, and a dark grey `·` when its status cannot be read.
- The sidebar title counts the sessions by dot: `sessions ● 1 ◉ 1 ○ 2 · 1`.
- `?` explains the dots.
- `harbour --demo` shows a made-up session in each state.

## v0.2.0 (2026-11-20)
- The selected session has a cyan bar on its left edge in place of the `›` marker, and the side that has the keys (the sidebar or the pane) has a cyan border.

## v0.1.0 (2026-11-20)
- `harbour` with no argument, on a terminal, opens a sidebar of Claude Code sessions beside a live pane.
- `n` starts a session in the current directory, `j` and `k` select one, `Ctrl-l` or Enter focuses its pane and `Ctrl-h` comes back.
- Keys, pastes and the mouse reach the focused session, and the sidebar shows when a session has exited and with what code.
- `x` ends a session or removes one that has exited, `?` shows the keys, and `q` asks first while a session runs.
- `harbour --demo` shows the same with made-up sessions, no `claude` needed.

## v0.0.1 (2026-11-19)
- harbour installs from Homebrew, a release tarball or Nix, and answers `--version`, `--help` and `changelog`.
