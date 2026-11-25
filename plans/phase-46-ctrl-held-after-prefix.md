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
- The builder's: `after` drops Ctrl as it drops Shift. The help line reads `Ctrl may stay held
  for the key after the prefix: C-/ C-z acts as C-/ z;` and `a digit with Ctrl held needs a
  terminal that reports extended keys.`, under the line of the jumps.
- From review round 1: a letter with Shift held after the prefix is its capital whatever case
  the terminal reports (`typed`, moved from `src/app/key.rs` to `src/keymap.rs`), so
  `C-/ C-S-x` closes as `C-/ X` does; the width keys' repeat takes Ctrl held too, so
  `C-/ C-= C-=` widens twice; Ctrl+C after the prefix shows the changelog, as `c` does; the
  digits with Ctrl held need extended keys (a legacy terminal sends C-2 as NUL, C-3 as Esc, C-4
  as C-\ and C-7 as C-_, which are not special-cased), so the help line and the fragment say so;
  a key whose character needs Shift on the user's layout is a known limit
  (`docs/known-limits.md`).

## States

| Key after the prefix | Today | Now |
| --- | --- | --- |
| a bound key, bare or with Shift | its action | its action |
| a bound key with Ctrl, or Ctrl and Shift | `does nothing` | its action, as without Ctrl |
| a lower-case letter with Shift (csi-u form), Ctrl or not | the lower-case action | the capital's action |
| C-c | `does nothing` | the changelog, as `c` |
| C-= or C-- again within 500 ms of a width key | sent to the program | widens or narrows again |
| a digit 1 to 9 with Ctrl (extended keys on) | `does nothing` | the jump |
| C-2, C-3, C-4, C-7 with extended keys off (NUL, Esc, C-\, C-_) | nothing, a cancel, nothing, the prefix sent on | nothing, a cancel, `side`, the prefix sent on |
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

`Keymap::after` drops Ctrl as it drops Shift, after matching the prefix again and Esc, so every
action key and the digit jumps act the same with Ctrl held, Ctrl and Shift as Shift alone, and
Alt still does nothing. Nothing else in `src/` changed but one `harbour --help` line. Tests:
`tests/keymap.rs`, a new table of `after` (four rows red before the change, seven guards shown
red by breaking the code they cover); `C-; C-z` zooming in process (`tests/app/bindings.rs`) and
`C-/ C-1`, `C-/ C-2` jumping end to end (`tests/e2e/keys.rs`), both red against the old
`after`; the help line in `tests/cli.rs`, red first. Shipped as one task in draft PR #225.
Phase 45's ready, merged and released were read from GitHub into `plans/metrics.md` (v0.40.0).

Review round 1 found one must and two shoulds, fixed in one ship. Ctrl-Shift-x in the csi-u
form arrives as `x` with Shift and ran `end`, not `close`: `after` now reads the key through
`typed`, moved to `src/keymap.rs` from `src/app/key.rs` and shared with the sidebar and the
rename popup, so a letter with Shift is its capital with or without Ctrl (red rows in
`tests/keymap.rs`). The width keys' repeat takes Ctrl held too (red in
`tests/app/width_keys.rs`). The help line and the fragment now say a digit with Ctrl held needs
extended keys. A `tests/keymap.rs` row pins Ctrl+C after the prefix as the changelog (a guard,
shown red by breaking `after`), and `docs/known-limits.md` gained the keys that need Shift.

## Hurdles

- `test-seam` seven guard rows of `tests/keymap.rs` (Esc, Alt, Ctrl and Alt, the prefix again twice, a bare key, an unbound Ctrl key) held before the change and were green on their first run; each was shown red by breaking the code it covers, then restored → code
- `tooling` `tests/app/prefix.rs` sits at 672 of the 700 lines `tests/source_size.rs` allows, so the in-process wiring test went into `tests/app/bindings.rs`, beside the other `C-;` test → none
- `review-loop` round 1 found states the plan's table missed: a lower-case letter reported with Shift (csi-u), the width keys' repeat with Ctrl held, and the digits a legacy terminal sends as other keys (C-3 Esc, C-4 C-\, C-7 C-_) → code
