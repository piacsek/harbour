# Phase 39: X ends a running session and removes its card

Issue #161. Today `x` on a running card asks `End? y/n`, and a yes ends the session but leaves its
card in the sidebar, marked `exited`; a second `x` removes it. The owner kept `x` as it is and put
the one-press close on a new key, `X`.

## Verified facts

- **`x` runs `end_or_remove`** (`src/app/key.rs`): on a running card it asks `Question::End` with
  the editors' unsaved state, else it calls `remove_selected` at once, which drops the card,
  forgets its conversation and keeps the selection in range.
- **`answer_yes` ends the session and its editors** (`src/app/mod.rs`) through `session::end_all`,
  which waits for every process it hung up on, then removes the card only when the session had
  already exited.
- **A key matched whatever its case** (`src/keymap.rs`): `action_on` and `clash` compared with
  `eq_ignore_ascii_case`, so `X` ran `end` and a binding on `X` clashed with `end` on `x`.
  `tests/config/keys.rs` pinned both: `zoom = "F"` answers `f`, and `zoom = "f"` with `find = "F"`
  was refused.
- **The help fills a 20-row terminal with no row to spare**: 14 key rows and 4 legend lines, each
  row at most 57 columns with the `C-␣` prefix (`tests/help.rs`, `tests/app/help.rs`).

## Decided

- The owner's: `x` is unchanged. A new action `close`, default key `X`, configurable under
  `[keys]` like the others: on a running card it asks the same question as `x` (with the
  unsaved-editor warning), and a yes ends the session and its editors and removes the card, the
  selection landing where removing an ended card leaves it; a no does nothing. On an ended or
  stopped card it removes it at once, as `x` does.
- The owner's: a key matches its exact case first, and falls back to the other case only when
  nothing is bound in that exact case; two bindings clash only on the same character. `zoom = "F"`
  still answers `f` when nothing else is on `f`, `zoom = "f"` with `find = "F"` now loads, and a
  file holding only `end = "x"` still loads beside `close` on `X`.
- The help keeps 14 rows: the session row becomes `n/x/X` `start / end or forget / end & forget,
  C-/ too`, 56 columns with `C-␣`. With `close` unbound the row reads as before; ` a session` is
  dropped only when `close` shares the row, so every combination fits 57 columns.
- `harbour --help` gains two lines after the picker's, saying what `x` and `X` each do.
- The question stays `Question::End`; which key asked is an `App` field, taken when the question is
  answered or dismissed. After round 1 of the review, `X` asks `End and remove? y/n`, so a Caps
  Lock `x` cannot pass for `x`.
- After round 1: a file that binds another action to `X` (say `end = "X"`) still loads; the unwritten
  default of `close` yields to it and `close` is left unbound. Two written keys on one character
  still refuse. `close` on `j`, `J`, `k` or `K` is refused as `new` and `end` are, since the case
  fallback would hand either case of `j` to it.

## States

| Card | Key | What the user sees |
| --- | --- | --- |
| Running | `x`, then `y` | the session ends, its card stays, `exited` |
| Running | `X` (asks `End and remove? y/n`), then `y` | the session and its editors end, the card goes, the next card selected (the one above when it was the last) |
| Running | `x` or `X`, then any other key | the question goes, nothing ends |
| Exited or stopped | `x` or `X` | the card goes at once, no question |
| Exited, an editor running | `x` or `X` | the question, a yes ends the editor and removes the card |
| Any | `C-/ X` | as `X` |
| `close = "none"` | `X` | `x`'s action, by the case fallback |
| `end = "X"` written | `X` | `end`'s action; `close` unbound |

## Outcome

- `X` (the `close` action) ends a running session and removes its card on a yes to the same
  question `x` asks; on an ended card it removes it at once. `x` is unchanged.
- Keys match their exact case first, falling back to the other case only when that case has no
  binding of its own; a clash is the same character only. Both rules are the owner's.
- The help's session row names all three keys in its 14 rows; `harbour --help` and the README key
  table follow. `docs/details.png` does not show the help, so it was not retaken.

## Hurdles

- `design` the issue first changed `x`; the owner moved the new behaviour to `X`, which the keymap's case-blind matching could not hold apart from `x`, so the case rule was the owner's to decide before any code → none
- `review-loop` round 1 found that the new default `X` made a file binding another action to `X` refuse to load, and that `close` escaped the `j`/`k` rule; a written key now takes `X` from `close`'s default → code
- `design` an own help row for `X` overflowed the 14 rows a 20-row terminal fits; the row was merged with `n/x` within 57 columns → none
