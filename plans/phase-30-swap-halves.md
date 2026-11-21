# Phase 30: swap Claude Code and the editor in side by side

Issue #106. While side by side (phase 29), Alt-{ moves the focused half to the left and Alt-}
to the right, as tmux's prefix `{` and `}` swap panes. With two halves each key ends in a swap
or in nothing. The order is one setting for the whole harbour until it quits.

## Verified facts

- **The bytes**, captured in a scratch tmux 3.7c server (`tmux -S <short relative socket> -f
  /dev/null`, the scratchpad's absolute path being too long for a socket) by a program that
  turns on modifyOtherKeys 2 (`\e[>4;2m`, what harbour writes) and dumps what it reads:

  | tmux setup | `send-keys 'M-{'` | `send-keys 'M-}'` | `send-keys 'M-S-{'` | `send-keys 'M-S-}'` |
  | --- | --- | --- | --- | --- |
  | `extended-keys on`, `extended-keys-format xterm` | `CSI 27;3;123~` | `CSI 27;3;125~` | `CSI 27;4;123~` | `CSI 27;4;125~` |
  | `extended-keys on`, `extended-keys-format csi-u` | `CSI 123;3u` | `CSI 125;3u` | `CSI 123;4u` | `CSI 125;4u` |

  So the brace arrives as its own code point (123, 125), with Alt alone or with Alt and Shift.
  `M-S-[` arrived as `CSI 27;4;91~` (`[` with Alt and Shift), a different key that harbour
  leaves alone: reading it as `{` would assume a US layout.
- **Claude Code** (2.1.330, the installed binary): its strings name no `meta+` or `alt+`
  binding on `{`, `}`, `[` or `]`; its Meta bindings are letters, arrows, Space, Tab, Enter,
  Home and End. A user may still bind one. Not run interactively; and with extended keys on,
  harbour takes the two keys before Claude Code could see them.
- **Redraw on a swap**: each program keeps its half's size, so a swap sends no new size unless
  the width is odd (`Ratio(1, 2)` of 121 columns gives halves of 60 and 61), when each program
  gets the other half's width and one SIGWINCH.

## Decided

- Everything under "Decided" and "States" in the issue.
- **One flag, `swapped`, on `App`**, false at start, never written to disk. `halves` gives the
  two halves in program order, Claude Code's first, so `pane` stays Claude Code's inner area and
  `beside` the editor's, wherever they sit: the mouse, the cursor, the sizes and new sessions
  follow without a change. `enter_side` starts the editor at its own half's size.
- **The keys**: `{` or `}`, Alt with or without Shift, a press, not `ESCAPED`; the same guard as
  Alt-\ (an editor that has focus gets them). While side by side and Claude Code has focus,
  Alt-{ sets Claude Code on the left and Alt-} on the right; outside side by side they go on to
  the session as before (changed in review round 1 from "nothing happens").
- **C-h and C-l keep left and right**: swapped, C-h from Claude Code opens or shows the editor
  as C-l does unswapped; C-l from the editor gives the focus to Claude Code unless the editor
  keeps it (`wants_ctrl_l`, the mirror of `wants_ctrl_h`: anything but Neovim's normal modes
  without a blocking prompt in the rightmost window keeps C-l, so a `:terminal` shell still
  clears with C-l and `<C-w>l` still moves right); C-h from the editor, on the left, leaves the
  mode for the sidebar unless the editor keeps it (`wants_ctrl_h`). The issue names only
  `wants_ctrl_h`; mirroring it for C-l keeps a swapped Neovim working as an unswapped one.
- **The borders**: the headline and the left program's name over the left half, the right
  program's name and the counts over the right; the focused program's name bold.
- **The help**: the `M-\` row of `KEYS` becomes `M-\ M-{ M-}` with the text `side by side;
  M-{/M-} swap the halves`, no new row, so the help still fits 20 rows beside the narrowest
  sidebar. `harbour --help` gains one line after the side by side note.
- **Log**: `key name="M-{"` and `key name="M-}"` when harbour acts on them.

## States

| State | Alt-{ / Alt-} |
| --- | --- |
| Side by side, Claude Code focused on the left | Alt-} swaps, Alt-{ does nothing |
| Side by side, Claude Code focused on the right | Alt-{ swaps, Alt-} does nothing |
| Side by side, the editor half focused | the editor gets the key, as with Alt-\ |
| Side by side under 120 columns (the focused half shown alone) | the order changes and shows once the terminal is wide enough |
| The normal view, or zoom | the key reaches the session as before (round 1: was nothing happens) |
| A question, the help view, the picker, the session list or the usage popup | nothing happens |
| Another session is jumped to or picked | it is shown in the order set |
| The mode is left and entered again | the order set is kept |
| Quit and start again | Claude Code on the left |
| Extended keys unavailable (`ESC {`) | the keys reach the session as before |
| Swapped, the session ends | the mode ends as unswapped; the order is kept for the next time |
| Swapped, the editor exits | Claude Code alone, as Alt-z shows it |
| Swapped, the terminal narrows under 120 and widens again | the focused half alone, then the swapped halves again |
| Swapped, a jump (C-1 to C-9, a pick by name, a resume) | the target side by side, swapped, the half it last focused keeping the focus |
| Swapped, C-h from Claude Code (right) | the editor half gets the focus |
| Swapped, C-l from the editor (left) | Claude Code gets the focus, unless the editor keeps C-l |
| Swapped, C-h from the editor (left) | the sidebar comes back focused, unless the editor keeps C-h |
| Swapped, C-l from Claude Code (right) | Claude Code gets C-l |
| Swapped, a click in the left half | the editor focused, the press reaches it |
| Swapped, a drag from the right half across the middle | Claude Code gets every event, clamped to its half |
| Swapped, `harbour edit` | the file opens in the editor on the left, the editor focused |
| Swapped, Alt-\ from Claude Code | the sidebar back; Alt-\ again shows the swapped order |

### Text

| Width | What the border shows |
| --- | --- |
| 120, swapped | left: the headline it has unswapped, then `acme-edit: <dir>` in the room left, the directory cut from its start; right: `claude` and the counts |
| A CJK title, swapped | the same headline as unswapped, cut by display columns with `…`, never wider than the left half |
| `nerd_font` on, swapped | each name with its icon, over its own half |

The narrowest sidebar is not a row: the sidebar is hidden while side by side.

## Tasks

1. This plan, its `PLAN.md` and `plans/metrics.md` rows, phases 28 and 29's after-merge cells.
2. The keys and the swap (`src/app/side.rs`, `tests/app/side_swap.rs`): Alt-} from Claude Code
   puts it on the right, sized to its half; Alt-{ back; each guard red first.
3. C-h and C-l swapped, `wants_ctrl_l`.
4. The borders swapped.
5. The mouse swapped, a jump swapped, the mode left and entered again.
6. Through tmux (`tests/e2e/keys.rs`): both forms swap; `ESC {` with `extended-keys off` reaches
   the session.
7. The help row, the README key table, `harbour --help`, the log names, the fragment, the
   Layout of `AGENTS.md`, a `side-swapped` shot, `docs/keys.png` and `docs/help.png` again.
8. Outcome, Hurdles, metrics.

## Outcome

### Sweep

Range: phases 25 to 29. `scripts/sweep-report.sh` starts after phase 25, the last `### Sweep`,
so it reports phases 26 to 29; phase 25's own `## Hurdles` were read by hand. The report counted
23 hurdles for phases 26 to 29 (`test-seam` 10, `process` 4, `review-loop` 4, `design` 3,
`tooling` 2) and phase 25 added 4 (`test-seam`, `tooling`, `process`, `review-loop`); 14 of the
27 went `→ none`. Ready, merged and released of phases 28 and 29 were read from GitHub. The
owner decided in advance that every finding becomes an issue or is declined, no rule and no
backlog line is applied, and nobody is asked; the orchestrator filed the issues.

- A feature merged mid-phase and touched the same help rows, headroom test, README key table,
  key screenshots and `AGENTS.md` lines, in phases 27, 28 and 29, each `→ none` (phase 25 took
  in phase 24 the same way); phase 29
  waited for phase 28's merge and took 2 h 16 min to its clean review. The orchestration skill
  already puts issues that add a key in one lane; the run did not → issue #109.
- A ship stages with `git add -A` after its gates, so the worktree cannot be edited while one
  runs: phases 27 and 29, and phase 24 in the last range, each `→ none` → issue #110.
- Review round 1 found states the plan's table did not list, in phases 25, 27, 28 and 29: what
  changes while a view is open, what the user's own setup adds, and every way into a reused
  path. The states-table rule exists and did not prevent them, since it names no such rows →
  issue #111.
- Phase 28's merge started no workflow run, so its changes were released about 80 minutes later
  with phase 29's, as v0.24.0; nothing reported it → issue #112.
- A test that passed before the change existed, in phases 25, 26 and 27 (tests of an absence or
  of code that already behaved), each shown red by breaking the code → already issue #96, still
  open; nothing new.
- The help view had no spare row, in phases 26 and 29 → already issue #98, still open.
- Phase 26's two tests whose red run existed only on the CI runner (a race macOS does not
  reproduce, a child holding the terminal) → declined: one phase, and "Known limits" already
  says the tests run on Linux only in CI.
- Phase 26's interactive `/usage` screen that could not be probed past the trust question →
  declined: a fact of Claude Code, and the `-p` snapshot is the design.
- Phase 25's tmux socket path too long under the scratchpad → declined: one-off, worked around
  with a short `TMUX_TMPDIR`.
- Phase 27's help test missed in a red run, and phase 28's five fixture traps → declined: each
  fixed in code where it arose.
- Every row of phases 26 to 29 says the notice of the last ship was not known when it was
  written → declined: the row is written before that ship by construction.

Metrics: phases 25 to 29 each took 2 review rounds. Build start to the clean review ran 1 h 13
min to 1 h 29 min for phases 25 to 28 and 2 h 16 min for phase 29, the wait of #109. The
builder's notices ran 121014 to 162549 tokens and the reviewer's rounds 87098 to 163822, with
no trend across phases 26 to 29. The orchestrator's context grew from 684129 to 825147 over phases
26 to 29 in one shared session.

### Phase

- Alt-} and Alt-{ swap Claude Code and its editor side by side through one `swapped` flag on
  `App`; `halves` gives the halves in program order, so the mouse, the cursor, the sizes and a
  jump follow each program without their own change.
- Swapped, C-h and C-l keep their left and right; `wants_ctrl_l` mirrors `wants_ctrl_h` for
  Neovim's rightmost window, checked against a real clean Neovim.
- The borders keep the headline on the left and the counts on the right, whichever program sits
  there.
- The help row deviates from the plan: the key column is padded to 10 columns, and
  `M-\ M-{ M-}` takes 11, so the key stays `M-\` and its text says `side by side (120+
  columns); M-{/M-} swap`, which still fits the 20 rows. `harbour --help` gained one line, the
  log names `M-{` and `M-}`, and `docs/side-swapped.png` is the new shot.
- Verified facts: tmux 3.7c with `extended-keys` on sends Alt-{ and Alt-} in both forms (with
  Shift as modifier 4), and Claude Code 2.1.330 binds neither key.

## Hurdles

- `design` the help's key column is 10 columns wide, so the planned `M-\ M-{ M-}` key did not
  fit and the keys moved into the row's text → code
- `test-seam` the mouse, jump and end-to-end tests of the swap passed before the change, since
  the halves already followed the programs; each was shown red by breaking the code → none
- `process` the end-to-end tests were written while a ship waited on the other worktree's gates
  lock, and had to be set aside before its `git add -A` → none
- `review-loop` round 1 found the swap keys swallowed outside side by side, the swapped headline
  squeezed to its minimum by the editor's name, and a `--help` line made wrong by the swap → code
- `ci` a border test passed on macOS only: its long temporary directory left room at the end of
  the half, Linux's short one filled it; the corner cell is asserted instead → code
