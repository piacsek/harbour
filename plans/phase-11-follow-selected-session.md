# Phase 11: tmux shortcuts that use the current directory follow the selected session

## Context

Issue #11. A tmux binding that opens something "here" (`split-window -c '#{pane_current_path}'`,
`new-window -c '#{pane_current_path}'`, a popup with lazygit) reads the directory of the pane's
foreground process. In harbour's pane that is harbour itself, which stays where it was opened,
so the binding opens there whatever session is selected. Harbour now moves its own working
directory to where the selected session works, so those bindings open in the right repository
or worktree without any change to the user's tmux configuration.

## Verified facts (2026-11-20, `main` at 1451741, tmux 3.7c, macOS)

Checked in a scratch server only (`tmux -L <scratch> -f /dev/null`), never the user's.

- A program's `chdir` shows in `#{pane_current_path}` at the next query: polled every 100 ms,
  it was first seen 33 ms after the call. tmux reads the directory of the pane tty's foreground
  process, so nothing has to be told.
- After the directory is deleted, tmux still reports the deleted path.
- Under a shell, tmux reports the foreground program's directory (the program, not the shell),
  and goes back to the shell's directory once the program exits.
- Harbour's sessions run on PTYs of their own, so the foreground process of harbour's pane tty
  is harbour: its working directory is what tmux reports.
- `portable_pty` resolves a relative program, and relative `PATH` entries, against the
  directory the command is given (`cwd`), which `Session::start` always sets; harbour's own
  working directory never decides where a program is found or where it starts.
- `std::env::set_current_dir` changes the whole process. In-process tests run in parallel
  threads of one process, so no in-process test may really move it.

## Where the selected session works

The same directory the editor of phase 6 opens at, read the same way (`editor::root` over the
session's last seen directory), so a tmux split and `Ctrl-l` agree:

| State | Harbour's working directory (what `#{pane_current_path}` shows) |
| --- | --- |
| No session yet, or the last one removed | where harbour was opened |
| The selected session works inside a repository | the root of that repository |
| Claude Code moved into a worktree (the transcript or the registry names it) | the root of that worktree, within half a second |
| The selected session works outside any repository (a `.git` at the home directory itself does not count) | that directory itself |
| The selection moves (`j`, `k`, a new session, a removed row) | where the newly selected session works, at the next frame |
| The directory is gone | where harbour was opened, within half a second |
| The directory is there but harbour cannot enter it (not searchable) | where harbour was opened; tried again at the next change or check |
| The selected session exited | where it was last seen working, as for its editor |
| The picker popup, the help or a question is open | unchanged: the selection does not move |
| Not running inside tmux | the same moves; nothing reads them |

## What depended on harbour's working directory staying put

| What | How it stays right |
| --- | --- |
| The picker's first directory and where `n` starts without a picker | `App::home`, read once at start: "where harbour was opened" |
| A relative `HARBOUR_SESSION_COMMAND` | resolved against the session's own directory by `portable_pty`, as before; unchanged |
| A relative `HARBOUR_EDITOR` | resolved against the editor's directory, as before; unchanged |
| A relative `HARBOUR_DIR_PICKER` | checked and run from where harbour was opened, as before (`Pick::open` gets `home`) |
| A relative `CLAUDE_CONFIG_DIR` (or the registry and transcripts under it) | made absolute at start, before harbour moves |
| A relative `TMPDIR` (the picker's scratch directory, Neovim's socket) | read once at start and made absolute; the picker and the editor are handed it |
| The demo's registry under a relative `TMPDIR` | already absolute: `tempfile` joins a relative parent to the working directory when it creates it, before harbour moves |
| A relative `XDG_STATE_HOME` | already ignored: the summary file needs an absolute one |
| A relative `HOME` | not supported: POSIX `HOME` is absolute; nothing new reads it relative |
| Child processes inheriting the directory | every spawn sets its directory (`Session::start` sets `cwd` and `PWD`) |
| `harbour status` | a separate process with its own directory; unchanged |
| `--demo` | follows its made-up checkouts like a real run |
| In-process tests | `App::new` moves nothing; only the terminal interface (`run`, `--demo`) hands the app the real move |

## Decisions

- Harbour moves only when the target differs from where it last moved, so a check every half
  second costs a few `stat` calls and no `chdir`.
- A directory harbour cannot enter falls back to where harbour was opened, silently: the
  selected session is still usable, and no message would help the user fix a tmux split.
- An exited session keeps the directory it was last seen in, the same rule as its editor.
- Harbour holds the selected session's directory open like any shell does, so it can keep a
  volume busy while selected.
- A relative `CLAUDE_CONFIG_DIR` is read from where harbour was opened, as before this phase;
  sessions still inherit it unchanged, so a Claude Code started elsewhere reads it from its own
  directory, as it always did.
- tmux reports the directory of the pane's foreground process group leader. Started from an
  interactive shell, or as the pane's own command, that is harbour. Started by a script without
  job control (`sh -c 'harbour; read'`), it is the script, and the pane's directory does not
  follow: nothing in harbour can change that, so it is left as is.

## Tasks

1. The plan, its rows in `PLAN.md` and `plans/metrics.md` (with phase 6's backfill), the
   fragment.
2. A follow module: the target from the selected session's last seen directory and a move
   behind a seam; the app moves at each frame when the selection or the directory changed, or
   every half second; in-process tests with a recording move for every row of the table.
3. The terminal interface hands the app the real `chdir`; end-to-end tests read
   `#{pane_current_path}` for each row.
4. Relative `CLAUDE_CONFIG_DIR` and `TMPDIR` made absolute at start, each with an end-to-end
   test.
5. Guidelines: the layout lines in `AGENTS.md`, the README checked.

## Outcome

- Harbour moves its own working directory to where the selected session works (`src/follow.rs`,
  called at each frame from `draw`), so a tmux split, window or popup opened at
  `#{pane_current_path}` opens there. In-process tests hand the app a recording move
  (`tests/app/moving.rs`, one test per row of the table); only the terminal interface hands it
  the real `chdir`, proven in `tests/e2e/moving.rs` by reading `#{pane_current_path}`.
- A relative `CLAUDE_CONFIG_DIR` (registry and transcripts) and a relative `TMPDIR` (the
  picker's scratch directory and Neovim's socket) are made absolute at start; each has an
  end-to-end test that failed before the fix.
- The README now says "the directory harbour was opened in" where it said "the current
  directory", which now moves.
- Nothing visible changed on screen; the screenshots are unchanged.

## Hurdles

- `test-seam` The e2e launcher runs harbour under a non-interactive `sh -c`, so tmux reported the shell's directory and the first end-to-end tests stayed red with the move wired in; harbour now runs as the pane's own process in those tests (`Server::exec`) → rule in `tests/AGENTS.md`
- `design` The plan listed the demo's registry under a relative `TMPDIR` as broken by the move; its test would not go red, because `tempfile` already makes a relative parent absolute when it creates the directory, so that change was dropped → none
