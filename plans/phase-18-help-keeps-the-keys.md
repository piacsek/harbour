# Phase 18: the help view keeps the keys, `harbour --help` the notes

## Context

Issue #69. The help view (`?`) has to show whole on a 20-row terminal and on 80x24, and it is
full: phase 14 fit its log note only by merging two other notes. Five planned features each add
a key, and so a row: #36 (zoom, Alt-z), #37 (width, Alt-minus and Alt-=), #39 (jump by name,
Alt-s), #41 (jump by number, C-1 to C-9) and #42 (side by side, Alt-\). The owner chose shorter
notes: the view keeps `KEYS` and the legend of the four states, and the notes move to
`harbour --help`, so nothing is lost.

## Verified facts (2026-11-21, `main` at 7c68e86)

- `draw_help` in `src/app/draw.rs` draws over the whole frame, whatever the layout: a bordered
  `Paragraph`, wrapped without trimming, of `KEYS` (each `"{key:<10} {what}"`), then
  `state::legend()` (4 lines), then `NOTES` (6 lines). The border takes 2 rows and 2 columns.
- `NOTES` in `src/help.rs` is used only by `draw_help`; `KEYS` also by `tests/readme.rs`.
- `USAGE` in `src/main.rs` already explains `HARBOUR_LOG` and `harbour log` at length, and the
  tmux `#{pane_current_path}` rule; it says nothing about `HARBOUR_DIR_PICKER`,
  `HARBOUR_EDITOR`, the C-h rule in Neovim, C-Backspace, vim-tmux-navigator or the newline
  under tmux.
- The two-column layout starts at 60 columns (`NARROW` in `src/app/split.rs`), so the narrowest
  sidebar is 20 columns, at a 60-column terminal; under 60 is the narrow layout. The help covers
  both alike, so only the terminal's width and height matter to it.
- Rows today, drawn in process (`Screen`, 120x20 by default):

| Rows | 120x20 (18 inside) | 80x24 (22 inside) |
| --- | --- | --- |
| `KEYS` | 8, one each | 8, one each (the longest, `x`, is 76 of 78 columns) |
| legend | 4 | 4 |
| `NOTES` | 6, one each | 8: the fzf note and the C-Backspace note wrap to two |
| used | 18 of 18, none to spare | 20 of 22 |

- At 60x20 (the narrowest sidebar) `KEYS` wraps to 12 rows and the notes are cut after the
  first; at 59x20 (narrow layout) to 13 rows; at 50x16 the view holds `KEYS` alone, cut at `q`.

## What the view shows

The pointer line reads `harbour --help has the rest.`, after the legend (the issue's wording,
the shortest that says where to look). It is drawn last, so it is the first row a short
terminal cuts; no rule drops it, since it fits at every size the issue names.

| Terminal | Rows used after | Shows |
| --- | --- | --- |
| 120x20 | 8 + 4 + 1 = 13 of 18 | the keys, the legend, the pointer; 5 rows to spare |
| 80x24 | 13 of 22 | the same, one row each; 9 rows to spare |
| 60x20, the narrowest sidebar | 12 + 4 + 1 = 17 of 18 | everything, the keys wrapped |
| 59x20, the narrow layout | 13 + 4 + 1 = 18 of 18 | everything, the keys wrapped |
| 50x16 | 14 of 14 | the keys, cut as today: unchanged, the smallest option |
| a CJK name | none | the view draws only `KEYS`, the legend and the pointer, all fixed ASCII and glyphs; no session text reaches it |

## How many key rows fit

The binding limit is 20 rows: 18 inside the border. With the pointer, 5 more one-row keys fit
before the view is full, which is exactly the five of #36, #37 (one row for both width keys),
#39, #41 and #42, as long as each description stays within 107 columns (one row at 120
columns). A sixth key needs the pointer row. At 80x24 the five fit with 4 rows to spare, each
staying one row while its description is at most 67 columns.

## Where the notes go

`USAGE` gains them right after the paragraph that opens the interface, led by
`In harbour, ? shows the keys and what the dots mean.`, one note to a line, wrapped at 80
columns like the rest. The `HARBOUR_LOG` note is not repeated: `USAGE` already says more about
the log than the note did. `NOTES` goes, so no dead code is left.

## Tasks

1. This plan, its `PLAN.md` and `plans/metrics.md` rows, phase 17's cells read from GitHub.
2. `harbour --help` prints the notes the view showed (`tests/cli.rs`).
3. The help view drops the notes and shows the pointer (`tests/app/help.rs`); `NOTES` goes; the
   fragment, the screenshots and the `AGENTS.md` line for `src/help.rs`.

## Outcome

- The help view shows `KEYS`, the legend and `harbour --help has the rest.`: 13 rows, 5 to
  spare at 120x20 and 9 at 80x24, pinned by `the_whole_help_shows_on_a_20_row_terminal` and
  `the_whole_help_shows_on_an_80_by_24_terminal`; it shows whole at 60x20 and 59x20 too.
  `no_note_shows_in_the_help` fails when a note comes back to the view.
- `harbour --help` carries the five notes it lacked, right after the paragraph on the interface,
  one to a line; `HARBOUR_LOG` was already explained there, so its note is not repeated.
- `NOTES` is gone; the pointer is `REST` in `src/help.rs`, beside `KEYS`.
- The five keys of #36, #37, #39, #41 and #42 fit at 120x20 with the pointer, as one row each
  within 107 columns; a sixth needs the pointer row or a rule that drops it.
- `docs/keys.png` and `docs/help.png` were shot again; no other image changed.
- 1 review round, with no must or should finding. Decided, not changed (the orchestrator's
  call): the pointer line stays last, after the legend, so a terminal too short for the whole
  view loses the pointer before the keys or the legend.
- The exact headroom (5 rows at 120x20) is pinned by one test,
  `the_help_leaves_five_rows_for_new_keys_on_a_20_row_terminal`, so the next key's builder
  changes one number on purpose; the 20-row and 80x24 tests assert only that the whole view
  shows.

## Hurdles

- `process` Three builders shared the machine's gates before the gates lock reached `main`, so each ship waited on a `pgrep` loop until the lock landed and was merged in → none
