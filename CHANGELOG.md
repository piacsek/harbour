# Changelog

## v0.27.0 (2026-11-21)
- `c` in the sidebar shows what changed in each release, newest first, over the whole screen, as
  `?` shows the help. `j` and `k`, the arrows or the mouse wheel scroll a line, Ctrl-d and
  Ctrl-u half a page, PageDown and PageUp a whole page, and any other key closes it. A build without releases says it has no release notes.

## v0.26.0 (2026-11-21)
- The usage popup of `u` draws Claude Code's report as bars: one per limit, coloured green,
  yellow or red by how full it is, and one per share of each breakdown. `t` shows the report as
  printed and back; a report harbour cannot read shows as printed, as before.

## v0.25.0 (2026-11-21)
- Side by side, Alt-} puts Claude Code in the right half and its editor in the left one, and
  Alt-{ puts them back. C-h and C-l still move left and right, clicks and borders follow each
  program to its half, and a jump keeps the order. Like Alt-\, it needs the extended keys and
  never comes from the editor half.

## v0.24.2 (2026-11-21)
- When harbour is ended by `kill`, an interrupt sent from outside, or its terminal or tmux pane
  closing, it now ends its sessions and cleans up as a quit does, and gives the terminal back
  with the mouse, bracketed paste and extended keys turned off, so Ctrl+C and Ctrl+D work again
  in the shell after it. It then ends by that same signal, so a shell or a script sees it was
  killed or interrupted. A second `kill` or interrupt ends a harbour that is stuck at once.
  Any other signal, SIGKILL included, still leaves the terminal needing `reset`.

## v0.24.1 (2026-11-21)
- `harbour --help` says that under tmux a terminal tmux does not know, such as Ghostty, also needs `extkeys` in `terminal-features` for Ctrl-1 to Ctrl-9 and Shift-Enter.

## v0.24.0 (2026-11-21)
- Alt-s lists every session in fzf, running, ended and stopped alike, by title, repository and
  branch, with each one's time, or `exited` or `stopped` in its place; typing filters the list,
  Enter jumps to the one picked as Ctrl-1 to Ctrl-9 do (a stopped one resumes), and Esc closes
  it and changes nothing. With a picker other than fzf it
  says the finder needs fzf.
- Alt-s works only where the terminal sends it as an extended key (under tmux, `extended-keys
  on`), and never from the editor. Under tmux, Esc then `s` typed fast in Claude Code arrives as
  Alt-s and opens the list.
- The help view says `C-l/Enter  focus or resume; C-l in Claude Code: editor`, so the whole
  help still fits 20 rows beside the narrowest sidebar.
- Alt-\ hides the sidebar and shows the selected session's Claude Code on the left and its
  editor on the right, each in half the terminal; Alt-\ again brings the sidebar back. It needs
  a terminal at least 120 columns wide and one that sends Alt-\ as an extended key (under tmux,
  `extended-keys` on), and never acts from the editor half: C-h there first.
- Side by side, C-l and C-h move the focus between the halves, a click focuses the half it
  lands in, and only the focused half shows a cursor. Below 120 columns the focused half fills
  the screen until the terminal is wide enough again.

## v0.23.0 (2026-11-21)
- Alt-minus makes the sidebar 2 columns narrower and Alt-equals 2 columns wider, from the
  sidebar or the Claude Code pane, between 15 columns and half the terminal, until harbour
  quits. A held key repeats, and Shift held as well changes nothing.
- They work only where the terminal sends them as extended keys (under tmux, `extended-keys on`),
  and never from the editor. Under tmux, Esc then `-` or `=` typed fast in Claude Code arrives
  as the Alt key and changes the width.
- A session that has ended keeps its last screen whole when the sidebar, the terminal or a zoom
  changes the pane's width; it was cut at the right edge for good.

## v0.22.0 (2026-11-21)
- `u` in the sidebar shows Claude Code's usage and limits in a popup; Esc or `q` closes it.

## v0.21.0 (2026-11-21)
- Alt-z zooms the selected session to the whole screen, and back. The sidebar hides and the
  pane's top border shows the session's dot, title, repository and branch, and time, then the
  names, with the counts at the right. C-h from Claude Code ends the zoom, and C-1 to C-9 jump
  to another session still zoomed.
- Alt-z zooms only where the terminal sends it as an extended key (under tmux,
  `extended-keys on`), and never from an editor: C-h first. Under tmux, Esc then z typed fast in Claude Code
  arrives as Alt-z and zooms.
- The help says `q` quits "after a yes while anything runs", a shorter line that leaves room for
  the new key.

## v0.20.0 (2026-11-21)
- With `nerd_font = true`, a card's third line puts an icon before its repository, its branch,
  its pull request and its count of running subagents, in place of the dots and the word
  "subagents", and ends with a Neovim icon while the session has Neovim open. A narrow sidebar
  drops the facts from the end as before, and an icon never shows without some of its text.

## v0.19.0 (2026-11-21)
- Ctrl-1 to Ctrl-8 select the session card at that position and focus its pane, Ctrl-9 the
  last card, from the sidebar or from inside a pane; a stopped card resumes as Enter does.
  Under tmux they need `extended-keys on`; without it the keys reach the session as before.

## v0.18.0 (2026-11-21)
- `nerd_font = true` in the config file puts an icon before `claude` and before `nvim` in the
  pane's border. It needs a terminal font with Nerd Font glyphs, so it is off by default, and an
  editor other than Neovim gets no icon. `harbour config` shows the setting and where it comes
  from, and a value other than `true` or `false` is refused like any other bad setting.

## v0.17.0 (2026-11-21)
- Harbour reads its settings from `$XDG_CONFIG_HOME/harbour/config.toml`, else
  `~/.config/harbour/config.toml`, at start: `editor`, `picker` and `session_command` (each a
  program with its arguments), `sidebar_width` and `accent`. A `HARBOUR_` variable still wins
  over the file; without a file nothing changes.
- `sidebar_width` sets the sidebar's columns, borders included, with the pane keeping at least
  half, and `accent` colours the focused border and the selected card's bar (cyan by default).
- `harbour config` prints the file's path and every setting with where it comes from: the file,
  a variable or the default. A bad file is named with its line and key in one line, here and at
  start.
- A `picker` or `session_command` from the file whose program is named `fzf` or `claude` runs
  as the default does (fzf over the list, claude with its session id), with the arguments given;
  a configured picker that cannot start says so instead of being skipped. A `claude` whose own
  arguments already pick a conversation (`--continue`, `--resume`, `--session-id`) runs as given.

## v0.16.1 (2026-11-21)
- An editor or the directory picker that asks for extended keys, as Neovim does, now gets each
  key in the form it asked for, so Ctrl+Esc, Ctrl+Tab, Ctrl+Enter, Shift+Enter, Ctrl with a
  digit or punctuation, Ctrl+Shift with a letter, and F13 and above reach it as themselves.
  Ctrl+I, Ctrl+M and Ctrl+[ reach it apart from Tab, Enter and Esc when the terminal tells them
  apart (under tmux, with `extended-keys on`).
- Claude Code gets the same keys as before, whatever it asks for.
- With extended keys on in tmux, Ctrl+M in the sidebar acts as Enter, and Ctrl+Shift+H and
  Ctrl+Shift+L act as Ctrl+H and Ctrl+L where harbour uses those keys.

## v0.16.0 (2026-11-21)
- While the sidebar has focus, the pane shows no cursor, for Claude Code and for the editor; it
  comes back at the same place, with its shape, as soon as the pane has focus again.

## v0.15.0 (2026-11-21)
- The cursor in a pane takes the shape its program asks for: a thin bar in Neovim's insert mode,
  an underline in its replace mode, a block in normal mode. It is the terminal's own cursor,
  so it blinks and is coloured as the terminal draws it, and harbour gives the terminal back
  its default shape when it quits.

## v0.14.0 (2026-11-21)
- The help view (`?`) shows only the keys and what the dots mean, with a line saying
  `harbour --help` has the rest, so new keys have room.
- `harbour --help` carries the notes the help view used to show: the directory picker and the
  editor settings, the C-h rule in Neovim, C-Backspace, vim-tmux-navigator and the newline
  under tmux.

## v0.13.2 (2026-11-21)
- Under tmux with `extended-keys on`, Shift+Enter and Ctrl+Enter in a session start a new line
  instead of sending the message, whichever `extended-keys-format` tmux uses.

## v0.13.1 (2026-11-21)
- Inside tmux, the editor harbour opens beside a session keeps harbour's `TMUX` and
  `TMUX_PANE`, so a tmux command it runs acts on harbour's pane: a test runner that drives
  tmux opens its split beside harbour instead of failing with "not inside tmux session".
  Claude Code sessions and the directory picker still start outside tmux.

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
