# Changelog

## v0.13.0 (2026-11-21)
- The selected card no longer shows its last prompt while its session works or idles, so the sidebar stays still; what a blocked, exited or stopped card says still shows on it.
- A working card that is not selected shows its last prompt, dimmed, as an idle one does, instead of its last tool call.

## v0.12.1 (2026-11-21)
- v0.12.1: internal changes only.

## v0.12.0 (2026-11-21)
- The sidebar title leaves out the session counts that are zero, as `harbour status` already
  does: `sessions ● 1 ○ 2` instead of `sessions ● 1 ◉ 0 ○ 2 · 0`, and `sessions` alone with
  none.

## v0.11.0 (2026-11-21)
- With `HARBOUR_LOG` set to anything but empty, harbour keeps a private log of what it saw and
  decided: its start and quit, the keys it acted on, where input went (by byte count only),
  focus and selection, what it refused, each session and editor starting and exiting, every
  `harbour edit` request and Neovim call, frames of 100 ms or more, and the screen errors it
  recovered from. It never records what was typed, pasted or printed, or a session's title.
  One file a run under harbour's state folder, removed after 7 days.
- `harbour log` prints the path of the newest log, so `tail -f "$(harbour log)"` follows it.

## v0.10.0 (2026-11-20)
- `harbour edit <file>` opens a file in the editor of the harbour session whose repository or
  worktree holds it, from a script or another terminal: harbour selects that session and shows
  its editor with the file open and the keys in it. When several hold the file, the one whose
  repository or worktree lies deepest wins, then the selected one, then the one used last. The editor must be Neovim; one is started with the file
  when the session has none open. Anything else prints why: it exits 3 when no harbour runs or
  answers, no session holds the file, or a Neovim harbour started did not answer in time or exited, so a script can open it itself, and 2 when harbour or
  Neovim refused it (Neovim waiting for a key, for one) or a harbour went silent once it had the
  file, since it may still open it.

## v0.9.0 (2026-11-20)
- Sessions come back after harbour closes: each conversation harbour started is remembered,
  and the next harbour shows it as a stopped card at its place in the sidebar. Enter or
  `Ctrl-l` resumes it in its directory; `x` forgets it.

## v0.8.0 (2026-11-20)
- Harbour works in the directory of the selected session: the root of its repository or
  worktree, or the directory itself outside one. A tmux shortcut that opens a split, a window
  or a popup at `#{pane_current_path}` now opens where the selected session works, and follows
  when another session is selected or Claude Code moves into a worktree. With no session, or
  when that directory is gone, it is where harbour was opened.
  It follows when harbour is typed at a shell prompt, or when a tmux pane command or a
  wrapper script ends with `exec harbour`.

## v0.7.0 (2026-11-20)
- Each session has an editor beside its Claude Code pane: `Ctrl-l` from Claude Code opens Neovim at the root of the repository or worktree Claude Code is in, and the pane shows one or the other. The pane's top border names both and marks the one shown, with the editor's directory.
- `Ctrl-h` from Neovim's leftmost window in normal mode goes back to Claude Code; anywhere else in Neovim it stays there. From the sidebar, `Ctrl-l` and Enter go to what the pane last showed.
- When Claude Code moves to another worktree, `Ctrl-l` opens a second editor there and the first keeps running, hidden, with its buffers; moving back shows it again, and `e` in the sidebar shows each of a session's editors in turn. Quitting the editor returns to Claude Code.
- `HARBOUR_EDITOR` names another editor. `x` and `q` end a session's editors with it, and their questions say when a Neovim has unsaved changes.

## v0.6.0 (2026-11-20)
- Each session in the sidebar is a card of three lines: its title and how long it has been in
  its state; what it is doing, what it waits for, its last prompt or how it exited; and its
  repository, branch, pull request and running subagents. The cards scroll to keep the
  selected one in view.

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
