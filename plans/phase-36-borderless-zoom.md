# Phase 36: zoom without borders

Alt-z shows the selected session's Claude Code alone, inside a box whose top border carries the
session's headline, the pane's names and the counts. The owner wants zoom without the box: the
headline and the counts on one plain top row, Claude Code on every row below it, edge to edge.
Side by side (Alt-\) keeps its borders, its narrow form included.

## Verified facts

- **One branch draws both**: `src/app/draw.rs` gives the pane `bordered(..)` and, when
  `self.zoomed || self.side_by_side` without two halves, `zoom_border` (`src/app/zoom.rs`) titles
  it. Side by side narrower than 120 columns, the focused half filling the screen, takes that
  same branch today.
- **Everything sized from the pane follows `self.pane`**: the session's PTY resize, the cursor
  cell, mouse clamping (`src/mouse.rs` gets the pane's area), so a bigger inner area needs no
  change there.
- **Zoom always shows Claude Code**: `zoom_by_key` returns false over a shown editor, so the
  zoomed pane never holds an editor.
- **The focus is never in question while zoomed**: the pane always has focus (`settle_zoom` drops
  zoom otherwise), so the thick accent border carried no information there.
- **`tests/app/zoom_border.rs`** pins the boxed form (`┏ ● Fix the login … ━ sessions ● 1 ┓`);
  `scripts/screenshots.sh` shoots `zoom` at 160x26.

## Decided

- **Zoom draws no border.** Row 1 of the terminal is the top row; Claude Code gets rows 2 to the
  last and every column.
- **The top row holds what the border's title held, laid out the same**: the headline (`src/zoom.rs`
  `info`, its dot guaranteed, up to 16 columns of title, the time), then the names (`claude |
  <editor>: <directory>`, `src/tabs.rs`), the counts right-aligned. Same fitting rules, with the
  room the full width minus nothing (today it is the width minus 2 for the corners).
- **The gap between left and right is blank**, no rule characters, no background colour.
- **Side by side keeps every border**, the narrow form (under 120 columns) included: it stays on
  today's boxed `zoom_border`, so only `self.zoomed` takes the new path.
- **A terminal 1 row tall** shows the top row alone and gives Claude Code no rows (its screen
  stays at the 2-row floor `src/session.rs` keeps); nothing is drawn below.

## States

| State | What the user sees |
| --- | --- |
| Zoomed, wide terminal, titled session | `● Fix the login  acme-api · main  3m  claude | nvim: api` … `sessions ● 1` on row 1, Claude Code from row 2, no box |
| Zoomed, no transcript title yet | the session's name as title, as today |
| Zoomed, no editor ever opened | no names, the headline and the counts only |
| Zoomed, 20 columns | the counts kept, the dot and one column of title, as the narrow boxed border fits today (2 more columns of room) |
| Zoomed, CJK title | cut by display width, never half a wide character |
| Zoomed, 1 row | the top row only |
| Zoomed, mouse click on row 1 | nothing reaches Claude Code (outside its area) |
| Zoomed, click on row 2 col 0 | Claude Code gets row 1 col 1 (its own origin) |
| Zoomed, cursor | Claude Code's cursor cell offset by row 1, never on the top row |
| Alt-z again | the sidebar and the boxed pane come back |
| Side by side, 120 columns or more | both halves boxed, as today |
| Side by side, under 120 columns | the focused half boxed with today's title, as today |
| Zoom dropped (session ends, focus lost) | the boxed layout, as today |

## Tasks

1. Zoom draws no border: the pane is the whole terminal minus row 1, the top row drawn with the
   headline, names and counts; `tests/app/zoom_border.rs` rewritten for the plain row, the
   side-by-side narrow case pinned boxed.
2. The `zoom` screenshot rerun, `harbour --help`'s Alt-z lines checked, the `AGENTS.md` layout
   lines for `src/app/zoom.rs` updated, the fragment `changes/phase-36-borderless-zoom.md`
   (`kind: feat`).

## Outcome

## Hurdles
