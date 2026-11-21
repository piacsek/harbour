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
- **Where and with what.** It runs in the selected session's last seen directory when that is a
  directory, else in harbour's own, with harbour's environment (so harbour's
  `CLAUDE_CONFIG_DIR`), `TMUX` and `TMUX_PANE` removed like the picker's.
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
| `u` in the sidebar | the popup titled ` usage `, `loading…` inside until the process prints |
| `claude` slow to start | `loading…` for as long as the screen stays blank |
| it prints and exits 0 | what it printed stays in the popup until Esc or `q` |
| it exits 0 having printed nothing | `printed nothing` |
| it exits non-zero (not logged in, no network) | what it printed, and `exit <code>` in the popup's title |
| `claude` missing or not executable | no popup; `Could not start <program>: <error>` on the message line |
| the session command is not `claude` | no popup; `Usage needs Claude Code; the session command runs <program>` |
| its output wider than the popup | wrapped by Claude Code at the PTY width, which is the popup's inner width |
| its output longer than the popup | the top of it; the rest is cut |
| terminal smaller than 80 by 30 | the popup takes the terminal's size |
| narrow layout (under 60 columns), the narrowest sidebar | the popup over the whole terminal, as in the wide layout |
| a CJK line in its output | drawn by the terminal widget cell by cell, two columns each, cut at the border |
| a key other than Esc or `q`, a paste, a mouse event while open | sent to the popup's process; nothing visible once it has exited; nothing reaches a session or the sidebar |
| a second `u` while open | sent to the popup like any other key; no second popup |
| Esc or `q` | the popup closes and its process group is hung up, then killed |
| quit (`q` after the popup closes) | as before; a popup still open when harbour exits ends with harbour |
| a session selected whose directory is gone | it runs in harbour's own directory |

## Tasks

1. `u` opens the popup with the report, Esc and `q` close it; nothing stays behind (card,
   remembered file, process), in the wide and the narrow layout.
2. Keys, paste and mouse reach only the popup while it is open.
3. Loading, printed nothing, exit code, cannot start, not `claude`; the argv and the directory.
4. The help row, the README table, the headroom test at 3, the demo's made-up report and its
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

## Hurdles

- `design` the `u` row pushed the help view past 20 rows beside the narrowest sidebar, so the `q` row was shortened → code
- `test-seam` the interactive `/usage` screen could not be probed past the trust question, so the popup settled on the `-p` snapshot → none
- `test-seam` seven of the popup's state tests were green on their first run and had to be shown red by breaking the code → none
