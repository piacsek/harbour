# Phase 16: only the session counts above zero in the sidebar title

## Context

Issue #38. The sidebar title always showed four counts, zeros included
(`sessions ● 1 ◉ 0 ○ 2 · 0`). A count now shows only while it is above zero, in the same order
and colours, as `harbour status` already does for tmux. Phase 15 is held for another issue, so
this phase is 16.

## Verified facts (2026-11-20, `main` at e499835)

- `title` in `src/state.rs` tries `counted(" sessions ", counts, " ")` and falls back to
  `counted(" ", counts, "")` when it is wider than the sidebar's inner width; `counted` pushes
  every glyph and its count, zeros included.
- `status_line` in `src/state.rs` already filters out zero counts.
- `src/app/draw.rs` hands `title` the sidebar's width less its two borders; the narrowest sidebar
  gives 18, so `" sessions "` (10 columns) always fits there and the empty title below is
  reachable only through `title` itself.
- A ratatui 0.30 `Block` draws a title of `" "` as a gap in its top border (`┌ ────┐`); an
  empty `Line` leaves the border whole (`┌─────┐`).
- The demo's sessions count `● 1 ◉ 1 ○ 2 · 1`, no zero, so `docs/details.png` and the narrow
  shot do not change; a new shot ends the demo's blocked session (`j x y`) so its title reads
  `● 1 ○ 3 · 1`.

## What the user sees

| Sessions | Sidebar | Title before | Title after |
| --- | --- | --- | --- |
| 1 working, 2 idle | wide | `sessions ● 1 ◉ 0 ○ 2 · 0` | `sessions ● 1 ○ 2` |
| 1 blocked | wide | `sessions ● 0 ◉ 1 ○ 0 · 0` | `sessions ◉ 1` |
| none | wide | `sessions ● 0 ◉ 0 ○ 0 · 0` | `sessions` |
| 1 working, 2 idle | too narrow for the word | `●1 ◉0 ○2 ·0` | `●1 ○2` |
| 10 working, 10 blocked, 10 idle, 10 unknown | narrowest (18 columns) | `●10 ◉10 ○10 ·10` | the same |
| none | too narrow for the word | `●0 ◉0 ○0 ·0` | no title, the border whole |
| a session in `東京` (CJK name) | narrowest | its counts | its counts: the title holds only glyphs and digits, so a card's name never reaches it |

## Tasks

1. This plan and its rows; `title` leaves out every zero count, and with none and no room for
   the word gives no title; the in-process tests that asserted zeros follow.
2. A `shoot` of the demo with its blocked session ended, so a screenshot shows a count gone;
   the fragment.

## Outcome

- The sidebar title shows only the counts above zero, in the same order and colours:
  `sessions ● 1 ○ 2`, `sessions ◉ 1`, `sessions` alone with none; the narrow form drops the
  zeros too, and with none and no room for the word there is no title, so the border stays
  whole (a lone space would have cut it).
- The title and `harbour status` now follow one rule. The help legend still explains all four
  glyphs.
- `docs/details.png` and every other shot came out the same, since the demo counts one of each
  state; `docs/ended.png`, a new shot of the demo with its blocked session ended, shows the title
  without `◉`.

## Hurdles

- `design` The demo counts one session in each state, so no existing screenshot could show a zero left out; the orchestrator chose a new shot that ends the blocked session over changing the demo's sessions → code
