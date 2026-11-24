# Phase 40: reorder sidebar cards with J and K

Issue #165. A card's place in the sidebar is fixed today: new sessions land at the end and
nothing moves them. With the sidebar focused, `J` moves the selected card one place down and `K`
one place up, and the new order comes back after a restart.

## Verified facts

- **The sidebar's `j`/`k`** are the last arms of the match in `src/app/key.rs`, after the arms of
  `new`, `end` and `close`, which ask `Keymap::action_on`. They match the key's code only, so a
  `j` with the Shift flag selects the next card today.
- **A typed `J` arrives as `J` with the Shift flag** (`src/input.rs`, `utf8`); under
  `modifyOtherKeys` it can arrive as `j` with the Shift flag (`ESC [ 27 ; 2 ; 106 ~`).
- **Everything tied to a card lives on its `Entry`** (`src/app/mod.rs`): its editors, the shown
  editor and `on_editor`. Zoom, side by side and the swap are `App` flags read through
  `selected`, so swapping two entries and moving `selected` with them keeps them on the card.
  `C-/ 1..9` jumps by index, so it follows the new order.
- **Positions on disk** (`src/memory.rs`): each remembered conversation's file holds its
  `position`; `recall` sorts by it. `keep` in `src/app/remember.rs` rewrites a running card's
  file when its index changed, but a stopped card's file is never rewritten.
- **The config refuses `new`, `end` and `close` on `j`, `J`, `k` or `K`** (`src/config.rs`), so a
  binding on `J` or `K` only reaches the app through `App::with_keymap`.
- **The help fills a 20-row terminal with no row to spare**: 14 key rows and 4 legend lines,
  each row at most 57 columns with the `C-␣` prefix (`tests/help.rs`, `tests/app/help.rs`);
  issue #98 tracks the lasting fix.

## Decided

- The orchestrator's: `J` and `K` are fixed keys like `j` and `k`, not under `[keys]`. They act
  only with the sidebar focused; in a pane they go to the program. A binding of `new`, `end` or
  `close` on `J` or `K` wins, by the order of the match.
- `J` counts with or without the Shift flag, and so does `j` with the Shift flag; `K` alike.
- A move saves both swapped cards' positions at once, each as its index, when the card has a
  remembered conversation whose file is still there: a running card from what it last kept, a
  stopped card from its id, directory and title. A card whose conversation is open elsewhere is
  left alone, its file being the other harbour's.
- The help keeps 14 rows: an own row for `J/K` would push it past 20 rows, so the `j/k` row
  becomes `j/k J/K` `select / move the card down / up`, which keeps the asked wording
  `move the card down / up` and fits 57 columns. The README key table follows. `docs/keys.png`
  is retaken; `docs/details.png` and `docs/help.png` do not show the help's rows.

## States

| Before | Key | What the user sees |
| --- | --- | --- |
| Sidebar focused, a card in the middle | `J` | the card one place lower, still selected |
| Sidebar focused, a card in the middle | `K` | the card one place higher, still selected |
| The last card | `J` | nothing |
| The first card | `K` | nothing |
| No card | `J` or `K` | nothing |
| One card | `J` or `K` | nothing |
| `j` with the Shift flag | | as `J` |
| Pane focused, or zoomed, or side by side | `J`, `K` | the program gets them |
| `new`, `end` or `close` bound on `J` or `K` (only through `with_keymap`) | that key | the action, no move |
| A card with editors moved | `J`, then Enter | its pane, its editor |
| Moved, then `C-/ 1` | | the card now first |
| Two stopped cards swapped, harbour restarted | | they come back in the new order |
| A running card swapped with a stopped one, harbour quit and restarted | | the new order |
| A card's file gone (forgotten by another harbour) | `J` | the card moves; no file comes back |
| A card open elsewhere | `J` | the card moves; its file is not written |
| A card with no conversation yet (no transcript) | `J` | the card moves; nothing to save |
| The help at 60x20 and 59x20, prefix `C-Space` | `C-/ ?` | the `j/k J/K` row whole, the help whole |

The move draws no new text on a card: the narrowest sidebar and a CJK name draw as before.

## Tasks

1. `J`/`K` swap the selected card with its neighbour, the selection following
   (`tests/app/reordering.rs`).
2. The help row and the README key table (`tests/help.rs`, `tests/app/help.rs`).
3. Both swapped cards save their positions (`tests/app/reordering.rs`).
4. One end-to-end test through tmux (`tests/e2e/cards.rs`), the screenshot, the fragment.

## Outcome

`J` and `K` in the sidebar swap the selected card with its neighbour, the selection following, in
#177. A move saves both cards' positions at once, so stopped cards come back in the new order; a
file another harbour holds or one already forgotten is left alone. The help's first row reads
`j/k J/K  select / move the card down / up`, mirrored by the README; `docs/keys.png` was
retaken (the other shots changed only in rasterising and were left out).

### Sweep

Not run: the owner parked the sweep (#128), and the one due before this phase (#170) waits with
it. Phase 39's ready, merged and released were read from GitHub into `plans/metrics.md`.

## Hurdles

- `design` an own help row for `J/K` overflowed the 14 rows a 20-row terminal fits; it was merged into the `j/k` row within 57 columns → none
- `tooling` `scripts/screenshots.sh` rewrote every image though only the help changed; the unchanged shots were checked out again by hand → none
