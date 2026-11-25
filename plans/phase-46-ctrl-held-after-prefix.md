# Phase 46: Ctrl may stay held for the key after the prefix

Issue #224. With `prefix = "C-;"`, typing `C-; C-s` says `C-; C-s does nothing`: the key after
the prefix only works bare. After the prefix, a key with Ctrl held now does what the same key
does without it, for every action and the digit jumps, as in tmux.

## Verified facts

- **`Keymap::after` (`src/keymap.rs`) returns `After::Unknown` for any modifier but Shift**, after
  it matched the prefix again and a bare Esc. Everything else goes through it: `prefix_key` in
  `src/app/prefix.rs` acts on its answer and names the key in the unknown-key message with
  `key_name`, which writes `C-` for Ctrl.
- **The prefix waits before anything else in the key path but the popups** (`key_handled` in
  `src/app/key.rs`), so a Ctrl key after it never reaches C-h, C-j, C-k, C-l or Ctrl+C handling.
  C-Backspace is turned into C-h before it, so after the prefix it acts as `h` (`sidebar`).
- **A terminal sends some Ctrl keys as other keys** without extended keys: C-m Enter, C-i Tab,
  C-h Backspace, C-[ Esc, C-2 NUL. Those reach `after` as another key, and C-[ as Esc cancels the
  prefix, as today.
- **The width keys' repeat** (`repeated` in `src/app/prefix.rs`) takes the same key alone, Shift
  aside, within 500 ms.

## Decided

- The orchestrator's: every action key and the digit jumps take Ctrl held; Ctrl and Shift held
  act as Shift alone; the prefix again is matched first, unchanged; Alt held, with or without
  Ctrl, still does nothing; Esc unchanged; no new setting; the unknown-key message keeps
  `key_name`'s naming (`C-; C-w does nothing`); `harbour --help` gains one line; the help view,
  the README and the screenshots do not change; the keys a terminal sends as another key are not
  special-cased.
- The builder's: `after` drops Ctrl as it drops Shift, so nothing else in `src/` changes. The
  width keys' repeat stays bare only: `C-/ C-=` widens once, and the repeat within 500 ms is `=`
  alone, as after `C-/ =`. The help line reads `Ctrl may stay held for the key after the prefix:
  C-/ C-z acts as C-/ z.`, under the line of the jumps.

## States

| Key after the prefix | Today | Now |
| --- | --- | --- |
| a bound key, bare or with Shift | its action | its action |
| a bound key with Ctrl, or Ctrl and Shift | `does nothing` | its action, as without Ctrl |
| a digit 1 to 9 with Ctrl (extended keys on) | `does nothing` | the jump |
| C-2 with extended keys off (NUL) | `does nothing` | `does nothing` (another key) |
| C-m, C-i, C-h with extended keys off | Enter, Tab, Backspace: `does nothing` | the same |
| C-[ | Esc: cancels | cancels |
| an unbound key with Ctrl | `C-/ C-w does nothing` | the same |
| the prefix again | sends the prefix | sends the prefix |
| Alt, or Ctrl and Alt, with any key | `does nothing` | `does nothing` |
| Esc | cancels | cancels |

No text is drawn by this phase, so no narrow-sidebar or CJK rows; `docs/details.png` is expected
to stay the same.

## Tasks

1. `Keymap::after` takes Ctrl held (`tests/keymap.rs`, a new table of `after`), the wiring in
   process (`tests/app/bindings.rs`: `C-; C-z` zooms) and end to end (`tests/e2e/keys.rs`:
   `C-/ C-1` and `C-/ C-2` jump with extended keys on), `harbour --help` (`tests/cli.rs`),
   `src/AGENTS.md`, `tests/AGENTS.md`, the fragment.

## Outcome

## Hurdles
