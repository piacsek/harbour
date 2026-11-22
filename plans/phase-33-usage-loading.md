# Phase 33: a loading animation in the usage popup

Issue #123. The usage popup of `u` showed the report's text as soon as its process printed, then
switched to the bars once it had finished, so every open flickered from text to bars. While the
report is still being read the popup now shows a loading animation and nothing of the report.

## Verified facts

- **The main loop draws a frame on every pass** (`step` in `src/tui.rs`): `frame` first, then a
  read of the terminal that waits at most 30 ms. So harbour already redraws at least every 30 ms
  whether or not anything changed; ratatui writes only the cells that differ. The animation
  needs no way to ask for a frame, and nothing starts redrawing once it is gone: the loop is the
  same with or without the popup. `src/tui.rs` does not change.
- **When the report counts as finished** (`draw_usage` in `src/app/usage.rs`): its process has
  exited and its output is read to its end, or half a second has passed since the exit
  (`HELD`). Only then is the text parsed into a `Report`; the parsed report is kept for the
  popup's life.
- **What drew the popup before**: the bars once a report parsed and `raw` is off; else the
  session's screen once anything printed; else `printed nothing` once finished, else `loading…`.
  `t` was taken by harbour only while a parsed report exists; otherwise every key but Esc and
  `q` went to the report's process.
- **The limit bars** are 24 cells at 78 inner columns, one cell less per column under that, down
  to 4 (`lines` in `src/usage.rs`); the popup is 80 by 30, so its inside is 78 by 28.
- **The demo's report** (`src/demo.rs`, `USAGE`) prints at once and exits, so the animation
  shows there for at most a frame or two: too short for a screenshot.

## Decided

- Everything under "What changes", "Decided" and "States" in the issue.
- **A pure function** in `src/usage.rs`, `loading(step, width, height, accent)`, draws the
  popup's inside while loading: the bar, a blank row and `reading usage…`, centred, with
  `  t text · Esc close` on the last row, as `lines` puts its hint. The bar has the limit bar's cells for
  the width; the patch is centred on cell `step` modulo the cells: `█` on it, `▓` one cell
  away, `▒` two away, in the accent, every other cell `░` dimmed. At 24 cells a sweep is 24
  steps of 80 ms, 1.92 s; a narrower bar sweeps in fewer steps at the same speed per cell, the
  smallest option.
- **Rows**: with 4 rows or more the hint takes the last row and the three rows of bar, blank and
  label are centred in the whole inside, moved up when they would reach the hint's row; with
  exactly 3 rows, the three rows without the hint; with 1 or 2 rows, the label alone, centred.
- **The step comes from the clock**: the time since the popup opened, divided by 80 ms, read at
  each frame.
- **`t` (or `T`, Shift or Caps Lock, no other modifier) is taken by harbour while the report has
  not finished**, as well as while a parsed report exists: it switches between the animation and
  the text as it arrives (the session's screen, empty while nothing printed). It reuses the
  `raw` flag, so a `t` pressed while loading leaves the text shown once the report finishes,
  with ` t bars ` on the bottom border when it parsed. Each press logs `usage view=raw` or
  `usage view=bars` as `t` does on the bars, the smallest option: no new log line, and `bars`
  there means the view harbour draws itself.
- **The text shown by `t` while loading has no hint**: the report's own screen fills the inside,
  as the raw view after the bars does, and the bottom border names nothing, since `t bars` would
  promise bars that may never come. The smallest option.
- **Review round 1** (decided by the orchestrator for the owner): the loading hint reads
  `t text · Esc close`, so a report that hangs on a retry or a login message is never a bar
  forever with no way to see why; `t` before anything printed shows a dim `nothing printed yet`
  and `t animation · Esc close` on the last row instead of an empty popup, the smallest option
  that names the way back. Whether the report has finished is worked out by one helper that both
  a key and a frame call, so a `t` between the end of the report and the next frame acts on the
  finished report.
- **The screenshot** `usage-loading` runs the interface without `--demo`, through a config file
  whose `session_command` is a script named `claude` in the scratch `HOME` that sleeps, so the
  report never finishes and the animation shows; the config and the script are removed after
  the shot, as for the `accent` and `icons` shots. The demo's report is left as it is.

## States

| State | The popup shows, or what happens |
| --- | --- |
| Opened, nothing printed yet | the animation, `t text · Esc close` on the last row |
| The report prints, its process still runs | the animation |
| The report has finished and reads as bars | the bars, with no text in between |
| The report has finished and does not read as bars | the printed text, as today |
| The report printed nothing | `printed nothing`, no animation |
| Its command failed | ` exit <code> ` in the title and what it printed, or `printed nothing`, no animation |
| Something the report started keeps the terminal open | the animation until half a second after the exit |
| `t` or `T` while loading | the text as it arrives; before anything printed a dim `nothing printed yet` and `t animation · Esc close` on the last row; `t` again, the animation |
| Ctrl-t, Alt-t, or any other key but Esc and `q` while loading | goes to the report's process, as today |
| `t` was pressed while loading and the report finishes and parses | the printed text stays, ` t bars ` on the bottom border |
| `t` was pressed while loading and the report finishes without parsing | the printed text, no ` t bars `; `t` goes to the process again |
| A report that never finishes | the animation until Esc or `q`; `t` shows why |
| Esc or `q` while loading | closes the popup, ends its process, logs `usage closed`, as today |
| A paste or a click while loading | goes to the report's process, as today |
| The popup opened again | the animation from its first step, never the text, whatever was shown before |
| The popup is narrower than the bar (under 78 inner columns) | the bar shrinks as the limit bars do, down to 4 cells; the label is cut with `…` when wider than the room |
| The popup has 3 inner rows | the bar, a blank row and the label, no hint |
| The popup has 1 or 2 inner rows | the label alone, centred; no bar, no hint |
| The terminal is resized while loading | the bar and the label are centred again at the next frame |
| The terminal is resized while loading in the text view | the report's screen keeps its width once something printed, as today |
| `--demo` | the same animation, for as long as the demo's report takes (a frame or two) |
| The accent is one of the 16 colour names or `#rrggbb` | the patch in that colour, the rest of the bar dim |
| No key, no output, nothing else changes | the animation still moves: the loop draws a frame at least every 30 ms |
| The popup closes or the report finishes | the animation stops; the loop goes on as before, no more and no less often |
| The narrowest sidebar (15 columns) | unaffected: the popup is centred on the whole terminal, 80 by 30 or the terminal when smaller, never on the sidebar |
| A session or directory with a CJK name | unaffected: the popup draws no session or directory name; its label and hints are fixed text |

## Tasks

1. This plan, its `PLAN.md` and `plans/metrics.md` rows, phase 32's after-merge cells.
2. `loading` in `src/usage.rs`, with its tests in `tests/usage_loading.rs`.
3. The popup draws it while the report loads, from the clock, never the text before the bars
   (`tests/app/usage_states.rs`, `tests/app/usage_bars.rs`); tests of a running report that
   read its text press `t` first.
4. `t` while loading.
5. The fragment, the Layout of `AGENTS.md`, the `usage-loading` shot.
6. Outcome, Hurdles, metrics.

## Outcome

- Shipped as decided above: `loading` in `src/usage.rs` (15 tests in `tests/usage/loading.rs`),
  drawn by the popup until the report has finished, from the clock, and `t` while loading.
  `src/tui.rs` is unchanged.
- Tasks 2, 3 and 4 shipped as one commit: `loading` alone would have been dead code
  (`tests/dead_code.rs`), and the tests of a report that never finishes read its text only
  through `t` while loading, so neither half went green without the other.
- Every test of a report that keeps running (`tests/app/usage.rs`, `with_usage_open` in
  `tests/app/handoff.rs`) now presses `t` before reading its text; `acme-secret` in one of them
  became `acme-hidden`, since its `t` now switches the view.
- The `usage-loading` shot runs without `--demo` through a scratch `claude` script that sleeps;
  no other image changed.
- `t` while loading switches the view, so a report waiting for typed input could not get a `t`;
  `claude -p` takes none, declined for good and written into "Known limits" in `AGENTS.md`.

## Hurdles

- `test-seam` Every test that read the text of a report still running had to press `t` first, found only by running the whole `app` target (three outside the usage files) → none
- `process` Tasks 2 to 4 could not ship apart: the drawing function would have been dead code and the running-report tests needed `t` → none
- `design` The first version of the shot captured whatever frame was showing, so the image changed on every run; the shot now waits for one fixed frame → code
- `test-seam` A test asserted 20 dim cells in a row, which only the first step guarantees, so a slow first frame could fail it; it now counts the bar's cells → code
- `review-loop` The second round asked for the usage tests as one folder, a time bound on the shot's frame wait and a draw-driven wait in place of a fixed sleep → code
- `review-loop` `t` while loading never reaching the report was declined for good → known limit
