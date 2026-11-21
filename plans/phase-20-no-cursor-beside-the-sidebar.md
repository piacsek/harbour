# Phase 20: no cursor in the pane while the sidebar has focus

## Context

Issue #35. With the sidebar focused, the pane on the right still shows its program's cursor,
though keys go to the sidebar. Since phase 19 the pane's cursor is the terminal's real one,
placed after each frame at `App::cursor_position()`; this phase makes that position `None` while
the sidebar has focus, for the Claude Code pane and for the editor. The screen a session
receives is untouched. Phase 20 is a multiple of 5, so it opens with the sweep of phases 15 to
19 (issue #81).

## Verified facts (2026-11-21, `main` at f5ae3b1)

- `compose` in `src/app/draw.rs` sets the cursor in one place for the pane:
  `cursor = placed(session, self.pane)` for the shown session (Claude Code or the editor, both
  through `Entry::shown`). After it, an exited session clears it, a failure message over it
  clears it, the help view sets `(None, 0)` and the picker popup sets its own.
- `pane_focused` (`src/app/mod.rs`) says which side has focus. `n` starts a session and keeps
  the sidebar focused; Enter, C-l, `e`, a click in the pane and an open from `harbour edit`
  focus the pane; C-h over Claude Code and a click in the sidebar focus the sidebar.
- In the narrow layout (under 60 columns) `one_side` in `src/app/split.rs` shows the pane only
  while it has focus, so the pane is never drawn beside a focused sidebar there.
- The question (`x`, `q`), the help view (`?`) and the picker (`n`) are opened by sidebar keys
  only (`src/app/key.rs`), so each is shown while the sidebar has focus.
- `tui::frame` writes `ESC[?25l` before every frame and shows the cursor only when
  `cursor_position()` is `Some`, so `None` keeps the terminal's cursor hidden. The shape is
  sent from `cursor_shape()` whether a cursor shows or not.
- tmux 3.7c reports a pane's cursor visibility as `#{cursor_flag}` (1 shown, 0 hidden), so an
  end-to-end test reads what harbour left its pane.

## What the user sees

| State | Program shows its cursor | Program hid its cursor |
| --- | --- | --- |
| Pane has focus, Claude Code shown | cursor shown at its cell (as today) | none (as today) |
| Pane has focus, editor shown | cursor shown at its cell (as today) | none (as today) |
| Sidebar has focus, Claude Code shown | none | none (as today) |
| Sidebar has focus, editor shown (a click in the sidebar, or `j`/`k` to a session showing its editor) | none | none |
| Focus back to the pane (Enter, C-l, `e`, a click in the pane) | cursor shown at the same cell, with the program's shape | none |
| Another session selected while the sidebar has focus (`j`/`k`) | none; that session's shape is sent while the cursor is hidden, as phase 19 does for a hidden cursor | none |
| Narrow layout (under 60 columns), sidebar focused | the sidebar alone, no cursor (as today) | none |
| Narrow layout, pane focused | the pane alone, its cursor (as today) | none |
| Narrowest two-column layout (60 columns) | none while the sidebar has focus; the pane's cursor at its cell once the pane has it | none |
| A question at the bottom of the sidebar (`x`, `q`, asked from the sidebar) | none; the question rows never get a cursor (as today) | none |
| Help view (`?`, from the sidebar) | none, default shape (as today) | none |
| Picker popup (`n`, from the sidebar) | the picker's own cursor and shape (as today) | the picker's |
| Exited session, dimmed | none (as today) | none |
| Stopped card or no session | none (as today) | none |
| A failure message over the pane's bottom rows | none under it, as today; none at all while the sidebar has focus | none |
| A CJK line under the cursor | none while the sidebar has focus; on the wide character's first column once the pane has it | none |
| The session's own screen | untouched: the program's cursor position and visibility stay as it set them, only what harbour draws changes | untouched |

Decided: the shape stays the shown program's while the sidebar has focus, so moving focus
between the sidebar and the pane writes no shape at all, only the cursor's show or its
absence. This is the smallest change (the shape already ignores a hidden cursor) and the cursor
comes back with the right shape in the same frame. A question is always asked from the sidebar,
so it now shows no cursor; "as today" holds for what it draws and for its rows never getting a
cursor.

## Tasks

1. The sweep, this plan, the `PLAN.md` and `plans/metrics.md` rows, and the times of phases 18
   and 19 read from GitHub.
2. The pane's cursor is placed only while the pane has focus (`src/app/draw.rs`): in-process
   tests for Claude Code and the editor, the frame writing no show, and an end-to-end test of
   `#{cursor_flag}` under tmux; the tests of phase 19 that read the cursor after `n` press Enter
   first.
3. Screenshots, fragment, outcome.

## Outcome

### Sweep

Range: phases 15 to 19. `scripts/sweep-report.sh` starts after phase 15, the last
`### Sweep`, so it reports phases 16 to 19; phase 15's own `## Outcome` and `## Hurdles` were
read by hand. All five are on `main`. The report counted 12 hurdles for phases 16 to 19
(`test-seam` 4, `process` 3, `review-loop` 3, `design` 2) and phase 15 added 2 (`tooling`,
`process`); 11 of the 14 went `→ none`. Ready, merged and released of phases 18 and 19 were read
from GitHub. The owner decided in advance that every finding becomes a proposed issue or is
declined, no rule and no backlog line is applied, and nobody is asked; the orchestrator files
the issues. Findings marked "orchestrator" come from the orchestrator's notes on the lanes of
this range (the fix branches #72, #73 and #74 have no plan file).

- Phase 19 needed all 3 review rounds, and each round's hurdle says a states table row per
  frame step (what is written to the terminal, in which order) would have listed what it found;
  the plan rule asks for states, not for the order of writes → issue.
- The round cap (3, then merge with no must left) lets the fixes of the last round reach `main`
  unreviewed: phase 19's round 3 fixes (the cursor at the last changed cell, a panic mid-frame,
  the dimmed pane) shipped after the last review (phase 19, orchestrator) → issue.
- `ship.sh` piped against the rule (phase 15's hurdle, three times in the range by the
  orchestrator's count): prose alone does not hold it, and a piped ship can hide a failure, as
  `gates.sh | tail` once hid one (root `AGENTS.md`, Development) → issue.
- One builder's PR body overwritten by another's, through body files in the one scratchpad
  root parallel builders share (orchestrator); `AGENTS.md` asks to fetch a body back after an
  edit, which caught it only afterwards → issue.
- Every local gate failed while the machine ran on battery (orchestrator): the timing tests lose
  their waits when the machine slows down, and nothing names the cause → issue.
- Phase numbers taken out of order: phase 16 merged while 15 was held, so `sweep-issue.sh`
  (highest phase plus one) never announced the phase 15 sweep (phase 16, `→ none`); phases 18
  and 19 were started in reverse order. Twice in five phases → issue.
- The wait rule that deadlocked: builders waited in `pgrep` loops for each other's gates
  (phase 18, `→ none`; orchestrator) → declined: `ship.sh` takes turns by itself since #70
  (#74), and the lock's gaps are #78.
- `test-seam` hurdles (phases 17 and 19: tests waiting on a line the change hid, functions dead
  until their caller ships, keys sent while the sidebar had focus, a panic hook hiding a red
  run) → declined: four different problems, each fixed where it arose.
- `design` hurdles (phase 16's demo unable to show a zero, phase 17's rule narrowed after
  review) → declined: two different problems, each fixed in code.
- The size gate stopping a ship (phase 17) → declined: the gate did its job.
- `openpty` failing under another worktree's load (phase 15) → declined: the class of #70 and
  #78.

Metrics (from `plans/metrics.md`): review rounds 1, 1, 2, 1, 3 for phases 15 to 19; start to
release 43 min 17 s, 37 min 35 s, 1 h 31 min 46 s, 36 min 35 s and 1 h 45 min 21 s. The Claude
reviewer per round stays between 96965 and 149335 (phase 18 unknown), inside the 84k to 159k of
the last sweep; no trend. Builder notices: 2, 1, 3, 1 and 3 per phase, each between 93152 and
151809, the last ship of each unknown. The orchestrator's context grew from 249861 (phase 15's
start) to 507096 (phase 19's end), but every phase shared one session with others built in
parallel, so no two rows compare and no trend can be read. Process drift: none; every phase had
a worktree, a review and a metrics row.

### Built

- The pane's cursor is placed only while the pane has focus: one line in `compose`
  (`src/app/draw.rs`) drops the position of the shown session's cursor while the sidebar has
  focus. The shape stays the program's, so moving focus writes no shape; the picker, the help
  view, an exited session and a failure message keep their own rules, applied after it.
- Red run: four new in-process tests in `tests/app/cursor.rs` and the reworded question test
  failed on the cursor still placed beside the focused sidebar (`left: Some(Position { x: 62,
  y: 3 })`, `right: None` at `tests/app/cursor.rs:91`, `:102` and `:118`, `x: 63` for the editor
  at `:140`, and the frame test finding `ESC[?25h` at `:169`); the new end-to-end test
  `the_terminal_hides_the_cursor_while_the_sidebar_has_focus` failed with `#{cursor_flag}` still
  `1` (`tests/e2e/cursor.rs:84`). All passed after the change.
- Four tests of phase 19 that read the cursor right after `n` now press Enter first, through
  `focused_on_the_pane`, since `n` keeps the sidebar focused; the frame-order test of a session
  switch presses Enter after `k`, so its frame shows the cursor.
- A question is asked only from the sidebar, so it now shows no cursor; its rows still never get
  one.
- `scripts/screenshots.sh` rewrote every image under `docs/`, each byte-identical to `main`'s: a
  screenshot cannot show the terminal's real cursor.

## Hurdles

- `test-seam` Five cursor tests of phase 19 read a shown cursor right after `n` or `k`, which keep the sidebar focused, so they had to press Enter first before the change could land, the same assumption as phase 19's end-to-end hurdle → code
