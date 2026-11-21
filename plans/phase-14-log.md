# Phase 14: a log of what harbour saw and decided

## Context

Issue #55. harbour records nothing about what it does: when a key seems lost, a session does not
start or `harbour edit` is refused, the only evidence is what was on screen at that moment, and
a panic of the terminal parser that harbour contains leaves no trace. With `HARBOUR_LOG` set, a
running harbour writes one line per event to a private file of its own; `harbour log` prints
where the newest one is. Off by default.

## Verified facts (2026-11-20, `main` at cb138d0, macOS 25.5, Rust 1.98)

- Opening a FIFO nobody reads for writing blocks the opener (checked with `mkfifo` and a Perl
  `open` under a 1 s `alarm`: it never returned). A writer thread that opens its own file is
  the only place such a stall can land; `std::sync::mpsc::SyncSender::try_send` returns at once
  with `Full` or `Disconnected` and never blocks the sender.
- `summary::directory_from_env` already gives `$XDG_STATE_HOME/harbour/status`, else
  `$HOME/.local/state/harbour/status`, and nothing when neither is absolute; `tui::remembered`
  takes its sibling with `with_file_name`. The log takes `with_file_name("log")` the same way.
- `tests/session.rs` already has a session (`wide_at_the_edge`) whose screen cut by a resize
  makes `vt100` panic, which `session::contained` catches: a contained panic can be counted
  there without a new trigger.
- `App::key` and `App::paste` already wrap `key_handled` and `paste_handled`, so what a key did
  can be logged after it was handled, in one place.
- `src/app/mod.rs` is 650 lines and `tests/e2e/main.rs` 675: the new code goes in new files
  (`src/log.rs`, `src/app/logging.rs`, `tests/log.rs`, `tests/app/logging.rs`,
  `tests/e2e/log.rs`), and the two files take only the calls into them.

## Decisions

- **`HARBOUR_LOG` with any non-empty value turns the log on**; unset or empty, it is off. The
  smallest rule: no value is refused, so a typo never stops harbour from starting.
- **Slow frame: 100 ms.** A frame of `App::draw` normally takes a few milliseconds and the
  loop polls every 30 ms; the reads on the draw thread are bounded (see Known limits) and a
  Neovim call waits at most 100 ms (`nvim::PATIENCE`) on the key thread. 100 ms is about where
  a key's echo starts to feel late, so a frame over it is what the log is for, and a lower
  threshold would fill the log with the ordinary transcript reads of every 500 ms.
- **One writer thread per log**, fed by a bounded channel (`sync_channel`, 4096 lines) through
  `try_send`: a line that finds the channel full is dropped, never waited for. The thread makes
  the folder (0700, its mode set again when it exists), removes the logs older than 7 days, opens
  `<pid>.log` (0600, truncated: a reused pid replaces the old file) and writes each line as it
  comes, unbuffered, so a kill loses at most the lines still in the channel. At quit harbour
  waits up to 500 ms for the thread to write what is queued (`Log::finish`), so `quit` reaches
  the file unless the disk stalls. The file is opened without following a symlink, and a folder
  that is a symlink is never used. Any failure ends the
  thread; the senders then find the channel closed and drop every line, silently.
- **The app is handed its log** (`App::with_log`), as it is handed its summary and listener, so
  in-process tests read the lines from a channel of their own (`Log::sending`) without a file or
  a wait. Each editor gets the app's log to time its calls to Neovim. A session counts the
  panics it contained; the app reads that count each frame and logs it.
- **A line** is `<UTC time with milliseconds> <event> <field>=<value> …`. Paths and messages are
  written as Rust debug strings (quoted, a newline escaped), so one event is always one line.
- **Sessions are named by harbour's number for them (`session=3`) and their directory**, plus
  Claude Code's session id when harbour passes or resumes one. Titles, prompts, tool calls,
  typed or pasted text and anything a session prints are never written: a key or paste sent to a
  session or the picker is `input to=<session> bytes=<n>`, `n` the bytes sent (a paste counts
  its bracketed paste marks when the session asked for them). A key harbour acts on itself is
  logged by name (`key name="j"`), so a bound key pressed with the sidebar focused is logged by
  its character; any other key is `key name=other`.
- **Mouse events sent to a session are not logged**: under mouse mode `?1003` every move would be
  a line. A click that changes the selection or the focus is logged as that change.
- **Refusals are logged as the message harbour shows** (`refused message="…"`), whenever the
  message it shows changes, so a start, resume, editor or picker refusal is caught in one place.
- **`harbour edit` requests are logged where the app serves them**: the verb, the file, the
  reply and how long it took, and a request whose deadline passed before the app reached it. A
  request the listener refuses on its own thread (another user, a path that no longer resolves,
  too slow to arrive) never reaches the app and is not logged.
- **Start and quit** are logged by the app: start at its first frame, with the version, the
  terminal's size and the settings in effect (session command, picker, editor); quit when it
  stops running.
- **`harbour log`** prints the path of the newest `<digits>.log` regular file of the log folder
  by modification time and exits 0, or prints nothing and exits 1. It needs no running harbour.
- **The end-to-end tests run harbour with `HARBOUR_LOG=1`** and print its log when a test fails
  (the `Server` remembers the scratch `HOME` and prints the newest log when dropped while
  panicking).

## States

| State | What the user sees |
| --- | --- |
| `HARBOUR_LOG` unset or empty | nothing is written, no folder is made, harbour as today |
| `HARBOUR_LOG=1` or any other non-empty value | the log is written as above |
| `XDG_STATE_HOME` absolute | the log is under `$XDG_STATE_HOME/harbour/log/` |
| `XDG_STATE_HOME` unset or relative, `HOME` absolute | under `~/.local/state/harbour/log/` |
| Neither absolute | no log; `harbour log` prints nothing and exits 1 |
| The folder cannot be made, made private or written | harbour runs without a log |
| The folder exists with a looser mode | its mode is set to 0700 again |
| The disk fills or a write fails mid-run | the log stops there, harbour carries on |
| The file's path stalls (a FIFO, a hung network disk) | lines queue up to 4096, then are dropped; keys and frames are never delayed |
| Two harbours at once | one file each, by pid |
| A pid reused by a later run | the old file is replaced |
| A log file older than 7 days | removed when a harbour with the log on starts, unless its pid is alive (an idle harbour writes nothing for days) |
| A symlink at `<pid>.log`, or a log folder that is a symlink | no log; the target is never written or made private |
| harbour quits while lines are queued | it waits up to 500 ms for them, then exits |
| A file in the folder not named `<digits>.log`, or not a regular file | left alone, never printed by `harbour log` |
| harbour killed by a signal | the lines written so far stay; the ones still queued are lost |
| `--demo`, `harbour status`, `harbour edit` (the sender) | no log |
| `harbour log` with no log in the folder, or no folder | prints nothing, exits 1 |
| The help at 120x20 and 80x24 | every key, the legend and all six notes show; the merged C-h note wraps to two lines at 80 columns |
| The help on the narrowest terminal (60x20, the narrowest sidebar; 56x26, the narrow layout) | the notes wrap by words and the help is cut at the bottom, as before this phase: the tmux note and the `HARBOUR_LOG` note do not show at 60x20, nor at 56x26; `harbour --help` carries the `HARBOUR_LOG` text in full |
| A session directory with a CJK name | the help shows no name; the log writes it as is inside quotes (`directory="/acme/東京-api"`), a name that is not UTF-8 as `\xNN` escapes |

## Tasks

1. This plan, its rows in `PLAN.md` and `plans/metrics.md`.
2. `src/log.rs`: the writer, the folder, the sweep, the time stamp, the newest log;
   `harbour log`; `HARBOUR_LOG` read in `tui::run`.
3. The app's events: start and quit, keys, input counts, focus and selection, refusals.
4. A session's life: started, exited, resumed, editors started and exited.
5. `harbour edit` requests and every call to Neovim.
6. Slow frames and contained panics.
7. The end-to-end tests: logging on, the log printed on a failure, the marker test, the off
   test.
8. `--help`, the usage, `README.md`, `NOTES`, `AGENTS.md`'s layout, the fragment, the outcome.

## Outcome

- Built as planned, in eight tasks, on PR #62. `HARBOUR_LOG` turns the log on; `harbour log`
  prints the newest log's path or exits 1. The decisions above held as written.
- A C-h that a blocking Neovim keeps is logged as `input to=editor ... bytes=1`, never as a key:
  the fake Neovim of `tests/app/handing.rs` already covered it, so no new seam was needed.
- The help gained the `HARBOUR_LOG` note without growing: the two C-h notes became one line
  ("C-Backspace acts as C-h. In Neovim, C-h goes back only from the leftmost window, in normal
  mode."), so the whole help still fits 120x20 and 80x24. `docs/help.png` and `docs/keys.png`
  were shot again; `docs/details.png` did not change.
- One new seam, `App::with_slow_frame`, on the `SEAMS` list: it lets a test set the slow-frame
  threshold to zero or a minute instead of waiting 100 ms.

## Hurdles

- `test-seam` The first app test of a contained panic passed without any panic: the fixture's
  wide character was never overwritten after the resize, so vt100 never panicked; it now redraws
  that row on `SIGWINCH`, and the red run was shown by breaking the count → code
- `process` The builder handed back early while another worktree's gates ran, instead of
  waiting in the foreground for `scripts/gates.sh` to finish → none
- `test-seam` Review round 1 asked that the app skip its log bookkeeping (the panic counts, the lives, the focus) when the log is off, proved by a test that fails today; nothing an app test can reach shows that bookkeeping, so it needs a new seam, and the change was left out → backlog
- `test-seam` The sweep test named its old log `4100.log`, a pid alive on the build machine, so skipping live pids made it fail; it now uses a pid that cannot exist → code
- `design` The seventh help note pushed the help past 20 rows, which held exactly 18 lines with
  no slack; two notes were merged into one line → code
