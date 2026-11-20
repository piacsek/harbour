# Phase 3: whether each agent is working, idle or waiting on you

## Context

With several sessions open the user wants to see at a glance which agent is working, which is
idle and, above all, which one is blocked waiting on them. Today the sidebar dot only says
whether the process runs or has exited. Issue #6 asks for each dot to follow Claude Code's own
status, with tmux-agents' glyphs and colours, and for the sidebar title to count them.

## Verified facts (2026-11-20, `main` at 7ce71e3)

- Claude Code 2.1.325 to 2.1.329 keep one `<pid>.json` per live session in
  `~/.claude/sessions/`, next to a `<pid>.<hash>.key` file. Each JSON names `pid`, `status`
  (`idle` and `busy` seen live), `statusUpdatedAt` and a dozen other fields; `waitingFor` was in
  none of the files looked at, since none was waiting. Read from the files on this machine, no
  `claude` started.
- tmux-agents parses these files with `serde` (`src/registry.rs`): `status` is an enum of
  `busy`, `shell`, `idle`, `waiting` with `#[serde(other)]` falling back to unknown, unknown
  fields ignored, `waitingFor` optional. `src/state.rs` maps `busy` and `shell` to working,
  `waiting` to blocked, and gives the glyphs `●` (working), `◉` (blocked), `○` (idle and
  unknown) and the styles yellow, red, dim and dark grey.
- The process harbour starts is the one the file is named after (issue #6, checked 2026-11-19
  with Claude Code 2.1.328). `portable_pty`'s `Child::process_id` gives that pid; `session.rs`
  already uses it to hang up the process group.
- `App::draw` builds each sidebar row as a `String` (`● name` running, `○ name exited<code>`)
  and the title is the plain ` sessions `. The drive loop of `tui.rs` draws every 30 ms or
  sooner, so a 500 ms read can hang off `draw`.
- A session inherits harbour's environment, so a fixture can read `CLAUDE_CONFIG_DIR`. The e2e
  launcher unsets `XDG_CONFIG_HOME` and `XDG_STATE_HOME` beside its scratch `HOME`, not
  `CLAUDE_CONFIG_DIR`, which outranks `HOME` for the registry.
- `serde` and `serde_json` are not yet dependencies of harbour.

## States

What the user sees in a session's row, for every state the registry and the process can be in.

| The process | Its registry file | Dot | Colour | Row text |
| --- | --- | --- | --- | --- |
| running | `status` `busy` or `shell` | `●` | yellow | name |
| running | `status` `waiting`, with `waitingFor` | `◉` | red | name, then what it waits for in red |
| running | `status` `waiting`, no `waitingFor` | `◉` | red | name |
| running | `status` `idle` | `○` | dim | name |
| running | absent (not written yet, not Claude Code, removed) | `·` | dark grey | name |
| running | unreadable, not JSON, `status` missing or a value not listed | `·` | dark grey | name |
| running | no registry directory (no `CLAUDE_CONFIG_DIR` and no `HOME`) | `·` | dark grey | name |
| running | last written more than 2 s before the session started (left by an earlier process with the same pid) | `·` | dark grey | name |
| exited | anything | `○` | dim | name, "exited" and the code, as today |

The title counts the dots: ` sessions ● <working> ◉ <blocked> ○ <the rest> `, all three always
shown, zeros included. A change in the file shows within about 500 ms, the read interval.

## Decisions

- **An exit outranks the file.** Claude Code removes its file on exit, but a crash may leave it;
  harbour never reads the file of an exited session.
- **The file is found by the pid harbour started, with no search,** under
  `$CLAUDE_CONFIG_DIR/sessions`, else `$HOME/.claude/sessions`. An empty `CLAUDE_CONFIG_DIR`
  counts as unset, so harbour never reads a `sessions` folder relative to where it runs.
- **Only the dot and the waiting reason are coloured.** The name keeps its style (bold when
  selected), so cyan keeps meaning "where you are".
- **The title always carries the three counts**, even with no session, so its shape never moves.
  When ` sessions ● a ◉ b ○ c ` does not fit the sidebar (under about 72 columns), the word
  `sessions` goes and the counts stay. `○` counts idle, exited and unknown alike.
- **A waiting reason always shows, at least in part.** When the name and the reason do not fit,
  the name is cut with `…` to what is left after the reason, but never below half the row, and
  the reason is cut at the border. Other rows keep their full name; issue #9 redoes the card.
- **Unknown has its own glyph, `·`,** so idle and unknown differ by shape, not by shade alone.
- **A file last written more than 2 s before its session started is ignored**, as left by an
  earlier process with the same pid; the 2 s allow for file systems with coarse timestamps.
- **The registry is read on the draw thread.** The files are a few hundred bytes on local disk,
  read at most every 500 ms; a slow config directory is a known limit in `AGENTS.md`.
- **The demo writes its own registry** in a temporary directory: each made-up session writes its
  file the way Claude Code does, so the demo goes through the same code and never reads the
  user's real registry. The directory is removed when the demo ends.
- **The e2e launcher unsets `CLAUDE_CONFIG_DIR`** beside its scratch `HOME`, as it does the XDG
  variables, so no e2e test reads the real registry.

## Tasks

One red-green-refactor cycle each, shipped with `scripts/ship.sh`.

1. This plan, its rows in `PLAN.md` and `plans/metrics.md`, and the phase 2 backfill.
2. A running session's dot follows its registry file: working, idle and unknown, with the parser
   from tmux-agents and its tests, the 500 ms read, and the fragment.
3. Blocked: the red `◉` and what it waits for; exited keeps its row with a dim dot.
4. The title counts the dots.
5. The real binary reads `CLAUDE_CONFIG_DIR` (an e2e test with a fixture that writes its file),
   and the help view explains the dots.
6. The demo shows one session in each state, and the screenshots are rerun.

## Outcome

- Shipped as planned in six tasks on PR #13: each sidebar row's dot follows Claude Code's own
  session file (`src/registry.rs` reads it, `src/state.rs` turns it into a dot, a reason, the
  title counts and the help legend), read at most every 500 ms from `draw`.
- A session harbour cannot read (not Claude Code, no file yet, an unknown status, a malformed
  file) shows a dark grey `·` (a `○` until review round 1), so every running row of a build
  without a registry moved from `●` to that `·`, and the older tests moved with it.
- The binary's wiring to `CLAUDE_CONFIG_DIR` and `HOME` landed with task 2 instead of task 5:
  without a caller in `src/`, `with_registry` would have been test-only, and `tests/dead_code.rs`
  refuses that. Task 5 kept the e2e tests and the help legend.
- `serde` and `serde_json` are new dependencies, for the session file.
- The demo shows a session in each state (working, blocked on a permission prompt, idle,
  exited 1, unknown) by writing its own registry under the temp dir, removed when it ends; the
  screenshots were rerun.

## Hurdles

- `test-seam` The exited-dot test passed on its first run, since task 2 already dimmed an exited dot; as in phase 2, a test that is green at once proves nothing until the code is broken → none
- `tooling` A `sed` that moved the demo's running rows from `●` to `○` also rewrote the new registry e2e tests, which then failed until restored by hand → none
- `review-loop` Round 1 found the title and the waiting reason cut off at the narrowest sidebars, which the plan's state table never covered by width → code
- `test-seam` Two tests quoted `$$` in single quotes, so they wrote a literal `$$.json` and tested a missing file instead of the malformed and stale files they named → code
- `process` The dead-code rule pulled the binary's registry wiring from task 5 into task 2, so the plan's task order did not hold → none
