# Phase 26: Claude Code's usage and limits in a popup

## Context

Issue #29. `u` in the sidebar opens a centred popup, 80 by 30 cells or the terminal when it is
smaller, that runs Claude Code's usage report and shows what it printed, the way `n` hosts the
directory picker. While it is open every key, paste and mouse event goes to it; Esc and `q`
close it and end its process. Its process is never a card, never remembered, never resumed.

## Verified facts (2026-11-21, `main` at 329b0f3)

Probed by hand in a scratch directory under the temporary directory, `claude` 2.1.330, in a
scratch tmux server at 78 by 28:

- Interactive `claude /usage` in a folder Claude Code has not been told to trust shows the
  workspace trust question first ("Yes, I trust this folder / No, exit"); Esc answers no, it
  prints that it did not start because the folder is not trusted and exits 0. No folder above
  the temporary directory is trusted, so the interactive Usage screen, what Esc and `q` do in
  it, and whether it writes a transcript were not verified.
- `claude -p --no-session-persistence /usage` skips the trust question (`-p` does), prints plain
  text in about 3 s and exits 0: a line on what powers the usage (a subscription), then
  `Current session: N% used · resets <date> (<zone>)`, the same for the current week across all
  models and for one model, then "What's contributing to your usage?" with sections for the
  last hours (requests, sessions, tips, top skills, subagents). About 22 lines at 78 columns;
  lines wrap at the PTY width.
- What it left behind: an empty `<config>/projects/<encoded scratch directory>/memory/` folder,
  and a `<config>/session-env/<uuid>/` folder holding the user's `SessionStart` hook script
  (the hook ran). Nothing in `<config>/sessions/`, the history file or the projects list of the
  global configuration, and no `.jsonl` transcript, so nothing Claude Code or harbour would
  offer to resume. Both folders were removed after the probe.
- Re-probed in review round 1 in the fixed folder of the decision below, with
  `claude -p --no-session-persistence --setting-sources user /usage`: it exits 0 with the same
  report and leaves the same two kinds of folder, `<config>/projects/<encoded usage folder>/memory/`
  (empty, one for that folder, so made once) and `<config>/session-env/<uuid>/` with the user's
  `SessionStart` hook script (one per press: user settings still load). Nothing in the history
  file or the projects list. Both removed after the probe.
- `claude --help` documents `--setting-sources <sources>`: "Comma-separated list of setting
  sources to load (user, project, local)." Under `-p` it says the workspace trust dialog is
  skipped and "Only use this in directories you trust."
- `--no-session-persistence` works only with `--print`. `--bare` would skip hooks but reads only
  an API key, never a subscription login, so it is not used.
- Whether `-p /usage` makes a model call or counts against the limits is not known: its output
  shows no model reply and its figures look fetched, but neither the output nor `claude --help`
  says. An account billed by API key, which has no limits, could not be probed.
- tui-term draws a screen's rows from the top: a PTY taller than the popup shows the start of
  what was printed, the session line first, and cuts the rest.

## Decisions

- **A snapshot, not the interactive screen.** The popup runs
  `<claude> -p --no-session-persistence /usage` and keeps what it printed after the process
  exits: the route whose leftovers were checked. Trade-off: no Status, Config or Stats tabs and
  no live refresh; closing and pressing `u` again runs it afresh.
- **Which program.** `<claude>` is the program of the configured session command when its file
  name is `claude` (its own arguments dropped, since they are for an interactive session);
  any other program refuses with `Usage needs Claude Code; the session command runs <program>`
  on the sidebar's message line, and no popup opens. The configured program, not a bare
  `claude` on `PATH`, because a user who points harbour at a wrapped or pinned `claude` expects
  that one.
- **Where and with what.** It always runs in harbour's own private folder, `usage/` beside
  `status/` under the state folder (`$XDG_STATE_HOME/harbour/usage`, else
  `~/.local/state/harbour/usage`), made on demand for its owner only (0700, closed again when
  found open), with `--setting-sources user`, harbour's environment (so harbour's
  `CLAUDE_CONFIG_DIR`), `TMUX` and `TMUX_PANE` removed like the picker's. With no state folder,
  or one that cannot be made private, `u` opens no popup and says why on the message line; it
  never falls back to a project directory. The first design ran it in the selected session's
  last seen directory, else harbour's: dropped in review, because `-p` skips the folder trust
  question, so pressing `u` beside a session in a folder never trusted (a freshly cloned
  repository) would run that folder's project hooks. The report belongs to the account, not to
  a folder.
- **Its PTY is 200 rows by the popup's inner width** and never resized in rows, so a short
  popup shows the top of the report rather than its end.
- **The help row.** `u  show Claude Code's usage and limits` joins `KEYS` before `?`. With it
  the help view no longer fit 20 rows beside the narrowest sidebar, so the `q` row is shortened
  to `quit, after a yes while anything runs`; the headroom test drops from 4 rows to 3.
- The log names `u` when harbour acts on it and `to=usage` with a byte count for what the popup
  receives; it never records what the popup shows.

## What the user sees

| State | What the user sees |
| --- | --- |
| `u` in the sidebar | the popup titled ` usage `, `loading…` inside while nothing is printed; the screen as soon as anything prints |
| `claude` slow to start | `loading…` for as long as the screen stays blank |
| it prints and exits 0 | what it printed stays in the popup until Esc or `q` |
| it exits 0 having printed nothing | `printed nothing`, once its output is read to its end |
| it exits while a child it started keeps the terminal open | decided half a second after the exit: what it printed, `printed nothing` or the exit |
| it exits non-zero (not logged in, no network) | what it printed, and `exit <code>` in the popup's title |
| `claude` missing or not executable | no popup; `Could not start <program>: <error>` on the message line |
| the session command is not `claude` | no popup; `Usage needs Claude Code; the session command runs <program>` |
| its output wider than the popup | wrapped by Claude Code at the PTY width, which is the popup's inner width |
| its output longer than the popup | the top of it; the rest is cut |
| the terminal shrinks after the report printed | the PTY keeps its width, so the popup shows the left part; widening shows it all again |
| terminal smaller than 80 by 30 | the popup takes the terminal's size |
| narrow layout (under 60 columns), the narrowest sidebar | the popup over the whole terminal, as in the wide layout |
| a CJK line in its output | drawn by the terminal widget cell by cell, two columns each, cut at the border |
| a key other than Esc or `q`, a paste, a mouse event while open | sent to the popup's process; nothing visible once it has exited; nothing reaches a session or the sidebar |
| a second `u` while open | sent to the popup like any other key; no second popup |
| Esc or `q` | the popup closes and its process group is hung up, then killed |
| quit (`q` after the popup closes) | as before; a popup still open when harbour exits ends with harbour |
| a session selected, its folder holding project settings | it runs in harbour's private folder, with user settings only; the project's settings and hooks never load |
| no state folder (neither `XDG_STATE_HOME` nor `HOME` absolute) | no popup; `Usage needs a state folder; set XDG_STATE_HOME or HOME` |
| the private folder cannot be made (a file in its path, not its owner's) | no popup; `Could not make <folder>: <error>` |
| the private folder exists open to others | closed to 0700 before it runs |
| Ctrl-U, Alt-U | nothing; only `u` or `U` (Shift-U, or `u` with Caps Lock) opens it |
| `Q` (Shift-Q, or `q` with Caps Lock) while open | the popup closes like `q`; harbour does not quit |
| `u` while a session is zoomed (M-z) | the sidebar is hidden and the zoomed pane has the keys, so `u` reaches that session like any letter; no popup |
| M-z while the popup is open | sent to the popup like any other key; nothing zooms |
| `harbour edit <file>` while it is open | `none` to an ask and a refusal to an open, as while the picker or a question waits; the popup stays |

## Tasks

1. `u` opens the popup with the report, Esc and `q` close it; no card, remembered file or
   process stays behind, in the wide and the narrow layout.
2. Keys, paste and mouse reach only the popup while it is open.
3. Loading, printed nothing, exit code, cannot start, not `claude`; the argv and the directory.
4. The help row, the README table, the headroom test, the demo's made-up report and its
   screenshot, the fragment.

## Outcome

- `u` opens the usage popup as planned, on the `-p` snapshot route; the interactive Usage screen
  stays unverified, since a trusted folder was never probed.
- The program comes from the session command, so a non-`claude` command refuses with a message
  instead of opening a popup; the demo hands the app its own made-up report.
- The help view's `q` row is shortened to keep the help on 20 rows beside the narrowest sidebar,
  and the headroom test now asks for 3 free rows.
- `docs/usage.png` is new and `docs/keys.png` is shot again; `docs/help.png` did not change.
- Whether `-p /usage` makes a model call or counts against the limits stays unknown.
- Merging phase 25 (zoom) brought its `M-z` row and the same shortened `q` row. With both new
  rows the help no longer fit beside the narrowest sidebar, so the `e` row is shortened to
  `show the next editor, one per directory`, and the headroom test now asks for 2 free rows.
- Review round 1: the report runs in harbour's private `usage/` folder with
  `--setting-sources user`, never in a session's folder; `harbour edit` is answered as while the
  picker waits; `printed nothing` and the exit show only once its output is read to its end; its close is
  logged as `usage closed`; only `u` and Shift-U open it. Each press still leaves a
  `session-env/<id>/` folder in the Claude Code configuration (a known limit).
- The "printed nothing" flash could not be made to happen on macOS: the old logic showed it in
  none of 90 popups of a report that prints and exits at once, and output written after the
  exit never arrives there, since the PTY is cut off when its leader exits. The fix is guarded
  by a session test (drained only once the output is read to its end) and a popup test that
  counts the frames showing `printed nothing`.
- Review round 2 found no must or should; its last fixes: `U` opens the popup and `Q` closes
  it, with Shift or Caps Lock, and a finished report keeps its PTY width, so shrinking the
  terminal no longer cuts it for good. A report whose child keeps the terminal open after the
  exit stayed on `loading…` on the CI runner, though not on macOS or in a local Linux
  container: the popup now decides half a second after the exit even when the end of output
  never comes. Half a second is far longer than reading what a finished process left in the
  PTY takes, and short enough not to read as a hang.
- 2 review rounds.

## Hurdles

- `design` the `u` row pushed the help view past 20 rows beside the narrowest sidebar, so the `q` row was shortened → code
- `test-seam` the interactive `/usage` screen could not be probed past the trust question, so the popup settled on the `-p` snapshot → none
- `test-seam` seven of the popup's state tests were green on their first run and had to be shown red by breaking the code → none
- `design` the first design ran the report in the session's folder, where `-p` skips the trust question and project hooks run → code
- `test-seam` the race between seeing the exit and reading the output could not be reproduced on macOS, so its popup test is green against the old logic → none
- `test-seam` a child holding the terminal after the exit kept the output open on the CI runner only, so the red run was CI's, not a local one → none
